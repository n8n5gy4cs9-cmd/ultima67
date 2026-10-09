//! Planet / star-map art. 64x64 bodies, ship, bifrost gate, starfield.
use crate::canvas::*;
use crate::manifest::Atlas;

pub struct Body {
    pub id: &'static str,
    pub colors: [Col; 3],
    pub bands: f32,
    pub noise: f32,
    pub cap: bool,
    pub ring: bool,
    pub craters: bool,
    pub glow: bool,
}

const fn b(id: &'static str, a: Col, bb: Col, cc: Col, bands: f32, noise: f32) -> Body {
    Body { id, colors: [a, bb, cc], bands, noise, cap: false, ring: false, craters: false, glow: false }
}

pub fn bodies() -> Vec<Body> {
    vec![
        Body { glow: true, ..b("sol", rgb(255, 230, 120), rgb(255, 160, 40), rgb(255, 250, 200), 0.0, 5.0) },
        Body { craters: true, ..b("dvalinn", rgb(150, 120, 100), rgb(90, 60, 50), rgb(240, 120, 40), 0.0, 6.0) },
        b("vanaheimr", rgb(220, 190, 120), rgb(90, 160, 90), rgb(240, 240, 230), 3.0, 4.0),
        Body { cap: true, ..b("midgard", rgb(50, 110, 190), rgb(70, 150, 80), rgb(240, 245, 250), 0.0, 3.5) },
        Body { craters: true, ..b("maani", rgb(190, 190, 195), rgb(120, 120, 130), rgb(230, 230, 235), 0.0, 6.0) },
        Body { cap: true, ..b("muspelheimr", rgb(190, 80, 50), rgb(120, 40, 30), rgb(250, 140, 60), 0.0, 5.0) },
        Body { craters: true, ..b("svartalfheimr", rgb(80, 75, 85), rgb(45, 40, 55), rgb(140, 130, 150), 0.0, 8.0) },
        b("asgard", rgb(230, 200, 150), rgb(200, 130, 90), rgb(250, 235, 210), 9.0, 3.0),
        Body { ring: true, ..b("jotunheimr", rgb(215, 195, 140), rgb(185, 160, 110), rgb(240, 225, 180), 7.0, 2.5) },
        b("niflheimr", rgb(150, 220, 230), rgb(90, 170, 200), rgb(220, 250, 255), 2.0, 3.0),
        b("hel", rgb(45, 60, 140), rgb(25, 30, 90), rgb(110, 200, 220), 3.0, 3.5),
        Body { glow: true, ..b("ginnungagap", rgb(25, 10, 50), rgb(120, 50, 180), rgb(255, 140, 240), 4.0, 2.0) },
        b("vainola", rgb(60, 150, 80), rgb(40, 100, 60), rgb(210, 240, 190), 0.0, 3.5),
        Body { cap: true, ..b("pohjola", rgb(190, 215, 235), rgb(90, 130, 170), rgb(250, 255, 255), 0.0, 3.0) },
        b("tuonela", rgb(40, 35, 60), rgb(20, 18, 35), rgb(130, 120, 190), 2.0, 4.0),
        b("ilma", rgb(120, 170, 230), rgb(230, 235, 255), rgb(255, 240, 150), 6.0, 2.0),
        Body { craters: true, ..b("sampola", rgb(150, 130, 80), rgb(100, 80, 50), rgb(240, 200, 90), 0.0, 6.0) },
    ]
}

pub fn planet(bd: &Body, idx: usize) -> Canvas {
    let n = 64;
    let mut c = Canvas::new(n, n);
    let r = 28.0;
    let seed = idx as u64 * 977 + 13;
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = ((x as f32 - 31.5) / r, (y as f32 - 31.5) / r);
            let d2 = dx * dx + dy * dy;
            if d2 > 1.0 {
                continue;
            }
            let dz = (1.0 - d2).sqrt();
            let (u, v) = (dx.atan2(dz) * 2.0, dy.asin() * 2.0);
            let mut t = vnoise(u * bd.noise + 10.0, v * bd.noise, seed);
            if bd.bands > 0.0 {
                t = (t * 0.45 + 0.55 * ((v * bd.bands * 2.0 + t * 1.5).sin() * 0.5 + 0.5)).clamp(0.0, 1.0);
            }
            let mut col = if t < 0.5 { mix(bd.colors[1], bd.colors[0], t * 2.0) } else { mix(bd.colors[0], bd.colors[2], (t - 0.5) * 0.7) };
            if bd.cap && dy.abs() > 0.82 {
                col = bd.colors[2];
            }
            if bd.craters && noise((u * 6.0) as i32, (v * 6.0) as i32, seed) > 0.86 {
                col = darken(col, 0.35);
            }
            let light = if bd.glow { 1.0 } else { (0.25 + 0.95 * (-dx * 0.6 - dy * 0.5 + dz * 0.6)).clamp(0.12, 1.1) };
            let mut col = if light > 1.0 { lighten(col, light - 1.0) } else { darken(col, 1.0 - light) };
            if bd.glow {
                col = lighten(col, (dz * 0.1).min(0.1));
            }
            c.set(x, y, col);
        }
    }
    if bd.ring {
        for k in 0..720 {
            let t = k as f32 * std::f32::consts::PI / 360.0;
            for (rx, ry) in [(31.0f32, 8.0f32), (28.5, 7.0)] {
                let (x, y) = (31.5 + t.cos() * rx, 34.0 + t.sin() * ry);
                let front = t.sin() > 0.0;
                let (dx, dy) = ((x - 31.5) / 28.0, (y - 31.5) / 28.0);
                if front || dx * dx + dy * dy > 1.0 {
                    c.set(x as i32, y as i32, if rx > 30.0 { rgb(225, 205, 160) } else { rgb(190, 165, 120) });
                }
            }
        }
    }
    c.outline(rgb(10, 8, 18));
    c
}

