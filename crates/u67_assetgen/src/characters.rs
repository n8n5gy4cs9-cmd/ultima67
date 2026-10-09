//! Character sprites. 24x24 cells. Row = archetype*4 + dir (N,E,S,W). Columns:
//! 0 idle, 1-4 walk, 5-7 attack, 8-10 die.
use crate::canvas::*;
use crate::manifest::Atlas;

pub const CELL: i32 = 24;
pub const COLS: i32 = 11;
const OUT: Col = rgb(18, 12, 16);

#[derive(Clone, Copy, PartialEq)]
pub enum Helm {
    None,
    Horned,
    Winged,
    Hood,
    Band,
    Visor,
    Antlers,
    Hat,
    Crown,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Gun {
    None,
    Rifle,
    Axe,
    Staff,
    Spear,
}

#[derive(Clone, Copy)]
pub struct Look {
    pub name: &'static str,
    pub skin: Col,
    pub hair: Col,
    pub top: Col,
    pub bottom: Col,
    pub accent: Col,
    pub helm: Helm,
    pub beard: bool,
    pub hold: Gun,
    /// body width bonus (troll/jotun) and height shrink (dwarf)
    pub bulk: i32,
    pub short: i32,
    pub glow_eyes: Option<Col>,
    pub skirt: bool,
}

const SKIN: Col = rgb(235, 190, 150);
const PALE: Col = rgb(170, 200, 215);

const fn look(name: &'static str, skin: Col, hair: Col, top: Col, bottom: Col, accent: Col) -> Look {
    Look { name, skin, hair, top, bottom, accent, helm: Helm::None, beard: false, hold: Gun::None, bulk: 0, short: 0, glow_eyes: None, skirt: false }
}

pub const HUMANOIDS: &[Look] = &[
    Look { helm: Helm::None, beard: true, hold: Gun::Rifle, ..look("player", SKIN, rgb(120, 70, 35), rgb(70, 95, 130), rgb(60, 55, 50), rgb(200, 170, 60)) },
    Look { helm: Helm::Winged, hold: Gun::Spear, ..look("valkyrie", SKIN, rgb(240, 215, 120), rgb(200, 205, 215), rgb(120, 40, 50), rgb(240, 240, 250)) },
    Look { beard: true, short: 4, bulk: 1, hold: Gun::Axe, helm: Helm::None, ..look("dwarf", rgb(225, 165, 130), rgb(190, 80, 40), rgb(110, 70, 40), rgb(70, 60, 55), rgb(220, 160, 50)) },
    Look { helm: Helm::Band, skirt: true, hold: Gun::Staff, ..look("singer", SKIN, rgb(230, 225, 215), rgb(235, 235, 235), rgb(200, 50, 60), rgb(60, 90, 190)) },
    Look { helm: Helm::Hood, skirt: true, ..look("volva", SKIN, rgb(30, 25, 45), rgb(95, 55, 150), rgb(60, 35, 100), rgb(90, 240, 220)) },
    Look { helm: Helm::Horned, beard: true, bulk: 2, hold: Gun::Axe, ..look("berserker", rgb(215, 160, 125), rgb(70, 40, 25), rgb(100, 70, 50), rgb(70, 50, 40), rgb(190, 190, 180)) },
    Look { beard: false, ..look("villager_m", SKIN, rgb(140, 100, 60), rgb(150, 120, 80), rgb(90, 75, 60), rgb(190, 60, 50)) },
    Look { skirt: true, helm: Helm::Band, ..look("villager_f", SKIN, rgb(190, 130, 60), rgb(80, 130, 150), rgb(120, 80, 70), rgb(240, 220, 120)) },
    Look { helm: Helm::Visor, hold: Gun::Rifle, bulk: 1, ..look("merc", SKIN, rgb(30, 30, 30), rgb(35, 38, 45), rgb(30, 32, 38), rgb(220, 60, 50)) },
    Look { glow_eyes: Some(rgb(110, 255, 230)), helm: Helm::Horned, hold: Gun::Axe, ..look("draugr", PALE, rgb(60, 70, 80), rgb(70, 90, 100), rgb(55, 65, 75), rgb(120, 190, 200)) },
    Look { bulk: 3, glow_eyes: Some(rgb(255, 220, 60)), ..look("troll", rgb(110, 130, 100), rgb(60, 70, 50), rgb(100, 85, 60), rgb(80, 70, 55), rgb(150, 150, 120)) },
    Look { helm: Helm::Antlers, skirt: true, glow_eyes: Some(rgb(80, 255, 255)), beard: false, hold: Gun::Staff, ..look("louhi", rgb(150, 190, 200), rgb(220, 240, 245), rgb(40, 70, 90), rgb(30, 50, 70), rgb(120, 230, 255)) },
    Look { bulk: 4, glow_eyes: Some(rgb(255, 120, 60)), beard: true, helm: Helm::Horned, hold: Gun::Axe, ..look("jotun", rgb(170, 205, 235), rgb(230, 240, 250), rgb(90, 120, 160), rgb(70, 95, 130), rgb(230, 240, 255)) },
    Look { helm: Helm::Hat, beard: true, hold: Gun::Staff, ..look("skald", SKIN, rgb(220, 220, 220), rgb(70, 70, 110), rgb(60, 60, 80), rgb(200, 170, 60)) },
    Look { helm: Helm::Crown, beard: true, ..look("jarl", SKIN, rgb(180, 120, 50), rgb(130, 40, 40), rgb(70, 55, 45), rgb(235, 190, 70)) },
    Look { helm: Helm::Visor, hold: Gun::Rifle, ..look("warden_guard", SKIN, rgb(90, 60, 40), rgb(60, 85, 70), rgb(50, 55, 50), rgb(210, 170, 60)) },
];

pub const CREATURES: &[(&str, Col, Col, i32)] = &[
    ("wolf", rgb(120, 120, 130), rgb(210, 210, 215), 0),
    ("fenrir", rgb(40, 40, 55), rgb(255, 60, 40), 3),
    ("ice_wolf", rgb(210, 225, 240), rgb(120, 220, 255), 1),
];

pub fn names() -> Vec<&'static str> {
    HUMANOIDS.iter().map(|l| l.name).chain(CREATURES.iter().map(|c| c.0)).collect()
}

