//! UI atlas: panels (9-slice), slots, buttons, bars, paperdoll, cursors, runic glyphs.
use crate::canvas::*;
use crate::manifest::Atlas;

const BORDER: Col = rgb(26, 18, 14);
const PANEL: Col = rgb(62, 44, 30);
const PANEL_L: Col = rgb(98, 72, 48);
const PANEL_D: Col = rgb(40, 28, 20);
const GOLD: Col = rgb(224, 180, 80);
const INK: Col = rgb(235, 220, 190);

pub const FUTHARK: usize = 24;

pub fn panel(w: i32, h: i32) -> Canvas {
    let mut c = Canvas::filled(w, h, PANEL);
    for y in 0..h {
        for x in 0..w {
            if noise(x / 2, y / 2, 5) > 0.7 {
                c.set(x, y, mix(PANEL, PANEL_D, 0.5));
            }
        }
    }
    c.frame(0, 0, w, h, BORDER);
    c.frame(1, 1, w - 2, h - 2, GOLD);
    c.frame(2, 2, w - 4, h - 4, PANEL_D);
    c.rect(3, 3, w - 6, 1, PANEL_L);
    for (x, y) in [(3, 3), (w - 5, 3), (3, h - 5), (w - 5, h - 5)] {
        c.rect(x, y, 2, 2, GOLD);
    }
    c
}

pub fn slot() -> Canvas {
    let mut c = Canvas::filled(18, 18, rgb(30, 22, 18));
    c.frame(0, 0, 18, 18, BORDER);
    c.frame(1, 1, 16, 16, rgb(90, 70, 50));
    c.rect(2, 2, 14, 1, rgb(18, 12, 10));
    c
}

pub fn button(pressed: bool) -> Canvas {
    let mut c = Canvas::filled(48, 16, if pressed { PANEL_D } else { PANEL_L });
    c.frame(0, 0, 48, 16, BORDER);
    c.rect(1, 1, 46, 1, if pressed { PANEL } else { lighten(PANEL_L, 0.3) });
    c.rect(1, 14, 46, 1, if pressed { PANEL_L } else { PANEL_D });
    c
}

pub fn bar(fill: Option<Col>) -> Canvas {
    let mut c = Canvas::new(64, 8);
    match fill {
        None => {
            c.rect(0, 0, 64, 8, rgb(20, 14, 12));
            c.frame(0, 0, 64, 8, GOLD);
        }
        Some(col) => {
            c.rect(0, 0, 64, 8, col);
            c.rect(0, 0, 64, 2, lighten(col, 0.35));
            c.rect(0, 6, 64, 2, darken(col, 0.3));
        }
    }
    c
}

/// Paperdoll silhouette 80x96 with slot rects (positions listed in docs/ui_layout in the manifest notes).
pub fn paperdoll() -> Canvas {
    let mut c = Canvas::filled(80, 96, rgb(34, 24, 18));
    c.frame(0, 0, 80, 96, BORDER);
    let sil = rgb(52, 40, 32);
    c.ellipse(40, 22, 8, 9, sil);
    c.rect(28, 32, 24, 28, sil);
    c.rect(22, 33, 6, 24, sil);
    c.rect(52, 33, 6, 24, sil);
    c.rect(30, 60, 8, 28, sil);
    c.rect(42, 60, 8, 28, sil);
    c
}

/// Slot positions on the paperdoll sheet (x, y) in 18x18 cells; the game places Slot frames here.
pub const PAPERDOLL_SLOTS: &[(&str, i32, i32)] = &[
    ("head", 31, 4),
    ("neck", 52, 8),
    ("torso", 31, 34),
    ("back", 6, 8),
    ("hand_l", 6, 36),
    ("hand_r", 56, 36),
    ("ring_l", 6, 60),
    ("ring_r", 56, 60),
    ("legs", 31, 58),
    ("feet", 31, 76),
    ("ammo", 56, 76),
];

pub fn cursor(kind: &str) -> Canvas {
    let mut c = Canvas::new(16, 16);
    match kind {
        "arrow" => {
            c.tri((1, 1), (1, 12), (9, 9), INK);
            c.line(5, 9, 9, 14, INK);
        }
        "hand" => {
            c.rect(5, 6, 7, 7, INK);
            for x in [5, 7, 9, 11] {
                c.rect(x, 2, 1, 5, INK);
            }
            c.rect(2, 8, 3, 2, INK);
        }
        "target" => {
            c.ellipse(8, 8, 6, 6, rgb(230, 60, 50));
            c.ellipse(8, 8, 4, 4, CLEAR);
            c.line(8, 0, 8, 15, rgb(230, 60, 50));
            c.line(0, 8, 15, 8, rgb(230, 60, 50));
        }
        _ => {
            c.ellipse(8, 8, 5, 5, INK);
            c.ellipse(8, 8, 2, 2, CLEAR);
        }
    }
    c.outline(BORDER);
    c
}

pub fn portrait_frame() -> Canvas {
    let mut c = Canvas::new(48, 48);
    c.frame(0, 0, 48, 48, BORDER);
    c.frame(1, 1, 46, 46, GOLD);
    c.frame(2, 2, 44, 44, PANEL_D);
    c
}

