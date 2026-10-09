//! Pure logic behind list menus: shops, crafting, travel, journal, spells.
use crate::data::*;
use crate::script;
use u67_world::combat::item_value;
use u67_world::db::Db;
use u67_world::items::{self, Kind};
use u67_world::map::World;
use u67_world::quests::Category;
use u67_world::script::Cond;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    pub label: String,
    pub tag: String,
    pub enabled: bool,
    pub header: bool,
}

fn row(label: impl Into<String>, tag: impl Into<String>, enabled: bool) -> Row {
    Row { label: label.into(), tag: tag.into(), enabled, header: false }
}
fn header(label: impl Into<String>) -> Row {
    Row { label: label.into(), tag: String::new(), enabled: false, header: true }
}

fn name(id: &str) -> &str {
    items::get(id).map_or(id, |d| d.name)
}

// ------------------------------------------------------------------ shops
pub fn buy_price(db: &Db, shop: &str, item: &str) -> Option<u32> {
    let s = db.shops.get(shop)?;
    let e = s.stock.iter().find(|e| e.item == item)?;
    Some(e.price.unwrap_or_else(|| item_value(item)).max(1))
}

pub fn sell_price(db: &Db, shop: &str, item: &str) -> u32 {
    let rate = db.shops.get(shop).map_or(0.5, |s| s.buy_rate);
    ((item_value(item) as f32 * rate).floor() as u32).max(1)
}

fn currency(db: &Db, shop: &str) -> String {
    db.shops.get(shop).and_then(|s| s.currency.clone()).unwrap_or_else(|| "silver".into())
}

fn flatten(v: &[u67_world::inventory::Item], out: &mut Vec<(String, u32)>) {
    for i in v {
        match out.iter_mut().find(|(id, _)| *id == i.id) {
            Some(e) => e.1 += i.qty,
            None => out.push((i.id.clone(), i.qty)),
        }
        flatten(&i.contents, out);
    }
}

pub fn shop_rows(d: &GameData, db: &Db, shop: &str, selling: bool) -> Vec<Row> {
    let mut rows = vec![];
    let cur = currency(db, shop);
    let have = d.players[0].inventory.count(&cur);
    rows.push(header(format!("{}   [{}: {}]", if selling { "SELLING (your goods)" } else { "BUYING" }, name(&cur), have)));
    if selling {
        let mut all = vec![];
        flatten(&d.players[0].inventory.pack, &mut all);
        all.sort();
        for (id, n) in all {
            if id == cur || items::get(&id).is_some_and(|i| i.kind == Kind::Container) {
                continue;
            }
            rows.push(row(format!("{} x{}   -> {} each", name(&id), n, sell_price(db, shop, &id)), format!("sell:{id}"), true));
        }
    } else if let Some(s) = db.shops.get(shop) {
        for e in &s.stock {
            let p = buy_price(db, shop, &e.item).unwrap_or(1);
            rows.push(row(format!("{}   {} {}", name(&e.item), p, name(&cur)), format!("buy:{}", e.item), have >= p));
        }
    }
    rows
}

pub fn buy(d: &mut GameData, db: &Db, shop: &str, item: &str) -> Result<String, String> {
    let price = buy_price(db, shop, item).ok_or("not for sale")?;
    let cur = currency(db, shop);
    if d.players[0].inventory.count(&cur) < price {
        return Err(format!("You need {price} {}.", name(&cur)));
    }
    d.players[0].inventory.add(item, 1).map_err(|e| match e {
        u67_world::inventory::InvError::TooHeavy => "You cannot carry that much.".to_string(),
        e => format!("{e:?}"),
    })?;
    d.players[0].inventory.remove(&cur, price);
    Ok(format!("Bought {} for {price}.", name(item)))
}

pub fn sell(d: &mut GameData, db: &Db, shop: &str, item: &str) -> Result<String, String> {
    if d.players[0].inventory.count(item) == 0 {
        return Err("You do not have that.".into());
    }
    let p = sell_price(db, shop, item);
    let cur = currency(db, shop);
    d.players[0].inventory.remove(item, 1);
    // currency is always accepted even if overweight
    let inv = &mut d.players[0].inventory;
    let saved = inv.max_weight;
    inv.max_weight = 1e9;
    inv.add(&cur, p).ok();
    inv.max_weight = saved;
    Ok(format!("Sold {} for {p}.", name(item)))
}