// dir: 0=N 1=E 2=S 3=W. (W is drawn as flipped E.)
fn humanoid_side(l: &Look, dir: usize, frame: usize) -> Canvas {
    let mut c = Canvas::new(CELL, CELL);
    let flip = dir == 3;
    let d = if flip { 1 } else { dir };
    let (walk, attack, die) = match frame {
        1..=4 => (frame, 0, 0),
        5..=7 => (0, frame - 4, 0),
        8..=10 => (0, 0, frame - 7),
        _ => (0, 0, 0),
    };
    let phase = [0, 1, 0, -1, 0][walk.min(4)];
    let bob = if walk == 1 || walk == 3 { -1 } else { 0 };
    let top_y = 9 + l.short + bob;
    let bw = 8 + l.bulk;
    let bx = 12 - bw / 2;
    let leg_h = 5 - l.short / 2;
    let leg_y = top_y + 7;
    // legs
    let ls = if d == 1 { phase * 2 } else { 0 };
    let lw = 3 + l.bulk / 2;
    if l.skirt {
        c.rect(bx, leg_y - 2, bw, leg_h + 2, l.top);
        c.rect(bx, leg_y + leg_h - 2, bw, 1, l.accent);
        c.rect(bx + 1, leg_y + leg_h, 2, 1, l.bottom);
        c.rect(bx + bw - 3, leg_y + leg_h, 2, 1, l.bottom);
    } else if d == 1 {
        c.rect(12 - 2 + ls, leg_y, lw, leg_h, l.bottom);
        c.rect(12 - 2 - ls, leg_y, lw, leg_h, darken(l.bottom, 0.25));
        c.rect(12 - 2 + ls, leg_y + leg_h - 1, lw + 1, 1, rgb(40, 30, 25));
    } else {
        let off = if walk > 0 { phase } else { 0 };
        c.rect(bx + 1, leg_y + off.max(0), lw, leg_h - off.max(0), l.bottom);
        c.rect(bx + bw - 1 - lw, leg_y + (-off).max(0), lw, leg_h - (-off).max(0), darken(l.bottom, 0.15));
        c.rect(bx + 1, leg_y + leg_h - 1, lw, 1, rgb(40, 30, 25));
        c.rect(bx + bw - 1 - lw, leg_y + leg_h - 1, lw, 1, rgb(40, 30, 25));
    }
    // torso
    c.rect(bx, top_y, bw, 8, l.top);
    c.rect(bx, top_y + 6, bw, 1, l.accent); // belt / hem
    c.rect(bx, top_y, bw, 1, lighten(l.top, 0.25));
    if d == 2 {
        c.rect(12 - 1, top_y + 1, 2, 5, darken(l.top, 0.2)); // front seam
    }
    // arms
    let aw = 2 + l.bulk / 3;
    let swing = if d == 1 { phase } else { 0 };
    let arm_y = top_y + 1;
    let reach = match attack {
        1 => -3,
        2 => 4,
        3 => 2,
        _ => 0,
    };
    if d == 1 {
        c.rect(12 - 1 + swing, arm_y, aw, 6, darken(l.top, 0.15));
        c.rect(12 - 1 + swing, arm_y + 5, aw, 2, l.skin);
    } else {
        c.rect(bx - aw, arm_y, aw, 6, l.top);
        c.rect(bx + bw, arm_y, aw, 6, l.top);
        c.rect(bx - aw, arm_y + 5, aw, 2, l.skin);
        c.rect(bx + bw, arm_y + 5, aw, 2, l.skin);
    }
    // held item
    if l.hold != Gun::None {
        let hy = arm_y + 4;
        let hx = if d == 1 { 12 + 1 + swing + reach } else { bx + bw + aw };
        match (l.hold, d) {
            (Gun::Rifle, 1) => {
                c.rect(hx, hy - 1, 9, 2, rgb(45, 48, 55));
                c.rect(hx - 1, hy - 1, 3, 3, rgb(110, 75, 40));
                if attack == 2 {
                    c.rect(hx + 9, hy - 1, 3, 3, rgb(255, 230, 120));
                }
            }
            (Gun::Rifle, 2) => c.rect(hx - 1, hy - 4 + reach / 2, 2, 6, rgb(45, 48, 55)),
            (Gun::Rifle, _) => c.rect(hx - 1, hy - 3, 2, 5, rgb(45, 48, 55)),
            (Gun::Axe, 1) => {
                c.rect(hx, hy - 6, 1, 9, rgb(110, 75, 40));
                c.rect(hx, hy - 6, 4, 4, rgb(180, 185, 195));
            }
            (Gun::Axe, _) => {
                c.rect(hx, hy - 7, 1, 10, rgb(110, 75, 40));
                c.rect(hx - 1, hy - 8, 4, 3, rgb(180, 185, 195));
            }
            (Gun::Staff, _) => {
                c.rect(hx, hy - 11, 1, 16, rgb(120, 85, 50));
                c.rect(hx - 1, hy - 12, 3, 2, l.accent);
            }
            (Gun::Spear, _) => {
                c.rect(hx, hy - 12, 1, 18, rgb(150, 110, 60));
                c.tri((hx, hy - 15), (hx - 1, hy - 11), (hx + 1, hy - 11), rgb(210, 215, 225));
            }
            _ => {}
        }
    }
    // head
    let hy = top_y - 6;
    let hw = 6 + l.bulk;
    let hx = 12 - hw / 2;
    c.rect(hx, hy, hw, 6, l.skin);
    match d {
        0 => c.rect(hx, hy, hw, 6, l.hair),
        2 => {
            c.rect(hx, hy, hw, 2, l.hair);
            let eye = l.glow_eyes.unwrap_or(rgb(30, 30, 40));
            c.set(hx + 1, hy + 3, eye);
            c.set(hx + hw - 2, hy + 3, eye);
        }
        _ => {
            c.rect(hx, hy, hw, 2, l.hair);
            c.rect(hx, hy, 2, 5, l.hair);
            c.set(hx + hw - 2, hy + 3, l.glow_eyes.unwrap_or(rgb(30, 30, 40)));
            c.set(hx + hw, hy + 4, darken(l.skin, 0.15));
        }
    }
    if l.beard && d != 0 {
        c.rect(hx, hy + 4, hw, 3 + l.bulk / 2, l.hair);
        if d == 2 {
            c.set(hx + hw / 2, hy + 3, darken(l.skin, 0.2));
        }
    }
    match l.helm {
        Helm::Horned => {
            c.rect(hx - 1, hy - 1, hw + 2, 3, rgb(160, 165, 175));
            c.rect(hx - 2, hy - 3, 1, 4, rgb(240, 235, 215));
            c.rect(hx + hw + 1, hy - 3, 1, 4, rgb(240, 235, 215));
        }
        Helm::Winged => {
            c.rect(hx - 1, hy - 1, hw + 2, 3, rgb(210, 215, 225));
            c.tri((hx - 1, hy - 1), (hx - 4, hy - 4), (hx - 1, hy + 1), rgb(250, 250, 255));
            c.tri((hx + hw, hy - 1), (hx + hw + 3, hy - 4), (hx + hw, hy + 1), rgb(250, 250, 255));
        }
        Helm::Hood => {
            c.rect(hx - 1, hy - 1, hw + 2, 8, l.top);
            if d == 2 {
                c.rect(hx + 1, hy + 2, hw - 2, 3, l.skin);
                c.set(hx + 1, hy + 3, l.glow_eyes.unwrap_or(l.accent));
                c.set(hx + hw - 2, hy + 3, l.glow_eyes.unwrap_or(l.accent));
            }
        }
        Helm::Band => c.rect(hx, hy + 1, hw, 1, l.accent),
        Helm::Visor => {
            c.rect(hx - 1, hy - 1, hw + 2, 3, rgb(45, 48, 55));
            if d != 0 {
                c.rect(hx, hy + 2, hw, 2, rgb(220, 60, 50));
            }
        }
        Helm::Antlers => {
            c.line(hx, hy, hx - 3, hy - 5, l.accent);
            c.line(hx + hw - 1, hy, hx + hw + 2, hy - 5, l.accent);
            c.line(hx - 2, hy - 3, hx - 4, hy - 3, l.accent);
            c.line(hx + hw + 1, hy - 3, hx + hw + 3, hy - 3, l.accent);
        }
        Helm::Hat => {
            c.rect(hx - 2, hy - 1, hw + 4, 2, l.top);
            c.rect(hx, hy - 4, hw, 3, l.top);
        }
        Helm::Crown => {
            c.rect(hx, hy - 2, hw, 2, l.accent);
            for i in (0..hw).step_by(2) {
                c.set(hx + i, hy - 3, l.accent);
            }
        }
        Helm::None => {}
    }
    // death poses
    if die > 0 {
        let mut body = c.clone();
        if die >= 2 {
            body = body.rot90();
            if die == 3 {
                body.tint(rgb(60, 20, 20), 0.25);
            }
        }
        let mut o = Canvas::new(CELL, CELL);
        o.blit(&body, 0, if die == 1 { 3 } else { 5 });
        c = o;
    }
    if flip {
        c = c.flip_h();
    }
    c.outline(OUT);
    c
}

