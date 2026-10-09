# ASSETS

> **Status: skeleton.** `u67_assetgen` (T3.2+) will generate every placeholder below and regenerate this file from `assets/manifest.ron` (T3.11).
> **To replace an asset:** overwrite the file at its path with one of the same name and format, then (dev build) it hot-reloads, or restart. Keep size/format listed in the notes.

## 1. Clean list (use | name | path | source)

| Use | Name | Path | Source |
|---|---|---|---|
| Terrain tileset | terrain | `assets/gfx/terrain.png` | generated placeholder (planned) |
| Object sprites | objects | `assets/gfx/objects.png` | generated placeholder (planned) |
| Characters | characters | `assets/gfx/characters.png` | generated placeholder (planned) |
| Items/icons | items | `assets/gfx/items.png` | generated placeholder (planned) |
| UI | ui | `assets/gfx/ui.png` | generated placeholder (planned) |
| Space/planet art | space | `assets/gfx/space.png` | generated placeholder (planned) |
| SFX (footstep, gun, cannon, hit, door, ui, spell, ambient) | sfx_* | `assets/sfx/*.wav` | generated placeholder (planned) |
| Music (menu, towns, wilds, space, combat, boss) | music_* | `assets/music/*.ogg` | generated placeholder (planned) |
| Maps | midgard + bodies | `assets/maps/*.ron` | procedural (planned) |

## 2. Detailed notes (fill as generated)

Conventions planned:
- Tiles 16x16 (terrain), objects/characters 16x16 or 16x32 on a grid sheet; Starbound-like chunky pixel style with Norse motifs, limited palette per biome.
- Characters: 4 directions x (idle 1, walk 4, attack 3, die 3 frames).
- SFX: 16-bit mono WAV 44.1 kHz. Music: OGG loops.
- Each entry in the manifest has: `id, kind, path, source (generated|reference|custom), status (placeholder|final), notes`.
