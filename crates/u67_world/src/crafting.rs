//! Crafting recipes (forge, campfire, workbench, alchemy table, Sampo forge).
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    /// "forge" | "campfire" | "workbench" | "alchemy" | "sampo_forge"
    pub station: String,
    pub inputs: Vec<(String, u32)>,
    pub output: (String, u32),
    #[serde(default)]
    pub level: u32,
    /// optional: needs this flag (e.g. learned plan)
    #[serde(default)]
    pub needs_flag: Option<String>,
}

/// Which object kinds act as which crafting station.
pub fn station_of(object_kind: &str) -> Option<&'static str> {
    match object_kind {
        "forge" => Some("forge"),
        "campfire" => Some("campfire"),
        "table" => Some("workbench"),
        _ => None,
    }
}