fn creature(name: &str, body: Col, eye: Col, size: i32, dir: usize, frame: usize) -> Canvas {
    let mut c = Canvas::new(CELL, CELL);
    let flip = dir == 3;
    let d = if flip { 1 } else { dir };
    let step: i32 = if (1..=4).contains(&frame) { [0, 2, 0, -2][frame - 1] } else { 0 };
    let lunge = if (5..=7).contains(&frame) { (frame as i32 - 4) * 2 } else { 0 };
    let sz = size;
    let dk = darken(body, 0.35);
    if d == 1 {
        c.rect(5 - sz, 11 - sz, 13 + sz, 6 + sz, body);
        c.rect(5 - sz, 11 - sz, 13 + sz, 2, lighten(body, 0.2));
        for (x, off) in [(6, step), (9, -step), (15, -step), (18, step)] {
            c.rect(x, 16 + sz / 2, 2, 5 - sz / 2 + (off / 2).abs().min(1), dk);
        }
        c.rect(1 - sz, 10 - sz, 5, 3, body); // tail
        c.rect(17 + lunge / 2, 8 - sz, 6 + sz, 6 + sz, body);
        c.tri((19 + lunge / 2, 8 - sz), (18 + lunge / 2, 5 - sz), (21 + lunge / 2, 8 - sz), dk);
        c.rect(21 + sz + lunge / 2, 12 - sz, 3, 2, dk);
        c.set(21 + lunge / 2, 10 - sz, eye);
    } else {
        c.rect(7 - sz, 8, 10 + sz * 2, 10, body);
        c.rect(8 - sz / 2, 3 + lunge / 2, 8 + sz, 7, body);
        c.tri((8 - sz / 2, 3), (7 - sz / 2, 0), (11, 3), dk);
        c.tri((16 + sz / 2, 3), (17 + sz / 2, 0), (13, 3), dk);
        if d == 2 {
            c.set(10, 6 + lunge / 2, eye);
            c.set(14, 6 + lunge / 2, eye);
            c.rect(11, 8 + lunge / 2, 2, 2, dk);
        }
        c.rect(8, 17, 3, 4 + step.abs() / 2, dk);
        c.rect(13, 17, 3, 4, dk);
    }
    let _ = name;
    if (8..=10).contains(&frame) {
        let mut o = Canvas::new(CELL, CELL);
        o.blit(&c.rot90(), 0, if frame == 8 { 2 } else { 4 });
        c = o;
    }
    if flip {
        c = c.flip_h();
    }
    c.outline(OUT);
    c
}

