//! Inventory icons, 16x16, one per item in `u67_world::items::ITEMS` (same order).
use crate::canvas::*;
use crate::manifest::Atlas;
use u67_world::items::{Kind, Slot, ITEMS};

pub const I: i32 = 16;
const OUT: Col = rgb(20, 14, 18);
const STEEL: Col = rgb(175, 182, 195);
const DARK: Col = rgb(55, 58, 68);
const WOODC: Col = rgb(130, 85, 45);
const GOLD: Col = rgb(235, 190, 70);

fn hash_col(id: &str) -> Col {
    let mut h: u32 = 2166136261;
    for b in id.bytes() {
        h = (h ^ b as u32).wrapping_mul(16777619);
    }
    rgb(90 + (h & 0x7f) as u8, 90 + ((h >> 8) & 0x7f) as u8, 90 + ((h >> 16) & 0x7f) as u8)
}

pub fn icon(idx: usize) -> Canvas {
    let d = &ITEMS[idx];
    let acc = hash_col(d.id);
    let mut c = Canvas::new(I, I);
    match d.kind {
        Kind::Gun => {
            let len = if d.weight > 4.0 { 13 } else if d.weight > 2.5 { 11 } else { 7 };
            let x0 = 8 - len / 2;
            if d.id == "vainamoinen_gun" {
                // kantele-shaped rifle: wooden body, gold strings
                c.tri((2, 12), (13, 3), (13, 12), rgb(150, 100, 50));
                for i in 0..4 {
                    c.line(4 + i * 2, 11, 12, 4 + i, GOLD);
                }
                c.rect(11, 6, 5, 2, DARK);
            } else {
                c.rect(x0, 7, len, 2, DARK);
                c.rect(x0, 7, 4, 3, WOODC);
                c.rect(x0 + 3, 9, 2, 3, WOODC);
                c.rect(x0 + len - 3, 6, 3, 1, STEEL);
                c.set(x0 + 5, 8, acc);
                if d.id == "gungnir_sniper" {
                    c.rect(6, 5, 4, 2, rgb(30, 30, 40));
                    c.set(7, 5, rgb(120, 220, 255));
                }
            }
        }
        Kind::Melee => {
            if d.id == "long_sword" || d.id == "puukko" {
                let l = if d.id == "puukko" { 6 } else { 11 };
                c.line(12, 3 + (11 - l) / 2, 12 - l + 2, 3 + l + (11 - l) / 2, STEEL);
                c.line(13, 4, 13 - l + 2, 4 + l, lighten(STEEL, 0.4));
                c.rect(3, 12, 5, 1, GOLD);
                c.rect(2, 13, 2, 2, WOODC);
            } else if d.id == "round_spear" {
                c.line(3, 13, 11, 5, WOODC);
                c.tri((14, 2), (10, 4), (12, 6), STEEL);
            } else if d.id == "mjolnir_drone" {
                c.rect(3, 3, 10, 5, rgb(90, 100, 125));
                c.rect(3, 3, 10, 1, rgb(150, 180, 230));
                c.rect(7, 8, 2, 6, WOODC);
                c.set(11, 5, rgb(120, 230, 255));
            } else {
                c.line(3, 14, 11, 4, WOODC);
                let big = d.id == "bear_axe";
                c.tri((9, 2), (14, 5 + big as i32), (10, 9), STEEL);
                c.tri((9, 2), (5, 4), (10, 6), darken(STEEL, 0.2));
            }
        }
        Kind::Ammo => match d.id {
            "cannon_ball" => c.ellipse(8, 8, 5, 5, rgb(45, 48, 55)),
            "gunpowder" => {
                c.rect(4, 5, 8, 9, WOODC);
                c.rect(4, 7, 8, 1, DARK);
                c.rect(4, 11, 8, 1, DARK);
                c.rect(7, 2, 2, 3, rgb(240, 90, 30));
            }
            "ammo_shell" => {
                for x in [4, 8] {
                    c.rect(x, 4, 3, 9, rgb(190, 50, 45));
                    c.rect(x, 11, 3, 2, GOLD);
                }
            }
            _ => {
                for x in [3, 6, 9] {
                    c.rect(x, 5, 2, 8, GOLD);
                    c.rect(x, 3, 2, 3, rgb(200, 120, 70));
                }
            }
        },
        Kind::Armor => match d.slot {
            Some(Slot::Head) => {
                c.ellipse(8, 9, 5, 5, STEEL);
                c.rect(3, 9, 10, 5, STEEL);
                c.rect(2, 4, 1, 4, rgb(240, 235, 215));
                c.rect(13, 4, 1, 4, rgb(240, 235, 215));
                c.rect(7, 9, 2, 5, DARK);
            }
            Some(Slot::Torso) => {
                c.rect(4, 3, 8, 11, rgb(70, 78, 90));
                c.rect(2, 3, 3, 4, rgb(70, 78, 90));
                c.rect(11, 3, 3, 4, rgb(70, 78, 90));
                c.rect(6, 3, 4, 2, rgb(30, 30, 38));
                c.rect(4, 9, 8, 1, GOLD);
            }
            Some(Slot::Back) => {
                c.tri((8, 2), (2, 14), (14, 14), rgb(105, 95, 90));
                c.rect(5, 2, 6, 2, rgb(220, 220, 225));
            }
            Some(Slot::Legs) => {
                c.rect(4, 2, 8, 3, DARK);
                c.rect(4, 5, 3, 9, STEEL);
                c.rect(9, 5, 3, 9, STEEL);
            }
            Some(Slot::Feet) => {
                c.rect(3, 6, 5, 7, WOODC);
                c.rect(3, 12, 8, 2, DARK);
                c.rect(9, 9, 4, 5, WOODC);
            }
            Some(Slot::HandL) => {
                c.ellipse(8, 8, 6, 6, rgb(170, 60, 55));
                c.ellipse(8, 8, 2, 2, STEEL);
                c.frame(2, 2, 12, 12, CLEAR);
            }
            _ => c.rect(4, 4, 8, 8, acc),
        },
        Kind::Food => match d.id {
            "mead" => {
                c.tri((4, 2), (12, 2), (8, 14), rgb(230, 190, 90));
                c.rect(4, 2, 8, 2, rgb(250, 240, 200));
                c.rect(7, 12, 2, 2, DARK);
            }
            "rye_bread" => {
                c.ellipse(8, 9, 6, 4, rgb(150, 100, 55));
                c.line(5, 7, 7, 10, rgb(110, 70, 35));
                c.line(9, 7, 11, 10, rgb(110, 70, 35));
            }
            "smoked_fish" => {
                c.ellipse(8, 8, 5, 3, rgb(190, 130, 70));
                c.tri((13, 8), (15, 5), (15, 11), rgb(190, 130, 70));
                c.set(4, 7, rgb(20, 20, 20));
            }
            _ => {
                c.rect(2, 4, 12, 9, rgb(235, 235, 235));
                c.rect(7, 5, 2, 7, rgb(200, 40, 40));
                c.rect(4, 7, 8, 2, rgb(200, 40, 40));
            }
        },
        Kind::Tool => match d.id {
            "torch" => {
                c.rect(7, 7, 2, 8, WOODC);
                c.tri((8, 1), (5, 8), (11, 8), rgb(240, 120, 30));
                c.tri((8, 4), (6, 8), (10, 8), rgb(255, 230, 120));
            }
            "lockpick" => {
                c.line(3, 13, 12, 4, STEEL);
                c.line(12, 4, 14, 6, STEEL);
                c.rect(2, 13, 3, 2, DARK);
            }
            "rope" => {
                c.ellipse(8, 8, 6, 6, rgb(190, 160, 100));
                c.ellipse(8, 8, 3, 3, CLEAR);
            }
            "vacuum_suit" => {
                c.ellipse(8, 6, 4, 4, rgb(235, 235, 240));
                c.ellipse(8, 6, 3, 2, rgb(80, 200, 230));
                c.rect(4, 10, 8, 5, rgb(235, 235, 240));
            }
            _ => {
                c.ellipse(5, 5, 3, 3, GOLD);
                c.ellipse(5, 5, 1, 1, CLEAR);
                c.rect(7, 7, 2, 7, GOLD);
                c.rect(9, 11, 3, 1, GOLD);
            }
        },
        Kind::Relic => match d.id {
            id if id.starts_with("sampo_shard") => {
                c.tri((8, 1), (3, 10), (8, 15), rgb(200, 140, 255));
                c.tri((8, 1), (13, 10), (8, 15), rgb(150, 90, 220));
                c.set(6, 6, rgb(255, 255, 255));
            }
            "gleipnir_chain" => {
                for i in 0..4 {
                    c.ellipse(4 + i * 3, 4 + i * 3, 2, 2, STEEL);
                    c.ellipse(4 + i * 3, 4 + i * 3, 1, 1, CLEAR);
                }
            }
            "odin_eye" => {
                c.ellipse(8, 8, 6, 4, rgb(245, 245, 235));
                c.ellipse(8, 8, 3, 3, rgb(60, 140, 220));
                c.set(8, 8, DARK);
            }
            "skidbladnir_ship" => {
                c.rect(2, 9, 12, 3, WOODC);
                c.tri((8, 2), (4, 9), (12, 9), rgb(240, 235, 220));
            }
            "draupnir_ring" => {
                c.ellipse(8, 9, 5, 5, GOLD);
                c.ellipse(8, 9, 3, 3, CLEAR);
                c.rect(7, 3, 3, 2, rgb(90, 220, 255));
            }
            _ => {
                c.ellipse(8, 8, 3, 3, GOLD);
                c.line(8, 2, 8, 5, GOLD);
                c.tri((6, 5), (10, 5), (8, 12), rgb(90, 200, 255));
            }
        },
        Kind::Reagent => {
            c.ellipse(8, 10, 5, 4, acc);
            c.rect(6, 4, 4, 3, darken(acc, 0.3));
            c.set(7, 9, lighten(acc, 0.5));
        }
        Kind::Container => match d.id {
            "backpack" => {
                c.rect(3, 4, 10, 10, rgb(150, 100, 55));
                c.ellipse(8, 4, 5, 2, rgb(170, 120, 65));
                c.rect(5, 9, 6, 4, rgb(120, 78, 40));
                c.rect(7, 10, 2, 1, GOLD);
            }
            "chest" => {
                c.rect(2, 6, 12, 8, WOODC);
                c.rect(2, 6, 12, 2, rgb(165, 115, 66));
                c.rect(7, 8, 2, 3, GOLD);
            }
            "pouch" => {
                c.ellipse(8, 10, 4, 4, rgb(160, 110, 70));
                c.rect(6, 4, 4, 3, rgb(160, 110, 70));
                c.rect(5, 6, 6, 1, rgb(90, 55, 30));
            }
            _ => {
                c.rect(2, 5, 12, 8, rgb(70, 85, 70));
                c.rect(2, 5, 12, 2, rgb(95, 115, 95));
                c.rect(7, 8, 2, 2, GOLD);
            }
        },
        Kind::Misc => {
            c.ellipse(8, 8, 5, 5, rgb(205, 210, 220));
            c.ellipse(8, 8, 3, 3, rgb(165, 170, 185));
            c.set(5, 5, rgb(255, 255, 255));
        }
    }
    c.outline(OUT);
    c
}

pub fn sheet(atlas: &mut Atlas) -> Canvas {
    let cols = 8;
    let rows = (ITEMS.len() as i32 + cols - 1) / cols;
    let mut s = Canvas::new(I * cols, I * rows);
    for (i, d) in ITEMS.iter().enumerate() {
        let (x, y) = ((i as i32 % cols) * I, (i as i32 / cols) * I);
        s.blit(&icon(i), x, y);
        atlas.add(d.id, x, y, I, I);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_item_has_icon() {
        for (i, d) in ITEMS.iter().enumerate() {
            assert!(icon(i).px.iter().filter(|p| p[3] > 0).count() > 12, "{}", d.id);
        }
    }
}
