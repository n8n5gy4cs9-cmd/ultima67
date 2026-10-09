//! Asset manifest + atlas index files. The game loads assets by these ids/paths.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub kind: String,
    pub path: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub source: String,
    pub status: String,
    pub notes: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub entries: Vec<Entry>,
}

impl Manifest {
    pub fn add(&mut self, id: &str, kind: &str, path: &str, use_: &str, notes: &str) {
        self.entries.push(Entry {
            id: id.into(),
            kind: kind.into(),
            path: path.into(),
            use_: use_.into(),
            source: "generated".into(),
            status: "placeholder".into(),
            notes: notes.into(),
        });
    }
    pub fn save(&self, root: &Path) -> Result<(), String> {
        let s = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(root.join("manifest.json"), s + "\n").map_err(|e| e.to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AtlasEntry {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Names -> pixel rectangles inside a sheet image.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Atlas {
    pub image: String,
    pub entries: Vec<AtlasEntry>,
}

impl Atlas {
    pub fn new(image: &str) -> Self {
        Self { image: image.into(), entries: vec![] }
    }
    pub fn add(&mut self, name: &str, x: i32, y: i32, w: i32, h: i32) {
        self.entries.push(AtlasEntry { name: name.into(), x, y, w, h });
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let s = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, s + "\n").map_err(|e| e.to_string())
    }
}

pub fn path_in(root: &Path, rel: &str) -> PathBuf {
    root.join(rel)
}
