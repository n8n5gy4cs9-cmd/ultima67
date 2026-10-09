# Technical Plan

## Tech choices
- **Rust + Bevy** (wgpu -> Metal on Apple Silicon, DX12/Vulkan elsewhere). ECS suits many NPCs/objects; multiple cameras give splitscreen. Pin one Bevy version and do not upgrade mid-project.
- Scripting: **Lua via `mlua`** (vendored feature, works on all platforms) for quests/dialogue/object use, mirroring U7 "usecode". Fallback: RON-only data if mlua is troublesome.
- Data formats: RON for config, JSON for manifest/content, PNG for sprites, OGG/WAV for audio, serde+bincode/RON for saves.
- Audio: Bevy audio (rodio). Music loops generated as WAV/OGG.
- Rendering: 2D tile renderer, Ultima 7-style 3/4 view (top-down tiles with tall objects, z-order by y + z). Optional isometric later; do not start there.

## Workspace layout
```
ultima67/
  Cargo.toml                 (workspace)
  crates/
    u67_core/                shared types, platform paths, RNG, math, serde save
    u67_world/               tile grid, chunks, objects, pathfinding, schedules, map IO
    u67_game/                Bevy app: states, rendering, input, UI, combat, AI   (binary: ultima67)
    u67_script/              Lua bindings, dialogue/quest engine
    u67_console/             command registry, parser, cheats, commands.txt generation
    u67_assetgen/            procedural sprite/sfx/music generator + manifest writer (binary)
    u67_mapgen/              procedural Midgard + planet maps; U7 importer (binary)
    u67_net/                 splitscreen input routing now; netcode later
  assets/                    gfx/ sfx/ music/ maps/ data/ manifest.ron
  tools/                     sync_tasks.py etc.
  docs/
```

## Key designs
**World.** 3072x3072 tiles in 192x192 chunks of 16x16 (as U7), 16 z-levels max. Terrain layer + object layer. Objects: `{shape_id, frame, x, y, z, flags, container?}`. Containers nest and are the basis of inventory.

**Simulation.** Fixed 30 Hz tick for gameplay; render interpolated. NPCs only simulated in full near the player; far NPCs advance by schedule teleport (U7 behaviour). Game clock: 1 real minute = 20 game minutes (tunable).

**Inventory.** Paperdoll slots (head, neck, torso, legs, feet, hands L/R, ammo, ring x2, back), backpack/containers as windows, drag-drop between container/ground/paperdoll, weight limit from STR. Modern addition: shift-click quick-move, hotbar 1-9 for equipped consumables/weapons.

**Controls.** WASD/left stick move; mouse/right stick aim; LMB use/attack; RMB look; E interact; I inventory (U7 style); Tab party; Space dodge/roll; R reload; 1-9 hotbar; \` console; F2 cheats; F5/F9 quicksave/load. All rebindable. U7 point-and-click movement kept as an option.

**Combat.** Real-time. Melee (axe/spear/sword), ranged (bows), firearms (pistol, shotgun, rifle, SMG, kantele-rifle) with ammo types, spread, recoil, cover. **Cannons:** world objects with state {empty, loaded(powder,ball), lit}; player aims with mouse, fires; projectile arc with splash damage; mountable on ships and fortifications (U6).

**Ships.** Vehicle entity with deck map; wind/engine movement on water, "space longship" mode on the star map. Cannons mounted via deck slots.

**Magic.** Seidr/runes with reagents (U7) and Kalevala *laulu* (sung spells: choose a short rune sequence). Mana from "väki" stat.

**Multiplayer.** Phase 1: local splitscreen, one world, N cameras, input devices mapped to player entities (`u67_net::LocalSeat`). Arena mode reuses the same code with classes. Netcode (optional) = deterministic lockstep, which the fixed tick enables.

**Asset pipeline.** Everything the game loads is listed in `assets/manifest.ron` (id, kind, path, source, status). Game loads by id, so replacing the file at the path replaces the asset. `u67_assetgen` writes placeholders and `ASSETS.md`. Dev builds hot-reload.

**Map pipeline.** (a) `u67_mapgen --procedural` produces the shipped Midgård with the U7 macro-layout rebuilt from scratch (no EA data). (b) `u67_mapgen --import-u7 <path>` lets a user who owns U7 import their copy through Exult-style readers and apply the Norse remap table. Output goes in `assets/maps/` only locally and is git-ignored if derived from original data.

## Roadmap order
P1 study -> P2 skeleton -> P3 assets (needed by everything visual) -> P4 world -> P5 gameplay -> P6 content -> P7 console (start the registry early, in P2/P5, as it speeds testing) -> P8 multiplayer -> P9 polish.

## Risks
- Bevy compile times: use `opt-level=1` dev profile with deps at 3, dynamic linking in dev.
- Scope: build vertical slice first (one town, one dungeon, one planet) before widening.
- Licence: GPL-3.0-or-later (ports from Exult/Nuvie/OpenNox allowed).
