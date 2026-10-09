//! Shared primitives: coordinates, deterministic RNG, game clock, platform paths.
use serde::{Deserialize, Serialize};

pub const GAME_NAME: &str = "Ultima67";
pub const CREDIT: &str = "By Crowelian 2026 + Sonnet";
pub const CHUNK: i32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
}

impl TilePos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn step(self, d: Dir) -> Self {
        let (dx, dy) = d.delta();
        Self::new(self.x + dx, self.y + dy)
    }
    pub fn chunk(self) -> (i32, i32) {
        (self.x.div_euclid(CHUNK), self.y.div_euclid(CHUNK))
    }
    pub fn manhattan(self, o: Self) -> i32 {
        (self.x - o.x).abs() + (self.y - o.y).abs()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];
    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::N => (0, -1),
            Dir::E => (1, 0),
            Dir::S => (0, 1),
            Dir::W => (-1, 0),
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Small deterministic RNG (SplitMix64). Same seed => same sequence on every platform.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0,1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    /// Uniform in [lo,hi). Returns lo if hi<=lo.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u64() % (hi - lo) as u64) as i32
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }
    pub fn pick<'a, T>(&mut self, s: &'a [T]) -> &'a T {
        &s[self.range(0, s.len() as i32) as usize]
    }
}

/// In-game time. 1 real second = `SCALE` game seconds by default (1 real min = 20 game min).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GameClock {
    pub minutes: u64,
    frac: f32,
    pub scale: f32,
}

impl Default for GameClock {
    fn default() -> Self {
        Self { minutes: 8 * 60, frac: 0.0, scale: 20.0 }
    }
}

impl GameClock {
    pub fn advance(&mut self, real_seconds: f32) {
        self.frac += real_seconds * self.scale / 60.0;
        let whole = self.frac.floor();
        self.minutes += whole as u64;
        self.frac -= whole;
    }
    pub fn hour(&self) -> u32 {
        ((self.minutes / 60) % 24) as u32
    }
    pub fn minute(&self) -> u32 {
        (self.minutes % 60) as u32
    }
    pub fn day(&self) -> u64 {
        self.minutes / (24 * 60)
    }
    pub fn set_hm(&mut self, h: u32, m: u32) {
        self.minutes = self.day() * 1440 + (h.min(23) * 60 + m.min(59)) as u64;
    }
    pub fn is_night(&self) -> bool {
        let h = self.hour();
        !(6..20).contains(&h)
    }
    /// 0.0 = full dark, 1.0 = full daylight (smooth dawn/dusk).
    pub fn daylight(&self) -> f32 {
        let t = self.hour() as f32 + self.minute() as f32 / 60.0;
        let d = ((t - 6.0) / 2.0).clamp(0.0, 1.0) - ((t - 19.0) / 2.0).clamp(0.0, 1.0);
        0.15 + 0.85 * d
    }
}

pub mod platform {
    use directories::ProjectDirs;
    use std::path::PathBuf;
    fn dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("", "Crowelian", "Ultima67")
    }
    pub fn save_dir() -> PathBuf {
        dirs().map(|d| d.data_dir().join("saves")).unwrap_or_else(|| "saves".into())
    }
    pub fn config_dir() -> PathBuf {
        dirs().map(|d| d.config_dir().to_path_buf()).unwrap_or_else(|| ".".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rng_is_deterministic() {
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        assert!(a.f32() < 1.0);
    }
    #[test]
    fn chunk_negative() {
        assert_eq!(TilePos::new(-1, 16).chunk(), (-1, 1));
    }
    #[test]
    fn clock_runs() {
        let mut c = GameClock::default();
        assert_eq!(c.hour(), 8);
        c.advance(60.0); // 1 real minute = 20 game minutes
        assert_eq!(c.minute(), 20);
        c.set_hm(23, 0);
        assert!(c.is_night() && c.daylight() < 0.3);
    }
}

pub mod noise {
    //! Deterministic value noise used by map generation.
    pub fn hash(x: i32, y: i32, seed: u64) -> f32 {
        let mut h = (x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (y as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ seed.wrapping_mul(0x1656_67B1_9E37_79F9);
        h ^= h >> 33;
        h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
        h ^= h >> 29;
        (h >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn value(x: f32, y: f32, seed: u64) -> f32 {
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (fx, fy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
        let n = |dx: i32, dy: i32| hash(x0 as i32 + dx, y0 as i32 + dy, seed);
        let a = n(0, 0) + (n(1, 0) - n(0, 0)) * fx;
        let b = n(0, 1) + (n(1, 1) - n(0, 1)) * fx;
        a + (b - a) * fy
    }
    /// Fractal Brownian motion in roughly [0,1].
    pub fn fbm(x: f32, y: f32, seed: u64, octaves: u32) -> f32 {
        let (mut sum, mut amp, mut freq, mut norm) = (0.0, 1.0, 1.0, 0.0);
        for o in 0..octaves {
            sum += value(x * freq, y * freq, seed.wrapping_add(o as u64 * 101)) * amp;
            norm += amp;
            amp *= 0.5;
            freq *= 2.0;
        }
        sum / norm
    }
}
