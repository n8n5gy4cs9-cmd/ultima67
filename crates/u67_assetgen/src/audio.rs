//! Placeholder SFX + music synthesis. Output: 16-bit mono WAV.
use u67_core::Rng;

pub const SFX_RATE: u32 = 22050;
pub const MUSIC_RATE: u32 = 16000;

pub fn wav_bytes(samples: &[f32], rate: u32) -> Vec<u8> {
    let data: Vec<u8> = samples.iter().flat_map(|s| ((s.clamp(-1.0, 1.0) * 32000.0) as i16).to_le_bytes()).collect();
    let mut o = Vec::with_capacity(44 + data.len());
    o.extend(b"RIFF");
    o.extend((36 + data.len() as u32).to_le_bytes());
    o.extend(b"WAVEfmt ");
    o.extend(16u32.to_le_bytes());
    o.extend(1u16.to_le_bytes()); // PCM
    o.extend(1u16.to_le_bytes()); // mono
    o.extend(rate.to_le_bytes());
    o.extend((rate * 2).to_le_bytes());
    o.extend(2u16.to_le_bytes());
    o.extend(16u16.to_le_bytes());
    o.extend(b"data");
    o.extend((data.len() as u32).to_le_bytes());
    o.extend(data);
    o
}

fn n(rate: u32, secs: f32) -> usize {
    (rate as f32 * secs) as usize
}
fn env_exp(i: usize, rate: u32, decay: f32) -> f32 {
    (-(i as f32 / rate as f32) * decay).exp()
}
fn tau() -> f32 {
    std::f32::consts::TAU
}

struct Lp {
    y: f32,
    a: f32,
}
impl Lp {
    fn new(a: f32) -> Self {
        Self { y: 0.0, a }
    }
    fn p(&mut self, x: f32) -> f32 {
        self.y += self.a * (x - self.y);
        self.y
    }
}

pub struct Sfx {
    pub id: &'static str,
    pub use_: &'static str,
    pub gen: fn(&mut Rng) -> Vec<f32>,
}

fn noise_burst(r: &mut Rng, secs: f32, decay: f32, lp: f32) -> Vec<f32> {
    let mut f = Lp::new(lp);
    (0..n(SFX_RATE, secs)).map(|i| f.p(r.f32() * 2.0 - 1.0) * env_exp(i, SFX_RATE, decay)).collect()
}
fn tone(freq: impl Fn(f32) -> f32, secs: f32, decay: f32, square: bool) -> Vec<f32> {
    let mut ph = 0.0f32;
    (0..n(SFX_RATE, secs))
        .map(|i| {
            let t = i as f32 / SFX_RATE as f32;
            ph += tau() * freq(t) / SFX_RATE as f32;
            let s = if square { ph.sin().signum() * 0.5 } else { ph.sin() };
            s * env_exp(i, SFX_RATE, decay)
        })
        .collect()
}
fn mixv(a: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    let l = a.len().max(b.len());
    (0..l).map(|i| a.get(i).copied().unwrap_or(0.0) + b.get(i).copied().unwrap_or(0.0)).collect()
}
fn scale(mut a: Vec<f32>, g: f32) -> Vec<f32> {
    a.iter_mut().for_each(|x| *x *= g);
    a
}

