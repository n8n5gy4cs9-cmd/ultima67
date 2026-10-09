# PRD - Ultima67

**By Crowelian 2026 + Sonnet.** Machine-readable task state lives in `PRD.json`.

## 1. Vision
A full single-player (plus splitscreen co-op) RPG with the feel of Ultima 7: a huge seamless map, NPCs with daily schedules, keyword dialogue, a drag-and-drop paperdoll/backpack inventory, party play. Re-skinned as Nordic/Finnish myth in the modern day: runestones next to assault rifles, Norse gods as ancient entities, Kalevala singers as spellcasters.

## 2. Pillars
1. **U7 soul:** world simulation, inventory, dialogue, exploration.
2. **Modern feel:** WASD/gamepad, mouse aim, hotkeys, readable UI, quicksave.
3. **U6 spice:** shootable cannons, ships, ship-to-ship fights.
4. **Nox spice:** class-based arena multiplayer (splitscreen), spell feel.
5. **Moddable:** all assets replaceable (ASSETS.md), content in data files, console + cheats.
6. **Simple:** fewest systems that deliver the above. Optional tasks are marked optional.

## 3. Setting
See `docs/LORE.md`. Sol system with Norse names is the main play area. A second 5-planet Kalevala system is a late-game expansion region.

## 4. Platforms
- Primary: macOS Apple Silicon (M1 Max), native arm64.
- Later: Windows x86_64. Optional: Linux.
- Rule: no platform-specific code outside `u67_core::platform`.

## 5. Functional requirements
| ID | Requirement | Tasks |
|---|---|---|
| F1 | Midgård map derived from U7's layout, modified into a Norse world | P4 |
| F2 | U7 inventory system retained (paperdoll, nested containers, drag-drop) | T5.2 |
| F3 | Modern controls with rebinding and gamepad | T5.1, T2.5 |
| F4 | Real-time combat incl. modern guns and cannons | T5.8, T5.9 |
| F5 | Ships/space-longships and star-map travel | T5.10, T6.1 |
| F6 | Two systems: Sol (main) + Kalevala (5 planets) | P6 |
| F7 | Main quest, hotspots, side + optional missions | T6.4-T6.6 |
| F8 | Cheats menu + developer console, all commands in `commands.txt` | P7 |
| F9 | Splitscreen co-op (2-4), OpenNox-style arena mode | P8 |
| F10 | All sprites/sfx/music generated as placeholders; documented in `ASSETS.md`, trivially replaceable | P3 |
| F11 | Save/load | T2.7 |

## 6. Non-functional
- 60 fps on M1 Max at 1080p+; load < 5 s.
- Deterministic simulation tick (fixed timestep) to allow replays / netcode later.
- `cargo fmt`, `clippy -D warnings`, and tests pass in CI.
- Licence hygiene: see `docs/REFERENCES.md`.

## 7. Out of scope (for now)
Online matchmaking, voice acting, mobile, mod workshop.

## 8. Milestones
M1 walk around Midgård with placeholder art (P2-P4) -> M2 inventory/dialogue/combat (P5) -> M3 space travel + quest (P6) -> M4 cheats/console (P7) -> M5 splitscreen (P8) -> M6 release builds (P9).
