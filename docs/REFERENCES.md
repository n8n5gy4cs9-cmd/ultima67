# References & licensing

Repos to study (clone to `../refs/`, **outside** this repo):
- https://github.com/exult/exult - Ultima 7 engine reimplementation (C++, GPL-2.0+). Study: `shapes/`, `usecode/`, `actors.cc`, `schedule.cc`, `gamemap.cc`, `chunks`.
- https://github.com/nuvie/nuvie - Ultima 6 engine (C++, GPL-2.0+). Study: cannons, ships, combat, UI, `Actor`, `Script/`.
- https://github.com/noxworld-dev/opennox - Nox engine reimplementation (Go + C, **check LICENSE**). Study: classes, spells, netcode, map format, quest/arena modes.
- https://github.com/exult - org page (also holds `exult-sdk`, tools, `exultstudio`).

## What "clone" means here
We are writing an **original Rust engine** inspired by these projects, not a verbatim port.
- Algorithms and file-format knowledge may be re-implemented from reading the sources.
- **Copying code** from GPL repos into this repo makes it GPL. Decision (default): this project is **GPL-2.0-or-later** if any code is ported; otherwise keep it MIT/Apache. Record the choice in `LICENSE` at T2.1 and note every ported file here.
- **Original game data** (Ultima 6/7 maps, shapes, music, sfx; Nox assets) is copyrighted by EA/Westwood. It is NOT in those engine repos (they require the user's own copy). **Do not commit it.** The shipped game uses procedurally generated assets + a procedural Midgård; the U7 importer only processes files the user legally owns.
- Therefore "copy sfx + music from reference" is only done when the file is licence-clean (T3.13, optional). Default is generated placeholders.

## Study notes (fill during P1)
| Topic | Source file(s) | Notes |
|---|---|---|
| Chunk/map format | | |
| Shape/frame format | | |
| Usecode VM | | |
| NPC schedules | | |
| Inventory/containers | | |
| Cannons (U6) | | |
| Ships (U6) | | |
| Nox classes/spells | | |
| Nox netcode | | |