pub const SFX: &[Sfx] = &[
    Sfx { id: "footstep", use_: "walking", gen: |r| scale(noise_burst(r, 0.09, 40.0, 0.25), 0.6) },
    Sfx { id: "footstep_snow", use_: "walking on snow", gen: |r| scale(noise_burst(r, 0.14, 25.0, 0.12), 0.6) },
    Sfx { id: "gun_pistol", use_: "pistol shot", gen: |r| scale(mixv(noise_burst(r, 0.25, 22.0, 0.7), tone(|t| 220.0 * (-t * 25.0).exp() + 50.0, 0.2, 18.0, false)), 0.8) },
    Sfx { id: "gun_rifle", use_: "rifle shot", gen: |r| scale(mixv(noise_burst(r, 0.4, 14.0, 0.5), tone(|t| 140.0 * (-t * 18.0).exp() + 40.0, 0.3, 12.0, false)), 0.85) },
    Sfx { id: "gun_shotgun", use_: "shotgun blast", gen: |r| scale(mixv(noise_burst(r, 0.5, 9.0, 0.35), tone(|t| 90.0 * (-t * 12.0).exp() + 30.0, 0.4, 8.0, false)), 0.9) },
    Sfx { id: "reload", use_: "reloading a gun", gen: |r| mixv(scale(noise_burst(r, 0.05, 60.0, 0.9), 0.7), {
        let mut v = vec![0.0; n(SFX_RATE, 0.18)];
        v.extend(scale(noise_burst(r, 0.07, 50.0, 0.8), 0.8));
        v
    }) },
    Sfx { id: "cannon_fire", use_: "cannon firing", gen: |r| scale(mixv(noise_burst(r, 1.0, 4.5, 0.18), tone(|t| 70.0 * (-t * 6.0).exp() + 28.0, 0.9, 4.0, false)), 1.0) },
    Sfx { id: "explosion", use_: "explosions / cannonball impact", gen: |r| scale(mixv(noise_burst(r, 1.2, 3.5, 0.12), tone(|t| 55.0 * (-t * 4.0).exp() + 22.0, 1.0, 3.0, false)), 1.0) },
    Sfx { id: "hit_flesh", use_: "melee hit on creature", gen: |r| scale(mixv(noise_burst(r, 0.12, 30.0, 0.3), tone(|t| 120.0 - t * 300.0, 0.1, 30.0, false)), 0.9) },
    Sfx { id: "hit_metal", use_: "weapon on armor", gen: |r| scale(mixv(noise_burst(r, 0.08, 50.0, 0.9), tone(|_| 1480.0, 0.4, 12.0, false)), 0.5) },
    Sfx { id: "swing", use_: "melee swing", gen: |r| scale(noise_burst(r, 0.2, 14.0, 0.4), 0.4) },
    Sfx { id: "death", use_: "creature death", gen: |_| tone(|t| 260.0 * (-t * 3.0).exp() + 50.0, 0.7, 4.0, true) },
    Sfx { id: "door_open", use_: "door open", gen: |r| mixv(scale(noise_burst(r, 0.3, 8.0, 0.08), 0.5), tone(|t| 90.0 + t * 80.0, 0.3, 8.0, true)) },
    Sfx { id: "door_close", use_: "door close", gen: |r| scale(mixv(noise_burst(r, 0.15, 30.0, 0.15), tone(|_| 80.0, 0.15, 25.0, false)), 0.9) },
    Sfx { id: "chest_open", use_: "chest/container open", gen: |r| mixv(scale(noise_burst(r, 0.2, 10.0, 0.1), 0.4), tone(|t| 200.0 + t * 400.0, 0.2, 12.0, true)) },
    Sfx { id: "pickup", use_: "pick up item", gen: |_| tone(|t| if t < 0.05 { 660.0 } else { 990.0 }, 0.14, 14.0, false) },
    Sfx { id: "drop", use_: "drop item", gen: |_| tone(|t| 300.0 - t * 400.0, 0.1, 25.0, false) },
    Sfx { id: "coin", use_: "money", gen: |_| mixv(tone(|_| 1760.0, 0.25, 12.0, false), tone(|_| 2349.0, 0.3, 9.0, false)) },
    Sfx { id: "ui_click", use_: "button click", gen: |_| scale(tone(|_| 880.0, 0.04, 60.0, true), 0.5) },
    Sfx { id: "ui_open", use_: "open inventory / menu", gen: |_| scale(tone(|t| 400.0 + t * 1500.0, 0.15, 12.0, false), 0.6) },
    Sfx { id: "ui_close", use_: "close menu", gen: |_| scale(tone(|t| 900.0 - t * 1500.0, 0.15, 12.0, false), 0.6) },
    Sfx { id: "ui_error", use_: "invalid action / console error", gen: |_| scale(tone(|_| 140.0, 0.25, 8.0, true), 0.7) },
    Sfx { id: "spell_cast", use_: "casting a spell (seidr/laulu)", gen: |_| {
        let a = tone(|t| 440.0 + (t * 30.0).sin() * 30.0 + t * 700.0, 0.7, 3.0, false);
        let b = tone(|t| 660.0 + t * 1050.0, 0.7, 3.5, false);
        mixv(scale(a, 0.5), scale(b, 0.3))
    } },
    Sfx { id: "spell_fizzle", use_: "failed spell", gen: |r| scale(noise_burst(r, 0.4, 7.0, 0.3), 0.5) },
    Sfx { id: "level_up", use_: "level up / quest done", gen: |_| {
        let mut v = vec![];
        for f in [523.0, 659.0, 784.0, 1047.0] {
            v.extend(tone(move |_| f, 0.18, 6.0, false));
        }
        v
    } },
    Sfx { id: "quest_start", use_: "new quest", gen: |_| {
        let mut v = tone(|_| 392.0, 0.2, 6.0, false);
        v.extend(tone(|_| 587.0, 0.3, 5.0, false));
        v
    } },
    Sfx { id: "bifrost", use_: "bifrost travel", gen: |r| mixv(scale(noise_burst(r, 1.5, 2.0, 0.05), 0.3), tone(|t| 200.0 + t * 900.0, 1.5, 1.5, false)) },
    Sfx { id: "ship_creak", use_: "ship movement", gen: |_| scale(tone(|t| 110.0 + (t * 12.0).sin() * 8.0, 0.6, 3.0, true), 0.4) },
    Sfx { id: "splash", use_: "water splash", gen: |r| scale(noise_burst(r, 0.5, 6.0, 0.2), 0.6) },
    Sfx { id: "ambient_wind", use_: "wind loop (snow, mountains, space-planets)", gen: |r| {
        let mut f = Lp::new(0.03);
        let len = n(SFX_RATE, 4.0);
        let v: Vec<f32> = (0..len).map(|i| f.p(r.f32() * 2.0 - 1.0) * (0.6 + 0.4 * (i as f32 / len as f32 * tau()).sin()) * 3.0).collect();
        loopfade(v)
    } },
    Sfx { id: "ambient_forest", use_: "forest loop (birds + rustle)", gen: |r| {
        let mut f = Lp::new(0.06);
        let len = n(SFX_RATE, 4.0);
        let mut v: Vec<f32> = (0..len).map(|_| f.p(r.f32() * 2.0 - 1.0) * 0.8).collect();
        for _ in 0..5 {
            let st = r.range(0, len as i32 - 4000) as usize;
            let fr = 2000.0 + r.f32() * 1500.0;
            for (i, s) in tone(move |t| fr + (t * 40.0).sin() * 300.0, 0.12, 20.0, false).into_iter().enumerate() {
                v[st + i] += s * 0.3;
            }
        }
        loopfade(v)
    } },
    Sfx { id: "ambient_sea", use_: "waves loop", gen: |r| {
        let mut f = Lp::new(0.05);
        let len = n(SFX_RATE, 5.0);
        let v: Vec<f32> = (0..len).map(|i| f.p(r.f32() * 2.0 - 1.0) * (0.5 + 0.5 * (i as f32 / SFX_RATE as f32 * tau() / 5.0).sin().abs()) * 2.5).collect();
        loopfade(v)
    } },
    Sfx { id: "ambient_cave", use_: "dungeon/cave loop (drips + drone)", gen: |r| {
        let len = n(SFX_RATE, 4.0);
        let mut v: Vec<f32> = (0..len).map(|i| (i as f32 * tau() * 55.0 / SFX_RATE as f32).sin() * 0.15).collect();
        for _ in 0..3 {
            let st = r.range(0, len as i32 - 6000) as usize;
            for (i, s) in tone(|t| 1200.0 - t * 2000.0, 0.1, 30.0, false).into_iter().enumerate() {
                v[st + i] += s * 0.4;
            }
        }
        loopfade(v)
    } },
    Sfx { id: "ambient_space", use_: "ship / space station hum loop", gen: |_| {
        let len = n(SFX_RATE, 4.0);
        loopfade((0..len).map(|i| ((i as f32 * tau() * 60.0 / SFX_RATE as f32).sin() * 0.3) + ((i as f32 * tau() * 90.0 / SFX_RATE as f32).sin() * 0.15)).collect())
    } },
    Sfx { id: "wolf_howl", use_: "wolf / Fenrir", gen: |_| scale(tone(|t| 300.0 + 250.0 * (t * 1.8).min(1.0) - 150.0 * (t - 1.0).max(0.0), 1.6, 1.5, false), 0.6) },
    Sfx { id: "kantele", use_: "kantele pluck (Väinämöinen's gun, singing magic)", gen: |_| {
        let mut v = vec![];
        for f in [392.0, 494.0, 587.0] {
            v.extend(mixv(tone(move |_| f, 0.5, 6.0, false), scale(tone(move |_| f * 2.0, 0.5, 9.0, false), 0.4)));
        }
        v
    } },
];

