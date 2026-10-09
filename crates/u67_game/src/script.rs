//! Executes JSON scripts (`u67_world::script`) against the game state, evaluates conditions,
//! and advances quests. Pure (no Bevy): engine-side things come back as [`Effect`]s.
use crate::apply::Effect;
use crate::data::*;
use u67_world::db::Db;
use u67_world::dialogue::CondCtx;
use u67_world::items;
use u67_world::map::World;
use u67_world::script::{Cond, Step};

pub struct Ctx<'a> {
    pub d: &'a GameData,
}

impl CondCtx for Ctx<'_> {
    fn check(&self, c: &Cond) -> bool {
        check(self.d, c)
    }
}

pub fn check(d: &GameData, c: &Cond) -> bool {
    match c {
        Cond::Flag { flag } => d.flags.contains(flag),
        Cond::NotFlag { flag } => !d.flags.contains(flag),
        Cond::HasItem { item, count } => d.players[0].inventory.count(item) >= *count,
        Cond::Counter { counter, min } => d.counters.get(counter).copied().unwrap_or(0) >= *min,
        Cond::Quest { quest, state } => {
            let s = d.quests.get(quest).map(|q| q.state);
            matches!((state.as_str(), s), ("none", None) | ("active", Some(QuestState::Active)) | ("done", Some(QuestState::Done)))
        }
        Cond::MinLevel { level } => d.players[0].stats.level >= *level,
        Cond::InMap { map } => &d.current_map == map,
        Cond::Night => d.clock.is_night(),
        Cond::Day => !d.clock.is_night(),
        Cond::InParty { npc } => d.party.contains(npc),
        Cond::And { all } => all.iter().all(|c| check(d, c)),
        Cond::Or { any } => any.iter().any(|c| check(d, c)),
        Cond::Not { cond } => !check(d, cond),
    }
}

#[derive(Debug, Default)]
pub struct Out {
    pub lines: Vec<String>,
    pub toasts: Vec<String>,
    pub effects: Vec<Effect>,
}

pub const MAX_PARTY: usize = 5;

/// Give an item ignoring the weight limit (quest rewards must never be lost).
fn give_forced(d: &mut GameData, item: &str, count: u32) {
    let inv = &mut d.players[0].inventory;
    let saved = inv.max_weight;
    inv.max_weight = 1e9;
    inv.add(item, count).ok();
    inv.max_weight = saved;
}

pub fn run(d: &mut GameData, db: &Db, world: &World, steps: &[Step]) -> Out {
    let mut o = Out::default();
    run_into(d, db, world, steps, &mut o);
    o
}

