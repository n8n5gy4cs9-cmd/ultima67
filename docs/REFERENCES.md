# Reference policy: port from upstream, GPL-3.0-or-later

User decision (2026-10-09): **use the upstream code and ideas freely**, and the user will supply their own legal Ultima 7 data at the end.

Upstream (cloned to `../refs/`, outside this repo, never committed):
- Exult (Ultima 7) - GPL-2.0-or-later - github.com/exult/exult
- Nuvie (Ultima 6) - GPL-2.0(+) - github.com/nuvie/nuvie
- OpenNox (Nox) - GPL-3.0 - github.com/noxworld-dev/opennox

## Rules
1. **This project is GPL-3.0-or-later** (compatible with all three). Porting/translating upstream code to Rust is allowed.
2. **Attribution:** any file that is a port or close adaptation starts with a header comment: `// Derived from <project> <path> (GPL-x), (c) its authors. Rust port (c) 2026 Crowelian + Sonnet.` and is listed in the table below.
3. **Original game data is NOT free:** Ultima 6/7 and Nox maps, graphics, music, sfx are EA/Westwood property. Never commit them. The user provides their legal copy locally (git-ignored `assets/original/`). The repo ships only generated placeholder assets + a procedural Midgard so it works without original data.
4. Prefer idiomatic Rust over line-by-line translation (no unsafe pointer-style ports); write tests for ported logic.
5. Do not add dependencies with GPL-incompatible licences.

## Where to look upstream (map of ideas)
| Topic | Upstream place to start |
|---|---|
| Map/chunk/object model | exult `gamemap.cc`, `chunks.cc`, `objs/`, `shapes/` |
| Shape/frame/palette file formats | exult `shapes/`, `files/`, `vgafile.cc`, `data/` |
| Usecode VM (scripting) | exult `usecode/` |
| NPC schedules / AI / combat | exult `schedule.cc`, `actors.cc`, `combat.cc` |
| Inventory / paperdoll / gumps | exult `gumps/` |
| Cheat screen ideas | exult `cheat.cc`, `cheat_screen.cc` |
| U6 cannons, ships, UI | nuvie `Actor*`, `Script/`, `Party`, `MapWindow`, `ViewManager` |
| Nox classes, spells, netcode | opennox `internal/`, `common/`, `noxnet` etc. |

## Files derived from upstream (keep updated)
| Our file | Derived from | Licence |
|---|---|---|
| (none yet) | | |

## Study notes (own words, fill during P1)
| Topic | Notes |
|---|---|
| Chunk/map format | |
| Shape/frame format | |
| Usecode VM | |
| Schedules | |
| Cannons / ships | |
| Nox classes/spells | |