fn loopfade(mut v: Vec<f32>) -> Vec<f32> {
    let f = (v.len() / 20).max(1);
    for i in 0..f {
        let t = i as f32 / f as f32;
        let j = v.len() - f + i;
        v[j] = v[j] * (1.0 - t) + v[i] * t;
    }
    let l = v.len() - f;
    v.truncate(l);
    v
}

// ---------------- music ----------------
pub struct Track {
    pub id: &'static str,
    pub use_: &'static str,
    /// semitone offsets of the scale
    pub scale: &'static [i32],
    pub root: f32,
    pub bpm: f32,
    pub drone: bool,
    pub drive: bool,
    pub seed: u64,
}

pub const DORIAN: &[i32] = &[0, 2, 3, 5, 7, 9, 10];
pub const PHRYG: &[i32] = &[0, 1, 3, 5, 7, 8, 10];
pub const MINOR_P: &[i32] = &[0, 3, 5, 7, 10];
pub const MAJOR_P: &[i32] = &[0, 2, 4, 7, 9];
pub const LYDIAN: &[i32] = &[0, 2, 4, 6, 7, 9, 11];

pub const TRACKS: &[Track] = &[
    Track { id: "menu", use_: "main menu", scale: DORIAN, root: 146.8, bpm: 64.0, drone: true, drive: false, seed: 1 },
    Track { id: "midgard_town", use_: "towns of Midgard", scale: MAJOR_P, root: 196.0, bpm: 92.0, drone: false, drive: false, seed: 2 },
    Track { id: "midgard_wilds", use_: "overworld, forests, fjords", scale: DORIAN, root: 164.8, bpm: 72.0, drone: true, drive: false, seed: 3 },
    Track { id: "dungeon", use_: "caves, ruins, barrows", scale: PHRYG, root: 110.0, bpm: 54.0, drone: true, drive: false, seed: 4 },
    Track { id: "space", use_: "star map and space travel", scale: LYDIAN, root: 220.0, bpm: 60.0, drone: true, drive: false, seed: 5 },
    Track { id: "combat", use_: "fights", scale: PHRYG, root: 130.8, bpm: 140.0, drone: false, drive: true, seed: 6 },
    Track { id: "boss", use_: "boss fights", scale: MINOR_P, root: 98.0, bpm: 156.0, drone: true, drive: true, seed: 7 },
    Track { id: "kalevala", use_: "Kalevala system (kantele/runolaulu feel)", scale: MINOR_P, root: 196.0, bpm: 80.0, drone: true, drive: false, seed: 8 },
    Track { id: "hel", use_: "Hel, Tuonela and the dead", scale: PHRYG, root: 92.5, bpm: 48.0, drone: true, drive: false, seed: 9 },
];

