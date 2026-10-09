# Ultima67

**By Crowelian 2026 + Sonnet** · GPL-3.0-or-later · Rust + [Bevy](https://bevyengine.org) 0.16

An **Ultima 7**-style open-world RPG remade as **Midgård**, with **Ultima 6** touches (shootable cannons, ships) and Nox-flavoured magic. Norse myth and the Finnish *Kalevala*, set in the modern day — runestones next to rifles — across our Solar System (Norse names) and a second five-planet Kalevala system behind the Sampo Gate.

Primary target: **macOS Apple Silicon (M1 Max)**. Windows and Linux are built by CI too. Solo adventure or **1–4 player local split-screen co-op**.

## Play

```bash
cargo run --release -p u67_game        # first run generates Midgård (a few seconds in release)
```
Assets are generated placeholders; see [`ASSETS.md`](ASSETS.md) for how to replace any of them. Regenerate with `cargo run -p u67_assetgen`.

### Controls (rebind in `settings.ron`, shown in the Options menu)
| Action | Keyboard / mouse (P1) | Keyboard 2 (P2) | Gamepad |
|---|---|---|---|
| Move / run | WASD / Shift | arrows / R-Shift | left stick / LB |
| Aim | mouse | — (faces move dir) | right stick |
| Attack | left click / Space | `.` | RT |
| Interact, talk, loot, board, revive | E / right click | `/` | A |
| Roll (i-frames) / reload | Ctrl / R | `,` / `;` | B / X |
| Inventory / journal / map | I / J / M | `'` | Y |
| Spells (3 hotkeys), book, sing runes | 1-3, B, L | 7-9 | D-pad |
| Console / cheats menu | `` ` `` or F1 / F2 | | |
| Quick save / load / pause | F5 / F9 / Esc | | Start |

Inventory is the Ultima 7 one: paperdoll + nested backpacks, drag & drop (shift-click quick-move, right-click use/open/equip, click outside to drop).

### Co-op
Choose *Players: 2-4* in the main menu (or `splitscreen 3` in the console). Shared world, quests and flags; each player has their own character, inventory, camera and HUD. Downed players can be revived by a teammate (interact) or recover on their own. Menus (inventory, dialogue, shops…) open full-screen for the player who triggered them.

### Cheats & console
Press `` ` `` for the console, F2 for the cheats menu. Commands are easy to remember: `give vainamoinen_gun`, `tp kaupang`, `planet maani`, `god`, `spawn troll 2`, `time 23:00`, `weather storm`… The complete list is in [`commands.txt`](commands.txt) (generated from the code; `help` in-game).

## Repository map
| Path | What |
|---|---|
| `crates/u67_core` | coordinates, RNG, clock, noise, save paths |
| `crates/u67_world` | tiles, maps, items, inventory, pathfinding, schedules, **rules & content schema** (combat, quests, dialogue, scripts, spells, crafting) |
| `crates/u67_console` | command registry/parser/autocomplete, generates `commands.txt` |
| `crates/u67_assetgen` | generates every sprite / sound / music placeholder + `assets/manifest.json` + `ASSETS.md` |
| `crates/u67_mapgen` | procedural Midgård (16 towns, caves, rune rings), 16 other worlds |
| `crates/u67_import` | **Ultima 7 importer**: your own U7 files -> Midgård (`docs/formats/u7.md`) |
| `crates/u67_game` | the Bevy game: rendering, input, seats, combat, AI, UI |
| `assets/data/*.json` | **all content**: quests, dialogue, NPCs, shops, creatures, spawns, loot, spells, recipes, lore |
| `docs/` | plan, lore bible, references/licensing |
| `tools/` | task sync, macOS/Windows/Linux packaging |

Project tracking for contributors/AI: [`HANDOFF.md`](HANDOFF.md) → [`TASKS.md`](TASKS.md) ← `PRD.json` · [`PRD.md`](PRD.md) · [`docs/PLAN.md`](docs/PLAN.md) · [`docs/LORE.md`](docs/LORE.md) · [`docs/REFERENCES.md`](docs/REFERENCES.md).

## Build & package
```bash
cargo test --workspace                 # ~70 tests incl. content validation, balance model
tools/bundle_macos.sh                  # Ultima67.app + .dmg (Apple Silicon by default)
tools/package_linux.sh                 # tar.gz (+ AppImage if appimagetool exists)
powershell -File tools/package_windows.ps1
```
Linux build deps: `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libx11-dev`.

## Your Ultima 7 data
Put the contents of your own legally-owned `STATIC` folder in `assets/original/u7/` (git-ignored; never committed) and run `cargo run --release -p u67_import --bin import_u7`: the U7 world map is converted into Midgård (see [`docs/formats/u7.md`](docs/formats/u7.md)). The generated Midgård works without them.
