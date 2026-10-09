use crate::actions::*;
use u67_world::items;

pub struct Args<'a>(pub &'a [&'a str]);

impl<'a> Args<'a> {
    fn req(&self, i: usize, name: &str) -> Result<&'a str, String> {
        self.0.get(i).copied().ok_or_else(|| format!("missing <{name}>"))
    }
    fn opt(&self, i: usize) -> Option<&'a str> {
        self.0.get(i).copied()
    }
    fn num<T: std::str::FromStr>(&self, i: usize, name: &str) -> Result<T, String> {
        let s = self.req(i, name)?;
        s.parse().map_err(|_| format!("<{name}> must be a number, got '{s}'"))
    }
    fn opt_num<T: std::str::FromStr>(&self, i: usize, name: &str) -> Result<Option<T>, String> {
        match self.opt(i) {
            None => Ok(None),
            Some(s) => s.parse().map(Some).map_err(|_| format!("<{name}> must be a number, got '{s}'")),
        }
    }
    fn toggle(&self, i: usize) -> Result<Option<bool>, String> {
        match self.opt(i) {
            None => Ok(None),
            Some("on" | "1" | "true") => Ok(Some(true)),
            Some("off" | "0" | "false") => Ok(Some(false)),
            Some(o) => Err(format!("expected on|off, got '{o}'")),
        }
    }
    fn item(&self, i: usize) -> Result<String, String> {
        let id = self.req(i, "item_id")?;
        if items::get(id).is_some() {
            return Ok(id.into());
        }
        let near: Vec<_> = items::ITEMS.iter().filter(|d| d.id.contains(id)).take(5).map(|d| d.id).collect();
        Err(if near.is_empty() { format!("unknown item '{id}' (try: items)") } else { format!("unknown item '{id}'. Did you mean: {}", near.join(", ")) })
    }
}

type Parser = fn(&Args) -> Result<Action, String>;

pub struct Command {
    pub category: &'static str,
    pub name: &'static str,
    pub usage: &'static str,
    pub help: &'static str,
    pub parse: Parser,
}

pub struct Registry {
    pub commands: Vec<Command>,
}

fn c(category: &'static str, name: &'static str, usage: &'static str, help: &'static str, parse: Parser) -> Command {
    Command { category, name, usage, help, parse }
}

macro_rules! toggle_cmd {
    ($cat:expr, $name:expr, $help:expr, $cheat:expr) => {
        c($cat, $name, "[on|off]", $help, |a| Ok(Action::Toggle($cheat, a.toggle(0)?)))
    };
}