fn osc_tri(ph: f32) -> f32 {
    let p = ph.fract();
    4.0 * (p - 0.5).abs() - 1.0
}

pub fn render_track(t: &Track, secs: f32) -> Vec<f32> {
    let rate = MUSIC_RATE;
    let len = n(rate, secs);
    let mut out = vec![0.0f32; len];
    let mut r = Rng::new(t.seed * 7919);
    let beat = 60.0 / t.bpm;
    let note = |deg: i32, oct: i32| -> f32 {
        let sc = t.scale;
        let o = deg.div_euclid(sc.len() as i32);
        let d = deg.rem_euclid(sc.len() as i32) as usize;
        t.root * 2f32.powf((sc[d] + 12 * (o + oct)) as f32 / 12.0)
    };
    let add_note = |out: &mut Vec<f32>, start: f32, dur: f32, f: f32, vol: f32, sq: bool| {
        let s0 = (start * rate as f32) as usize;
        let nn = (dur * rate as f32) as usize;
        for i in 0..nn {
            let j = s0 + i;
            if j >= out.len() {
                break;
            }
            let ph = f * i as f32 / rate as f32;
            let w = if sq { osc_tri(ph) * 0.6 + (ph * tau()).sin().signum() * 0.2 } else { osc_tri(ph) };
            let tt = i as f32 / nn as f32;
            let env = (tt * 12.0).min(1.0) * (1.0 - tt).powf(1.5);
            out[j] += w * env * vol;
        }
    };
    let bars = (secs / (beat * 4.0)).floor() as i32;
    let mut deg = 2;
    for bar in 0..bars {
        let b0 = bar as f32 * beat * 4.0;
        if t.drone {
            add_note(&mut out, b0, beat * 4.0, note(0, -1), 0.18, false);
            add_note(&mut out, b0, beat * 4.0, note(4, -1), 0.08, false);
        }
        if t.drive {
            for k in 0..8 {
                add_note(&mut out, b0 + k as f32 * beat * 0.5, beat * 0.4, note(if k % 4 == 0 { 0 } else { -2 }, -1), 0.2, true);
            }
        }
        // melody: random walk phrase repeated with variation
        let mut pos = 0.0;
        while pos < 4.0 {
            let dur = *r.pick(&[0.5, 0.5, 1.0, 1.0, 1.5, 2.0]);
            let dur = if pos + dur > 4.0 { 4.0 - pos } else { dur };
            deg = (deg + r.range(-2, 3)).clamp(-2, 9);
            if r.chance(0.85) {
                add_note(&mut out, b0 + pos * beat, dur * beat * 0.95, note(deg, 0), 0.22, t.drive);
            }
            pos += dur;
        }
        if !t.drive && bar % 2 == 1 {
            add_note(&mut out, b0, beat * 2.0, note(deg + 2, 1), 0.07, false); // shimmer
        }
    }
    // loop crossfade + normalise
    let out = loopfade(out);
    let peak = out.iter().fold(0.0f32, |m, x| m.max(x.abs())).max(0.001);
    out.into_iter().map(|x| x / peak * 0.7).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wav_header_and_length() {
        let w = wav_bytes(&[0.0, 0.5, -0.5], 8000);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(w.len(), 44 + 6);
    }
    #[test]
    fn all_sfx_nonempty_not_clipping_silent() {
        for s in SFX {
            let v = (s.gen)(&mut Rng::new(1));
            assert!(v.len() > 500, "{}", s.id);
            let peak = v.iter().fold(0.0f32, |m, x| m.max(x.abs()));
            assert!(peak > 0.05, "{} silent", s.id);
            assert!(v.iter().all(|x| x.is_finite()));
        }
    }
    #[test]
    fn music_is_deterministic() {
        let a = render_track(&TRACKS[0], 6.0);
        assert_eq!(a, render_track(&TRACKS[0], 6.0));
        assert!(a.len() > 16000 * 5);
    }
}
