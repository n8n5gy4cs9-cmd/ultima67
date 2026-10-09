//! Applies console [`Action`]s to [`GameData`]. Pure and unit-tested; effects that need the
//! engine (spawning, screenshots...) are returned as [`Effect`]s.
use crate::data::*;
use u67_console::{Action, QuestOp, Stat};
use u67_core::TilePos;
use u67_world::items::{self, Kind};
use u67_world::map::World;

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    Spawn { id: String, count: u32 },
    SpawnShip(String),
    SpawnCannon,
    KillAll,
    Screenshot,
    Quit,
    ReloadAssets,
    ReloadData,
    Splitscreen(u8),
    Save(Option<u8>),
    Load(Option<u8>),
    Weather(String),
    Exec(String),
    Cast(String),
    OpenShop(String),
    OpenCraft(String),
    Sleep,
    Sfx(String),
    Toast(String),
    Teleported,
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub lines: Vec<String>,
    pub effects: Vec<Effect>,
}

impl Outcome {
    fn say(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
}

pub const SPAWNABLE: &[&str] = &[
    "wolf", "ice_wolf", "fenrir", "draugr", "troll", "jotun", "merc", "louhi", "villager_m", "villager_f", "valkyrie", "dwarf", "singer", "volva", "berserker", "skald", "jarl", "warden_guard",
];

fn list_wrapped(items: &[String], per_line: usize) -> Vec<String> {
    items.chunks(per_line).map(|c| c.join("  ")).collect()
}

pub fn apply(d: &mut GameData, world: &World, a: Action) -> Outcome {
    let mut o = Outcome::default();
    let cur_map = d.current_map.clone();
    macro_rules! p {
        () => {
            d.players[0]
        };
    }
    match a {
        Action::Give { item, count } => match p!().inventory.add(&item, count) {
            Ok(()) => o.say(format!("gave {count} x {item}")),
            Err(e) => o.say(format!("cannot give {item}: {e:?}")),
        },
        Action::GiveAllWeapons | Action::GiveAllItems => {
            let all = matches!(a, Action::GiveAllItems);
            let saved = p!().inventory.max_weight;
            p!().inventory.max_weight = 1e9;
            let mut n = 0;
            for it in items::ITEMS {
                if all || matches!(it.kind, Kind::Melee | Kind::Gun) {
                    p!().inventory.add(it.id, if it.stackable { 20 } else { 1 }).ok();
                    n += 1;
                }
            }
            p!().inventory.max_weight = saved;
            o.say(format!("added {n} kinds of items (overweight is fine, cheaters!)"));
        }
        Action::Take { item, count } => {
            let n = p!().inventory.remove(&item, count);
            o.say(format!("removed {n} x {item}"));
        }
        Action::DropAll => {
            let n = p!().inventory.pack.len();
            p!().inventory.pack.clear();
            o.say(format!("dropped {n} stacks (they vanish for now)"));
        }
        Action::Ammo(c) => {
            let ammo = p!().inventory.equipped.get(&items::Slot::HandR).and_then(|i| items::get(&i.id)).and_then(|d| d.ammo);
            match ammo {
                Some(id) => {
                    p!().inventory.add(id, c.unwrap_or(50)).ok();
                    o.say(format!("+{} {id}", c.unwrap_or(50)));
                }
                None => o.say("no gun equipped in the right hand"),
            }
        }
        Action::Money(n) => {
            if n >= 0 {
                p!().inventory.add("silver", n as u32).ok();
            } else {
                p!().inventory.remove("silver", (-n) as u32);
            }
            let have = p!().inventory.count("silver");
            o.say(format!("silver: {have}"));
        }
        Action::Items(f) => {
            let ids: Vec<String> = items::ITEMS.iter().filter(|i| f.as_ref().is_none_or(|f| i.id.contains(f.as_str()))).map(|i| i.id.to_string()).collect();
            o.lines.extend(list_wrapped(&ids, 4));
            o.say(format!("{} items", ids.len()));
        }
        Action::Toggle(c, v) => {
            let nv = v.unwrap_or(!d.cheats.get(c));
            d.cheats.set(c, nv);
            o.say(format!("{}: {}", c.command(), if nv { "on" } else { "off" }));
        }
        Action::Heal(n) => {
            let m = p!().stats.max_hp();
            p!().stats.hp = n.map_or(m, |n| (p!().stats.hp + n as i32).min(m));
            o.say(format!("hp {}/{}", p!().stats.hp, m));
        }
        Action::Mana(n) => {
            let m = p!().stats.max_mana();
            p!().stats.mana = n.map_or(m, |n| (p!().stats.mana + n as i32).min(m));
            o.say(format!("mana {}/{}", p!().stats.mana, m));
        }
        Action::Xp(n) => {
            let g = p!().stats.add_xp(n);
            o.say(format!("xp {} (level {}{})", p!().stats.xp, p!().stats.level, if g > 0 { ", LEVEL UP!" } else { "" }));
        }
        Action::Level(l) => {
            let l = l.clamp(1, 99);
            p!().stats.level = l;
            p!().stats.xp = xp_for_level(l);
            let (h, m) = (p!().stats.max_hp(), p!().stats.max_mana());
            p!().stats.hp = h;
            p!().stats.mana = m;
            o.say(format!("level {l}"));
        }
        Action::SetStat(s, v) => {
            match s {
                Stat::Str => p!().stats.str_ = v,
                Stat::Dex => p!().stats.dex = v,
                Stat::Int => p!().stats.int = v,
                Stat::Vaki => p!().stats.vaki = v,
            }
            p!().inventory.max_weight = p!().stats.str_ as f32 * 2.0 + 20.0;
            o.say(format!("{s:?} = {v}"));
        }
        Action::Skill { name, value } => {
            p!().skills.insert(name.clone(), value);
            o.say(format!("skill {name} = {value}"));
        }
        Action::KillAll => o.effects.push(Effect::KillAll),
        Action::Tp(place) => {
            let here = world.maps.get(&cur_map).and_then(|m| m.places.get(&place).copied());
            let found = here.map(|p| (cur_map.clone(), p)).or_else(|| world.find_place(&place).map(|(m, p)| (m.to_string(), p)));
            match found {
                Some((map, pos)) => {
                    d.current_map = map.clone();
                    p!().pos = [pos.x as f32 + 0.5, pos.y as f32 + 0.5];
                    d.map_dirty = true;
                    o.say(format!("teleported to {place} ({map} {},{})", pos.x, pos.y));
                }
                None => {
                    let near: Vec<_> = world.maps.values().flat_map(|m| m.places.keys()).filter(|k| k.contains(&place)).take(6).cloned().collect();
                    o.say(if near.is_empty() { format!("no place '{place}' (try: places)") } else { format!("no place '{place}'. Did you mean: {}", near.join(", ")) });
                }
            }
        }
        Action::TpXy { x, y } => match world.maps.get(&cur_map) {
            Some(m) if m.in_bounds(TilePos::new(x, y)) => {
                p!().pos = [x as f32 + 0.5, y as f32 + 0.5];
                d.map_dirty = true;
                o.say(format!("at {x},{y}"));
            }
            _ => o.say("out of bounds"),
        },
        Action::Pos => o.say(format!("{} {:.1},{:.1} t={:02}:{:02} day {}", d.current_map, p!().pos[0], p!().pos[1], d.clock.hour(), d.clock.minute(), d.clock.day() + 1)),
        Action::Planet(id) => match world.maps.get(&id) {
            Some(m) if u67_mapgen::planets::BODIES.iter().any(|b| b.id == id) || id == "midgard" => {
                let land = m.places.get("landing").or_else(|| m.places.get("start")).copied().unwrap_or(TilePos::new(m.width / 2, m.height / 2));
                d.current_map = id.clone();
                p!().pos = [land.x as f32 + 0.5, land.y as f32 + 0.5];
                d.map_dirty = true;
                o.say(format!("arrived at {id}"));
            }
            _ => o.say(format!("unknown body '{id}' (try: planets)")),
        },
        Action::Planets => {
            o.say("midgard (Sol system home)");
            for b in u67_mapgen::planets::BODIES {
                o.say(format!("{:<14} {:<10} {}", b.id, b.system, b.name));
            }
        }
        Action::Places(f) => {
            let mut v: Vec<String> = world.maps.get(&cur_map).map(|m| m.places.keys().filter(|k| f.as_ref().is_none_or(|f| k.contains(f.as_str()))).cloned().collect()).unwrap_or_default();
            let total = v.len();
            v.truncate(48);
            o.lines.extend(list_wrapped(&v, 4));
            o.say(format!("{total} places on {cur_map}"));
        }
        Action::RevealMap => {
            d.map_revealed = true;
            o.say("map revealed");
        }
        Action::UnlockTravel => {
            for b in u67_mapgen::planets::BODIES {
                d.travel_unlocked.insert(b.id.to_string());
            }
            o.say("all fast-travel nodes unlocked");
        }
        Action::Time { hour, minute } => {
            d.clock.set_hm(hour, minute);
            o.say(format!("time {hour:02}:{minute:02}"));
        }
        Action::Timescale(x) => {
            d.clock.scale = 20.0 * x;
            o.say(format!("timescale x{x}"));
        }
        Action::SetWeather(w) => {
            d.weather = format!("{w:?}").to_lowercase();
            o.effects.push(Effect::Weather(d.weather.clone()));
            o.say(format!("weather: {}", d.weather));
        }
        Action::Spawn { id, count } => {
            if SPAWNABLE.contains(&id.as_str()) {
                o.effects.push(Effect::Spawn { id, count });
            } else {
                o.say(format!("unknown creature '{id}' (try: creatures)"));
            }
        }
        Action::SpawnShip(id) => o.effects.push(Effect::SpawnShip(id)),
        Action::SpawnCannon => o.effects.push(Effect::SpawnCannon),
        Action::Creatures(f) => {
            let v: Vec<String> = SPAWNABLE.iter().filter(|s| f.as_ref().is_none_or(|f| s.contains(f.as_str()))).map(|s| s.to_string()).collect();
            o.lines.extend(list_wrapped(&v, 5));
        }
        Action::SpellsAll => {
            d.spells_all = true;
            d.spells_known.insert("*".into());
            o.say("all spells learned");
        }
        Action::Reagents => {
            for it in items::ITEMS.iter().filter(|i| i.kind == Kind::Reagent) {
                p!().inventory.add(it.id, 20).ok();
            }
            o.say("reagents filled");
        }
        Action::Cast(s) => o.effects.push(Effect::Cast(s)),
        Action::Quest(op, id) => match op {
            QuestOp::List => {
                if d.quests.is_empty() {
                    o.say("no quests");
                }
                for (k, v) in &d.quests {
                    o.say(format!("{k}: {:?} (step {})", v.state, v.step));
                }
            }
            QuestOp::Start => {
                let id = id.unwrap_or_default();
                d.quests.insert(id.clone(), QuestProgress { state: QuestState::Active, step: 0 });
                o.say(format!("quest started: {id}"));
            }
            QuestOp::Complete => {
                let id = id.unwrap_or_default();
                let step = d.quests.get(&id).map_or(0, |q| q.step);
                d.quests.insert(id.clone(), QuestProgress { state: QuestState::Done, step });
                o.say(format!("quest completed: {id}"));
            }
            QuestOp::Reset => {
                let id = id.unwrap_or_default();
                d.quests.remove(&id);
                o.say(format!("quest reset: {id}"));
            }
        },
        Action::Flag { name, value } => match value {
            None => o.say(format!("{name} = {}", d.flags.contains(&name) as u8)),
            Some(true) => {
                d.flags.insert(name.clone());
                o.say(format!("{name} = 1"));
            }
            Some(false) => {
                d.flags.remove(&name);
                o.say(format!("{name} = 0"));
            }
        },
        Action::Party { add, npc } => {
            if add && !d.party.contains(&npc) {
                d.party.push(npc.clone());
            } else if !add {
                d.party.retain(|n| *n != npc);
            }
            o.say(format!("party: {}", if d.party.is_empty() { "(empty)".into() } else { d.party.join(", ") }));
        }
        Action::Ending(c) => {
            d.flags.insert(format!("ending_{c}"));
            o.say(format!("ending {c} flagged"));
        }
        Action::Save(s) => o.effects.push(Effect::Save(s)),
        Action::Load(s) => o.effects.push(Effect::Load(s)),
        Action::Quit => o.effects.push(Effect::Quit),
        Action::ReloadAssets => o.effects.push(Effect::ReloadAssets),
        Action::ReloadData => o.effects.push(Effect::ReloadData),
        Action::Screenshot => o.effects.push(Effect::Screenshot),
        Action::Seed(n) => {
            d.seed = n;
            o.say(format!("seed {n} (applies to new games)"));
        }
        Action::Splitscreen(n) => o.effects.push(Effect::Splitscreen(n)),
        Action::SetVar { name, value } => {
            d.cvars.insert(name.clone(), value.clone());
            o.say(format!("{name} = {value}"));
        }
        Action::GetVar(n) => o.say(format!("{n} = {}", d.cvars.get(&n).map_or("(unset)", |s| s.as_str()))),
        Action::Exec(f) => o.effects.push(Effect::Exec(f)),
        Action::Help(_) | Action::Cmds | Action::Clear | Action::Echo(_) => {}
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use u67_console::{Cheat, Registry};

    fn setup() -> (GameData, World) {
        let w = u67_mapgen::generate_world(67);
        let s = w.maps["midgard"].places["start"];
        (GameData::new(67, "midgard", [s.x as f32, s.y as f32]), w)
    }
    fn run(d: &mut GameData, w: &World, line: &str) -> Outcome {
        let a = Registry::standard().parse(line).unwrap().unwrap();
        apply(d, w, a)
    }

    #[test]
    fn give_vainamoinen_gun_and_equip() {
        let (mut d, w) = setup();
        run(&mut d, &w, "give vainamoinen_gun");
        assert_eq!(d.players[0].inventory.count("vainamoinen_gun"), 1);
    }
    #[test]
    fn god_toggle_and_levels() {
        let (mut d, w) = setup();
        run(&mut d, &w, "god");
        assert!(d.cheats.god);
        run(&mut d, &w, "god off");
        assert!(!d.cheats.god);
        run(&mut d, &w, "level 5");
        assert_eq!(d.players[0].stats.level, 5);
        run(&mut d, &w, "xp 5000");
        assert!(d.players[0].stats.level > 5);
    }
    #[test]
    fn teleports_across_maps() {
        let (mut d, w) = setup();
        run(&mut d, &w, "tp birka");
        assert_eq!(d.current_map, "midgard");
        run(&mut d, &w, "planet maani");
        assert_eq!(d.current_map, "maani");
        assert!(d.map_dirty);
        let o = run(&mut d, &w, "tp skoll_crater");
        assert!(o.lines[0].starts_with("teleported"));
        let o = run(&mut d, &w, "tp nowhere_xyz");
        assert!(o.lines[0].contains("no place"));
        let o = run(&mut d, &w, "tp brok");
        assert!(o.lines[0].contains("brokkr_forge"), "{:?}", o.lines);
    }
    #[test]
    fn flags_quests_party_money() {
        let (mut d, w) = setup();
        run(&mut d, &w, "flag met_skald 1");
        assert!(d.flags.contains("met_skald"));
        run(&mut d, &w, "quest start norns");
        assert_eq!(d.quests["norns"].state, QuestState::Active);
        run(&mut d, &w, "quest complete norns");
        assert_eq!(d.quests["norns"].state, QuestState::Done);
        run(&mut d, &w, "party add valkyrie");
        assert_eq!(d.party, vec!["valkyrie"]);
        run(&mut d, &w, "money 100");
        assert_eq!(d.players[0].inventory.count("silver"), 125);
        run(&mut d, &w, "time 23:30");
        assert!(d.clock.is_night());
    }
    #[test]
    fn effects_are_returned() {
        let (mut d, w) = setup();
        assert_eq!(run(&mut d, &w, "spawn wolf 3").effects, vec![Effect::Spawn { id: "wolf".into(), count: 3 }]);
        assert!(run(&mut d, &w, "spawn dragon").effects.is_empty());
        assert_eq!(run(&mut d, &w, "quit").effects, vec![Effect::Quit]);
        let _ = Cheat::God;
    }
    #[test]
    fn give_all_items_ignores_weight() {
        let (mut d, w) = setup();
        run(&mut d, &w, "give_all_items");
        assert!(d.players[0].inventory.count("bear_axe") >= 1);
        assert!(d.players[0].inventory.max_weight < 1000.0);
    }
}