fn run_into(d: &mut GameData, db: &Db, world: &World, steps: &[Step], o: &mut Out) {
    for s in steps {
        match s {
            Step::Say { text } => o.lines.push(text.clone()),
            Step::Toast { text } => o.toasts.push(text.clone()),
            Step::SetFlag { flag } => {
                d.flags.insert(flag.clone());
            }
            Step::ClearFlag { flag } => {
                d.flags.remove(flag);
            }
            Step::Inc { counter } => *d.counters.entry(counter.clone()).or_insert(0) += 1,
            Step::Give { item, count } => {
                give_forced(d, item, *count);
                let name = items::get(item).map_or(item.as_str(), |i| i.name);
                o.toasts.push(format!("Received {count} x {name}"));
            }
            Step::Take { item, count } => {
                d.players[0].inventory.remove(item, *count);
            }
            Step::Money { amount } => {
                if *amount >= 0 {
                    give_forced(d, "silver", *amount as u32);
                    o.toasts.push(format!("+{amount} silver"));
                } else {
                    d.players[0].inventory.remove("silver", (-amount) as u32);
                }
            }
            Step::Xp { amount } => {
                let g = d.players[0].stats.add_xp(*amount);
                o.toasts.push(format!("+{amount} XP{}", if g > 0 { format!(" - LEVEL UP! (Lv {})", d.players[0].stats.level) } else { String::new() }));
                if g > 0 {
                    o.effects.push(Effect::Sfx("level_up".into()));
                }
            }
            Step::Heal => {
                let (h, m) = (d.players[0].stats.max_hp(), d.players[0].stats.max_mana());
                d.players[0].stats.hp = h;
                d.players[0].stats.mana = m;
            }
            Step::StartQuest { quest } => {
                if !d.quests.contains_key(quest) {
                    d.quests.insert(quest.clone(), QuestProgress { state: QuestState::Active, step: 0 });
                    o.toasts.push(format!("New quest: {}", db.quest(quest).map_or(quest.as_str(), |q| q.title.as_str())));
                    o.effects.push(Effect::Sfx("quest_start".into()));
                }
            }
            Step::CompleteQuest { quest } => complete_quest(d, db, world, quest, o),
            Step::JoinParty { npc } => {
                if d.party.len() < MAX_PARTY && !d.party.contains(npc) {
                    d.party.push(npc.clone());
                    o.toasts.push(format!("{} joins your party", db.npcs.iter().find(|n| &n.id == npc).map_or(npc.as_str(), |n| n.name.as_str())));
                } else if d.party.len() >= MAX_PARTY {
                    o.lines.push("(Your party is full.)".into());
                }
            }
            Step::LeaveParty { npc } => d.party.retain(|n| n != npc),
            Step::Teleport { place } => {
                if let Some((m, p)) = world.find_place(place) {
                    d.current_map = m.to_string();
                    d.players[0].pos = [p.x as f32 + 0.5, p.y as f32 + 0.5];
                    d.map_dirty = true;
                    o.effects.push(Effect::Teleported);
                }
            }
            Step::Travel { body } => {
                if let Some(m) = world.maps.get(body) {
                    let land = m.places.get("landing").or_else(|| m.places.get("start")).copied().unwrap_or_default();
                    d.current_map = body.clone();
                    d.players[0].pos = [land.x as f32 + 0.5, land.y as f32 + 0.5];
                    d.map_dirty = true;
                    o.effects.push(Effect::Teleported);
                }
            }
            Step::Sfx { id } => o.effects.push(Effect::Sfx(id.clone())),
            Step::Spawn { id, count } => o.effects.push(Effect::Spawn { id: id.clone(), count: *count }),
            Step::Shop { shop } => o.effects.push(Effect::OpenShop(shop.clone())),
            Step::Craft { station } => o.effects.push(Effect::OpenCraft(station.clone())),
            Step::Sleep => {
                d.clock.minutes = (d.clock.day() + if d.clock.hour() >= 7 { 1 } else { 0 }) * 1440 + 7 * 60;
                let (h, m) = (d.players[0].stats.max_hp(), d.players[0].stats.max_mana());
                d.players[0].stats.hp = h;
                d.players[0].stats.mana = m;
                o.effects.push(Effect::Sleep);
            }
            Step::UnlockTravel { node } => {
                d.travel_unlocked.insert(node.clone());
            }
            Step::Learn { spell } => {
                if d.spells_known.insert(spell.clone()) {
                    o.toasts.push(format!("Learned spell: {}", db.spells.iter().find(|s| &s.id == spell).map_or(spell.as_str(), |s| s.name.as_str())));
                }
            }
            Step::If { cond, then, els } => {
                let branch = if check(d, cond) { then } else { els };
                run_into(d, db, world, branch, o);
            }
        }
    }
}

fn complete_quest(d: &mut GameData, db: &Db, world: &World, quest: &str, o: &mut Out) {
    let step = d.quests.get(quest).map_or(0, |q| q.step);
    if matches!(d.quests.get(quest), Some(q) if q.state == QuestState::Done) {
        return;
    }
    d.quests.insert(quest.to_string(), QuestProgress { state: QuestState::Done, step });
    if let Some(def) = db.quest(quest) {
        o.toasts.push(format!("Quest complete: {}", def.title));
        o.effects.push(Effect::Sfx("level_up".into()));
        let reward = def.reward.clone();
        run_into(d, db, world, &reward, o);
    }
}

