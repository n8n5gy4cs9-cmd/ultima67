//! Interaction with the world: NPCs, objects, ground items, cannons and ships.
use crate::app::{AppState, Game, WorldRes};
use crate::audio::SfxEvent;
use crate::combat::{Cursor, ProjKind, SpawnProjectile};
use crate::seats::{ActiveSeat, Intents, Operating, PlayerRt};
use crate::data::{ObjState, Vehicle};
use crate::db_res::DbRes;
use crate::input::KeyMap;
use crate::npc::{self, Npcs};
use crate::persist;
use crate::render::{ChunkIndex, DirtyChunks};
use crate::settings::Action;
use crate::ui::{EffectEvent, Toast};
use bevy::prelude::*;
use u67_core::{Dir, Rng, TilePos};
use u67_world::map::Map;
use u67_world::tiles;

/// Requests to open UI screens (handled by `gui`).
#[derive(Event, Clone, Debug)]
pub enum UiRequest {
    Dialogue(usize),
    Container(usize),
    Shop(String),
    Craft(String),
    Travel { by_ship: bool },
    Journal,
    Spells,
    Map,
    Laulu,
    Cheats,
}

pub const INTERACTIVE: &[&str] = &["door_wood", "door_open", "chest", "barrel", "crate", "signpost", "runestone", "well", "bed", "forge", "campfire", "table", "bifrost_node", "wreck", "cannon", "longship"];
const REACH: f32 = 1.9;

/// Nearest interactive object to `pos` among `candidates`.
pub fn nearest_object(map: &Map, candidates: impl Iterator<Item = usize>, pos: Vec2, reach: f32) -> Option<usize> {
    candidates
        .filter_map(|i| {
            let o = map.objects.get(i)?;
            if !INTERACTIVE.contains(&o.kind.as_str()) {
                return None;
            }
            let c = Vec2::new(o.pos.x as f32 + 0.5, o.pos.y as f32 + 0.5);
            let d = c.distance(pos);
            (d <= reach).then_some((i, d))
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(i, _)| i)
}

fn town_name(map: &Map, p: TilePos) -> Option<&'static str> {
    u67_mapgen::midgard::TOWNS.iter().filter_map(|(id, name, ..)| map.places.get(*id).map(|c| (*name, c.manhattan(p)))).min_by_key(|(_, d)| *d).map(|(n, _)| n)
}

fn seed_for(map: &str, p: TilePos, seed: u64) -> u64 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15;
    for b in map.bytes() {
        h = (h ^ b as u64).wrapping_mul(0x100_0000_01B3);
    }
    h ^ ((p.x as u64) << 20) ^ (p.y as u64) ^ 0xABCD
}

/// Roll the loot for a world container the first time it is opened.
pub fn roll_container(db: &u67_world::db::Db, kind: &str, map: &str, p: TilePos, seed: u64) -> Vec<u67_world::inventory::Item> {
    let mut rng = Rng::new(seed_for(map, p, seed));
    let table = match kind {
        "chest" => {
            if map != "midgard" && rng.chance(0.45) || rng.chance(0.12) { "chest_rare" } else { "chest_common" }
        }
        "barrel" => "barrel_common",
        _ => "crate_common",
    };
    db.loot.get(table).map(|t| t.roll(&mut rng)).unwrap_or_default().into_iter().map(|(i, n)| u67_world::inventory::Item::new(&i, n)).collect()
}