// ------------------------------------------------------------------ crafting
pub fn craft_rows(d: &GameData, db: &Db, station: &str) -> Vec<Row> {
    let mut rows = vec![header(format!("CRAFTING: {station}"))];
    for r in db.recipes.iter().filter(|r| r.station == station) {
        let ins: Vec<String> = r.inputs.iter().map(|(i, n)| format!("{n} {}", name(i))).collect();
        let (ok, _) = can_craft(d, db, &r.id);
        rows.push(row(format!("{} <- {}", r.name, ins.join(", ")), format!("craft:{}", r.id), ok.is_ok()));
    }
    rows
}

pub fn can_craft(d: &GameData, db: &Db, id: &str) -> (Result<(), String>, bool) {
    let Some(r) = db.recipes.iter().find(|r| r.id == id) else { return (Err("unknown recipe".into()), false) };
    if let Some(f) = &r.needs_flag {
        if !d.flags.contains(f) {
            return (Err("You do not know this craft yet (visit the right place).".into()), false);
        }
    }
    if d.players[0].stats.level < r.level {
        return (Err(format!("Requires level {}.", r.level)), false);
    }
    for (i, n) in &r.inputs {
        if d.players[0].inventory.count(i) < *n {
            return (Err(format!("Missing {} x{}.", name(i), n)), false);
        }
    }
    (Ok(()), true)
}

pub fn craft(d: &mut GameData, db: &Db, id: &str) -> Result<String, String> {
    can_craft(d, db, id).0?;
    let r = db.recipes.iter().find(|r| r.id == id).unwrap().clone();
    for (i, n) in &r.inputs {
        d.players[0].inventory.remove(i, *n);
    }
    let inv = &mut d.players[0].inventory;
    let saved = inv.max_weight;
    inv.max_weight = 1e9;
    inv.add(&r.output.0, r.output.1).ok();
    inv.max_weight = saved;
    let sk = d.players[0].skills.entry("crafting".into()).or_insert(0);
    *sk += 1;
    Ok(format!("Crafted {} x{}.", name(&r.output.0), r.output.1))
}

// ------------------------------------------------------------------ travel
/// Quest-progress requirement to reach a body by ship.
pub fn travel_requirement(body: &str) -> Cond {
    let done = |q: &str| Cond::Quest { quest: q.into(), state: "done".into() };
    match body {
        "midgard" => Cond::Flag { flag: "game_started".into() },
        "maani" | "dvalinn" | "vanaheimr" => done("m03_three_threads"),
        "muspelheimr" | "svartalfheimr" | "jotunheimr" => done("m05_fenrir_prime"),
        "asgard" | "niflheimr" => done("m07_surtr_bunker"),
        "hel" => done("m09_odins_vault"),
        "ginnungagap" => done("m10_hel_bargain"),
        _ => Cond::Flag { flag: "sampo_gate_open".into() },
    }
}

pub fn can_travel(d: &GameData, body: &str, by_ship: bool) -> bool {
    body == "midgard" || d.travel_unlocked.contains(body) || (by_ship && script::check(d, &travel_requirement(body)))
}

pub fn travel_rows(d: &GameData, by_ship: bool) -> Vec<Row> {
    let mut rows = vec![header(if by_ship { "STAR MAP (ship)" } else { "BIFROST NODE (visited places only)" })];
    rows.push(row("Midgård - Earth (home)", "go:midgard", d.current_map != "midgard"));
    let mut last = "";
    for b in u67_mapgen::planets::BODIES {
        if b.system != last {
            rows.push(header(if b.system == "sol" { "-- Sol system --" } else { "-- Kalevala system (beyond the Sampo Gate) --" }));
            last = b.system;
        }
        let ok = can_travel(d, b.id, by_ship) && d.current_map != b.id;
        let suffix = if d.current_map == b.id {
            "  (you are here)"
        } else if !can_travel(d, b.id, by_ship) {
            "  (locked)"
        } else {
            ""
        };
        rows.push(row(format!("{}{}", b.name, suffix), format!("go:{}", b.id), ok));
    }
    rows
}

pub fn travel(d: &mut GameData, world: &World, db: &Db, body: &str, by_ship: bool) -> Result<String, String> {
    if !can_travel(d, body, by_ship) {
        return Err("You cannot go there yet.".into());
    }
    if !world.maps.contains_key(body) {
        return Err("No such place.".into());
    }
    let out = script::run(d, db, world, &[u67_world::script::Step::Travel { body: body.to_string() }]);
    d.travel_unlocked.insert(body.to_string());
    d.vehicle = None;
    let _ = out;
    Ok(format!("You arrive at {}.", body))
}