impl Registry {
    pub fn standard() -> Self {
        let mut v = vec![
            // --- console ---
            c("console", "help", "[command]", "list commands / show usage", |a| Ok(Action::Help(a.opt(0).map(Into::into)))),
            c("console", "cmds", "", "print all commands", |_| Ok(Action::Cmds)),
            c("console", "clear", "", "clear console output", |_| Ok(Action::Clear)),
            c("console", "echo", "<text>", "print text", |a| Ok(Action::Echo(a.0.join(" ")))),
            c("console", "exec", "<file>", "run a script of commands", |a| Ok(Action::Exec(a.req(0, "file")?.into()))),
            // --- items ---
            c("items", "give", "<item_id> [count]", "add item to backpack (give vainamoinen_gun)", |a| {
                Ok(Action::Give { item: a.item(0)?, count: a.opt_num(1, "count")?.unwrap_or(1) })
            }),
            c("items", "give_all_weapons", "", "add one of every weapon", |_| Ok(Action::GiveAllWeapons)),
            c("items", "give_all_items", "", "add one of every item", |_| Ok(Action::GiveAllItems)),
            c("items", "take", "<item_id> [count]", "remove item", |a| Ok(Action::Take { item: a.item(0)?, count: a.opt_num(1, "count")?.unwrap_or(1) })),
            c("items", "drop_all", "", "drop inventory on ground", |_| Ok(Action::DropAll)),
            c("items", "ammo", "[count]", "refill ammo for equipped gun", |a| Ok(Action::Ammo(a.opt_num(0, "count")?))),
            c("items", "money", "<amount>", "add silver", |a| Ok(Action::Money(a.num(0, "amount")?))),
            c("items", "items", "[filter]", "list item ids", |a| Ok(Action::Items(a.opt(0).map(Into::into)))),
            // --- player ---
            toggle_cmd!("player", "god", "invulnerable", Cheat::God),
            c("player", "heal", "[amount]", "heal player (full if omitted)", |a| Ok(Action::Heal(a.opt_num(0, "amount")?))),
            c("player", "mana", "[amount]", "restore mana (väki)", |a| Ok(Action::Mana(a.opt_num(0, "amount")?))),
            toggle_cmd!("player", "infinite_ammo", "never reload/consume ammo", Cheat::InfiniteAmmo),
            toggle_cmd!("player", "noclip", "walk through walls", Cheat::Noclip),
            toggle_cmd!("player", "fast", "3x move speed", Cheat::Fast),
            c("player", "xp", "<amount>", "add experience", |a| Ok(Action::Xp(a.num(0, "amount")?))),
            c("player", "level", "<n>", "set level", |a| Ok(Action::Level(a.num(0, "n")?))),
            c("player", "stat", "<str|dex|int|vaki> <n>", "set stat", |a| {
                let s = match a.req(0, "stat")? {
                    "str" => Stat::Str,
                    "dex" => Stat::Dex,
                    "int" => Stat::Int,
                    "vaki" => Stat::Vaki,
                    o => return Err(format!("unknown stat '{o}' (str|dex|int|vaki)")),
                };
                Ok(Action::SetStat(s, a.num(1, "n")?))
            }),
            c("player", "skill", "<name> <n>", "set skill", |a| Ok(Action::Skill { name: a.req(0, "name")?.into(), value: a.num(1, "n")? })),
            c("player", "kill_all", "", "kill all hostile NPCs nearby", |_| Ok(Action::KillAll)),
            toggle_cmd!("player", "invisible", "enemies ignore you", Cheat::Invisible),
            toggle_cmd!("player", "one_hit", "all attacks kill", Cheat::OneHit),
            // --- world ---
            c("world", "tp", "<place_id>", "teleport to named place (tp kaupang, tp mimir_well)", |a| Ok(Action::Tp(a.req(0, "place_id")?.into()))),
            c("world", "tpxy", "<x> <y>", "teleport to coordinates on current map", |a| Ok(Action::TpXy { x: a.num(0, "x")?, y: a.num(1, "y")? })),
            c("world", "pos", "", "print position", |_| Ok(Action::Pos)),
            c("world", "planet", "<body_id>", "travel to planet (planet maani, planet pohjola)", |a| Ok(Action::Planet(a.req(0, "body_id")?.into()))),
            c("world", "planets", "", "list bodies", |_| Ok(Action::Planets)),
            c("world", "places", "[filter]", "list teleport ids", |a| Ok(Action::Places(a.opt(0).map(Into::into)))),
            c("world", "reveal_map", "", "reveal full map", |_| Ok(Action::RevealMap)),
            c("world", "unlock_travel", "", "unlock all fast-travel nodes", |_| Ok(Action::UnlockTravel)),
            c("world", "time", "<HH:MM>", "set game time", |a| {
                let s = a.req(0, "HH:MM")?;
                let (h, m) = s.split_once(':').ok_or("time must look like 14:30")?;
                let (hour, minute): (u32, u32) = (h.parse().map_err(|_| "bad hour")?, m.parse().map_err(|_| "bad minute")?);
                if hour > 23 || minute > 59 {
                    return Err("time out of range".into());
                }
                Ok(Action::Time { hour, minute })
            }),
            c("world", "timescale", "<x>", "game time speed multiplier", |a| {
                let x: f32 = a.num(0, "x")?;
                if !(0.0..=1000.0).contains(&x) {
                    return Err("timescale must be 0..1000".into());
                }
                Ok(Action::Timescale(x))
            }),
            c("world", "weather", "<clear|rain|snow|storm|aurora>", "set weather", |a| {
                Ok(Action::SetWeather(match a.req(0, "weather")? {
                    "clear" => Weather::Clear,
                    "rain" => Weather::Rain,
                    "snow" => Weather::Snow,
                    "storm" => Weather::Storm,
                    "aurora" => Weather::Aurora,
                    o => return Err(format!("unknown weather '{o}'")),
                }))
            }),
            // --- spawn ---
            c("spawn", "spawn", "<npc_or_creature_id> [count]", "spawn at cursor", |a| {
                Ok(Action::Spawn { id: a.req(0, "id")?.into(), count: a.opt_num(1, "count")?.unwrap_or(1) })
            }),
            c("spawn", "spawn_ship", "<ship_id>", "spawn ship", |a| Ok(Action::SpawnShip(a.req(0, "ship_id")?.into()))),
            c("spawn", "spawn_cannon", "", "spawn a cannon at cursor", |_| Ok(Action::SpawnCannon)),
            c("spawn", "creatures", "[filter]", "list spawnable ids", |a| Ok(Action::Creatures(a.opt(0).map(Into::into)))),
            // --- magic ---
            c("magic", "spells_all", "", "learn all spells", |_| Ok(Action::SpellsAll)),
            c("magic", "reagents", "", "fill reagents", |_| Ok(Action::Reagents)),
            c("magic", "cast", "<spell_id>", "cast without cost", |a| Ok(Action::Cast(a.req(0, "spell_id")?.into()))),
            // --- quests ---
            c("quests", "quest", "<list|start|complete|reset> [id]", "list quests / change quest state", |a| {
                let op = match a.req(0, "op")? {
                    "list" => QuestOp::List,
                    "start" => QuestOp::Start,
                    "complete" => QuestOp::Complete,
                    "reset" => QuestOp::Reset,
                    o => return Err(format!("unknown quest op '{o}'")),
                };
                if op != QuestOp::List && a.opt(1).is_none() {
                    return Err("missing <id>".into());
                }
                Ok(Action::Quest(op, a.opt(1).map(Into::into)))
            }),
            c("quests", "flag", "<name> [0|1]", "get/set story flag", |a| {
                let value = match a.opt(1) {
                    None => None,
                    Some("1" | "on" | "true") => Some(true),
                    Some("0" | "off" | "false") => Some(false),
                    Some(o) => return Err(format!("flag value must be 0|1, got '{o}'")),
                };
                Ok(Action::Flag { name: a.req(0, "name")?.into(), value })
            }),
            c("quests", "party", "<add|remove> <npc_id>", "party management", |a| {
                let add = match a.req(0, "op")? {
                    "add" => true,
                    "remove" => false,
                    o => return Err(format!("expected add|remove, got '{o}'")),
                };
                Ok(Action::Party { add, npc: a.req(1, "npc_id")?.into() })
            }),
            c("quests", "ending", "<a|b|c>", "jump to ending (debug)", |a| match a.req(0, "ending")? {
                "a" => Ok(Action::Ending('a')),
                "b" => Ok(Action::Ending('b')),
                "c" => Ok(Action::Ending('c')),
                o => Err(format!("ending must be a|b|c, got '{o}'")),
            }),
            // --- game ---
            c("game", "save", "[slot]", "save", |a| Ok(Action::Save(a.opt_num(0, "slot")?))),
            c("game", "load", "[slot]", "load", |a| Ok(Action::Load(a.opt_num(0, "slot")?))),
            c("game", "quit", "", "quit game", |_| Ok(Action::Quit)),
            c("game", "splitscreen", "<players 1-4>", "set local players (debug)", |a| {
                let n: u8 = a.num(0, "players")?;
                if !(1..=4).contains(&n) {
                    return Err("players must be 1-4".into());
                }
                Ok(Action::Splitscreen(n))
            }),
            // --- dev ---
            toggle_cmd!("dev", "fps", "show fps", Cheat::Fps),
            toggle_cmd!("dev", "debug_colliders", "draw collision boxes", Cheat::DebugColliders),
            toggle_cmd!("dev", "inspect", "entity inspector under cursor", Cheat::Inspect),
            c("dev", "reload_assets", "", "hot-reload assets", |_| Ok(Action::ReloadAssets)),
            c("dev", "reload_data", "", "reload data files", |_| Ok(Action::ReloadData)),
            c("dev", "screenshot", "", "save screenshot", |_| Ok(Action::Screenshot)),
            c("dev", "seed", "<n>", "set rng seed", |a| Ok(Action::Seed(a.num(0, "n")?))),
            c("dev", "set", "<cvar> <value>", "set config variable", |a| {
                Ok(Action::SetVar { name: a.req(0, "cvar")?.into(), value: a.req(1, "value")?.into() })
            }),
            c("dev", "get", "<cvar>", "read config variable", |a| Ok(Action::GetVar(a.req(0, "cvar")?.into()))),
        ];
        v.shrink_to_fit();
        Registry { commands: v }
    }

