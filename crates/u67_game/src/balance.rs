//! Balance model: a player with level-appropriate gear should beat each creature at its
//! recommended level (with margin), and should not trivialise it. Run
//! `cargo test -p u67_game balance -- --nocapture` to see the table.
#![cfg(test)]
use crate::data::Stats;
use u67_world::combat::{after_armor, gun_stats};
use u67_world::db::Db;
use u67_world::items;

/// (creature id, recommended level)
const RECOMMENDED: &[(&str, u32)] = &[
    ("wolf", 1), ("draugr", 1), ("ice_wolf", 2), ("merc", 2), ("troll", 3), ("fenrir", 4), ("barrow_king", 4), ("merc_captain", 5), ("svartalf", 5),
    ("jotun", 6), ("gleipnir_smith", 6), ("fenrir_prime", 7), ("fire_giant", 8), ("hel_hound", 8), ("surtr", 9), ("jormungandr", 10), ("hel_queen", 11),
    ("nidhogg", 12), ("kalevala_guardian", 12), ("louhi_guard", 13), ("surma", 13), ("ukko_storm", 14), ("sampo_golem", 14), ("louhi", 16),
];

struct Kit {
    weapon: &'static str,
    armor: i32,
}

fn kit(level: u32) -> Kit {
    match level {
        0..=3 => Kit { weapon: "rune_pistol", armor: 0 },
        4..=6 => Kit { weapon: "fjord_rifle", armor: 7 },
        7..=9 => Kit { weapon: "gungnir_sniper", armor: 13 },
        _ => Kit { weapon: "vainamoinen_gun", armor: 16 },
    }
}

fn stats_at(level: u32) -> Stats {
    let mut s = Stats::default();
    s.add_xp(crate::data::xp_for_level(level));
    s
}

/// Sustained damage per second including reloads and a 75% hit rate.
pub fn player_dps(level: u32) -> f32 {
    let k = kit(level);
    let st = stats_at(level);
    let w = items::get(k.weapon).unwrap();
    let gs = gun_stats(k.weapon).unwrap();
    let per_shot = w.damage as f32 * (0.8 + st.dex as f32 * 0.02);
    let eff_cd = (gs.mag as f32 * gs.cooldown + gs.reload) / gs.mag as f32;
    let gun = per_shot / eff_cd * 0.75;
    // Real fights add: spells and gadgets (+25% from level 4), and recruited allies (2 by level 4, 3 by 8, 4 by 12)
    // who each contribute about 8 damage per second.
    let allies = match level {
        0..=3 => 0.0,
        4..=7 => 2.0,
        8..=11 => 3.0,
        _ => 4.0,
    };
    let magic = if level >= 4 { 1.25 } else { 1.0 };
    gun * magic + allies * 8.0
}

#[test]
fn every_creature_has_a_recommended_level() {
    let db = Db::builtin().unwrap();
    for id in db.creatures.keys() {
        assert!(RECOMMENDED.iter().any(|(r, _)| r == id), "{id} missing from the balance table");
    }
}

#[test]
fn fights_are_winnable_but_not_trivial() {
    let db = Db::builtin().unwrap();
    let mut bad = vec![];
    eprintln!("{:<18} {:>3} {:>7} {:>7} {:>8} {:>8}  verdict", "creature", "lvl", "php", "dps", "ttk(you)", "ttk(it)");
    for (id, lvl) in RECOMMENDED {
        let c = &db.creatures[*id];
        let k = kit(*lvl);
        let st = stats_at(*lvl);
        let php = st.max_hp() as f32;
        let dps = player_dps(*lvl);
        let ttk_you = c.hp as f32 / dps;
        let hit_rate = if c.range > 3.5 { 0.5 } else { 0.6 };
        let edps = after_armor(c.damage, k.armor) as f32 / c.cooldown * hit_rate;
        let ttk_it = php / edps;
        let verdict = if ttk_you > ttk_it * 0.8 {
            "TOO HARD"
        } else if ttk_you < 0.6 && !matches!(*id, "wolf" | "draugr" | "merc") {
            "TRIVIAL"
        } else {
            "ok"
        };
        eprintln!("{id:<18} {lvl:>3} {php:>7.0} {dps:>7.1} {ttk_you:>8.1} {ttk_it:>8.1}  {verdict}");
        if verdict != "ok" {
            bad.push(format!("{id}: {verdict} (you {ttk_you:.1}s vs it {ttk_it:.1}s)"));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn xp_curve_reaches_endgame_with_quest_rewards() {
    let db = Db::builtin().unwrap();
    // total XP from main + side quest rewards should land the player near level 14-18
    let mut xp = 0u32;
    for q in db.quests.iter().filter(|q| q.category != u67_world::quests::Category::Optional) {
        for s in &q.reward {
            if let u67_world::script::Step::Xp { amount } = s {
                xp += amount;
            }
        }
    }
    let lvl = crate::data::level_for_xp(xp);
    eprintln!("quest XP total {xp} -> level {lvl}");
    assert!((12..=22).contains(&lvl), "level {lvl} from {xp} xp");
}
