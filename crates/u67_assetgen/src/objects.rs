//! Object sprites, 32x32 cells, up to 4 frames per object (row per object).
use crate::canvas::*;
use crate::manifest::Atlas;
use u67_world::objects::OBJECTS;

pub const S: i32 = 32;
const OUT: Col = rgb(20, 14, 18);
const WOOD: Col = rgb(120, 78, 42);
const WOOD_D: Col = rgb(80, 50, 28);
const WOOD_L: Col = rgb(165, 115, 66);
const IRON: Col = rgb(110, 115, 125);
const IRON_D: Col = rgb(60, 62, 72);
const GOLD: Col = rgb(235, 190, 70);
const STONE: Col = rgb(130, 132, 140);

fn pine(snow: bool) -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(14, 22, 4, 8, WOOD_D);
    let g = [rgb(30, 90, 55), rgb(40, 110, 62), rgb(55, 130, 70)];
    for (i, col) in g.iter().enumerate() {
        let y = 20 - i as i32 * 7;
        let w = 11 - i as i32 * 2;
        c.tri((16, y - 8), (16 - w, y + 3), (16 + w, y + 3), *col);
        if snow {
            c.tri((16, y - 8), (16 - w / 2, y - 2), (16 + w / 2, y - 2), rgb(235, 242, 250));
        }
    }
    c.outline(OUT);
    c
}

fn birch() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(14, 14, 4, 16, rgb(225, 225, 215));
    for y in (16..29).step_by(3) {
        c.rect(14, y, 2, 1, rgb(40, 40, 40));
    }
    c.ellipse(16, 10, 10, 8, rgb(130, 170, 60));
    c.ellipse(12, 9, 5, 4, rgb(165, 200, 80));
    c.ellipse(21, 12, 4, 3, rgb(100, 145, 50));
    c.outline(OUT);
    c
}

fn runestone(glow: bool) -> Canvas {
    let mut c = Canvas::new(S, S);
    c.ellipse(16, 16, 7, 13, STONE);
    c.rect(9, 18, 14, 12, STONE);
    c.rect(18, 8, 5, 22, darken(STONE, 0.2));
    let rune = if glow { rgb(110, 240, 255) } else { rgb(60, 70, 90) };
    c.line(16, 8, 16, 24, rune);
    c.line(16, 11, 20, 8, rune);
    c.line(16, 15, 20, 12, rune);
    c.line(16, 20, 12, 24, rune);
    c.line(12, 14, 16, 17, rune);
    c.outline(OUT);
    c
}

fn roof() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.tri((16, 4), (0, 22), (31, 22), rgb(110, 70, 40));
    c.rect(0, 20, 32, 6, rgb(90, 55, 32));
    for x in (2..30).step_by(4) {
        c.line(x, 20, x + 2, 25, darken(rgb(90, 55, 32), 0.3));
    }
    // dragon-head finials
    c.rect(15, 2, 2, 5, GOLD);
    c.set(14, 2, GOLD);
    c.set(17, 2, GOLD);
    c.outline(OUT);
    c
}

fn door(open: bool) -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(8, 6, 16, 24, WOOD_D);
    if open {
        c.rect(10, 8, 12, 22, rgb(25, 18, 14));
        c.rect(6, 8, 4, 22, WOOD);
    } else {
        c.rect(10, 8, 12, 22, WOOD);
        for x in [13, 17, 21] {
            c.rect(x, 8, 1, 22, WOOD_D);
        }
        c.rect(10, 14, 12, 2, IRON);
        c.rect(10, 24, 12, 2, IRON);
        c.set(20, 19, GOLD);
    }
    c.outline(OUT);
    c
}

fn chest(open: bool) -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(6, 16, 20, 11, WOOD);
    c.rect(6, 16, 20, 2, WOOD_L);
    if open {
        c.rect(6, 8, 20, 6, WOOD_D);
        c.rect(8, 10, 16, 3, rgb(240, 200, 80));
    } else {
        c.rect(6, 10, 20, 7, WOOD_L);
        c.rect(6, 10, 20, 1, WOOD);
    }
    c.rect(6, 20, 20, 2, IRON);
    c.rect(15, 15, 3, 5, GOLD);
    c.outline(OUT);
    c
}

fn bed() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(4, 14, 24, 12, WOOD);
    c.rect(5, 15, 22, 9, rgb(170, 60, 60));
    c.rect(5, 15, 7, 9, rgb(235, 235, 225));
    c.rect(4, 12, 3, 16, WOOD_D);
    c.outline(OUT);
    c
}

fn table() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(4, 14, 24, 6, WOOD_L);
    c.rect(4, 19, 24, 2, WOOD_D);
    c.rect(6, 21, 3, 8, WOOD_D);
    c.rect(23, 21, 3, 8, WOOD_D);
    c.rect(12, 11, 4, 3, rgb(210, 210, 215)); // mug
    c.outline(OUT);
    c
}

