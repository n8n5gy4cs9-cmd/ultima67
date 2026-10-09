//! NPC, shop and creature definitions (data tables loaded from JSON).
use crate::schedule::Activity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcDef {
    pub id: String,
    pub name: String,
    /// character sprite archetype
    pub arch: String,
    pub town: String,
    /// place names (see Map::places). `{town}` is replaced by the town id.
    pub home: String,
    pub work: String,
    #[serde(default)]
    pub tavern: Option<String>,
    #[serde(default)]
    pub work_activity: Option<Activity>,
    #[serde(default)]
    pub dialogue: Option<String>,
    #[serde(default)]
    pub shop: Option<String>,
    #[serde(default)]
    pub recruitable: bool,
    #[serde(default)]
    pub map: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockEntry {
    pub item: String,
    #[serde(default)]
    pub price: Option<u32>,
    #[serde(default = "default_qty")]
    pub qty: u32,
}
fn default_qty() -> u32 {
    99
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShopDef {
    pub id: String,
    pub name: String,
    pub stock: Vec<StockEntry>,
    /// Fraction of value paid when the player sells.
    #[serde(default = "default_rate")]
    pub buy_rate: f32,
    #[serde(default)]
    pub currency: Option<String>,
}
fn default_rate() -> f32 {
    0.5
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ai {
    Melee,
    Ranged,
    Boss,
    Passive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureDef {
    pub id: String,
    pub name: String,
    pub arch: String,
    pub hp: i32,
    pub speed: f32,
    pub damage: i32,
    pub range: f32,
    pub sight: f32,
    pub xp: u32,
    pub ai: Ai,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default)]
    pub loot: Option<String>,
    /// how often it attacks (seconds)
    #[serde(default = "default_cd")]
    pub cooldown: f32,
    #[serde(default)]
    pub flies: bool,
    #[serde(default)]
    pub swims: bool,
}
fn default_scale() -> f32 {
    1.0
}
fn default_cd() -> f32 {
    1.0
}

/// Where creatures live. Spawned when the player comes near, re-rolled over time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnDef {
    pub map: String,
    /// Map place name, or "*" for wilderness spawning around the player.
    pub place: String,
    pub creature: String,
    #[serde(default = "default_count")]
    pub count: u32,
    #[serde(default = "default_radius")]
    pub radius: f32,
    /// If true, the creature is a unique boss that never respawns once killed (flag `dead_<id>@<place>`).
    #[serde(default)]
    pub unique: bool,
    /// For "*": only on these tile names, e.g. ["snow","ice"].
    #[serde(default)]
    pub on: Vec<String>,
    /// For "*": only at night / day.
    #[serde(default)]
    pub night_only: bool,
}
fn default_count() -> u32 {
    1
}
fn default_radius() -> f32 {
    6.0
}
