//! Combat math and weapon stats. Pure and deterministic given an `Rng`.
use crate::items::{self, Kind};
use u67_core::Rng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GunStats {
    pub mag: u16,
    pub range: f32,
    pub cooldown: f32,
    pub spread_deg: f32,
    pub pellets: u8,
    pub speed: f32,
    pub reload: f32,
    /// special effect id: "", "stun", "flare"
    pub special: &'static str,
}

pub fn gun_stats(id: &str) -> Option<GunStats> {
    let g = |mag, range, cooldown, spread_deg, pellets, speed, reload, special| Some(GunStats { mag, range, cooldown, spread_deg, pellets, speed, reload, special });
    match id {
        "rune_pistol" => g(12, 9.0, 0.30, 3.0, 1, 22.0, 1.2, ""),
        "hunting_shotgun" => g(6, 6.5, 0.85, 12.0, 6, 20.0, 2.2, ""),
        "fjord_rifle" => g(8, 14.0, 0.75, 1.0, 1, 30.0, 1.8, ""),
        "skald_smg" => g(30, 8.0, 0.09, 6.0, 1, 24.0, 1.8, ""),
        "gungnir_sniper" => g(5, 26.0, 1.40, 0.0, 1, 45.0, 2.5, ""),
        "vainamoinen_gun" => g(10, 17.0, 0.50, 0.0, 1, 34.0, 1.5, "stun"),
        "flare_gun" => g(1, 12.0, 1.00, 2.0, 1, 16.0, 1.5, "flare"),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeleeStats {
    pub range: f32,
    pub cooldown: f32,
    pub arc_deg: f32,
}

pub fn melee_stats(id: &str) -> Option<MeleeStats> {
    let d = items::get(id)?;
    if d.kind != Kind::Melee {
        return None;
    }
    let reach = match id {
        "round_spear" => 1.9,
        "mjolnir_drone" => 1.8,
        "puukko" => 1.0,
        _ => 1.4,
    };
    Some(MeleeStats { range: reach, cooldown: 0.35 + d.weight * 0.08, arc_deg: if id == "round_spear" { 50.0 } else { 110.0 } })
}

/// Base price in silver.
pub fn item_value(id: &str) -> u32 {
    let Some(d) = items::get(id) else { return 1 };
    let base = match d.kind {
        Kind::Gun => 60 + d.damage as u32 * 6,
        Kind::Melee => 20 + d.damage as u32 * 4,
        Kind::Armor => 15 + d.armor as u32 * 12,
        Kind::Ammo => 1 + (d.weight * 20.0) as u32 / 10,
        Kind::Food => 3 + (d.weight * 10.0) as u32,
        Kind::Tool => 8 + (d.weight * 6.0) as u32,
        Kind::Reagent => 4,
        Kind::Container => 6 + (d.capacity as u32) / 2,
        Kind::Relic => 400,
        Kind::Misc => 1,
    };
    if id == "silver" {
        1
    } else {
        base
    }
}

pub fn heal_amount(id: &str) -> Option<i32> {
    match id {
        "medkit" => Some(40),
        "rye_bread" => Some(6),
        "smoked_fish" => Some(10),
        "mead" => Some(8),
        _ => None,
    }
}

/// Chance that an attack lands (0.05..0.95). `dex_att` attacker dexterity, `dex_def` defender's.
pub fn hit_chance(dex_att: i32, dex_def: i32, base: f32) -> f32 {
    (base + (dex_att - dex_def) as f32 * 0.01).clamp(0.05, 0.95)
}

/// Damage after armor: each armor point removes 6% (cap 70%), minimum 1.
pub fn after_armor(dmg: i32, armor: i32) -> i32 {
    let red = (armor as f32 * 0.06).min(0.7);
    ((dmg as f32 * (1.0 - red)).round() as i32).max(1)
}

/// Player attack damage: weapon damage scaled by STR (melee) or DEX (guns), +-20% variance, 5% crit x2.
pub fn roll_damage(weapon_dmg: i32, stat: u32, rng: &mut Rng) -> (i32, bool) {
    let scale = 0.8 + stat as f32 * 0.02;
    let var = 0.8 + rng.f32() * 0.4;
    let crit = rng.chance(0.05);
    let d = (weapon_dmg as f32 * scale * var * if crit { 2.0 } else { 1.0 }).round() as i32;
    (d.max(1), crit)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_gun_has_stats_and_ammo_match() {
        for d in items::ITEMS.iter().filter(|d| d.kind == Kind::Gun) {
            assert!(gun_stats(d.id).is_some(), "{}", d.id);
        }
        for d in items::ITEMS.iter().filter(|d| d.kind == Kind::Melee) {
            assert!(melee_stats(d.id).is_some(), "{}", d.id);
        }
    }
    #[test]
    fn armor_and_hit_bounds() {
        assert_eq!(after_armor(10, 0), 10);
        assert_eq!(after_armor(10, 5), 7);
        assert_eq!(after_armor(10, 50), 3);
        assert_eq!(after_armor(1, 50), 1);
        assert_eq!(hit_chance(100, 0, 0.8), 0.95);
        assert_eq!(hit_chance(0, 100, 0.8), 0.05);
    }
    #[test]
    fn damage_scales_with_stat() {
        let (lo, _) = roll_damage(10, 5, &mut Rng::new(3));
        let (hi, _) = roll_damage(10, 40, &mut Rng::new(3));
        assert!(hi > lo);
        assert!(item_value("vainamoinen_gun") > item_value("rune_pistol"));
    }
}