#[allow(clippy::too_many_arguments)]
fn interact(
    intents: Res<Intents>,
    mut game: ResMut<Game>,
    mut world: ResMut<WorldRes>,
    db: Res<DbRes>,
    npcs: Res<Npcs>,
    index: Res<ChunkIndex>,
    mut dirty: ResMut<DirtyChunks>,
    mut ui: EventWriter<UiRequest>,
    mut toast: ResMut<Toast>,
    mut sfx: EventWriter<SfxEvent>,
    mut fx: EventWriter<EffectEvent>,
    mut op: ResMut<Operating>,
    mut rt: ResMut<PlayerRt>,
    mut active: ResMut<ActiveSeat>,
) {
    for seat in 0..game.0.players.len() {
        if !intents.get(seat).interact || rt.list[seat].downed {
            continue;
        }
        // reviving a downed teammate takes priority
        let me = Vec2::from(game.0.players[seat].pos);
        if let Some(k) = (0..game.0.players.len()).find(|k| *k != seat && rt.list[*k].downed && Vec2::from(game.0.players[*k].pos).distance(me) < 1.8) {
            crate::combat::revive(&mut game, &mut rt, k, 0.4);
            *toast = Toast { text: format!("{} revived {}!", game.0.players[seat].name, game.0.players[k].name), timer: 3.0 };
            sfx.write(SfxEvent("level_up".into()));
            continue;
        }
        active.0 = seat;
        game.0.players.swap(0, seat);
        op.active = seat;
        rt.active = seat;
        interact_seat(seat, &mut game, &mut world, &db, &npcs, &index, &mut dirty, &mut ui, &mut toast, &mut sfx, &mut fx, &mut op, &mut rt);
        game.0.players.swap(0, seat);
    }
    op.active = 0;
    rt.active = 0;
}

