//! Pure game state (no Bevy types): everything that gets saved.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use u67_console::Cheat;
use u67_core::{Dir, GameClock};
use u67_world::inventory::{Inventory, Item};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub str_: u32,
    pub dex: u32,
    pub int: u32,
    pub vaki: u32,
    pub level: u32,
    pub xp: u32,
    pub hp: i32,
    pub mana: i32,
}

pub fn xp_for_level(level: u32) -> u32 {
    let l = level.saturating_sub(1);
    100 * l * l
}
pub fn level_for_xp(xp: u32) -> u32 {
    let mut l = 1;
    while xp_for_level(l + 1) <= xp {
        l += 1;
    }
    l
}

impl Default for Stats {
    fn default() -> Self {
        let mut s = Self { str_: 15, dex: 15, int: 12, vaki: 10, level: 1, xp: 0, hp: 0, mana: 0 };
        s.hp = s.max_hp();
        s.mana = s.max_mana();
        s
    }
}

impl Stats {
    pub fn max_hp(&self) -> i32 {
        (30 + self.str_ * 2 + self.level * 5) as i32
    }
    pub fn max_mana(&self) -> i32 {
        (self.vaki * 2 + self.int + self.level * 2) as i32
    }
    /// Add XP, returning how many levels were gained.
    pub fn add_xp(&mut self, xp: u32) -> u32 {
        let before = self.level;
        self.xp += xp;
        self.level = level_for_xp(self.xp);
        for l in before + 1..=self.level {
            self.str_ += 1;
            self.dex += 1;
            if l % 2 == 0 {
                self.int += 1;
                self.vaki += 1;
            }
        }
        if self.level > before {
            self.hp = self.max_hp();
            self.mana = self.max_mana();
        }
        self.level - before
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerData {
    pub name: String,
    /// Position in tile units (feet position).
    pub pos: [f32; 2],
    pub facing: Dir,
    pub stats: Stats,
    pub inventory: Inventory,
    pub skills: BTreeMap<String, u32>,
    /// Rounds currently loaded per gun id.
    #[serde(default)]
    pub loaded: BTreeMap<String, u16>,
    #[serde(default)]
    pub spell_slots: Vec<String>,
}

impl PlayerData {
    pub fn new(name: &str, pos: [f32; 2]) -> Self {
        let stats = Stats::default();
        let mut inventory = Inventory::new(stats.str_);
        inventory.add("backpack", 1).ok();
        for (id, n) in [("rune_pistol", 1), ("ammo_9mm", 30), ("rye_bread", 3), ("torch", 1), ("silver", 25)] {
            inventory.add(id, n).ok();
        }
        if let Some(i) = inventory.pack.iter().position(|it| it.id == "rune_pistol") {
            inventory.equip(&[i]).ok();
        }
        let mut loaded = BTreeMap::new();
        loaded.insert("rune_pistol".to_string(), 12u16);
        Self { name: name.into(), pos, facing: Dir::S, stats, inventory, skills: BTreeMap::new(), loaded, spell_slots: vec!["parane".into(), "ukon_nuoli".into(), "valo".into()] }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CheatState {
    pub god: bool,
    pub infinite_ammo: bool,
    pub noclip: bool,
    pub fast: bool,
    pub invisible: bool,
    pub one_hit: bool,
    pub fps: bool,
    pub debug_colliders: bool,
    pub inspect: bool,
}

impl CheatState {
    pub fn get(&self, c: Cheat) -> bool {
        match c {
            Cheat::God => self.god,
            Cheat::InfiniteAmmo => self.infinite_ammo,
            Cheat::Noclip => self.noclip,
            Cheat::Fast => self.fast,
            Cheat::Invisible => self.invisible,
            Cheat::OneHit => self.one_hit,
            Cheat::Fps => self.fps,
            Cheat::DebugColliders => self.debug_colliders,
            Cheat::Inspect => self.inspect,
        }
    }
    pub fn set(&mut self, c: Cheat, v: bool) {
        match c {
            Cheat::God => self.god = v,
            Cheat::InfiniteAmmo => self.infinite_ammo = v,
            Cheat::Noclip => self.noclip = v,
            Cheat::Fast => self.fast = v,
            Cheat::Invisible => self.invisible = v,
            Cheat::OneHit => self.one_hit = v,
            Cheat::Fps => self.fps = v,
            Cheat::DebugColliders => self.debug_colliders = v,
            Cheat::Inspect => self.inspect = v,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestState {
    Active,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestProgress {
    pub state: QuestState,
    pub step: u32,
}

/// Persistent changes to a world object (door open, chest looted...). Key: "map:x:y:original_kind".
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ObjState {
    pub kind: Option<String>,
    pub frame: Option<u8>,
    pub contents: Option<Vec<Item>>,
    pub loaded: bool,
    pub removed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroundItem {
    pub map: String,
    pub pos: [f32; 2],
    pub item: Item,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vehicle {
    pub kind: String,
    pub pos: [f32; 2],
    pub facing: Dir,
    pub hull: i32,
}

pub fn obj_key(map: &str, x: i32, y: i32, kind: &str) -> String {
    format!("{map}:{x}:{y}:{kind}")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameData {
    pub seed: u64,
    pub current_map: String,
    pub clock: GameClock,
    pub players: Vec<PlayerData>,
    pub flags: BTreeSet<String>,
    pub quests: BTreeMap<String, QuestProgress>,
    pub counters: BTreeMap<String, u32>,
    pub explored: BTreeSet<String>,
    pub obj_state: BTreeMap<String, ObjState>,
    pub tile_edits: Vec<(String, i32, i32, u16)>,
    pub ground: Vec<GroundItem>,
    pub spells_known: BTreeSet<String>,
    pub vehicle: Option<Vehicle>,
    /// Objects the player placed (docked ships, dropped cannons): (map, x, y, kind).
    #[serde(default)]
    pub placed: Vec<(String, i32, i32, String)>,
    pub cheats: CheatState,
    pub weather: String,
    pub map_revealed: bool,
    pub travel_unlocked: BTreeSet<String>,
    pub party: Vec<String>,
    pub spells_all: bool,
    pub cvars: BTreeMap<String, String>,
    /// Set when the active map or position was changed by game logic; the renderer rebuilds then clears it.
    #[serde(skip)]
    pub map_dirty: bool,
}

impl GameData {
    pub fn new(seed: u64, start_map: &str, start: [f32; 2]) -> Self {
        let mut d = Self {
            seed,
            current_map: start_map.into(),
            clock: GameClock::default(),
            players: vec![PlayerData::new("Rune-Warden", start)],
            flags: BTreeSet::new(),
            quests: BTreeMap::new(),
            counters: BTreeMap::new(),
            explored: BTreeSet::new(),
            obj_state: BTreeMap::new(),
            tile_edits: vec![],
            ground: vec![],
            spells_known: BTreeSet::new(),
            vehicle: None,
            placed: vec![],
            cheats: CheatState::default(),
            weather: "clear".into(),
            map_revealed: false,
            travel_unlocked: BTreeSet::new(),
            party: vec![],
            spells_all: false,
            cvars: BTreeMap::new(),
            map_dirty: true,
        };
        d.flags.insert("game_started".into());
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn levels() {
        assert_eq!(level_for_xp(0), 1);
        assert_eq!(level_for_xp(100), 2);
        assert_eq!(level_for_xp(399), 2);
        assert_eq!(level_for_xp(400), 3);
        let mut s = Stats::default();
        s.hp = 1;
        assert_eq!(s.add_xp(450), 2);
        assert_eq!(s.hp, s.max_hp());
    }
    #[test]
    fn new_player_has_kit() {
        let p = PlayerData::new("x", [1.0, 2.0]);
        assert_eq!(p.inventory.count("ammo_9mm"), 30);
    }
}
