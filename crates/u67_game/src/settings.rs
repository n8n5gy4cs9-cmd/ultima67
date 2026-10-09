//! User settings (RON): window, volumes, keybindings, gamepad. Written with defaults on first run.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Run,
    Interact,
    Attack,
    Inventory,
    Pause,
    Console,
    Cheats,
    QuickSave,
    QuickLoad,
    Reload,
    Roll,
    ZoomIn,
    ZoomOut,
    Map,
    Journal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PadMap {
    pub interact: String,
    pub attack: String,
    pub attack_alt: String,
    pub roll: String,
    pub reload: String,
    pub inventory: String,
    pub run: String,
    pub pause: String,
    pub spell1: String,
    pub spell2: String,
    pub spell3: String,
}

impl Default for PadMap {
    fn default() -> Self {
        let s = |x: &str| x.to_string();
        Self {
            interact: s("South"),
            attack: s("RightTrigger"),
            attack_alt: s("RightTrigger2"),
            roll: s("East"),
            reload: s("West"),
            inventory: s("North"),
            run: s("LeftTrigger"),
            pause: s("Start"),
            spell1: s("DPadLeft"),
            spell2: s("DPadUp"),
            spell3: s("DPadRight"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub fullscreen: bool,
    pub vsync: bool,
    pub zoom: f32,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub gamepad_deadzone: f32,
    pub text_scale: f32,
    /// Reload changed asset files while the game runs (dev convenience; images/audio).
    #[serde(default = "default_true")]
    pub hot_reload: bool,
    /// Orange/blue health bars instead of red/green.
    #[serde(default)]
    pub colorblind: bool,
    /// Gamepad button names per action (South, East, West, North, LeftTrigger, RightTrigger, LeftTrigger2,
    /// RightTrigger2, Start, Select, DPadUp, DPadDown, DPadLeft, DPadRight, LeftThumb, RightThumb).
    #[serde(default)]
    pub pad: PadMap,
    /// Action -> key names (see `input::key_from_name`). Several keys per action allowed.
    pub keys: BTreeMap<Action, Vec<String>>,
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        use Action::*;
        let k = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let keys = BTreeMap::from([
            (Up, k(&["W", "ArrowUp"])),
            (Down, k(&["S", "ArrowDown"])),
            (Left, k(&["A", "ArrowLeft"])),
            (Right, k(&["D", "ArrowRight"])),
            (Run, k(&["ShiftLeft"])),
            (Interact, k(&["E", "Enter"])),
            (Attack, k(&["Space"])),
            (Inventory, k(&["I", "Tab"])),
            (Pause, k(&["Escape"])),
            (Console, k(&["Backquote", "F1"])),
            (Cheats, k(&["F2"])),
            (QuickSave, k(&["F5"])),
            (QuickLoad, k(&["F9"])),
            (Reload, k(&["R"])),
            (Roll, k(&["ControlLeft"])),
            (ZoomIn, k(&["Equal", "NumpadAdd"])),
            (ZoomOut, k(&["Minus", "NumpadSubtract"])),
            (Map, k(&["M"])),
            (Journal, k(&["J"])),
        ]);
        Self {
            fullscreen: false,
            vsync: true,
            zoom: 3.0,
            master_volume: 0.8,
            music_volume: 0.5,
            sfx_volume: 0.8,
            gamepad_deadzone: 0.2,
            text_scale: 1.0,
            hot_reload: true,
            colorblind: false,
            pad: PadMap::default(),
            keys,
        }
    }
}

impl Settings {
    pub fn load_or_create(dir: &Path) -> Self {
        let path = dir.join("settings.ron");
        if let Ok(s) = std::fs::read_to_string(&path) {
            match ron::from_str::<Settings>(&s) {
                Ok(mut st) => {
                    // fill in any actions added in newer versions
                    for (a, k) in Settings::default().keys {
                        st.keys.entry(a).or_insert(k);
                    }
                    return st;
                }
                Err(e) => eprintln!("settings.ron invalid ({e}); using defaults"),
            }
        }
        let st = Settings::default();
        let _ = std::fs::create_dir_all(dir);
        let _ = st.save(dir);
        st
    }
    pub fn save(&self, dir: &Path) -> Result<(), String> {
        let s = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("settings.ron"), s).map_err(|e| e.to_string())
    }
    pub fn rebind(&mut self, a: Action, keys: Vec<String>) {
        self.keys.insert(a, keys);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_cover_all_actions_and_roundtrip() {
        let s = Settings::default();
        assert!(s.keys.len() >= 19);
        let dir = std::env::temp_dir().join(format!("u67_settings_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut a = Settings::load_or_create(&dir);
        assert!(dir.join("settings.ron").exists());
        a.rebind(Action::Interact, vec!["F".into()]);
        a.save(&dir).unwrap();
        let b = Settings::load_or_create(&dir);
        assert_eq!(b.keys[&Action::Interact], vec!["F"]);
        std::fs::write(dir.join("settings.ron"), "garbage(").unwrap();
        assert_eq!(Settings::load_or_create(&dir).zoom, 3.0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
