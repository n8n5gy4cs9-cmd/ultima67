//! Versioned save files: 10 slots + quicksave (slot 0 / "quick").
use crate::data::GameData;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const SAVE_VERSION: u32 = 1;
pub const SLOTS: u8 = 10;

#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u32,
    pub saved_at_minutes: u64,
    pub label: String,
    pub data: GameData,
}

pub fn slot_path(dir: &Path, slot: Option<u8>) -> PathBuf {
    match slot {
        None | Some(0) => dir.join("quick.json"),
        Some(n) => dir.join(format!("slot_{n:02}.json")),
    }
}

pub fn write(dir: &Path, slot: Option<u8>, d: &GameData) -> Result<PathBuf, String> {
    if let Some(n) = slot {
        if n > SLOTS {
            return Err(format!("slot must be 0..={SLOTS}"));
        }
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let label = format!("{} - day {} {:02}:{:02}", d.current_map, d.clock.day() + 1, d.clock.hour(), d.clock.minute());
    let f = SaveFile { version: SAVE_VERSION, saved_at_minutes: d.clock.minutes, label, data: d.clone() };
    let path = slot_path(dir, slot);
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&f).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?; // atomic-ish
    Ok(path)
}

pub fn read(dir: &Path, slot: Option<u8>) -> Result<GameData, String> {
    let path = slot_path(dir, slot);
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut v: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let ver = v["version"].as_u64().unwrap_or(0) as u32;
    if ver > SAVE_VERSION {
        return Err(format!("save is from a newer version ({ver} > {SAVE_VERSION})"));
    }
    migrate(&mut v, ver);
    let f: SaveFile = serde_json::from_value(v).map_err(|e| e.to_string())?;
    let mut d = f.data;
    d.map_dirty = true;
    Ok(d)
}

/// Upgrade old save JSON in place. Add one step per version bump.
fn migrate(_v: &mut serde_json::Value, _from: u32) {}

/// (slot, label) for every existing save.
pub fn list(dir: &Path) -> Vec<(Option<u8>, String)> {
    let mut out = vec![];
    for slot in std::iter::once(None).chain((1..=SLOTS).map(Some)) {
        if let Ok(b) = std::fs::read(slot_path(dir, slot)) {
            if let Ok(f) = serde_json::from_slice::<serde_json::Value>(&b) {
                out.push((slot, f["label"].as_str().unwrap_or("?").to_string()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tmpdir(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("u67_test_{n}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }
    #[test]
    fn roundtrip_and_slots() {
        let dir = tmpdir("save");
        let mut d = GameData::new(7, "midgard", [10.5, 20.5]);
        d.flags.insert("x".into());
        d.players[0].inventory.add("rope", 1).unwrap();
        write(&dir, Some(3), &d).unwrap();
        write(&dir, None, &d).unwrap();
        let r = read(&dir, Some(3)).unwrap();
        assert_eq!(r.players[0].pos, [10.5, 20.5]);
        assert!(r.flags.contains("x") && r.map_dirty);
        assert_eq!(r.players[0].inventory.count("rope"), 1);
        assert_eq!(list(&dir).len(), 2);
        assert!(read(&dir, Some(9)).is_err());
        assert!(write(&dir, Some(11), &d).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[test]
    fn rejects_future_version() {
        let dir = tmpdir("future");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(slot_path(&dir, Some(1)), br#"{"version":99}"#).unwrap();
        assert!(read(&dir, Some(1)).unwrap_err().contains("newer"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
