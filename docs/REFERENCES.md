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

## Study notes (own words; verified against ../refs on 2026-10-09)

### Exult (U7) - world & data
- **Scale:** tile = unit; chunk = 16x16 tiles; superchunk = 16x16 chunks (256x256 tiles); world = 12x12 superchunks = 192x192 chunks = **3072x3072 tiles**, wrapping at the edges (distance uses modular arithmetic). Our `u67_world` keeps CHUNK=16 and adds optional wrap.
- **Terrain:** `u7map` = 144 superchunks, each 16x16 little-endian u16 *chunk-template ids* (row-major, superchunk index = row*12+col). `u7chunks` = list of chunk templates, each 16x16 tiles of 2 bytes: shape = b0 + 256*(b1&3) (10 bits), frame = (b1>>2)&0x1f. Templates are shared by many map chunks (big memory win) - we copy this: `ChunkTemplate` table + per-chunk template id.
- **Objects:** `u7ifix` per superchunk = static (fixed) objects; `u7ireg` = dynamic objects incl. containers (nested recursively) saved in the savegame. Object has tile x,y (nibbles in chunk), lift (z), shape/frame, flags, optional container contents.
- **Flex archives:** header: 80-byte title, magic 0xffff1a00, count, magic2, padding to 128 bytes; then `count` pairs (offset u32, size u32). Used by shapes.vga, faces.vga, palettes.flx, usecode, etc.
- **Shapes:** `shapes.vga` holds RLE-compressed frames; first 0x96 (150) shapes are 8x8 flat terrain tiles; palettes in `palettes.flx`; `shapeinfo`/`tfa.dat`/`wgtvol.dat` give flags, weight/volume.
- **Schedules:** each NPC has up to 8 time-of-day slots (3-hour blocks) each with an activity + target tile. Activity types (our enum copies the *concept*): combat, pace(h/v), talk, dance, eat, farm, tend_shop, miner, hound, stand, loiter, wander, blacksmith, sleep, wait, sit, graze, bake, sew, shy, lab, thief, waiter, kid_games, eat_at_inn, duel, preach, patrol, desk_work, follow_avatar, walk_to_schedule. NPCs far from the player are teleported to their slot position instead of walking.
- **Time-queue** (`tqueue`): everything animated/AI-driven is an event scheduled at a future game tick. We use ECS timers instead.
- **Gumps:** the inventory is a stack of draggable windows (paperdoll, backpack, nested containers); drag item between them / the world. Statistics, spellbook, notebook gumps also exist.
- **Cheat screen** (`cheat.cc`): toggles (infinite light, wizard mode, map reveal, god/hero), NPC editor, time/teleport. Informs our F2 menu.

### Nuvie (U6) - cannons & ships
- **Cannon:** a world object with a facing frame; firing spawns a `CannonballEffect`: direction N/E/S/W (or from the frame), flight distance 5 tiles, a "toss" projectile animation that stops on blocking tiles/actors/objects; hitting a "missile boundary" tile spawns an explosion. The world is paused during the animation (we do NOT pause: real-time).
- **Ship:** a vehicle actor (OBJ_U6_SHIP); the player controls it while aboard; can be repaired; "Attack with ship cannons" prompts for a side/direction; many actions disabled while aboard.
- Our design extends this: load state (powder+ball), aim anywhere with mouse, ballistic arc, splash radius, ship broadside = N cannons fire together.

### OpenNox (Nox) - classes, spells, net
- **Classes:** Warrior, Conjurer, Wizard. **Spells** (file per spell, hints for our list): anchor, berserk, blind, counter, death, death ray, detect magic, fear, freeze, harpoon, haste, infravision, invisibility, invulnerability, light, magic wall, missiles, nullify, protect, run, slow, summon, trap, tread, vampirism, villain, warcry.
- Arena/CTF modes exist upstream but are **out of scope** (user decision: adventure + splitscreen co-op only). Netcode optional (T8.4).
- Our Norse mapping for player archetypes/spell schools: Warrior->Berserker, Conjurer->Volva (summoner), Wizard->Skald (sung spells).
