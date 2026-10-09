//! Generates ASSETS.md from the manifest.
use crate::manifest::Manifest;

pub fn assets_md(m: &Manifest) -> String {
    let mut s = String::new();
    s += "# ASSETS\n\n";
    s += "> **GENERATED** by `cargo run -p u67_assetgen` from `assets/manifest.json`. Do not edit by hand; edit the generator or the manifest notes.\n";
    s += "> **To replace an asset:** overwrite the file at its path with a file of the same name, format and size (see section 2), then restart (dev builds hot-reload). Set its `status` to `final` / `source` to `custom` in `assets/manifest.json` if you want the table to reflect it.\n";
    s += "> Everything below is a *generated placeholder* (no original game data). Regenerating overwrites files, so keep your replacements out of the generator's way: run `cargo run -p u67_assetgen -- --only <group>` or just don't regenerate.\n\n";
    s += "## 1. Clean list (use | name | path | source)\n\n| Use | Name | Path | Source |\n|---|---|---|---|\n";
    for e in &m.entries {
        s += &format!("| {} | `{}` | `assets/{}` | {} ({}) |\n", e.use_, e.id, e.path, e.source, e.status);
    }
    s += r#"
## 2. Details

### Graphics (PNG, RGBA, transparent background)
Every sheet has a sibling `*.json` atlas listing `name -> x,y,w,h`. The game slices sheets by name, so you can repaint a sheet freely as long as **cell size and positions stay the same**.

| Sheet | Cell | Layout |
|---|---|---|
| `gfx/terrain.png` | 16x16 | row = tile id (order of `u67_world::tiles::TILES`), 4 variants across |
| `gfx/objects.png` | 32x32 | row per object (`u67_world::objects::OBJECTS`), up to 4 frames across. cannon & longship frames: N,E,S,W. Last row = snowy pine |
| `gfx/characters.png` | 24x24 | 4 rows per archetype (N,E,S,W), 11 columns: idle, walk1-4, attack1-3, die1-3. Archetypes: see `characters.json` (humanoids then creatures) |
| `gfx/items.png` | 16x16 | 8 per row, order of `u67_world::items::ITEMS`; names in `items.json` |
| `gfx/ui.png` | mixed | panels (9-slice 48x48 / 96x48), slot 18x18, buttons 48x16, bars 64x8, paperdoll 80x96, cursors 16x16, 24 runes 12x16; see `ui.json` |
| `gfx/space.png` | 64x64 | 17 bodies (Sol system + Kalevala system), ship 4x32x32, bifrost gate, starfield |

Art direction: chunky Starbound-like pixel art, 1px dark outline, limited per-material palettes, Norse motifs (runes, longships, horned helms) mixed with modern items (kevlar, rifles, vacuum suits).

### Sound effects (`sfx/*.wav`)
16-bit mono PCM WAV, 22050 Hz. Loops (names starting `ambient_`) must loop seamlessly. Any length is fine; keep under ~2 MB. Replace with OGG only after enabling Bevy's `vorbis` feature and changing the extension in the manifest.

### Music (`music/*.wav`)
16-bit mono PCM WAV, 16000 Hz, 20 s seamless loops (generated placeholders: modal scales, drone + random-walk melody). Replace with longer/higher-quality tracks of the same name; stereo and other sample rates are fine once the loader is switched to OGG (see TASKS T3.13).

### Maps
Not stored in the repo: the game builds Midgard, caves and all planets at startup from seed 67 (`u67_mapgen::generate_world`). `cargo run --release -p u67_mapgen --bin gen_maps` dumps them as `.u67map` files (binary: header, JSON meta, RLE tiles) into `assets/maps/` (git-ignored); if a `.u67map` with the same name exists there the game loads it instead of generating, so you can hand-edit maps (map editor T4.9) or import your own Ultima 7 data (T4.2, local only, never committed).

### Original game data
If you own Ultima 7 / Ultima 6, put your files in `assets/original/` (git-ignored). Importers use them as optional upgrades; the repository never contains them.
"#;
    s
}