fn barrel() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.ellipse(16, 17, 9, 12, WOOD);
    c.rect(8, 8, 16, 18, WOOD);
    for y in [11, 21] {
        c.rect(7, y, 18, 2, IRON);
    }
    c.rect(11, 8, 2, 18, WOOD_L);
    c.outline(OUT);
    c
}

/// Cannon: frame 0=N 1=E 2=S 3=W.
fn cannon(dir: usize) -> Canvas {
    let mut c = Canvas::new(S, S);
    let wheel = |c: &mut Canvas, x: i32, y: i32| {
        c.ellipse(x, y, 4, 4, WOOD_D);
        c.ellipse(x, y, 2, 2, WOOD_L);
    };
    match dir {
        1 | 3 => {
            c.rect(6, 18, 20, 4, WOOD);
            wheel(&mut c, 12, 23);
            wheel(&mut c, 21, 23);
            c.rect(7, 12, 18, 7, rgb(45, 48, 55));
            c.rect(7, 12, 18, 2, rgb(95, 100, 112));
            c.rect(24, 11, 3, 9, rgb(35, 37, 44));
            c.set(11, 11, GOLD);
            if dir == 3 {
                c = c.flip_h();
            }
        }
        0 => {
            wheel(&mut c, 8, 22);
            wheel(&mut c, 24, 22);
            c.rect(10, 8, 12, 18, WOOD);
            c.rect(12, 4, 8, 20, rgb(45, 48, 55));
            c.rect(12, 4, 3, 20, rgb(95, 100, 112));
            c.rect(11, 3, 10, 3, rgb(30, 32, 38));
        }
        _ => {
            wheel(&mut c, 8, 22);
            wheel(&mut c, 24, 22);
            c.rect(10, 12, 12, 14, WOOD);
            c.ellipse(16, 14, 5, 6, rgb(45, 48, 55));
            c.ellipse(16, 15, 2, 3, rgb(10, 10, 12));
            c.rect(13, 8, 6, 3, rgb(95, 100, 112));
        }
    }
    c.outline(OUT);
    c
}

/// Longship: frame 0=N 1=E 2=S 3=W (N/S show the bow/stern end-on).
fn longship(dir: usize) -> Canvas {
    let mut c = Canvas::new(S, S);
    let hull = rgb(150, 98, 52);
    match dir {
        1 | 3 => {
            c.rect(2, 20, 28, 5, hull);
            c.tri((2, 25), (2, 20), (6, 25), hull);
            c.tri((29, 25), (29, 20), (25, 25), hull);
            c.rect(2, 20, 28, 1, WOOD_L);
            for x in (5..28).step_by(4) {
                c.ellipse(x, 22, 1, 1, rgb(200, 60, 50)); // shields
            }
            c.rect(15, 4, 2, 17, WOOD_D);
            for i in 0..4 {
                let col = if i % 2 == 0 { rgb(200, 50, 45) } else { rgb(240, 235, 220) };
                c.rect(8, 5 + i * 3, 16, 3, col);
            }
            c.rect(28, 14, 2, 7, hull); // prow
            c.rect(29, 12, 2, 3, GOLD);
            if dir == 3 {
                c = c.flip_h();
            }
        }
        _ => {
            c.rect(10, 8, 12, 20, hull);
            c.tri((10, 8), (22, 8), (16, 2), hull);
            c.rect(15, 4, 2, 18, WOOD_D);
            c.rect(6, 10, 20, 8, rgb(240, 235, 220));
            c.rect(6, 12, 20, 2, rgb(200, 50, 45));
            if dir == 0 {
                c.rect(15, 0, 2, 4, GOLD);
            }
        }
    }
    c.outline(OUT);
    c
}

fn sled(dir: usize) -> Canvas {
    let mut c = Canvas::new(S, S);
    let _ = dir;
    c.rect(4, 22, 24, 2, IRON);
    c.rect(4, 18, 24, 4, WOOD);
    c.rect(8, 14, 10, 4, rgb(170, 60, 60));
    c.line(4, 22, 2, 19, IRON);
    c.outline(OUT);
    c
}

fn campfire(f: i32) -> Canvas {
    let mut c = Canvas::new(S, S);
    c.line(8, 26, 24, 22, WOOD_D);
    c.line(8, 22, 24, 26, WOOD);
    let h = 8 + (f % 3) * 2;
    c.tri((16, 24 - h), (10, 24), (22, 24), rgb(235, 90, 25));
    c.tri((16, 24 - h + 4), (12, 24), (20, 24), rgb(255, 190, 50));
    c.tri((16, 24 - h + 8), (14, 24), (18, 24), rgb(255, 245, 160));
    c.outline(OUT);
    c
}