    pub fn find(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.name == name)
    }

    /// Parse one input line. `#` starts a comment.
    pub fn parse(&self, line: &str) -> Result<Option<Action>, String> {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            return Ok(None);
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let cmd = self.find(&parts[0].to_lowercase()).ok_or_else(|| {
            let near: Vec<_> = self.commands.iter().filter(|c| c.name.starts_with(&parts[0][..1.min(parts[0].len())])).take(4).map(|c| c.name).collect();
            if near.is_empty() { format!("unknown command '{}' (try: help)", parts[0]) } else { format!("unknown command '{}'. Similar: {}", parts[0], near.join(", ")) }
        })?;
        (cmd.parse)(&Args(&parts[1..])).map(Some).map_err(|e| format!("{e}\nusage: {} {}", cmd.name, cmd.usage))
    }

    /// Tab completion for the last word of `input`.
    pub fn complete(&self, input: &str) -> Vec<String> {
        let words: Vec<&str> = input.split_whitespace().collect();
        let trailing = input.ends_with(' ');
        if words.is_empty() || (words.len() == 1 && !trailing) {
            let p = words.first().copied().unwrap_or("");
            return self.commands.iter().filter(|c| c.name.starts_with(p)).map(|c| c.name.to_string()).collect();
        }
        let prefix = if trailing { "" } else { words[words.len() - 1] };
        let argn = if trailing { words.len() - 1 } else { words.len() - 2 };
        match (words[0], argn) {
            ("give" | "take", 0) => items::ITEMS.iter().filter(|d| d.id.starts_with(prefix)).map(|d| d.id.to_string()).collect(),
            ("god" | "noclip" | "fast" | "invisible" | "one_hit" | "infinite_ammo" | "fps" | "debug_colliders" | "inspect", 0) => {
                ["on", "off"].iter().filter(|s| s.starts_with(prefix)).map(|s| s.to_string()).collect()
            }
            ("weather", 0) => ["clear", "rain", "snow", "storm", "aurora"].iter().filter(|s| s.starts_with(prefix)).map(|s| s.to_string()).collect(),
            ("help", 0) => self.commands.iter().filter(|c| c.name.starts_with(prefix)).map(|c| c.name.to_string()).collect(),
            _ => vec![],
        }
    }

    /// Text of `commands.txt` (generated; a test keeps the file in sync).
    pub fn render_commands_txt(&self) -> String {
        let mut s = String::from(
            "# Ultima67 console commands. GENERATED from the registry:\n#   cargo run -q -p u67_console --bin gen_commands > commands.txt\n# Open the console with ` or F1; cheats menu with F2. Toggles: \"cmd [on|off]\".\n# Format: command <required> [optional] - description\n",
        );
        let mut last = "";
        for c in &self.commands {
            if c.category != last {
                s += &format!("\n# --- {} ---\n", c.category);
                last = c.category;
            }
            let left = if c.usage.is_empty() { c.name.to_string() } else { format!("{} {}", c.name, c.usage) };
            s += &format!("{left:<34}- {}\n", c.help);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(l: &str) -> Result<Option<Action>, String> {
        Registry::standard().parse(l)
    }
    #[test]
    fn give_vainamoinen_gun() {
        assert_eq!(p("give vainamoinen_gun").unwrap(), Some(Action::Give { item: "vainamoinen_gun".into(), count: 1 }));
        assert_eq!(p("give ammo_9mm 50").unwrap(), Some(Action::Give { item: "ammo_9mm".into(), count: 50 }));
    }
    #[test]
    fn errors_are_helpful() {
        assert!(p("give vainamoinen").unwrap_err().contains("vainamoinen_gun"));
        assert!(p("give").unwrap_err().contains("usage: give"));
        assert!(p("gdo").unwrap_err().contains("unknown command"));
        assert!(p("time 25:00").is_err());
        assert!(p("splitscreen 9").is_err());
    }
    #[test]
    fn toggles_and_comments() {
        assert_eq!(p("god on").unwrap(), Some(Action::Toggle(Cheat::God, Some(true))));
        assert_eq!(p("god").unwrap(), Some(Action::Toggle(Cheat::God, None)));
        assert_eq!(p("  # nothing").unwrap(), None);
        assert_eq!(p("GOD off # hi").unwrap(), Some(Action::Toggle(Cheat::God, Some(false))));
    }
    #[test]
    fn completion() {
        let r = Registry::standard();
        assert!(r.complete("gi").contains(&"give".to_string()));
        assert!(r.complete("give vain").contains(&"vainamoinen_gun".to_string()));
        assert_eq!(r.complete("weather s"), vec!["snow", "storm"]);
    }
    #[test]
    fn every_cheat_has_a_command() {
        let r = Registry::standard();
        for c in Cheat::ALL {
            assert!(r.find(c.command()).is_some(), "{c:?}");
            assert_eq!(r.parse(c.command()).unwrap(), Some(Action::Toggle(c, None)));
        }
    }
    #[test]
    fn commands_txt_in_sync() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../commands.txt");
        let file = std::fs::read_to_string(path).unwrap_or_default();
        assert_eq!(file, Registry::standard().render_commands_txt(), "run: cargo run -q -p u67_console --bin gen_commands > commands.txt");
    }
}
