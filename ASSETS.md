# ASSETS

> **GENERATED** by `cargo run -p u67_assetgen` from `assets/manifest.json`. Do not edit by hand; edit the generator or the manifest notes.
> **To replace an asset:** overwrite the file at its path with a file of the same name, format and size (see section 2), then restart (dev builds hot-reload). Set its `status` to `final` / `source` to `custom` in `assets/manifest.json` if you want the table to reflect it.
> Everything below is a *generated placeholder* (no original game data). Regenerating overwrites files, so keep your replacements out of the generator's way: run `cargo run -p u67_assetgen -- --only <group>` or just don't regenerate.

## 1. Clean list (use | name | path | source)

| Use | Name | Path | Source |
|---|---|---|---|
| terrain tiles | `terrain` | `assets/gfx/terrain.png` | generated (placeholder) |
| terrain tiles (name -> rect index) | `terrain_index` | `assets/gfx/terrain.json` | generated (placeholder) |
| world objects (trees, runestones, cannons, ships...) | `objects` | `assets/gfx/objects.png` | generated (placeholder) |
| world objects (trees, runestones, cannons, ships...) (name -> rect index) | `objects_index` | `assets/gfx/objects.json` | generated (placeholder) |
| characters, NPCs, creatures, bosses | `characters` | `assets/gfx/characters.png` | generated (placeholder) |
| characters, NPCs, creatures, bosses (name -> rect index) | `characters_index` | `assets/gfx/characters.json` | generated (placeholder) |
| inventory item icons | `items` | `assets/gfx/items.png` | generated (placeholder) |
| inventory item icons (name -> rect index) | `items_index` | `assets/gfx/items.json` | generated (placeholder) |
| UI panels, slots, bars, paperdoll, cursors, runes | `ui` | `assets/gfx/ui.png` | generated (placeholder) |
| UI panels, slots, bars, paperdoll, cursors, runes (name -> rect index) | `ui_index` | `assets/gfx/ui.json` | generated (placeholder) |
| planets, ship, bifrost gate, starfield | `space` | `assets/gfx/space.png` | generated (placeholder) |
| planets, ship, bifrost gate, starfield (name -> rect index) | `space_index` | `assets/gfx/space.json` | generated (placeholder) |
| walking | `sfx_footstep` | `assets/sfx/footstep.wav` | generated (placeholder) |
| walking on snow | `sfx_footstep_snow` | `assets/sfx/footstep_snow.wav` | generated (placeholder) |
| pistol shot | `sfx_gun_pistol` | `assets/sfx/gun_pistol.wav` | generated (placeholder) |
| rifle shot | `sfx_gun_rifle` | `assets/sfx/gun_rifle.wav` | generated (placeholder) |
| shotgun blast | `sfx_gun_shotgun` | `assets/sfx/gun_shotgun.wav` | generated (placeholder) |
| reloading a gun | `sfx_reload` | `assets/sfx/reload.wav` | generated (placeholder) |
| cannon firing | `sfx_cannon_fire` | `assets/sfx/cannon_fire.wav` | generated (placeholder) |
| explosions / cannonball impact | `sfx_explosion` | `assets/sfx/explosion.wav` | generated (placeholder) |
| melee hit on creature | `sfx_hit_flesh` | `assets/sfx/hit_flesh.wav` | generated (placeholder) |
| weapon on armor | `sfx_hit_metal` | `assets/sfx/hit_metal.wav` | generated (placeholder) |
| melee swing | `sfx_swing` | `assets/sfx/swing.wav` | generated (placeholder) |
| creature death | `sfx_death` | `assets/sfx/death.wav` | generated (placeholder) |
| door open | `sfx_door_open` | `assets/sfx/door_open.wav` | generated (placeholder) |
| door close | `sfx_door_close` | `assets/sfx/door_close.wav` | generated (placeholder) |
| chest/container open | `sfx_chest_open` | `assets/sfx/chest_open.wav` | generated (placeholder) |
| pick up item | `sfx_pickup` | `assets/sfx/pickup.wav` | generated (placeholder) |
| drop item | `sfx_drop` | `assets/sfx/drop.wav` | generated (placeholder) |
| money | `sfx_coin` | `assets/sfx/coin.wav` | generated (placeholder) |
| button click | `sfx_ui_click` | `assets/sfx/ui_click.wav` | generated (placeholder) |
| open inventory / menu | `sfx_ui_open` | `assets/sfx/ui_open.wav` | generated (placeholder) |
| close menu | `sfx_ui_close` | `assets/sfx/ui_close.wav` | generated (placeholder) |
| invalid action / console error | `sfx_ui_error` | `assets/sfx/ui_error.wav` | generated (placeholder) |
| casting a spell (seidr/laulu) | `sfx_spell_cast` | `assets/sfx/spell_cast.wav` | generated (placeholder) |
| failed spell | `sfx_spell_fizzle` | `assets/sfx/spell_fizzle.wav` | generated (placeholder) |
| level up / quest done | `sfx_level_up` | `assets/sfx/level_up.wav` | generated (placeholder) |
| new quest | `sfx_quest_start` | `assets/sfx/quest_start.wav` | generated (placeholder) |
| bifrost travel | `sfx_bifrost` | `assets/sfx/bifrost.wav` | generated (placeholder) |
| ship movement | `sfx_ship_creak` | `assets/sfx/ship_creak.wav` | generated (placeholder) |
| water splash | `sfx_splash` | `assets/sfx/splash.wav` | generated (placeholder) |
| wind loop (snow, mountains, space-planets) | `sfx_ambient_wind` | `assets/sfx/ambient_wind.wav` | generated (placeholder) |
| forest loop (birds + rustle) | `sfx_ambient_forest` | `assets/sfx/ambient_forest.wav` | generated (placeholder) |
| waves loop | `sfx_ambient_sea` | `assets/sfx/ambient_sea.wav` | generated (placeholder) |
| dungeon/cave loop (drips + drone) | `sfx_ambient_cave` | `assets/sfx/ambient_cave.wav` | generated (placeholder) |
| ship / space station hum loop | `sfx_ambient_space` | `assets/sfx/ambient_space.wav` | generated (placeholder) |
| wolf / Fenrir | `sfx_wolf_howl` | `assets/sfx/wolf_howl.wav` | generated (placeholder) |
| kantele pluck (Väinämöinen's gun, singing magic) | `sfx_kantele` | `assets/sfx/kantele.wav` | generated (placeholder) |
| main menu | `music_menu` | `assets/music/menu.wav` | generated (placeholder) |
| towns of Midgard | `music_midgard_town` | `assets/music/midgard_town.wav` | generated (placeholder) |
| overworld, forests, fjords | `music_midgard_wilds` | `assets/music/midgard_wilds.wav` | generated (placeholder) |
| caves, ruins, barrows | `music_dungeon` | `assets/music/dungeon.wav` | generated (placeholder) |
| star map and space travel | `music_space` | `assets/music/space.wav` | generated (placeholder) |
| fights | `music_combat` | `assets/music/combat.wav` | generated (placeholder) |
| boss fights | `music_boss` | `assets/music/boss.wav` | generated (placeholder) |
| Kalevala system (kantele/runolaulu feel) | `music_kalevala` | `assets/music/kalevala.wav` | generated (placeholder) |
| Hel, Tuonela and the dead | `music_hel` | `assets/music/hel.wav` | generated (placeholder) |

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

### Maps (`maps/*.json`)
Generated by `u67_mapgen` (procedural Midgard + planets). Replace by editing in the map editor (T4.9) or by importing your own Ultima 7 data (T4.2, local only, never committed).

### Original game data
If you own Ultima 7 / Ultima 6, put your files in `assets/original/` (git-ignored). Importers use them as optional upgrades; the repository never contains them.
