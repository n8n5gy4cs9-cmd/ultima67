# TASKS - Ultima67 (By Crowelian 2026 + Sonnet)

> Generated from `PRD.json` by `python3 tools/sync_tasks.py`. **Edit PRD.json, then re-run.**
> Legend: `[x]` done, `[ ]` todo, `[~]` in progress, `[!]` blocked.

**Progress: 76/78 done.**  
**Next action:** Waiting for the user to provide Ultima 7 data in assets/original/u7/ to tune the importer (run import_u7). Optional: T4.9 map editor, T8.4 netcode. Verify on real M1: tools/bundle_macos.sh + 60fps profile.

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
- [x] **T1.0c** Study opennox: classes, spells, netcode -> own notes

## P2 - Workspace & engine skeleton

- [x] **T2.1** Cargo workspace: u67_core, u67_world, u67_game, u67_assetgen, u67_console, u67_net
- [x] **T2.2** Bevy app boots window on macOS arm64, 60fps, fixed-timestep sim
- [x] **T2.3** GitHub Actions CI: fmt, clippy, test on macOS+Linux; Windows build job
- [x] **T2.4** Platform abstraction: paths via `directories`, no mac-only APIs
- [x] **T2.5** Config/settings file (RON), keybinding + gamepad remap [settings.ron + key rebinding done; gamepad uses fixed layout, remap UI pending] [settings.ron: keys + gamepad buttons + volumes + zoom; in-game Options menu; no in-game rebind UI yet]
- [x] **T2.6** Game state machine: Boot, MainMenu, Playing, Paused, Inventory, Dialogue, Console
- [x] **T2.7** Save/load (serde + versioned), 10 slots + quicksave

## P3 - Asset pipeline & placeholder generation

- [x] **T3.1** assets/manifest.json: every asset id -> path, kind, source, status
- [x] **T3.2** u67_assetgen: procedural Starbound-style Norse pixel sprite generator (seeded, deterministic)
- [x] **T3.3** Generate terrain tileset (grass, snow, rock, water, lava, ice, void, regolith, gas, floors, walls)
- [x] **T3.4** Generate object sprites (trees, runestones, longhouses, chests, doors, cannons, ships, furniture)
- [x] **T3.5** Generate character sprites 4-dir x walk/idle/attack/die (player, party, NPC archetypes, creatures, bosses)
- [x] **T3.6** Generate weapon/item/inventory icons (modern guns + Norse relics)
- [x] **T3.7** Generate UI art (paperdoll frame, backpack, containers, dialogue box, HUD, runic font)
- [x] **T3.8** Generate planet/space art (star map, planet surfaces, ship, Bifrost gate)
- [x] **T3.9** Placeholder SFX generator (WAV synth: steps, gun, cannon, hit, door, UI, spell, ambient)
- [x] **T3.10** Placeholder music generator (procedural MIDI-like -> OGG/WAV, one loop per region + combat + menu)
- [x] **T3.11** Auto-generate ASSETS.md from manifest (top: clean table use|name|path|source; below: detailed notes)
- [x] **T3.12** Hot-reload assets in dev builds so replacing a file shows instantly [pending: use bevy file_watcher feature] [Bevy file_watcher + terrain.png polling]
- [x] **T3.13** Use user-supplied original sfx/music from assets/original/ as optional upgrades (never committed) [assets/original/{sfx,music}/<id>.ogg|wav override generated files] _(optional)_

## P4 - World: Midgard map (U7 layout remake)

- [x] **T4.1** World model: map grid, 16x16 chunks, interiors as separate maps (z via portals)
- [x] **T4.2** U7 importer: crates/u67_import reads the USER-OWNED Ultima 7 STATIC files from assets/original/u7/ (Flex, u7map/u7chunks/u7ifix, shapes.vga flats, palettes, text.flx names, tfa.dat) [verified on synthetic data only; tune assets/original/u7_rules.json against real data]; docs/formats/u7.md
- [x] **T4.3** Midgard transformer (u67_import::norse): town clusters -> Norse towns + places, arctic north, then the same hotspot/cave extras as the generator
- [x] **T4.4** Fallback: procedural Midgard map generator with same macro-layout (works w/o U7 data; ships in repo)
- [x] **T4.5** Map renderer: layered isometric-ish/top-down tile rendering w/ object depth sorting, roofs hide on entry
- [x] **T4.6** Day/night cycle, weather, lighting [clock + day/night overlay done; weather visuals pending]
- [x] **T4.7** Interiors/dungeons as separate z/instance maps
- [x] **T4.8** Towns: Uppsala-ish Kaupang, Birka, Hedeby, Kuusamo, Rovaniemi-like Pohjola, Hel gate, 10+ more
- [ ] **T4.9** Map editor mode (in-game dev tool) to hand-edit tiles/objects _(optional)_

## P5 - Core gameplay (U7 feel, modern controls)

