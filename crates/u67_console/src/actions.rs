//! The result of parsing a command. Applied by the game (`u67_game`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cheat {
    God,
    InfiniteAmmo,
    Noclip,
    Fast,
    Invisible,
    OneHit,
    Fps,
    DebugColliders,
    Inspect,
}

impl Cheat {
    pub const ALL: [Cheat; 9] =
        [Cheat::God, Cheat::InfiniteAmmo, Cheat::Noclip, Cheat::Fast, Cheat::Invisible, Cheat::OneHit, Cheat::Fps, Cheat::DebugColliders, Cheat::Inspect];
    pub fn command(self) -> &'static str {
        match self {
            Cheat::God => "god",
            Cheat::InfiniteAmmo => "infinite_ammo",
            Cheat::Noclip => "noclip",
            Cheat::Fast => "fast",
            Cheat::Invisible => "invisible",
            Cheat::OneHit => "one_hit",
            Cheat::Fps => "fps",
            Cheat::DebugColliders => "debug_colliders",
            Cheat::Inspect => "inspect",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Cheat::God => "God mode",
            Cheat::InfiniteAmmo => "Infinite ammo",
            Cheat::Noclip => "No clip",
            Cheat::Fast => "Super speed",
            Cheat::Invisible => "Invisible",
            Cheat::OneHit => "One-hit kills",
            Cheat::Fps => "Show FPS",
            Cheat::DebugColliders => "Show colliders",
            Cheat::Inspect => "Entity inspector",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    Str,
    Dex,
    Int,
    Vaki,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Rain,
    Snow,
    Storm,
    Aurora,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestOp {
    List,
    Start,
    Complete,
    Reset,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    // console meta
    Help(Option<String>),
    Cmds,
    Clear,
    Echo(String),
    Exec(String),
    // items
    Give { item: String, count: u32 },
    GiveAllWeapons,
    GiveAllItems,
    Take { item: String, count: u32 },
    DropAll,
    Ammo(Option<u32>),
    Money(i64),
    Items(Option<String>),
    // player
    Toggle(Cheat, Option<bool>),
    Heal(Option<u32>),
    Mana(Option<u32>),
    Xp(u32),
    Level(u32),
    SetStat(Stat, u32),
    Skill { name: String, value: u32 },
    KillAll,
    // world
    Tp(String),
    TpXy { x: i32, y: i32 },
    Pos,
    Planet(String),
    Planets,
    Places(Option<String>),
    RevealMap,
    UnlockTravel,
    Time { hour: u32, minute: u32 },
    Timescale(f32),
    SetWeather(Weather),
    // spawn
    Spawn { id: String, count: u32 },
    SpawnShip(String),
    SpawnCannon,
    Creatures(Option<String>),
    // magic
    SpellsAll,
    Reagents,
    Cast(String),
    // quests / flags
    Quest(QuestOp, Option<String>),
    Flag { name: String, value: Option<bool> },
    Party { add: bool, npc: String },
    Ending(char),
    // game / dev
    Save(Option<u8>),
    Load(Option<u8>),
    Quit,
    ReloadAssets,
    ReloadData,
    Screenshot,
    Seed(u64),
    Splitscreen(u8),
    SetVar { name: String, value: String },
    GetVar(String),
}