// ------------------------------------------------------------------ cheats menu
/// Quick actions of the F2 cheats menu: (label, console command).
pub const CHEAT_ACTIONS: &[(&str, &str)] = &[
    ("Heal & restore mana", "heal"),
    ("Give Väinämöinen's Kantele-Rifle", "give vainamoinen_gun"),
    ("Give 1000 silver", "money 1000"),
    ("Give all weapons", "give_all_weapons"),
    ("Refill ammo", "ammo 100"),
    ("Cannon kit (10 balls + 10 powder)", "give cannon_ball 10"),
    ("  ... powder", "give gunpowder 10"),
    ("Level 10", "level 10"),
    ("Learn all spells (no reagents)", "spells_all"),
    ("Reveal map", "reveal_map"),
    ("Unlock all fast travel", "unlock_travel"),
    ("Time: noon", "time 12:00"),
    ("Time: midnight", "time 00:00"),
    ("Teleport: Kaupang", "tp kaupang"),
    ("Teleport: Mimir's Well", "tp mimir_well"),
    ("Teleport: Kaupang gate", "tp kaupang_gate"),
    ("Travel: Máni (Moon)", "planet maani"),
    ("Travel: Hel", "planet hel"),
    ("Travel: Väinölä", "planet vainola"),
    ("Spawn 3 wolves", "spawn wolf 3"),
    ("Spawn a troll", "spawn troll"),
    ("Kill all nearby enemies", "kill_all"),
    ("Place a cannon", "spawn_cannon"),
    ("Spawn a longship (needs water)", "spawn_ship longship"),
    ("Quicksave", "save"),
];

pub fn cheat_rows(d: &GameData) -> Vec<Row> {
    let mut rows = vec![header("CHEATS & DEBUG   (console: ` or F1; all commands in commands.txt)")];
    for c in u67_console::Cheat::ALL {
        let on = d.cheats.get(c);
        rows.push(row(format!("[{}] {}", if on { "x" } else { " " }, c.label()), format!("cmd:{}", c.command()), true));
    }
    rows.push(header("-- quick actions --"));
    for (label, cmd) in CHEAT_ACTIONS {
        rows.push(row(*label, format!("cmd:{cmd}"), true));
    }
    rows
}

// ------------------------------------------------------------------ journal / spells
pub fn journal_rows(d: &GameData, db: &Db) -> Vec<Row> {
    let mut rows = vec![];
    for (cat, title) in [(Category::Main, "MAIN QUESTS"), (Category::Side, "SIDE QUESTS"), (Category::Optional, "OPTIONAL TASKS")] {
        rows.push(header(title));
        for q in db.quests.iter().filter(|q| q.category == cat) {
            match d.quests.get(&q.id) {
                Some(p) if p.state == QuestState::Active => {
                    let step = q.steps.get(p.step as usize).map_or("...", |s| s.text.as_str());
                    rows.push(row(format!("> {}: {}", q.title, step), format!("q:{}", q.id), true));
                }
                Some(_) => rows.push(row(format!("  [done] {}", q.title), format!("q:{}", q.id), false)),
                None => {}
            }
        }
    }
    if rows.iter().all(|r| r.header) {
        rows.push(row("No quests yet. Talk to people!", "", false));
    }
    rows
}

pub fn spell_known(d: &GameData, db: &Db, id: &str) -> bool {
    d.spells_known.contains(id) || d.spells_known.contains("*") || db.spells.iter().any(|s| s.id == id && s.known)
}