pub fn ship(dir: usize) -> Canvas {
    let mut c = Canvas::new(32, 32);
    let hull = rgb(150, 98, 52);
    match dir {
        1 | 3 => {
            c.rect(3, 18, 26, 5, hull);
            c.tri((3, 23), (3, 18), (7, 23), hull);
            c.tri((28, 23), (28, 18), (24, 23), hull);
            c.rect(14, 4, 2, 15, rgb(80, 50, 28));
            c.rect(8, 6, 14, 10, rgb(235, 230, 215));
            c.rect(8, 9, 14, 2, rgb(200, 50, 45));
            c.rect(2, 22, 5, 3, rgb(120, 230, 255)); // engine glow
            c.rect(29, 14, 2, 4, rgb(235, 190, 70));
            if dir == 3 {
                c = c.flip_h();
            }
        }
        _ => {
            c.rect(11, 6, 10, 20, hull);
            c.rect(6, 8, 20, 8, rgb(235, 230, 215));
            c.rect(6, 11, 20, 2, rgb(200, 50, 45));
            c.rect(13, 26, 6, 3, rgb(120, 230, 255));
            if dir == 0 {
                c.rect(15, 2, 2, 4, rgb(235, 190, 70));
            }
        }
    }
    c.outline(rgb(10, 8, 18));
    c
}

pub fn gate() -> Canvas {
    let mut c = Canvas::new(64, 64);
    let cols = [rgb(255, 80, 80), rgb(255, 190, 60), rgb(255, 255, 90), rgb(90, 230, 120), rgb(80, 170, 255), rgb(190, 100, 255)];
    for (i, col) in cols.iter().enumerate() {
        let r = 30 - i as i32 * 2;
        for a in 0..=180 {
            let t = (180 - a) as f32 * std::f32::consts::PI / 180.0;
            let (x, y) = (32.0 + t.cos() * r as f32, 56.0 - t.sin() * r as f32);
            c.rect(x as i32, y as i32, 2, 2, *col);
        }
    }
    c.rect(0, 56, 64, 6, rgb(70, 60, 100));
    c.outline(rgb(10, 8, 18));
    c
}

pub fn starfield() -> Canvas {
    let mut c = Canvas::filled(128, 128, rgb(6, 6, 16));
    for y in 0..128 {
        for x in 0..128 {
            let n = noise(x, y, 4242);
            if n > 0.985 {
                c.set(x, y, rgb(255, 255, 255));
            } else if n > 0.96 {
                c.set(x, y, rgb(150, 160, 210));
            }
        }
    }
    c
}

pub fn sheet(atlas: &mut Atlas) -> Canvas {
    let bs = bodies();
    let mut s = Canvas::new(64 * 6, 64 * 3 + 32 + 64 + 128);
    for (i, bd) in bs.iter().enumerate() {
        let (x, y) = ((i as i32 % 6) * 64, (i as i32 / 6) * 64);
        s.blit(&planet(bd, i), x, y);
        atlas.add(bd.id, x, y, 64, 64);
    }
    let y = 64 * 3;
    for d in 0..4 {
        s.blit(&ship(d), d as i32 * 32, y);
        atlas.add(&format!("ship_{}", ["n", "e", "s", "w"][d]), d as i32 * 32, y, 32, 32);
    }
    s.blit(&gate(), 0, y + 32);
    atlas.add("bifrost_gate", 0, y + 32, 64, 64);
    s.blit(&starfield(), 0, y + 96);
    atlas.add("starfield", 0, y + 96, 128, 128);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bodies_draw_round() {
        for (i, b) in bodies().iter().enumerate() {
            let c = planet(b, i);
            assert!(c.get(32, 32)[3] == 255 && c.get(0, 0)[3] == 0, "{}", b.id);
        }
        assert_eq!(bodies().len(), 17);
    }
}