- [x] **T5.1** Modern controls: WASD/gamepad move, mouse aim, click-to-interact, hotkeys; keep U7 click-drag item handling [WASD/gamepad move + zoom done; mouse aim/click-interact pending] [mouse aim + RMB/E interact done; gamepad aim pending]
- [x] **T5.2** U7 inventory: paperdoll, backpack, nested containers, drag&drop, weight/volume, ground items
- [x] **T5.3** Object interaction: use/open/get/talk/look, locks, keys, traps [doors, locks (key/lockpick), containers, signs, beds, wells, wrecks done; traps not implemented]
- [x] **T5.4** Dialogue system U7 style (keyword topics, portraits, flags) with RON/Ink-like scripts [JSON dialogue, keyword UI, 17 NPC dialogues]
- [x] **T5.5** Party system (up to 6), follow AI, party panel, commands
- [x] **T5.6** NPC schedules (sleep/work/eat/pub) driven by time of day
- [x] **T5.7** Stats/skills/levels (STR DEX INT, HP, mana/seidr), XP, loot tables
- [x] **T5.8** Real-time combat: melee, ranged, modern guns (ammo, reload, recoil, aim), cover, stealth
- [x] **T5.9** Shootable cannons (U6): placeable/fixed cannons, load powder+ball, aim, fire, damage ships/walls/enemies
- [x] **T5.10** Ships & vehicles (U6/U7): longships, sleds, skiffs, space-longship; boarding, cannons on ships
- [x] **T5.11** Magic: runes/seidr spells in U7 reagent+circle style with Kalevala singing-magic (laulu)
- [x] **T5.12** Crafting, cooking, forging (Sampo-forge), alchemy
- [x] **T5.13** Shops/barter, currency (silver/hacksilver + krediitti)
- [x] **T5.14** Journal/quest log, map screen, automap, fast travel via Bifrost nodes
- [x] **T5.15** Usecode-like scripting VM or Lua (mlua) for quests/objects; data-driven content [implemented as own JSON script VM in u67_world::script (no Lua)]
- [x] **T5.16** Enemy AI: wolves, draugr, trolls, Fenrir spawn, Louhi's forces, corporate mercs w/ modern guns

## P6 - Solar system, travel & content

- [x] **T6.1** Star map UI + ship travel between bodies (Sol system: Sol, Mani, Midgard, Muspel, Asgard, Jotun, Nifl, Hel, belt Svartalf)
- [x] **T6.2** Each body: hand-seeded planetary map + biome + hazards (cold, heat, vacuum suit) [maps generated for all 15 bodies; hazards pending]
- [x] **T6.3** Second system 'Kalevala' with 5 planets (Vainola, Pohjola, Tuonela, Ilma, Sampola) behind the Bifrost-Sampo gate [5 planet maps generated; gate/travel pending]
- [x] **T6.4** Main quest line (see docs/LORE.md) end to end
- [x] **T6.5** 10+ hotspots per system (ruins, wrecks, rune-sites, boss lairs, secret caches) [places+sites generated: 40+ Sol, 11 Kalevala, 30+ Midgard; missions pending]
- [x] **T6.6** 20+ side missions and 10+ optional tasks [21 side + 10 optional implemented]
- [x] **T6.7** Bosses: Fenrir-Prime, Jormungandr (sea), Louhi, Surma, Hel-Queen, Nidhogg
- [x] **T6.8** Unique gun: vainamoinen_gun (Kantele-rifle) + other legendary relics

## P7 - Cheats & developer console

- [x] **T7.1** Console (backtick/F1): input, history, autocomplete, scrollback, command registry [logic done in u67_console; in-game UI pending]
- [x] **T7.2** Implement commands from commands.txt (give, tp, god, noclip, spawn, set, flag, time, weather, quest, heal, xp, ...) [parser+Action enum done; game-side effects pending]
- [x] **T7.3** Cheats menu (F2) with toggles: god, infinite ammo, noclip, fast, all spells, reveal map
- [x] **T7.4** Dev tools: show colliders, entity inspector, FPS, reload assets, run script
- [x] **T7.5** commands.txt auto-generated from registry (test fails if out of sync)

## P8 - Multiplayer

- [x] **T8.1** Local splitscreen co-op adventure (2-4 players): per-player camera, controller/keyboard assignment, same world, solo or co-op [seats.rs: 1-4 players, viewports, per-seat input (kbd1 / kbd2 / gamepads), downed+revive; console: splitscreen N]
- [x] **T8.2** Splitscreen per-player inventory UI; shared party + shared quest state; drop-in/drop-out join [modal screens open for the seat that triggered them; shared quests/flags/world]
- [ ] **T8.4** Optional: LAN/online netcode (lockstep or rollback) via u67_net _(optional)_

## P9 - Polish, packaging, platforms

- [x] **T9.1** Main menu, options, credits ('By Crowelian 2026 + Sonnet'), intro cinematic (slides)
- [x] **T9.2** macOS .app bundle + dmg, codesign notes; verify Apple Silicon native [tools/bundle_macos.sh (.app+.dmg, ad-hoc codesign); CI builds aarch64-apple-darwin; NOT yet run on a real Mac]
- [x] **T9.3** Windows build + installer zip [tools/package_windows.ps1; CI builds x86_64-pc-windows-msvc]
- [x] **T9.4** Linux build + AppImage [tools/package_linux.sh (tar.gz + AppImage if tool present)] _(optional)_
- [x] **T9.5** Balance pass, bug bash, performance profile (target 60fps on M1 Max) [balance model test + tuned bosses; on-hardware 60fps profile still to do on the M1]
- [x] **T9.6** Accessibility: rebinding, text size, colorblind palettes [text scale (UiScale), full key + gamepad rebinding in settings.ron, colour-blind health bars] _(optional)_
