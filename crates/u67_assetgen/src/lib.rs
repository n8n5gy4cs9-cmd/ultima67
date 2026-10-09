//! Procedural placeholder asset generation (sprites, sfx, music) + manifest + ASSETS.md.
pub mod audio;
pub mod canvas;
pub mod characters;
pub mod docgen;
pub mod icons;
pub mod manifest;
pub mod objects;
pub mod space;
pub mod terrain;
pub mod title;
pub mod ui;

use manifest::{Atlas, Manifest};
use std::path::Path;

pub const MUSIC_SECS: f32 = 20.0;

/// Generate every placeholder asset under `root` and return the manifest.
pub fn generate_all(root: &Path) -> Result<Manifest, String> {
    let mut m = Manifest::default();
    let mut sheet = |name: &str, use_: &str, notes: &str, f: &dyn Fn(&mut Atlas) -> canvas::Canvas| -> Result<(), String> {
        let mut a = Atlas::new(&format!("gfx/{name}.png"));
        f(&mut a).save(&root.join(format!("gfx/{name}.png")))?;
        a.save(&root.join(format!("gfx/{name}.json")))?;
        m.add(name, "sheet", &format!("gfx/{name}.png"), use_, notes);
        m.add(&format!("{name}_index"), "atlas", &format!("gfx/{name}.json"), &format!("{use_} (name -> rect index)"), "generated together with the sheet");
        Ok(())
    };
    sheet("terrain", "terrain tiles", "16x16, 4 variants per tile", &terrain::sheet)?;
    sheet("objects", "world objects (trees, runestones, cannons, ships...)", "32x32 cells", &objects::sheet)?;
    sheet("characters", "characters, NPCs, creatures, bosses", "24x24 cells, 11 frames x 4 dirs", &characters::sheet)?;
    sheet("items", "inventory item icons", "16x16", &icons::sheet)?;
    sheet("ui", "UI panels, slots, bars, paperdoll, cursors, runes", "mixed sizes", &ui::sheet)?;
    sheet("space", "planets, ship, bifrost gate, starfield", "64x64 bodies", &space::sheet)?;
    title::title().save(&root.join("gfx/title.png"))?;
    m.add("title", "image", "gfx/title.png", "main menu / intro background (320x180, scaled to the window)", "keep the 16:9 ratio");
    title::icon().save(&root.join("icon/icon.png"))?;
    m.add("app_icon", "image", "icon/icon.png", "application icon (macOS .icns / Windows .ico source, 256x256)", "used by tools/bundle_macos.sh");
    std::fs::create_dir_all(root.join("sfx")).map_err(|e| e.to_string())?;
    for s in audio::SFX {
        let mut r = u67_core::Rng::new(0xC0FFEE ^ s.id.len() as u64);
        let w = audio::wav_bytes(&(s.gen)(&mut r), audio::SFX_RATE);
        std::fs::write(root.join(format!("sfx/{}.wav", s.id)), w).map_err(|e| e.to_string())?;
        m.add(&format!("sfx_{}", s.id), "sfx", &format!("sfx/{}.wav", s.id), s.use_, "22050 Hz mono WAV");
    }
    std::fs::create_dir_all(root.join("music")).map_err(|e| e.to_string())?;
    for t in audio::TRACKS {
        let w = audio::wav_bytes(&audio::render_track(t, MUSIC_SECS), audio::MUSIC_RATE);
        std::fs::write(root.join(format!("music/{}.wav", t.id)), w).map_err(|e| e.to_string())?;
        m.add(&format!("music_{}", t.id), "music", &format!("music/{}.wav", t.id), t.use_, "16000 Hz mono WAV, 20 s loop");
    }
    m.entries.push(manifest::Entry {
        id: "font_ui".into(),
        kind: "font".into(),
        path: "fonts/UI.ttf".into(),
        use_: "all in-game text (embedded as the default font)".into(),
        source: "DejaVu Sans (free licence, see fonts/LICENSE-DejaVu.txt)".into(),
        status: "final".into(),
        notes: "TTF with Latin Extended + runic glyphs; replace with any TTF that has ä ö å and keep the filename (rebuild required: it is embedded with include_bytes!)"
            .into(),
    });
    Ok(m)
}
