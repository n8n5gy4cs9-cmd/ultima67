//! Terrain tileset: 16x16 tiles, 4 variants each. Row = tile id, column = variant.
use crate::canvas::*;
use crate::manifest::Atlas;
use u67_world::tiles::{self, TILES};

pub const T: i32 = 16;
pub const VARIANTS: i32 = 4;

fn jitter(c: Col, x: i32, y: i32, seed: u64, amt: f32) -> Col {
    let n = noise(x, y, seed) - 0.5;
    if n > 0.0 { lighten(c, n * amt) } else { darken(c, -n * amt) }
}

pub fn tile(id: usize, variant: i32) -> Canvas {
    let d = &TILES[id];
    let base = rgb(d.color[0], d.color[1], d.color[2]);
    let seed = id as u64 * 131 + variant as u64 * 7919 + 1;
    let mut c = Canvas::new(T, T);
    let dark = darken(base, 0.35);
    let light = lighten(base, 0.35);
    // base fill with chunky 2x2 noise
    for y in 0..T {
        for x in 0..T {
            c.set(x, y, jitter(base, x / 2, y / 2, seed, 0.28));
        }
    }
    let r = |i: i32, m: i32| (noise(i, variant, seed + 77) * m as f32) as i32;
    let t = tiles::TileId(id as u16);
    match t {
        tiles::GRASS | tiles::FOREST_FLOOR | tiles::SWAMP => {
            for i in 0..7 {
                let (x, y) = (r(i, T - 2) + 1, r(i + 50, T - 3) + 2);
                c.set(x, y, light);
                c.set(x, y - 1, light);
                c.set(x + 1, y, dark);
            }
            if t == tiles::FOREST_FLOOR {
                for i in 0..4 {
                    c.rect(r(i + 9, T - 2), r(i + 19, T - 2), 2, 1, rgb(150, 100, 40));
                }
            }
            if t == tiles::SWAMP {
                c.rect(r(1, 10), r(2, 10), 5, 3, rgb(50, 80, 90));
            }
        }
        tiles::SNOW | tiles::AURORA_ICE => {
            for i in 0..5 {
                c.set(r(i, T), r(i + 30, T), rgb(255, 255, 255));
            }
            if t == tiles::AURORA_ICE {
                for x in 0..T {
                    let g = ((x as f32 * 0.5 + variant as f32).sin() * 0.5 + 0.5) * 0.5;
                    for y in 0..T {
                        if (y + x / 2) % 5 == 0 {
                            c.set(x, y, mix(c.get(x, y), rgb(120, 255, 150), g));
                        }
                    }
                }
            }
        }
        tiles::ICE => {
            c.line(r(1, T), 0, r(2, T), T - 1, light);
            c.line(0, r(3, T), T - 1, r(4, T), lighten(base, 0.6));
        }
        tiles::ROCK | tiles::REGOLITH | tiles::ASH | tiles::MARS_DUST => {
            for i in 0..4 {
                let (x, y) = (r(i, T - 3), r(i + 9, T - 3));
                c.line(x, y, x + 2, y + 1, dark);
            }
            if t == tiles::REGOLITH {
                for i in 0..2 {
                    let (x, y) = (r(i + 3, T - 5) + 2, r(i + 7, T - 5) + 2);
                    c.ellipse(x, y, 2, 1, darken(base, 0.25));
                    c.set(x - 1, y - 1, light);
                }
            }
        }
        tiles::MOUNTAIN => {
            for i in 0..2 {
                let x = 3 + i * 8 + r(i, 3);
                c.tri((x, 3), (x - 4, 14), (x + 4, 14), darken(base, 0.1));
                c.tri((x, 3), (x, 14), (x + 4, 14), darken(base, 0.35));
                c.tri((x, 3), (x - 2, 7), (x + 2, 7), rgb(240, 245, 255));
            }
        }
        tiles::SAND => {
            for y in (2..T).step_by(4) {
                for x in 0..T {
                    if (x + y + variant) % 3 != 0 {
                        c.set(x, y + (x / 5) % 2, darken(base, 0.12));
                    }
                }
            }
        }
        tiles::WATER_SHALLOW | tiles::WATER_DEEP => {
            for i in 0..3 {
                let (x, y) = (r(i, T - 5), r(i + 20, T));
                c.rect(x, y, 4, 1, lighten(base, 0.45));
                c.rect(x + 1, y + 1, 2, 1, darken(base, 0.2));
            }
        }
        tiles::LAVA => {
            for i in 0..3 {
                let (x, y) = (r(i, T - 5), r(i + 20, T));
                c.line(x, y, x + 4, y + 1, rgb(255, 220, 60));
            }
            c.rect(r(9, 12), r(10, 12), 3, 2, rgb(70, 20, 10));
        }
        tiles::GAS => {
            for y in 0..T {
                for x in 0..T {
                    let v = vnoise(x as f32 / 4.0 + variant as f32 * 3.0, y as f32 / 2.5, seed);
                    c.set(x, y, mix(darken(base, 0.2), lighten(base, 0.3), v));
                }
            }
        }
        tiles::ROAD => {
            for i in 0..6 {
                c.set(r(i, T), r(i + 11, T), dark);
                c.set(r(i + 5, T), r(i + 31, T), light);
            }
        }
        tiles::FLOOR_WOOD => {
            for x in (0..T).step_by(4) {
                c.rect(x, 0, 1, T, dark);
            }
            for i in 0..4 {
                c.rect((i * 4 + variant) % T, r(i, T), 1, 2, darken(base, 0.5));
            }
        }
        tiles::FLOOR_STONE => {
            c.rect(0, 7, T, 1, dark);
            c.rect(7, 0, 1, 8, dark);
            c.rect(3, 8, 1, 8, dark);
            c.rect(12, 8, 1, 8, dark);
        }
        tiles::WALL_WOOD => {
            for y in (0..T).step_by(4) {
                c.rect(0, y, T, 1, darken(base, 0.5));
                c.rect(0, y + 1, T, 1, lighten(base, 0.15));
            }
        }
        tiles::WALL_STONE => {
            for (row, y) in (0..T).step_by(5).enumerate() {
                c.rect(0, y, T, 1, dark);
                let off = if row % 2 == 0 { 0 } else { 4 };
                for x in (off..T).step_by(8) {
                    c.rect(x, y, 1, 5, dark);
                }
            }
        }
        tiles::FARMLAND => {
            for y in (1..T).step_by(4) {
                c.rect(0, y, T, 1, dark);
                c.rect(0, y + 1, T, 1, lighten(base, 0.12));
            }
        }
        tiles::BIFROST => {
            let cols = [rgb(255, 80, 80), rgb(255, 190, 60), rgb(255, 255, 90), rgb(90, 230, 120), rgb(80, 170, 255), rgb(190, 100, 255)];
            for y in 0..T {
                for x in 0..T {
                    let k = ((x + y + variant * 3) / 3) as usize % cols.len();
                    c.set(x, y, mix(cols[k], rgb(255, 255, 255), 0.15 + 0.2 * noise(x, y, seed)));
                }
            }
        }
        tiles::VOID => {
            c.rect(0, 0, T, T, rgb(8, 8, 18));
            for i in 0..3 {
                c.set(r(i, T), r(i + 7, T), rgb(200, 210, 255));
            }
        }
        _ => {}
    }
    c
}

pub fn sheet(atlas: &mut Atlas) -> Canvas {
    let mut s = Canvas::new(T * VARIANTS, T * TILES.len() as i32);
    for (id, d) in TILES.iter().enumerate() {
        for v in 0..VARIANTS {
            s.blit(&tile(id, v), v * T, id as i32 * T);
        }
        atlas.add(d.name, 0, id as i32 * T, T * VARIANTS, T);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_tiles_opaque_and_deterministic() {
        for (id, d) in TILES.iter().enumerate() {
            let a = tile(id, 1);
            assert!(a.px.iter().all(|p| p[3] == 255), "{}", d.name);
            assert_eq!(a.px, tile(id, 1).px);
        }
        assert_ne!(tile(1, 0).px, tile(1, 1).px);
    }
}
