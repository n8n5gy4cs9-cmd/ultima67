# TASKS - Ultima67 (By Crowelian 2026 + Sonnet)

> Generated from `PRD.json` by `python3 tools/sync_tasks.py`. **Edit PRD.json, then re-run.**
> Legend: `[x]` done, `[ ]` todo, `[~]` in progress, `[!]` blocked.

**Progress: 11/79 done.**  
**Next action:** T2.1-T2.7 workspace; headless crates first (world, console, assetgen, mapgen), Bevy game last.

## P0 - Planning & docs

- [x] **T0.1** PRD.md, PRD.json, TASKS.md, HANDOFF.md
- [x] **T0.2** docs/PLAN.md architecture + roadmap
- [x] **T0.3** docs/LORE.md story, planets, factions, missions
- [x] **T0.4** docs/CHEATS.md + commands.txt (planned command set)
- [x] **T0.5** ASSETS.md skeleton + assets manifest format
- [x] **T0.6** docs/REFERENCES.md (exult/nuvie/opennox study notes + licensing)
- [x] **T0.7** tools/sync_tasks.py (PRD.json -> TASKS.md)

## P1 - Reference study

- [x] **T1.0** Study upstream in ../refs (exult, nuvie, opennox); fill docs/REFERENCES.md map + notes
- [x] **T1.0a** Study exult: world/chunk model, schedules, party AI, inventory -> own notes
- [x] **T1.0b** Study nuvie: cannons, ships, combat, UI -> own notes
- [x] **T1.0c** Study opennox: classes, spells, arena modes, netcode -> own notes

## P2 - Workspace & engine skeleton

- [ ] **T2.1** Cargo workspace: u67_core, u67_world, u67_game, u67_assetgen, u67_console, u67_net
- [ ] **T2.2** Bevy app boots window on macOS arm64, 60fps, fixed-timestep sim
- [ ] **T2.3** GitHub Actions CI: fmt, clippy, test on macOS+Linux; Windows build job
- [ ] **T2.4** Platform abstraction: paths via `directories`, no mac-only APIs
- [ ] **T2.5** Config/settings file (RON), keybinding + gamepad remap
- [ ] **T2.6** Game state machine: Boot, MainMenu, Playing, Paused, Inventory, Dialogue, Console
- [ ] **T2.7** Save/load (serde + versioned), 10 slots + quicksave

## P3 - Asset pipeline & placeholder generation

- [ ] **T3.1** assets/manifest.json: every asset id -> path, kind, source, status
- [ ] **T3.2** u67_assetgen: procedural Starbound-style Norse pixel sprite generator (seeded, deterministic)
- [ ] **T3.3** Generate terrain tileset (grass, snow, rock, water, lava, ice, void, regolith, gas, floors, walls)
- [ ] **T3.4** Generate object sprites (trees, runestones, longhouses, chests, doors, cannons, ships, furniture)
- [ ] **T3.5** Generate character sprites 4-dir x walk/idle/attack/die (player, party, NPC archetypes, creatures, bosses)
- [ ] **T3.6** Generate weapon/item/inventory icons (modern guns + Norse relics)
- [ ] **T3.7** Generate UI art (paperdoll frame, backpack, containers, dialogue box, HUD, runic font)
- [ ] **T3.8** Generate planet/space art (star map, planet surfaces, ship, Bifrost gate)
- [ ] **T3.9** Placeholder SFX generator (WAV synth: steps, gun, cannon, hit, door, UI, spell, ambient)
- [ ] **T3.10** Placeholder music generator (procedural MIDI-like -> OGG/WAV, one loop per region + combat + menu)
- [ ] **T3.11** Auto-generate ASSETS.md from manifest (top: clean table use|name|path|source; below: detailed notes)
- [ ] **T3.12** Hot-reload assets in dev builds so replacing a file shows instantly
- [ ] **T3.13** Use user-supplied original sfx/music from assets/original/ as optional upgrades (never committed) _(optional)_

## P4 - World: Midgard map (U7 layout remake)

- [ ] **T4.1** World model: map grid, 16x16 chunks, interiors as separate maps (z via portals)
- [ ] **T4.2** U7 importer: read the USER-OWNED Ultima 7 data from assets/original/ (port ideas/format readers from exult); output Midgard-remapped map. Never commit data.
- [ ] **T4.3** Midgard transformer: remap U7 terrain/objects to Norse equivalents (towns -> Norse/Finnish villages, Britannia coast -> fjords)
- [ ] **T4.4** Fallback: procedural Midgard map generator with same macro-layout (works w/o U7 data; ships in repo)
- [ ] **T4.5** Map renderer: layered isometric-ish/top-down tile rendering w/ object depth sorting, roofs hide on entry
- [ ] **T4.6** Day/night cycle, weather, lighting
- [ ] **T4.7** Interiors/dungeons as separate z/instance maps
- [ ] **T4.8** Towns: Uppsala-ish Kaupang, Birka, Hedeby, Kuusamo, Rovaniemi-like Pohjola, Hel gate, 10+ more
- [ ] **T4.9** Map editor mode (in-game dev tool) to hand-edit tiles/objects _(optional)_

