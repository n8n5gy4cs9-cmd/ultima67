//! Tiny RGBA pixel canvas with the drawing helpers the generators need.
use image::{ImageBuffer, Rgba};
use std::path::Path;

pub type Col = [u8; 4];
pub const CLEAR: Col = [0, 0, 0, 0];

pub const fn rgb(r: u8, g: u8, b: u8) -> Col {
    [r, g, b, 255]
}
pub fn mix(a: Col, b: Col, t: f32) -> Col {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round().clamp(0.0, 255.0) as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]), m(a[3], b[3])]
}
pub fn lighten(c: Col, t: f32) -> Col {
    mix(c, [255, 255, 255, c[3]], t)
}
pub fn darken(c: Col, t: f32) -> Col {
    mix(c, [0, 0, 0, c[3]], t)
}

/// Deterministic hash noise in [0,1) for integer coords + seed.
pub fn noise(x: i32, y: i32, seed: u64) -> f32 {
    let mut h = (x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (y as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ seed.wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 29;
    (h >> 40) as f32 / (1u64 << 24) as f32
}

/// Smooth value noise (bilinear), period-free.
pub fn vnoise(x: f32, y: f32, seed: u64) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (fx, fy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let n = |dx: i32, dy: i32| noise(x0 as i32 + dx, y0 as i32 + dy, seed);
    let a = n(0, 0) + (n(1, 0) - n(0, 0)) * fx;
    let b = n(0, 1) + (n(1, 1) - n(0, 1)) * fx;
    a + (b - a) * fy
}

#[derive(Clone)]
pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub px: Vec<Col>,
}

impl Canvas {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h, px: vec![CLEAR; (w * h) as usize] }
    }
    pub fn filled(w: i32, h: i32, c: Col) -> Self {
        Self { w, h, px: vec![c; (w * h) as usize] }
    }
    pub fn get(&self, x: i32, y: i32) -> Col {
        if x < 0 || y < 0 || x >= self.w || y >= self.h { CLEAR } else { self.px[(y * self.w + x) as usize] }
    }
    pub fn set(&mut self, x: i32, y: i32, c: Col) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Col) {
        for j in y..y + h {
            for i in x..x + w {
                self.set(i, j, c);
            }
        }
    }
    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, c: Col) {
        self.rect(x, y, w, 1, c);
        self.rect(x, y + h - 1, w, 1, c);
        self.rect(x, y, 1, h, c);
        self.rect(x + w - 1, y, 1, h, c);
    }
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Col) {
        let (mut x, mut y) = (x0, y0);
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let mut err = dx + dy;
        loop {
            self.set(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    pub fn ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, c: Col) {
        for y in -ry..=ry {
            for x in -rx..=rx {
                let (fx, fy) = (x as f32 / (rx as f32 + 0.5), y as f32 / (ry as f32 + 0.5));
                if fx * fx + fy * fy <= 1.0 {
                    self.set(cx + x, cy + y, c);
                }
            }
        }
    }
    /// Filled triangle (scanline).
    pub fn tri(&mut self, a: (i32, i32), b: (i32, i32), c: (i32, i32), col: Col) {
        let (miny, maxy) = (a.1.min(b.1).min(c.1), a.1.max(b.1).max(c.1));
        let (minx, maxx) = (a.0.min(b.0).min(c.0), a.0.max(b.0).max(c.0));
        let area = |p: (i32, i32), q: (i32, i32), r: (i32, i32)| (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0);
        let tot = area(a, b, c);
        if tot == 0 {
            return;
        }
        for y in miny..=maxy {
            for x in minx..=maxx {
                let p = (x, y);
                let (w0, w1, w2) = (area(b, c, p), area(c, a, p), area(a, b, p));
                let inside = if tot > 0 { w0 >= 0 && w1 >= 0 && w2 >= 0 } else { w0 <= 0 && w1 <= 0 && w2 <= 0 };
                if inside {
                    self.set(x, y, col);
                }
            }
        }
    }
    /// Add a 1px outline around opaque pixels (Starbound-ish chunky look).
    pub fn outline(&mut self, c: Col) {
        let src = self.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                if src.get(x, y)[3] == 0 && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| src.get(x + dx, y + dy)[3] > 0) {
                    self.set(x, y, c);
                }
            }
        }
    }
    pub fn blit(&mut self, s: &Canvas, dx: i32, dy: i32) {
        for y in 0..s.h {
            for x in 0..s.w {
                let p = s.get(x, y);
                if p[3] > 0 {
                    self.set(dx + x, dy + y, p);
                }
            }
        }
    }
    pub fn flip_h(&self) -> Canvas {
        let mut o = Canvas::new(self.w, self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                o.set(self.w - 1 - x, y, self.get(x, y));
            }
        }
        o
    }
    /// Rotate 90 degrees clockwise (square canvases keep size).
    pub fn rot90(&self) -> Canvas {
        let mut o = Canvas::new(self.h, self.w);
        for y in 0..self.h {
            for x in 0..self.w {
                o.set(self.h - 1 - y, x, self.get(x, y));
            }
        }
        o
    }
    pub fn tint(&mut self, c: Col, t: f32) {
        for p in &mut self.px {
            if p[3] > 0 {
                *p = mix(*p, [c[0], c[1], c[2], p[3]], t);
            }
        }
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(self.w as u32, self.h as u32);
        for (i, p) in self.px.iter().enumerate() {
            img.put_pixel((i as i32 % self.w) as u32, (i as i32 / self.w) as u32, Rgba(*p));
        }
        img.save(path).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn draw_basics() {
        let mut c = Canvas::new(8, 8);
        c.rect(2, 2, 3, 3, rgb(255, 0, 0));
        c.outline(rgb(0, 0, 0));
        assert_eq!(c.get(1, 2), rgb(0, 0, 0));
        assert_eq!(c.get(0, 0), CLEAR);
        let f = c.flip_h();
        assert_eq!(f.get(5, 3), rgb(255, 0, 0));
        assert_eq!(noise(3, 4, 9), noise(3, 4, 9));
    }
}