#[allow(clippy::too_many_arguments)]
fn interact_seat(
    _seat: usize,
    game: &mut Game,
    world: &mut WorldRes,
    db: &DbRes,
    npcs: &Npcs,
    index: &ChunkIndex,
    dirty: &mut DirtyChunks,
    ui: &mut EventWriter<UiRequest>,
    toast: &mut Toast,
    sfx: &mut EventWriter<SfxEvent>,
    fx: &mut EventWriter<EffectEvent>,
    op: &mut Operating,
    rt: &mut PlayerRt,
) {
    // aboard ship: disembark
    if game.0.vehicle.is_some() {
        disembark(game, world, dirty, toast);
        return;
    }
    if op.get().is_some() {
        op.set(None);
        *toast = Toast { text: "You step away from the cannon.".into(), timer: 1.5 };
        return;
    }
    let p = Vec2::from(game.0.players[0].pos) - Vec2::new(0.0, 0.4);
    let map_name = game.0.current_map.clone();
    // candidates
    let npc_i = npc::npc_near(&npcs.list, [p.x, p.y], REACH).filter(|i| !npcs.list[*i].in_party || true);
    let npc_d = npc_i.map(|i| Vec2::new(npcs.list[i].pos[0], npcs.list[i].pos[1] - 0.3).distance(p)).unwrap_or(99.0);
    let (pcx, pcy) = TilePos::new(p.x as i32, p.y as i32).chunk();
    let cands: Vec<usize> = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (pcx + dx, pcy + dy))).filter_map(|k| index.by_chunk.get(&k)).flatten().copied().collect();
    let obj_i = world.0.maps.get(&map_name).and_then(|m| nearest_object(m, cands.into_iter(), p, REACH));
    let obj_d = obj_i.and_then(|i| world.0.maps.get(&map_name).map(|m| Vec2::new(m.objects[i].pos.x as f32 + 0.5, m.objects[i].pos.y as f32 + 0.5).distance(p))).unwrap_or(99.0);
    let ground_i = game.0.ground.iter().enumerate().filter(|(_, g)| g.map == map_name).map(|(i, g)| (i, Vec2::new(g.pos[0], g.pos[1]).distance(p))).filter(|(_, d)| *d <= 1.3).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let ground_d = ground_i.map_or(99.0, |(_, d)| d);
    if npc_i.is_some() && npc_d <= obj_d && npc_d <= ground_d {
        ui.write(UiRequest::Dialogue(npc_i.unwrap()));
        return;
    }
    if let Some((gi, gd)) = ground_i {
        if gd <= obj_d {
            let msg = crate::combat::pickup(game, gi);
            sfx.write(SfxEvent("pickup".into()));
            *toast = Toast { text: msg, timer: 1.8 };
            return;
        }
    }
    let Some(oi) = obj_i else {
        return;
    };
    let kind = world.0.maps[&map_name].objects[oi].kind.clone();
    let opos = world.0.maps[&map_name].objects[oi].pos;
    let say = |toast: &mut Toast, t: String| *toast = Toast { text: t, timer: 3.5 };
    match kind.as_str() {
        "door_wood" | "door_open" => {
            let opening = kind == "door_wood";
            if opening && world.0.maps[&map_name].objects[oi].locked {
                let has_key = game.0.players[0].inventory.count("rune_key") > 0;
                let pick = game.0.players[0].inventory.count("lockpick") > 0 && Rng::new(seed_for(&map_name, opos, game.0.seed) ^ game.0.clock.minutes).chance(0.6);
                if !(has_key || pick) {
                    say(toast, "The door is locked.".into());
                    sfx.write(SfxEvent("ui_error".into()));
                    return;
                }
                world.0.maps.get_mut(&map_name).unwrap().objects[oi].locked = false;
            }
            if !opening && Vec2::new(opos.x as f32 + 0.5, opos.y as f32 + 0.5).distance(Vec2::from(game.0.players[0].pos)) < 0.9 {
                say(toast, "Something is in the way.".into());
                return;
            }
            let nk = if opening { "door_open" } else { "door_wood" };
            world.0.maps.get_mut(&map_name).unwrap().set_object_kind(oi, nk);
            persist::record(&mut game.0, &world.0, &map_name, oi, |s: &mut ObjState| s.kind = Some(nk.into()));
            dirty.0.insert(opos.chunk());
            sfx.write(SfxEvent(if opening { "door_open" } else { "door_close" }.into()));
        }
        "chest" | "barrel" | "crate" => {
            if world.0.maps[&map_name].objects[oi].contents.is_none() {
                let items = roll_container(&db.0, &kind, &map_name, opos, game.0.seed);
                world.0.maps.get_mut(&map_name).unwrap().objects[oi].contents = Some(items.clone());
                persist::record(&mut game.0, &world.0, &map_name, oi, |s| s.contents = Some(items));
            }
            sfx.write(SfxEvent("chest_open".into()));
            ui.write(UiRequest::Container(oi));
        }
        "signpost" => {
            let map = &world.0.maps[&map_name];
            let mut rng = Rng::new(seed_for(&map_name, opos, game.0.seed));
            let lines = db.0.signs.get("signpost").cloned().unwrap_or_default();
            let town = town_name(map, opos).map(|t| format!("Welcome to {t}.\n")).unwrap_or_default();
            say(toast, format!("{town}{}", lines.get(rng.range(0, lines.len().max(1) as i32) as usize).cloned().unwrap_or_default()));
        }
        "runestone" | "wreck" | "boulder" | "cannon" | "longship" | "bed" | "forge" | "campfire" | "table" | "well" | "bifrost_node" => match kind.as_str() {
            "well" => {
                let max = game.0.players[0].stats.max_hp();
                game.0.players[0].stats.hp = (game.0.players[0].stats.hp + 15).min(max);
                sfx.write(SfxEvent("splash".into()));
                let mut rng = Rng::new(game.0.clock.minutes);
                let lines = db.0.signs.get("well").cloned().unwrap_or_default();
                say(toast, format!("You drink. (+15 HP)\n{}", lines.get(rng.range(0, lines.len().max(1) as i32) as usize).cloned().unwrap_or_default()));
            }
            "runestone" => {
                let mut rng = Rng::new(seed_for(&map_name, opos, game.0.seed));
                let lines = db.0.signs.get("runestone").cloned().unwrap_or_default();
                say(toast, lines.get(rng.range(0, lines.len().max(1) as i32) as usize).cloned().unwrap_or_default());
            }
            "bed" => {
                let hp = game.0.players[0].stats.hp as f32 / game.0.players[0].stats.max_hp() as f32;
                if game.0.clock.is_night() || hp < 0.8 {
                    let out = crate::script::run(&mut game.0, &db.0, &world.0, &[u67_world::script::Step::Sleep]);
                    let _ = out;
                    say(toast, "You sleep until morning. HP and mana restored.".into());
                    sfx.write(SfxEvent("ui_close".into()));
                } else {
                    say(toast, "You are not tired.".into());
                }
            }
            "forge" | "campfire" | "table" => {
                fx.write(EffectEvent(crate::apply::Effect::OpenCraft(u67_world::crafting::station_of(&kind).unwrap_or("workbench").to_string())));
            }
            "bifrost_node" => {
                ui.write(UiRequest::Travel { by_ship: false });
                sfx.write(SfxEvent("bifrost".into()));
            }
            "wreck" => {
                let st = game.0.obj_state.get(&crate::data::obj_key(&map_name, opos.x, opos.y, "wreck")).is_some_and(|s| s.loaded);
                if st {
                    say(toast, "Already picked clean.".into());
                } else {
                    let mut rng = Rng::new(seed_for(&map_name, opos, game.0.seed));
                    if let Some(t) = db.0.loot.get("wreck_salvage") {
                        for (id, n) in t.roll(&mut rng) {
                            game.0.ground.push(crate::data::GroundItem { map: map_name.clone(), pos: [opos.x as f32 + 0.5 + rng.f32() - 0.5, opos.y as f32 + 1.2 + rng.f32() * 0.5], item: u67_world::inventory::Item::new(&id, n) });
                        }
                    }
                    persist::record(&mut game.0, &world.0, &map_name, oi, |s| s.loaded = true);
                    say(toast, "You salvage what you can from the wreck.".into());
                }
            }
            "cannon" => {
                let key = crate::data::obj_key(&map_name, opos.x, opos.y, "cannon");
                let loaded = game.0.obj_state.get(&key).is_some_and(|s| s.loaded);
                if !loaded {
                    let inv = &mut game.0.players[0].inventory;
                    if inv.count("gunpowder") > 0 && inv.count("cannon_ball") > 0 {
                        inv.remove("gunpowder", 1);
                        inv.remove("cannon_ball", 1);
                        persist::record(&mut game.0, &world.0, &map_name, oi, |s| s.loaded = true);
                        say(toast, "Cannon loaded (powder + ball). Press E again to take aim.".into());
                        sfx.write(SfxEvent("reload".into()));
                    } else {
                        say(toast, "To load a cannon you need 1 gunpowder and 1 cannon ball.".into());
                        sfx.write(SfxEvent("ui_error".into()));
                    }
                } else {
                    op.set(Some(oi));
                    rt.cooldown = 0.3;
                    say(toast, "Aim with the mouse. Click to FIRE. E to step away.".into());
                }
            }
            "longship" => {
                // board
                let ship = world.0.maps.get_mut(&map_name).unwrap().remove_object(oi);
                if let Some(s) = ship {
                    let key = crate::data::obj_key(&map_name, s.pos.x, s.pos.y, "longship");
                    game.0.obj_state.entry(key).or_default().removed = true;
                    game.0.placed.retain(|(m, x, y, k)| !(m == &map_name && *x == s.pos.x && *y == s.pos.y && k == "longship"));
                    game.0.vehicle = Some(Vehicle { kind: "longship".into(), pos: [s.pos.x as f32 + 0.5, s.pos.y as f32 + 0.5], facing: Dir::E, hull: 200 });
                    game.0.players[0].pos = [s.pos.x as f32 + 0.5, s.pos.y as f32 + 0.5];
                    game.0.flags.insert("boarded_ship".into());
                    dirty.0.insert(s.pos.chunk());
                    say(toast, "Aboard the longship! Sail with WASD over water, M = star map, E = disembark at a shore.".into());
                    sfx.write(SfxEvent("ship_creak".into()));
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn disembark(game: &mut Game, world: &mut WorldRes, dirty: &mut DirtyChunks, toast: &mut Toast) {
    let Some(v) = game.0.vehicle.clone() else { return };
    let map_name = game.0.current_map.clone();
    let Some(map) = world.0.maps.get_mut(&map_name) else { return };
    let c = TilePos::new(v.pos[0].floor() as i32, v.pos[1].floor() as i32);
    for r in 1..=3 {
        for dy in -r..=r {
            for dx in -r..=r {
                let t = TilePos::new(c.x + dx, c.y + dy);
                let d = tiles::def(map.tile(t));
                if map.walkable(t) && !d.water && !d.hazard {
                    // dock the ship where it floats
                    map.add_object("longship", c);
                    game.0.placed.push((map_name.clone(), c.x, c.y, "longship".into()));
                    game.0.vehicle = None;
                    game.0.players[0].pos = [t.x as f32 + 0.5, t.y as f32 + 0.5];
                    game.0.map_dirty = true;
                    dirty.0.insert(c.chunk());
                    *toast = Toast { text: "You step ashore. The ship waits for you.".into(), timer: 2.5 };
                    return;
                }
            }
        }
    }
    *toast = Toast { text: "No shore nearby to disembark.".into(), timer: 2.0 };
}

/// Fire a loaded cannon (while operating) or ship cannons (aboard).
#[allow(clippy::too_many_arguments)]
fn cannon_fire(
    intents: Res<Intents>,
    cursor: Res<Cursor>,
    mut game: ResMut<Game>,
    mut world: ResMut<WorldRes>,
    mut op: ResMut<Operating>,
    mut rt: ResMut<PlayerRt>,
    mut shots: EventWriter<SpawnProjectile>,
    mut sfx: EventWriter<SfxEvent>,
    mut toast: ResMut<Toast>,
    mut dirty: ResMut<DirtyChunks>,
) {
    for seat in 0..game.0.players.len() {
        game.0.players.swap(0, seat);
        op.active = seat;
        rt.active = seat;
        cannon_seat(seat, intents.get(seat), &cursor, &mut game, &mut world, &mut op, &mut rt, &mut shots, &mut sfx, &mut toast, &mut dirty);
        game.0.players.swap(0, seat);
    }
    op.active = 0;
    rt.active = 0;
}

#[allow(clippy::too_many_arguments)]
fn cannon_seat(
    seat: usize,
    intent: crate::seats::Intent,
    cursor: &Cursor,
    game: &mut Game,
    world: &mut WorldRes,
    op: &mut Operating,
    rt: &mut PlayerRt,
    shots: &mut EventWriter<SpawnProjectile>,
    sfx: &mut EventWriter<SfxEvent>,
    toast: &mut Toast,
    dirty: &mut DirtyChunks,
) {
    if rt.downed {
        return;
    }
    let map_name = game.0.current_map.clone();
    let pp = Vec2::from(game.0.players[0].pos);
    if let Some(oi) = op.get() {
        let Some(o) = world.0.maps.get(&map_name).and_then(|m| m.objects.get(oi)).cloned() else {
            op.set(None);
            return;
        };
        let cpos = Vec2::new(o.pos.x as f32 + 0.5, o.pos.y as f32 + 0.2);
        if cpos.distance(pp) > 3.2 {
            op.set(None);
            return;
        }
        let aim_to = if intent.aim != Vec2::ZERO { cpos + intent.aim * 8.0 } else if seat == 0 && cursor.valid { cursor.tile } else { return };
        let to = aim_to - cpos;
        let dir = to.normalize_or_zero();
        // rotate the barrel toward the aim
        let frame = match crate::combat::aim_dir(dir) {
            Dir::N => 0,
            Dir::E => 1,
            Dir::S => 2,
            Dir::W => 3,
        };
        if o.frame != frame {
            world.0.maps.get_mut(&map_name).unwrap().objects[oi].frame = frame;
            persist::record(&mut game.0, &world.0, &map_name, oi, |s| s.frame = Some(frame));
            dirty.0.insert(o.pos.chunk());
        }
        if intent.attack && rt.cooldown <= 0.0 {
            let key = crate::data::obj_key(&map_name, o.pos.x, o.pos.y, "cannon");
            if game.0.obj_state.get(&key).is_some_and(|s| s.loaded) {
                let range = to.length().clamp(3.0, 24.0);
                let tgt = cpos + dir * range;
                shots.write(SpawnProjectile { pos: cpos, vel: dir * 14.0, dmg: 90, friendly: true, range, kind: ProjKind::Cannonball, splash: 0.0, special: String::new(), cannon: true, target: Some(tgt), elem: String::new() });
                persist::record(&mut game.0, &world.0, &map_name, oi, |s| s.loaded = false);
                sfx.write(SfxEvent("cannon_fire".into()));
                rt.shake = 0.3;
                rt.cooldown = 0.6;
                op.set(None); // must reload
                *toast = Toast { text: "BOOM! Reload with E (1 powder + 1 ball).".into(), timer: 2.5 };
            }
        }
        return;
    }
    if let Some(v) = game.0.vehicle.clone() {
        let aim_to = if intent.aim != Vec2::ZERO { Some(Vec2::from(v.pos) + intent.aim * 8.0) } else if seat == 0 && cursor.valid { Some(cursor.tile) } else { None };
        if seat == 0 && intent.attack && rt.cooldown <= 0.0 && aim_to.is_some() {
            let inv = &mut game.0.players[0].inventory;
            if inv.count("gunpowder") > 0 && inv.count("cannon_ball") > 0 {
                inv.remove("gunpowder", 1);
                inv.remove("cannon_ball", 1);
                let from = Vec2::from(v.pos);
                let to = aim_to.unwrap() - from;
                let dir = to.normalize_or_zero();
                let range = to.length().clamp(3.0, 24.0);
                shots.write(SpawnProjectile { pos: from, vel: dir * 14.0, dmg: 90, friendly: true, range, kind: ProjKind::Cannonball, splash: 0.0, special: String::new(), cannon: true, target: Some(from + dir * range), elem: String::new() });
                sfx.write(SfxEvent("cannon_fire".into()));
                rt.shake = 0.35;
                rt.cooldown = 1.4;
            } else {
                *toast = Toast { text: "Out of powder or cannon balls!".into(), timer: 1.8 };
                sfx.write(SfxEvent("ui_error".into()));
            }
        }
    }
}

/// Sail the player's ship: move over water only.
pub fn boat_step(map: &Map, pos: Vec2, delta: Vec2) -> Vec2 {
    let ok = |p: Vec2| {
        let t = TilePos::new(p.x.floor() as i32, p.y.floor() as i32);
        map.in_bounds(t) && tiles::def(map.tile(t)).water && !map.blocked_by_object(t)
    };
    let mut q = pos;
    if ok(Vec2::new(q.x + delta.x, q.y)) {
        q.x += delta.x;
    }
    if ok(Vec2::new(q.x, q.y + delta.y)) {
        q.y += delta.y;
    }
    q
}

#[derive(Component)]
struct ShipSprite;

fn ship_visuals(mut commands: Commands, game: Res<Game>, sheets: Res<crate::render::Sheets>, mut q: Query<(Entity, &mut Transform, &mut Sprite), With<ShipSprite>>, mut players: Query<&mut Visibility, With<crate::player::Player>>) {
    for mut v in &mut players {
        *v = if game.0.vehicle.is_some() { Visibility::Hidden } else { Visibility::Inherited };
    }
    let row = u67_world::objects::OBJECTS.iter().position(|o| o.id == "longship").unwrap_or(0) as u32;
    match (&game.0.vehicle, q.iter_mut().next()) {
        (Some(v), None) => {
            commands.spawn((ShipSprite, Sprite { image: sheets.objects_img.clone(), texture_atlas: Some(TextureAtlas { layout: sheets.objects_layout.clone(), index: (row * 4 + 1) as usize }), anchor: bevy::sprite::Anchor::BottomCenter, ..default() }, Transform::from_xyz(0.0, 0.0, 2.0)));
            let _ = v;
        }
        (Some(v), Some((_, mut tf, mut sp))) => {
            let p = crate::render::tile_px(v.pos[0], v.pos[1] + 0.5);
            tf.translation = Vec3::new(p.x.round(), p.y.round(), 2.0 + v.pos[1] * 0.001);
            if let Some(a) = &mut sp.texture_atlas {
                let f = match v.facing {
                    Dir::N => 0,
                    Dir::E => 1,
                    Dir::S => 2,
                    Dir::W => 3,
                };
                a.index = (row * 4 + f) as usize;
            }
        }
        (None, Some((e, _, _))) => commands.entity(e).despawn(),
        _ => {}
    }
}

fn map_keys(kb: Res<ButtonInput<KeyCode>>, keys: Res<KeyMap>, game: Res<Game>, mut ui: EventWriter<UiRequest>, mut active: ResMut<ActiveSeat>) {
    active.0 = 0;
    if keys.just_pressed(Action::Map, &kb) {
        if game.0.vehicle.is_some() {
            ui.write(UiRequest::Travel { by_ship: true });
        } else {
            ui.write(UiRequest::Map);
        }
    }
    if keys.just_pressed(Action::Journal, &kb) {
        ui.write(UiRequest::Journal);
    }
    if keys.just_pressed(Action::Cheats, &kb) {
        ui.write(UiRequest::Cheats);
    }
    if kb.just_pressed(KeyCode::KeyB) {
        ui.write(UiRequest::Spells);
    }
    if kb.just_pressed(KeyCode::KeyL) {
        ui.write(UiRequest::Laulu);
    }
}

fn ready(game: Option<Res<Game>>, db: Option<Res<DbRes>>) -> bool {
    game.is_some() && db.is_some()
}

pub struct InteractPlugin;
impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiRequest>().add_systems(
            Update,
            (interact, cannon_fire, map_keys).run_if(ready).run_if(in_state(AppState::Playing)),
        );
        app.add_systems(Update, ship_visuals.run_if(ready).run_if(not(in_state(AppState::Boot))).run_if(not(in_state(AppState::MainMenu))).run_if(resource_exists::<crate::render::Sheets>));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_nearest_interactive_only() {
        let mut m = Map::new("t", 30, 30, tiles::GRASS);
        m.add_object("pine_tree", TilePos::new(5, 5));
        m.add_object("chest", TilePos::new(8, 5));
        m.add_object("door_wood", TilePos::new(6, 6));
        let idx = 0..m.objects.len();
        // tree is closest but not interactive
        assert_eq!(nearest_object(&m, idx.clone(), Vec2::new(5.5, 5.6), 2.0), Some(2));
        assert_eq!(nearest_object(&m, idx.clone(), Vec2::new(8.5, 6.0), 2.0), Some(1));
        assert_eq!(nearest_object(&m, idx, Vec2::new(20.0, 20.0), 2.0), None);
    }

    #[test]
    fn boats_stay_on_water() {
        let mut m = Map::new("t", 20, 20, tiles::WATER_DEEP);
        for y in 0..20 {
            m.set_tile(TilePos::new(10, y), tiles::GRASS);
        }
        let p = boat_step(&m, Vec2::new(8.5, 5.5), Vec2::new(1.8, 0.5));
        assert!(p.x < 10.0 && (p.y - 6.0).abs() < 1e-4, "{p}");
    }

    #[test]
    fn container_loot_is_deterministic() {
        let db = u67_world::db::Db::builtin().unwrap();
        let a = roll_container(&db, "chest", "midgard", TilePos::new(3, 4), 67);
        let b = roll_container(&db, "chest", "midgard", TilePos::new(3, 4), 67);
        assert_eq!(a, b);
        let c = roll_container(&db, "chest", "midgard", TilePos::new(9, 4), 67);
        let _ = c;
    }
}