pub fn spell_rows(d: &GameData, db: &Db) -> Vec<Row> {
    let mut rows = vec![header(format!("SPELLBOOK   mana {}/{}   (click to cast; sing runes with L)", d.players[0].stats.mana, d.players[0].stats.max_mana()))];
    for s in &db.spells {
        let known = spell_known(d, db, &s.id);
        let rg: Vec<String> = s.reagents.iter().map(|(r, n)| format!("{n} {}", name(r))).collect();
        rows.push(row(
            if known {
                format!("[{}] {} - {} mana ({})", s.circle, s.name, s.mana, if rg.is_empty() { "no reagents".into() } else { rg.join(", ") })
            } else {
                format!("[{}] ??? (not learned)", s.circle)
            },
            format!("cast:{}", s.id),
            known,
        ));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (GameData, Db, World) {
        let db = Db::builtin().unwrap();
        let w = u67_mapgen::generate_world(67);
        (GameData::new(67, "midgard", [100.0, 100.0]), db, w)
    }

    #[test]
    fn buy_and_sell() {
        let (mut d, db, _) = setup();
        assert_eq!(d.players[0].inventory.count("silver"), 25);
        assert!(buy(&mut d, &db, "smith_kaupang", "fjord_rifle").is_err());
        d.players[0].inventory.add("silver", 500).unwrap();
        let before = d.players[0].inventory.count("silver");
        buy(&mut d, &db, "smith_kaupang", "ammo_9mm").unwrap();
        assert_eq!(d.players[0].inventory.count("silver"), before - 1);
        let p = sell_price(&db, "smith_kaupang", "ammo_9mm");
        sell(&mut d, &db, "smith_kaupang", "ammo_9mm").unwrap();
        assert!(p >= 1);
        assert!(sell(&mut d, &db, "smith_kaupang", "gleipnir_chain").is_err());
        let rows = shop_rows(&d, &db, "smith_kaupang", false);
        assert!(rows.len() > 10 && rows[0].header);
        assert!(shop_rows(&d, &db, "smith_kaupang", true).iter().any(|r| r.tag == "sell:ammo_9mm"));
    }

    #[test]
    fn kalevala_shop_uses_krediitti() {
        let (mut d, db, _) = setup();
        assert!(buy(&mut d, &db, "kalevala_market", "medkit").is_err());
        d.players[0].inventory.add("krediitti", 50).unwrap();
        buy(&mut d, &db, "kalevala_market", "medkit").unwrap();
        assert_eq!(d.players[0].inventory.count("krediitti"), 30);
    }

    #[test]
    fn crafting_rules() {
        let (mut d, db, _) = setup();
        assert!(craft(&mut d, &db, "ingot").is_err());
        d.players[0].inventory.add("scrap_iron", 7).unwrap();
        craft(&mut d, &db, "ingot").unwrap();
        craft(&mut d, &db, "ingot").unwrap();
        assert_eq!(d.players[0].inventory.count("iron_ingot"), 2);
        assert_eq!(d.players[0].inventory.count("scrap_iron"), 1);
        // gated recipes
        d.players[0].inventory.add("reagent_bog_iron", 3).unwrap();
        d.players[0].inventory.add("reagent_amber", 2).unwrap();
        d.players[0].inventory.add("iron_ingot", 3).unwrap();
        assert!(craft(&mut d, &db, "gleipnir").unwrap_err().contains("do not know"));
        d.flags.insert("at_dvalinn_brokkr_forge".into());
        craft(&mut d, &db, "gleipnir").unwrap();
        assert_eq!(d.players[0].inventory.count("gleipnir_chain"), 1);
        // level gate
        d.players[0].inventory.add("iron_ingot", 3).unwrap();
        d.players[0].inventory.add("leather", 1).unwrap();
        assert!(craft(&mut d, &db, "long_sword").unwrap_err().contains("level"));
    }

    #[test]
    fn travel_gating_and_cheat() {
        let (mut d, db, w) = setup();
        assert!(!can_travel(&d, "maani", true));
        assert!(travel(&mut d, &w, &db, "maani", true).is_err());
        d.quests.insert("m03_three_threads".into(), QuestProgress { state: QuestState::Done, step: 4 });
        assert!(can_travel(&d, "maani", true) && !can_travel(&d, "maani", false));
        travel(&mut d, &w, &db, "maani", true).unwrap();
        assert_eq!(d.current_map, "maani");
        assert!(can_travel(&d, "maani", false), "arrival unlocks the node");
        assert!(!can_travel(&d, "pohjola", true));
        d.flags.insert("sampo_gate_open".into());
        assert!(can_travel(&d, "pohjola", true));
        let rows = travel_rows(&d, true);
        assert!(rows.iter().any(|r| r.tag == "go:pohjola" && r.enabled));
    }

    #[test]
    fn journal_lists_active_and_done() {
        let (mut d, db, w) = setup();
        script::tick_quests(&mut d, &db, &w);
        let rows = journal_rows(&d, &db);
        assert!(rows.iter().any(|r| r.label.contains("The Hum of the Gate") && r.enabled));
        d.quests.insert("m01_hum_of_the_gate".into(), QuestProgress { state: QuestState::Done, step: 4 });
        assert!(journal_rows(&d, &db).iter().any(|r| r.label.contains("[done]")));
    }

    #[test]
    fn every_cheat_action_is_a_valid_command() {
        let reg = u67_console::Registry::standard();
        for (label, cmd) in CHEAT_ACTIONS {
            assert!(matches!(reg.parse(cmd), Ok(Some(_))), "{label}: {cmd}");
        }
        let d = GameData::new(1, "midgard", [0.0, 0.0]);
        assert!(cheat_rows(&d).len() > 30);
    }

    #[test]
    fn spells_known_defaults() {
        let (mut d, db, _) = setup();
        assert!(spell_known(&d, &db, "valo"));
        assert!(!spell_known(&d, &db, "siirto"));
        d.spells_known.insert("siirto".into());
        assert!(spell_known(&d, &db, "siirto"));
        assert!(spell_rows(&d, &db).len() > 10);
    }
}