## P5 - Core gameplay (U7 feel, modern controls)

- [ ] **T5.1** Modern controls: WASD/gamepad move, mouse aim, click-to-interact, hotkeys; keep U7 click-drag item handling
- [ ] **T5.2** U7 inventory: paperdoll, backpack, nested containers, drag&drop, weight/volume, ground items
- [ ] **T5.3** Object interaction: use/open/get/talk/look, locks, keys, traps
- [ ] **T5.4** Dialogue system U7 style (keyword topics, portraits, flags) with RON/Ink-like scripts
- [ ] **T5.5** Party system (up to 6), follow AI, party panel, commands
- [ ] **T5.6** NPC schedules (sleep/work/eat/pub) driven by time of day
- [ ] **T5.7** Stats/skills/levels (STR DEX INT, HP, mana/seidr), XP, loot tables
- [ ] **T5.8** Real-time combat: melee, ranged, modern guns (ammo, reload, recoil, aim), cover, stealth
- [ ] **T5.9** Shootable cannons (U6): placeable/fixed cannons, load powder+ball, aim, fire, damage ships/walls/enemies
- [ ] **T5.10** Ships & vehicles (U6/U7): longships, sleds, skiffs, space-longship; boarding, cannons on ships
- [ ] **T5.11** Magic: runes/seidr spells in U7 reagent+circle style with Kalevala singing-magic (laulu)
- [ ] **T5.12** Crafting, cooking, forging (Sampo-forge), alchemy
- [ ] **T5.13** Shops/barter, currency (silver/hacksilver + krediitti)
- [ ] **T5.14** Journal/quest log, map screen, automap, fast travel via Bifrost nodes
- [ ] **T5.15** Usecode-like scripting VM or Lua (mlua) for quests/objects; data-driven content
- [ ] **T5.16** Enemy AI: wolves, draugr, trolls, Fenrir spawn, Louhi's forces, corporate mercs w/ modern guns

## P6 - Solar system, travel & content

- [ ] **T6.1** Star map UI + ship travel between bodies (Sol system: Sol, Mani, Midgard, Muspel, Asgard, Jotun, Nifl, Hel, belt Svartalf)
- [ ] **T6.2** Each body: hand-seeded planetary map + biome + hazards (cold, heat, vacuum suit)
- [ ] **T6.3** Second system 'Kalevala' with 5 planets (Vainola, Pohjola, Tuonela, Ilma, Sampola) behind the Bifrost-Sampo gate
- [ ] **T6.4** Main quest line (see docs/LORE.md) end to end
- [ ] **T6.5** 10+ hotspots per system (ruins, wrecks, rune-sites, boss lairs, secret caches)
- [ ] **T6.6** 20+ side missions and 10+ optional tasks
- [ ] **T6.7** Bosses: Fenrir-Prime, Jormungandr (sea), Louhi, Surma, Hel-Queen, Nidhogg
- [ ] **T6.8** Unique gun: vainamoinen_gun (Kantele-rifle) + other legendary relics

## P7 - Cheats & developer console

- [ ] **T7.1** Console (backtick/F1): input, history, autocomplete, scrollback, command registry
- [ ] **T7.2** Implement commands from commands.txt (give, tp, god, noclip, spawn, set, flag, time, weather, quest, heal, xp, ...)
- [ ] **T7.3** Cheats menu (F2) with toggles: god, infinite ammo, noclip, fast, all spells, reveal map
- [ ] **T7.4** Dev tools: show colliders, entity inspector, FPS, reload assets, run script
- [ ] **T7.5** commands.txt auto-generated from registry (test fails if out of sync)

## P8 - Multiplayer

- [ ] **T8.1** Local splitscreen co-op (2-4 players): per-player camera, controller/keyboard assignment, shared world
- [ ] **T8.2** Splitscreen per-player inventory UI + shared/separate party modes
- [ ] **T8.3** OpenNox-style Arena mode: classes Warrior/Conjurer/Wizard -> Berserker/Volva/Skald, deathmatch, CTF, on splitscreen
- [ ] **T8.4** Optional: LAN/online netcode (lockstep or rollback) via u67_net _(optional)_

## P9 - Polish, packaging, platforms

- [ ] **T9.1** Main menu, options, credits ('By Crowelian 2026 + Sonnet'), intro cinematic (slides)
- [ ] **T9.2** macOS .app bundle + dmg, codesign notes; verify Apple Silicon native
- [ ] **T9.3** Windows build + installer zip
- [ ] **T9.4** Linux build + AppImage _(optional)_
- [ ] **T9.5** Balance pass, bug bash, performance profile (target 60fps on M1 Max)
- [ ] **T9.6** Accessibility: rebinding, text size, colorblind palettes _(optional)_