/// Start quests whose `start_when` holds and advance active quests. Returns toasts/effects.
pub fn tick_quests(d: &mut GameData, db: &Db, world: &World) -> Out {
    let mut o = Out::default();
    for q in &db.quests {
        if !d.quests.contains_key(&q.id) {
            if let Some(c) = &q.start_when {
                if check(d, c) {
                    run_into(d, db, world, &[Step::StartQuest { quest: q.id.clone() }], &mut o);
                }
            }
        }
        // advance (bounded loop: at most all steps in one tick)
        for _ in 0..q.steps.len() + 1 {
            let Some(p) = d.quests.get(&q.id).copied() else { break };
            if p.state != QuestState::Active {
                break;
            }
            let idx = p.step as usize;
            if idx >= q.steps.len() {
                complete_quest(d, db, world, &q.id, &mut o);
                break;
            }
            if check(d, &q.steps[idx].when) {
                d.quests.insert(q.id.clone(), QuestProgress { state: QuestState::Active, step: p.step + 1 });
                if idx + 1 < q.steps.len() {
                    o.toasts.push(format!("{}: {}", q.title, q.steps[idx + 1].text));
                }
            } else {
                break;
            }
        }
    }
    o
}

/// Mark places the player is near as visited and chunks as explored. Returns toasts.
pub fn track_visits(d: &mut GameData, world: &World) -> Vec<String> {
    let mut toasts = vec![];
    let Some(map) = world.maps.get(&d.current_map) else { return toasts };
    let p = d.players[0].pos;
    // explored chunks (3x3 around the player)
    let (cx, cy) = u67_core::TilePos::new(p[0] as i32, p[1] as i32).chunk();
    for dy in -1..=1 {
        for dx in -1..=1 {
            let key = format!("{}:{}:{}", d.current_map, cx + dx, cy + dy);
            if d.explored.insert(key) && d.current_map == "midgard" {
                *d.counters.entry("chunks_explored".into()).or_insert(0) += 1;
            }
        }
    }
    for (name, pos) in &map.places {
        let (dx, dy) = (pos.x as f32 + 0.5 - p[0], pos.y as f32 + 0.5 - p[1]);
        if dx * dx + dy * dy > 36.0 {
            continue;
        }
        let flag = format!("at_{}_{}", d.current_map, name);
        if d.flags.insert(flag) && name.starts_with("rune_ring") {
            *d.counters.entry("runes_found".into()).or_insert(0) += 1;
            toasts.push(format!("Rune ring found ({}).", d.counters["runes_found"]));
        }
    }
    toasts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (GameData, Db, World) {
        let db = Db::builtin().expect("content parses");
        let w = u67_mapgen::generate_world(67);
        let s = w.maps["midgard"].places["start"];
        (GameData::new(67, "midgard", [s.x as f32, s.y as f32]), db, w)
    }

    #[test]
    fn content_cross_references_are_valid() {
        let (_, db, w) = setup();
        let problems = db.validate(&|p| w.find_place(p).is_some());
        assert!(problems.is_empty(), "{problems:#?}");
    }

    #[test]
    fn content_places_exist() {
        let (_, db, w) = setup();
        for s in &db.spawns {
            if s.place != "*" {
                assert!(w.maps.get(&s.map).is_some_and(|m| m.places.contains_key(&s.place)), "spawn place {}@{}", s.place, s.map);
            }
        }
        for n in &db.npcs {
            let map = n.map.clone().unwrap_or_else(|| "midgard".into());
            let m = &w.maps[&map];
            for pl in [Some(&n.home), Some(&n.work), n.tavern.as_ref()].into_iter().flatten() {
                assert!(m.places.contains_key(pl), "npc {} place {pl} missing in {map}", n.id);
            }
        }
    }

    #[test]
    fn main_quest_chain_progresses() {
        let (mut d, db, w) = setup();
        let o = tick_quests(&mut d, &db, &w);
        assert!(o.toasts.iter().any(|t| t.contains("The Hum of the Gate")));
        assert_eq!(d.quests["m01_hum_of_the_gate"].step, 0);
        d.flags.insert("talked_eirikr".into());
        d.players[0].inventory.add("ammo_9mm", 40).unwrap();
        d.flags.insert("at_midgard_kaupang_gate".into());
        d.counters.insert("kills_draugr".into(), 3);
        let o = tick_quests(&mut d, &db, &w);
        assert_eq!(d.quests["m01_hum_of_the_gate"].state, QuestState::Done, "{:?}", o.toasts);
        assert!(d.flags.contains("gate_secured"));
        assert!(d.players[0].stats.xp >= 150);
        // m02 auto-starts afterwards
        tick_quests(&mut d, &db, &w);
        assert!(d.quests.contains_key("m02_warband"));
    }

    #[test]
    fn script_steps_and_conditions() {
        let (mut d, db, w) = setup();
        let steps: Vec<Step> = serde_json::from_str(
            r#"[{"op":"give","item":"rope","count":2},{"op":"inc","counter":"x"},
          {"op":"if","cond":{"is":"counter","counter":"x","min":1},"then":[{"op":"set_flag","flag":"yes"}],"els":[{"op":"set_flag","flag":"no"}]},
          {"op":"join_party","npc":"sigrun"},{"op":"sleep"}]"#,
        )
        .unwrap();
        d.clock.set_hm(22, 0);
        let day = d.clock.day();
        let o = run(&mut d, &db, &w, &steps);
        assert_eq!(d.players[0].inventory.count("rope"), 2);
        assert!(d.flags.contains("yes") && !d.flags.contains("no"));
        assert_eq!(d.party, vec!["sigrun"]);
        assert_eq!(d.clock.hour(), 7);
        assert_eq!(d.clock.day(), day + 1);
        assert!(o.effects.contains(&Effect::Sleep));
    }

    #[test]
    fn party_is_capped() {
        let (mut d, db, w) = setup();
        for i in 0..8 {
            run(&mut d, &db, &w, &[Step::JoinParty { npc: format!("n{i}") }]);
        }
        assert_eq!(d.party.len(), MAX_PARTY);
    }

    #[test]
    fn visiting_places_sets_flags_and_counts_runes() {
        let (mut d, _, w) = setup();
        let rr = w.maps["midgard"].places.iter().find(|(k, _)| k.starts_with("rune_ring")).map(|(k, v)| (k.clone(), *v)).unwrap();
        d.players[0].pos = [rr.1.x as f32 + 0.5, rr.1.y as f32 + 0.5];
        let t = track_visits(&mut d, &w);
        assert!(d.flags.contains(&format!("at_midgard_{}", rr.0)));
        assert_eq!(d.counters["runes_found"], 1);
        assert_eq!(t.len(), 1);
        assert!(track_visits(&mut d, &w).is_empty());
    }

    #[test]
    fn dialogue_reaches_quest_flags() {
        use u67_world::dialogue::Session;
        let (mut d, db, w) = setup();
        let def = &db.dialogues["eirikr"];
        let mut s = Session::new();
        let r = s.greet(def, &Ctx { d: &d });
        run(&mut d, &db, &w, &r.steps);
        assert!(d.flags.contains("talked_eirikr"));
        let r = s.say(def, "gate", &Ctx { d: &d });
        assert!(r.text.contains("hum"));
        assert!(s.known.iter().any(|k| k == "Bifrost"));
        // topic gated behind quest completion
        assert!(s.say(def, "norns", &Ctx { d: &d }).text.contains("know nothing"));
    }
}
