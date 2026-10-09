//! Title screen art (320x180) and the application icon.
use crate::canvas::*;
use crate::space;

/// 5x7 glyphs for the title lettering.
fn glyph(c: char) -> [u8; 7] {
    match c {
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        _ => [0; 7],
    }
}

fn draw_text(c: &mut Canvas, text: &str, x: i32, y: i32, scale: i32, col: Col) {
    let mut cx = x;
    for ch in text.chars() {
        let g = glyph(ch);
        for (row, bits) in g.iter().enumerate() {
            for b in 0..5 {
                if bits & (1 << (4 - b)) != 0 {
                    c.rect(cx + b * scale, y + row as i32 * scale, scale, scale, col);
                }
            }
        }
        cx += 6 * scale;
    }
}

/// Nearest-neighbour resize.
pub fn resize(src: &Canvas, w: i32, h: i32) -> Canvas {
    let mut o = Canvas::new(w, h);
    for y in 0..h {
        for x in 0..w {
            o.set(x, y, src.get(x * src.w / w, y * src.h / h));
        }
    }
    o
}

pub fn title() -> Canvas {
    let (w, h) = (320, 180);
    let mut c = Canvas::new(w, h);
    for y in 0..h {
        let t = y as f32 / h as f32;
        let col = mix(rgb(6, 8, 30), rgb(70, 30, 90), t * t);
        c.rect(0, y, w, 1, col);
    }
    for y in 0..h {
        for x in 0..w {
            let n = noise(x, y, 7);
            if n > 0.992 {
                c.set(x, y, rgb(255, 255, 255));
            } else if n > 0.975 {
                c.set(x, y, rgb(160, 170, 220));
            }
        }
    }
    // the moon, and Midgard rising below the horizon
    let bodies = space::bodies();
    let moon = space::planet(&bodies[4], 4);
    c.blit(&resize(&moon, 30, 30), 268, 74);
    let g = bodies[3].colors;
    let earth_body = space::Body { id: "title_earth", colors: g, bands: 0.0, noise: 3.5, cap: false, ring: false, craters: false, glow: false };
    let earth = space::planet(&earth_body, 3);
    let big = resize(&earth, 420, 420);
    c.blit(&big, -50, 128);
    // atmosphere glow along the planet's upper rim
    for x in 0..w {
        for y in 100..140 {
            if c.get(x, y)[3] > 0 {
                continue;
            }
        }
    }
    // Bifrost: a rainbow arc across the sky
    let cols = [rgb(255, 80, 80), rgb(255, 170, 60), rgb(255, 240, 90), rgb(90, 230, 120), rgb(80, 170, 255), rgb(190, 100, 255)];
    for (i, col) in cols.iter().enumerate() {
        let r = 175 - i as i32 * 3;
        for a in 0..=1800 {
            let t = a as f32 * std::f32::consts::PI / 1800.0;
            let (x, y) = (160.0 + t.cos() * r as f32 * 1.1, 190.0 - t.sin() * r as f32 * 0.6);
            let (xi, yi) = (x as i32, y as i32);
            if c.get(xi, yi)[3] > 0 && yi < 130 {
                c.set(xi, yi, mix(c.get(xi, yi), *col, 0.55));
            }
            c.set(xi, yi, mix(c.get(xi, yi), *col, 0.55));
        }
    }
    // a longship sails the arc
    c.blit(&space::ship(1), 214, 62);
    // title lettering
    let mut t = Canvas::new(w, 60);
    draw_text(&mut t, "ULTIMA 67", 28, 6, 5, rgb(236, 190, 80));
    // light edge
    let mut hi = Canvas::new(w, 60);
    draw_text(&mut hi, "ULTIMA 67", 28, 5, 5, rgb(255, 235, 160));
    let mut shadow = Canvas::new(w, 60);
    draw_text(&mut shadow, "ULTIMA 67", 28, 9, 5, rgb(110, 50, 30));
    shadow.blit(&hi, 0, 0);
    shadow.blit(&t, 0, 0);
    shadow.outline(rgb(20, 10, 20));
    c.blit(&shadow, 0, 10);
    c
}

pub fn icon() -> Canvas {
    let mut c = Canvas::filled(256, 256, rgb(14, 12, 40));
    for y in 0..256 {
        for x in 0..256 {
            let n = noise(x, y, 11);
            if n > 0.99 {
                c.set(x, y, rgb(255, 255, 255));
            }
            let d = (((x - 128) as f32).powi(2) + ((y - 300) as f32).powi(2)).sqrt();
            if d < 150.0 {
                c.set(x, y, mix(rgb(40, 110, 190), rgb(60, 150, 80), noise(x / 6, y / 6, 3)));
            }
        }
    }
    let cols = [rgb(255, 80, 80), rgb(255, 170, 60), rgb(255, 240, 90), rgb(90, 230, 120), rgb(80, 170, 255), rgb(190, 100, 255)];
    for (i, col) in cols.iter().enumerate() {
        let r = 112 - i as i32 * 6;
        for a in 0..=1400 {
            let t = a as f32 * std::f32::consts::PI / 1400.0;
            c.rect((128.0 + t.cos() * r as f32) as i32, (170.0 - t.sin() * r as f32) as i32, 6, 6, *col);
        }
    }
    // runestone
    let mut rs = Canvas::new(64, 64);
    rs.ellipse(32, 30, 14, 26, rgb(135, 138, 150));
    rs.rect(18, 36, 28, 26, rgb(135, 138, 150));
    for (x0, y0, x1, y1) in [(32, 10, 32, 50), (32, 18, 42, 10), (32, 28, 42, 20), (32, 40, 22, 50)] {
        rs.line(x0, y0, x1, y1, rgb(110, 240, 255));
    }
    rs.outline(rgb(20, 14, 18));
    c.blit(&resize(&rs, 128, 128), 64, 100);
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn title_is_opaque_with_gold_lettering() {
        let t = title();
        assert_eq!((t.w, t.h), (320, 180));
        assert!(t.px.iter().all(|p| p[3] == 255));
        assert!(t.px.iter().any(|p| *p == rgb(236, 190, 80)));
        assert_eq!(icon().w, 256);
    }
}