pub fn frame(arch: usize, dir: usize, frame: usize) -> Canvas {
    if arch < HUMANOIDS.len() {
        humanoid_side(&HUMANOIDS[arch], dir, frame)
    } else {
        let (n, b, e, s) = CREATURES[arch - HUMANOIDS.len()];
        creature(n, b, e, s, dir, frame)
    }
}

pub fn sheet(atlas: &mut Atlas) -> Canvas {
    let n = names();
    let mut s = Canvas::new(CELL * COLS, CELL * 4 * n.len() as i32);
    for (a, name) in n.iter().enumerate() {
        for dir in 0..4 {
            let row = a * 4 + dir;
            for f in 0..COLS as usize {
                s.blit(&frame(a, dir, f), f as i32 * CELL, row as i32 * CELL);
            }
            let dn = ["n", "e", "s", "w"][dir];
            atlas.add(&format!("{name}_{dn}"), 0, row as i32 * CELL, CELL * COLS, CELL);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_frames_draw_and_walk_differs() {
        for a in 0..names().len() {
            for d in 0..4 {
                for f in 0..COLS as usize {
                    assert!(frame(a, d, f).px.iter().any(|p| p[3] > 0), "arch {a} dir {d} frame {f}");
                }
            }
        }
        assert_ne!(frame(0, 1, 1).px, frame(0, 1, 3).px);
        assert_eq!(frame(0, 1, 0).flip_h().px.len(), frame(0, 3, 0).px.len());
    }
}