fn forge(f: i32) -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(5, 12, 22, 16, rgb(95, 92, 100));
    c.rect(9, 17, 14, 8, if f == 0 { rgb(30, 20, 20) } else { rgb(255, 130, 30) });
    c.rect(12, 4, 8, 9, rgb(75, 72, 80));
    c.rect(2, 22, 6, 3, IRON_D); // anvil
    c.outline(OUT);
    c
}

fn well() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.ellipse(16, 22, 10, 6, STONE);
    c.ellipse(16, 21, 7, 4, rgb(40, 90, 160));
    c.rect(7, 6, 2, 16, WOOD_D);
    c.rect(23, 6, 2, 16, WOOD_D);
    c.rect(6, 4, 20, 3, WOOD);
    c.outline(OUT);
    c
}

fn boulder() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.ellipse(16, 20, 10, 8, rgb(105, 105, 115));
    c.ellipse(13, 17, 5, 3, rgb(140, 140, 150));
    c.outline(OUT);
    c
}

fn bifrost(f: i32) -> Canvas {
    let mut c = Canvas::new(S, S);
    let cols = [rgb(255, 80, 80), rgb(255, 200, 60), rgb(90, 230, 120), rgb(80, 170, 255), rgb(190, 100, 255)];
    c.ellipse(16, 24, 12, 5, rgb(40, 30, 70));
    for (i, col) in cols.iter().enumerate() {
        let h = 6 + ((i as i32 + f) % 3) * 3;
        c.rect(7 + i as i32 * 4, 22 - h - 6, 3, h + 6, *col);
    }
    c.ellipse(16, 22, 9, 3, rgb(255, 255, 255));
    c.outline(OUT);
    c
}

fn crate_() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(6, 10, 20, 18, WOOD_L);
    c.frame(6, 10, 20, 18, WOOD_D);
    c.line(6, 10, 25, 27, WOOD_D);
    c.line(25, 10, 6, 27, WOOD_D);
    c.outline(OUT);
    c
}

fn signpost() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(15, 8, 3, 22, WOOD_D);
    c.rect(6, 6, 20, 8, WOOD_L);
    c.rect(8, 9, 4, 1, WOOD_D);
    c.rect(14, 9, 8, 1, WOOD_D);
    c.outline(OUT);
    c
}

fn wreck() -> Canvas {
    let mut c = Canvas::new(S, S);
    c.rect(4, 20, 24, 5, rgb(90, 70, 50));
    c.tri((4, 25), (4, 14), (10, 25), rgb(90, 70, 50));
    c.rect(18, 8, 2, 14, WOOD_D);
    c.rect(18, 8, 8, 6, rgb(150, 145, 130));
    c.line(8, 22, 12, 28, IRON_D);
    c.outline(OUT);
    c
}

pub fn frames_of(id: &str, f: usize) -> Canvas {
    match id {
        "pine_tree" => pine(false),
        "birch_tree" => birch(),
        "runestone" => runestone(f == 1),
        "longhouse_roof" => roof(),
        "door_wood" => door(f == 1),
        "chest" => chest(f == 1),
        "bed" => bed(),
        "table" => table(),
        "barrel" => barrel(),
        "cannon" => cannon(f),
        "longship" => longship(f),
        "sled" => sled(f),
        "campfire" => campfire(f as i32),
        "forge" => forge((f % 2) as i32),
        "well" => well(),
        "boulder" => boulder(),
        "bifrost_node" => bifrost(f as i32),
        "crate" => crate_(),
        "signpost" => signpost(),
        "wreck" => wreck(),
        _ => {
            let mut c = Canvas::new(S, S);
            c.rect(8, 8, 16, 16, rgb(255, 0, 255));
            c
        }
    }
}

/// Snowy pine variant is added as an extra row (not in the object table).
pub fn sheet(atlas: &mut Atlas) -> Canvas {
    let rows = OBJECTS.len() as i32 + 1;
    let mut s = Canvas::new(S * 4, S * rows);
    for (row, d) in OBJECTS.iter().enumerate() {
        for f in 0..d.frames as usize {
            s.blit(&frames_of(d.id, f), f as i32 * S, row as i32 * S);
        }
        atlas.add(d.id, 0, row as i32 * S, S * d.frames as i32, S);
    }
    s.blit(&pine(true), 0, OBJECTS.len() as i32 * S);
    atlas.add("pine_tree_snow", 0, OBJECTS.len() as i32 * S, S, S);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_object_frame_draws_something() {
        for d in OBJECTS {
            for f in 0..d.frames as usize {
                let c = frames_of(d.id, f);
                assert!(c.px.iter().any(|p| p[3] > 0), "{} frame {f} empty", d.id);
                assert!(!c.px.iter().any(|p| *p == rgb(255, 0, 255)), "{} missing drawer", d.id);
            }
        }
    }
}