/// Elder Futhark inspired glyphs (decorative runic font), 12x16 each.
pub fn rune(i: usize) -> Canvas {
    let mut c = Canvas::new(12, 16);
    let col = rgb(110, 240, 255);
    c.line(6, 1, 6, 14, col); // staff
    match i % 24 {
        0 => c.line(6, 1, 10, 5, col),
        1 => {
            c.line(6, 1, 10, 5, col);
            c.line(10, 5, 6, 9, col);
        }
        2 => {
            c.line(6, 1, 10, 5, col);
            c.line(6, 5, 10, 9, col);
        }
        3 => {
            c.line(2, 1, 6, 5, col);
            c.line(2, 5, 6, 9, col);
        }
        4 => {
            c.line(6, 1, 2, 5, col);
            c.line(6, 1, 10, 5, col);
        }
        5 => c.line(2, 14, 10, 1, col),
        6 => {
            c.line(2, 1, 10, 14, col);
            c.line(10, 1, 2, 14, col);
        }
        7 => {
            c.line(2, 1, 2, 14, col);
            c.line(10, 1, 10, 14, col);
            c.line(2, 4, 10, 10, col);
        }
        8 => c.line(6, 4, 10, 8, col),
        9 => {
            c.line(2, 8, 6, 4, col);
            c.line(6, 12, 10, 8, col);
        }
        10 => c.line(2, 5, 10, 11, col),
        11 => {
            c.line(2, 2, 10, 8, col);
            c.line(10, 8, 2, 14, col);
        }
        12 => {
            c.line(6, 1, 2, 7, col);
            c.line(6, 1, 10, 7, col);
            c.line(2, 7, 6, 13, col);
            c.line(10, 7, 6, 13, col);
        }
        13 => {
            c.line(2, 1, 10, 7, col);
            c.line(2, 14, 10, 8, col);
        }
        14 => {
            c.line(2, 4, 6, 1, col);
            c.line(10, 4, 6, 1, col);
            c.line(2, 4, 10, 12, col);
        }
        15 => {
            c.line(2, 1, 6, 6, col);
            c.line(10, 1, 6, 6, col);
        }
        16 => {
            c.line(2, 14, 6, 9, col);
            c.line(10, 14, 6, 9, col);
        }
        17 => {
            c.line(2, 1, 6, 7, col);
            c.line(6, 7, 2, 14, col);
        }
        18 => {
            c.line(10, 1, 6, 7, col);
            c.line(6, 7, 10, 14, col);
        }
        19 => {
            c.line(2, 6, 10, 2, col);
            c.line(2, 12, 10, 8, col);
        }
        20 => {
            c.line(2, 4, 10, 4, col);
            c.line(2, 11, 10, 11, col);
        }
        21 => {
            c.line(6, 1, 2, 5, col);
            c.line(6, 1, 10, 5, col);
            c.line(6, 14, 2, 10, col);
        }
        22 => {
            c.line(2, 8, 6, 4, col);
            c.line(10, 8, 6, 4, col);
            c.line(2, 8, 6, 12, col);
            c.line(10, 8, 6, 12, col);
        }
        _ => {
            c.line(2, 1, 10, 5, col);
            c.line(10, 5, 2, 9, col);
        }
    }
    c
}

pub fn sheet(atlas: &mut Atlas) -> Canvas {
    let mut s = Canvas::new(256, 320);
    let put = |s: &mut Canvas, a: &mut Atlas, name: &str, c: Canvas, x: i32, y: i32| {
        s.blit(&c, x, y);
        a.add(name, x, y, c.w, c.h);
    };
    put(&mut s, atlas, "panel", panel(48, 48), 0, 0);
    put(&mut s, atlas, "dialogue_box", panel(96, 48), 52, 0);
    put(&mut s, atlas, "slot", slot(), 152, 0);
    put(&mut s, atlas, "button", button(false), 172, 0);
    put(&mut s, atlas, "button_pressed", button(true), 172, 18);
    put(&mut s, atlas, "bar_frame", bar(None), 0, 52);
    put(&mut s, atlas, "bar_hp", bar(Some(rgb(200, 50, 50))), 70, 52);
    put(&mut s, atlas, "bar_mana", bar(Some(rgb(60, 110, 230))), 0, 62);
    put(&mut s, atlas, "bar_stamina", bar(Some(rgb(90, 190, 80))), 70, 62);
    put(&mut s, atlas, "paperdoll", paperdoll(), 0, 72);
    put(&mut s, atlas, "portrait_frame", portrait_frame(), 84, 72);
    for (i, k) in ["arrow", "hand", "target", "look"].iter().enumerate() {
        put(&mut s, atlas, &format!("cursor_{k}"), cursor(k), 84 + i as i32 * 18, 124);
    }
    for i in 0..FUTHARK {
        put(&mut s, atlas, &format!("rune_{i}"), rune(i), (i as i32 % 16) * 14, 176 + (i as i32 / 16) * 18);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ui_sheet_has_entries_in_bounds() {
        let mut a = Atlas::new("x");
        let s = sheet(&mut a);
        for e in &a.entries {
            assert!(e.x >= 0 && e.y >= 0 && e.x + e.w <= s.w && e.y + e.h <= s.h, "{}", e.name);
        }
        assert!(a.entries.len() > 30);
    }
}
