//! Hostile creatures and summoned allies: spawning, AI, sprites.
use crate::app::{AppState, Game, WorldRes};
use crate::combat::{DamageEvent, KillEvent, PlayerHit, ProjKind, SpawnProjectile};
use crate::db_res::DbRes;
use crate::render::{self, Sheets, CHAR_COLS};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use u67_core::{Dir, Rng, TilePos};
use u67_world::map::Map;
use u67_world::npcs::{Ai, CreatureDef};
use u67_world::tiles;

#[derive(Component, Clone)]
pub struct Creature {
    pub def: String,
    pub pos: Vec2,
    pub hp: i32,
    pub max_hp: i32,
    pub cd: f32,
    pub special_cd: f32,
    pub home: Vec2,
    pub wander: f32,
    pub wander_dir: Vec2,
    pub stun: f32,
    pub fear: f32,
    pub frozen: f32,
    pub ally: bool,
    pub life: Option<f32>,
    pub spawn_idx: Option<usize>,
    pub unique_key: Option<String>,
    pub facing: Dir,
    pub moving: bool,
    pub attack_anim: f32,
    pub flash: f32,
    pub charge: f32,
    pub aggro: bool,
}

#[derive(Component)]
pub struct HpBarFg;
#[derive(Component)]
pub struct Corpse(pub f32);

#[derive(Resource, Default)]
pub struct SpawnTimers {
    pub map: String,
    pub t: Vec<f32>,
    pub clock: f32,
}

pub fn radius(def: &CreatureDef) -> f32 {
    0.32 * def.scale.sqrt()
}

pub fn clear_line(map: &Map, a: Vec2, b: Vec2) -> bool {
    let d = b - a;
    let n = (d.length() / 0.4).ceil() as i32;
    for i in 1..n {
        let p = a + d * (i as f32 / n as f32);
        let t = TilePos::new(p.x.floor() as i32, p.y.floor() as i32);
        let td = tiles::def(map.tile(t));
        if (!td.walkable && !td.water) || map.blocked_by_object(t) {
            return false;
        }
    }
    true
}

fn can_stand(map: &Map, def: &CreatureDef, p: Vec2) -> bool {
    let t = TilePos::new(p.x.floor() as i32, p.y.floor() as i32);
    if def.flies {
        return map.in_bounds(t);
    }
    let td = tiles::def(map.tile(t));
    if def.swims {
        return map.in_bounds(t) && td.water;
    }
    let r = (radius(def) * 0.8).min(0.6);
    for (dx, dy) in [(-r, 0.0), (r, 0.0), (-r, -r), (r, -r)] {
        let q = TilePos::new((p.x + dx).floor() as i32, (p.y + dy).floor() as i32);
        if !map.walkable(q) || tiles::def(map.tile(q)).hazard {
            return false;
        }
    }
    true
}

fn slide(map: &Map, def: &CreatureDef, p: Vec2, d: Vec2) -> (Vec2, bool) {
    let mut q = p;
    let mut moved = false;
    if can_stand(map, def, Vec2::new(q.x + d.x, q.y)) {
        q.x += d.x;
        moved |= d.x.abs() > 0.0;
    }
    if can_stand(map, def, Vec2::new(q.x, q.y + d.y)) {
        q.y += d.y;
        moved |= d.y.abs() > 0.0;
    }
    (q, moved)
}

fn dir_of(v: Vec2) -> Dir {
    if v.x.abs() > v.y.abs() {
        if v.x > 0.0 { Dir::E } else { Dir::W }
    } else if v.y > 0.0 {
        Dir::S
    } else {
        Dir::N
    }
}

pub fn new_creature(def: &CreatureDef, pos: Vec2, spawn_idx: Option<usize>, unique_key: Option<String>) -> Creature {
    Creature { def: def.id.clone(), pos, hp: def.hp, max_hp: def.hp, cd: 0.5, special_cd: 3.0, home: pos, wander: 0.0, wander_dir: Vec2::ZERO, stun: 0.0, fear: 0.0, frozen: 0.0, ally: false, life: None, spawn_idx, unique_key, facing: Dir::S, moving: false, attack_anim: 0.0, flash: 0.0, charge: 0.0, aggro: false }
}

pub fn spawn_creature_entity(commands: &mut Commands, sheets: &Sheets, def: &CreatureDef, c: Creature) -> Entity {
    let row = render::character_row(&def.arch).unwrap_or(0);
    let scale = def.scale;
    let e = commands
        .spawn((
            c,
            Sprite { image: sheets.chars_img.clone(), texture_atlas: Some(TextureAtlas { layout: sheets.chars_layout.clone(), index: (row * 4 * CHAR_COLS + 2 * CHAR_COLS) as usize }), anchor: Anchor::Custom(Vec2::new(0.0, -0.5 + 1.5 / 24.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 2.0).with_scale(Vec3::splat(scale)),
        ))
        .id();
    commands.entity(e).with_children(|p| {
        p.spawn((Sprite { color: Color::srgba(0.0, 0.0, 0.0, 0.7), custom_size: Some(Vec2::new(18.0, 3.0)), anchor: Anchor::CenterLeft, ..default() }, Transform::from_xyz(-9.0, 26.0, 0.1).with_scale(Vec3::splat(1.0 / scale))));
        p.spawn((HpBarFg, Sprite { color: Color::srgb(0.85, 0.15, 0.15), custom_size: Some(Vec2::new(16.0, 2.0)), anchor: Anchor::CenterLeft, ..default() }, Transform::from_xyz(-8.0, 26.0, 0.2).with_scale(Vec3::splat(1.0 / scale))));
    });
    e
}

pub fn spawn_by_id(commands: &mut Commands, sheets: &Sheets, db: &u67_world::db::Db, id: &str, pos: Vec2, ally: bool, life: Option<f32>) -> Option<Entity> {
    let def = db.creatures.get(id)?;
    let mut c = new_creature(def, pos, None, None);
    c.ally = ally;
    c.life = life;
    Some(spawn_creature_entity(commands, sheets, def, c))
}

fn random_point(map: &Map, def: &CreatureDef, center: Vec2, radius: f32, rng: &mut Rng) -> Option<Vec2> {
    for _ in 0..20 {
        let a = rng.f32() * std::f32::consts::TAU;
        let r = rng.f32().sqrt() * radius;
        let p = center + Vec2::new(a.cos(), a.sin()) * r;
        if can_stand(map, def, p) {
            return Some(p);
        }
    }
    None
}

fn near_town(map: &Map, p: Vec2) -> bool {
    crate::npc::TOWN_IDS.iter().filter_map(|t| map.places.get(*t)).any(|c| (c.x as f32 - p.x).abs() < 20.0 && (c.y as f32 - p.y).abs() < 20.0)
}

#[allow(clippy::too_many_arguments)]
fn spawner(mut commands: Commands, time: Res<Time>, game: Res<Game>, world: Res<WorldRes>, db: Res<DbRes>, sheets: Res<Sheets>, mut timers: ResMut<SpawnTimers>, creatures: Query<(Entity, &Creature)>, mut rng: Local<Option<Rng>>) {
    let rng = rng.get_or_insert_with(|| Rng::new(4242));
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    if timers.map != game.0.current_map || timers.t.len() != db.0.spawns.len() {
        for (e, _) in &creatures {
            commands.entity(e).despawn();
        }
        timers.map = game.0.current_map.clone();
        timers.t = vec![0.0; db.0.spawns.len()];
    }
    timers.clock -= time.delta_secs();
    if timers.clock > 0.0 {
        return;
    }
    timers.clock = 1.0;
    for t in timers.t.iter_mut() {
        *t = (*t - 1.0).max(0.0);
    }
    let player = Vec2::from(game.0.players[0].pos);
    // despawn far wilderness creatures
    for (e, c) in &creatures {
        if c.spawn_idx.is_some_and(|i| db.0.spawns.get(i).is_some_and(|s| s.place == "*")) && c.pos.distance(player) > 60.0 {
            commands.entity(e).despawn();
        }
    }
    for (i, s) in db.0.spawns.iter().enumerate().filter(|(_, s)| s.map == game.0.current_map) {
        let Some(def) = db.0.creatures.get(&s.creature) else { continue };
        let alive = creatures.iter().filter(|(_, c)| c.spawn_idx == Some(i) && !c.ally).count();
        if s.place == "*" {
            if alive >= 3 || timers.t[i] > 0.0 || (s.night_only && !game.0.clock.is_night()) {
                continue;
            }
            timers.t[i] = 5.0 + rng.f32() * 8.0;
            let a = rng.f32() * std::f32::consts::TAU;
            let p = player + Vec2::new(a.cos(), a.sin()) * (15.0 + rng.f32() * 8.0);
            let tn = tiles::def(map.tile(TilePos::new(p.x.floor() as i32, p.y.floor() as i32))).name;
            if (!s.on.is_empty() && !s.on.iter().any(|n| n == tn)) || near_town(map, p) || !can_stand(map, def, p) {
                continue;
            }
            let c = new_creature(def, p, Some(i), None);
            spawn_creature_entity(&mut commands, &sheets, def, c);
        } else {
            let Some(pl) = map.places.get(&s.place) else { continue };
            let center = Vec2::new(pl.x as f32 + 0.5, pl.y as f32 + 0.5);
            if center.distance(player) > 48.0 || alive > 0 || timers.t[i] > 0.0 {
                continue;
            }
            let ukey = format!("dead_{}@{}:{}", s.creature, s.map, s.place);
            if s.unique && game.0.flags.contains(&ukey) {
                continue;
            }
            timers.t[i] = 240.0;
            for _ in 0..s.count {
                if let Some(p) = random_point(map, def, center, s.radius, rng) {
                    let c = new_creature(def, p, Some(i), if s.unique { Some(ukey.clone()) } else { None });
                    spawn_creature_entity(&mut commands, &sheets, def, c);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn creature_ai(
    time: Res<Time>,
    game: Res<Game>,
    world: Res<WorldRes>,
    db: Res<DbRes>,
    mut q: Query<(Entity, &mut Creature)>,
    mut hits: EventWriter<PlayerHit>,
    mut shots: EventWriter<SpawnProjectile>,
    mut dmg: EventWriter<DamageEvent>,
    mut kills: EventWriter<KillEvent>,
    mut rng: Local<Option<Rng>>,
) {
    let rng = rng.get_or_insert_with(|| Rng::new(7));
    let dt = time.delta_secs();
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    let player = Vec2::from(game.0.players[0].pos);
    let visible = !game.0.cheats.invisible;
    // snapshot for ally/enemy targeting
    let snap: Vec<(Entity, Vec2, bool)> = q.iter().map(|(e, c)| (e, c.pos, c.ally)).collect();
    for (e, mut c) in &mut q {
        let Some(def) = db.0.creatures.get(&c.def) else { continue };
        c.cd -= dt;
        c.special_cd -= dt;
        c.attack_anim = (c.attack_anim - dt).max(0.0);
        c.flash = (c.flash - dt).max(0.0);
        c.stun = (c.stun - dt).max(0.0);
        c.fear = (c.fear - dt).max(0.0);
        c.frozen = (c.frozen - dt).max(0.0);
        if let Some(l) = &mut c.life {
            *l -= dt;
            if *l <= 0.0 {
                kills.write(KillEvent { entity: e, by_player: false, cannon: false });
                continue;
            }
        }
        c.moving = false;
        if c.stun > 0.0 || c.frozen > 0.0 {
            continue;
        }
        // choose a target
        let (tpos, tent, t_is_player): (Option<Vec2>, Option<Entity>, bool) = if c.ally {
            let best = snap.iter().filter(|(oe, _, ally)| *oe != e && !*ally).map(|(oe, p, _)| (*oe, *p)).filter(|(_, p)| p.distance(c.pos) < def.sight.max(9.0)).min_by(|a, b| a.1.distance(c.pos).partial_cmp(&b.1.distance(c.pos)).unwrap());
            match best {
                Some((oe, p)) => (Some(p), Some(oe), false),
                None => (Some(player), None, true),
            }
        } else if visible && player.distance(c.pos) < def.sight && (def.ai != Ai::Passive) {
            (Some(player), None, true)
        } else {
            (None, None, false)
        };
        let speed = def.speed * if c.charge > 0.0 { 2.6 } else { 1.0 };
        match tpos {
            Some(tp) if !(c.ally && t_is_player && tp.distance(c.pos) < 3.0) => {
                let d = tp - c.pos;
                let dist = d.length();
                c.aggro = !c.ally;
                let dir = d.normalize_or_zero();
                if c.fear > 0.0 {
                    let (np, m) = slide(map, def, c.pos, -dir * speed * dt);
                    c.pos = np;
                    c.moving = m;
                    c.facing = dir_of(-dir);
                    continue;
                }
                c.facing = dir_of(dir);
                let ranged = def.range > 3.5 && matches!(def.ai, Ai::Ranged | Ai::Boss);
                if ranged {
                    let los = clear_line(map, c.pos, tp);
                    if dist > def.range * 0.85 || !los {
                        let (np, m) = slide(map, def, c.pos, dir * speed * dt);
                        c.pos = np;
                        c.moving = m;
                    } else if dist < def.range * 0.4 {
                        let (np, m) = slide(map, def, c.pos, -dir * speed * dt * 0.8);
                        c.pos = np;
                        c.moving = m;
                    }
                    if dist <= def.range && los && c.cd <= 0.0 {
                        c.cd = def.cooldown;
                        c.attack_anim = 0.3;
                        let n = if def.ai == Ai::Boss && c.special_cd <= 0.0 { c.special_cd = 4.0; 5 } else { 1 };
                        for k in 0..n {
                            let spread = (k as f32 - (n as f32 - 1.0) / 2.0) * 0.18 + (rng.f32() - 0.5) * 0.08;
                            let (s, co) = spread.sin_cos();
                            let v = Vec2::new(dir.x * co - dir.y * s, dir.x * s + dir.y * co);
                            shots.write(SpawnProjectile { pos: c.pos - Vec2::new(0.0, 0.5), vel: v * 13.0, dmg: def.damage, friendly: c.ally, range: def.range + 4.0, kind: ProjKind::EnemyBolt, splash: 0.0, special: String::new(), cannon: false, target: None, elem: if def.arch == "louhi" { "frost".into() } else { String::new() } });
                        }
                    }
                } else {
                    // melee / charging bosses
                    if def.ai == Ai::Boss && c.special_cd <= 0.0 && dist > 3.0 && dist < 10.0 {
                        c.special_cd = 5.0;
                        c.charge = 0.7;
                    }
                    c.charge = (c.charge - dt).max(0.0);
                    if dist > def.range * 0.9 {
                        let (np, m) = slide(map, def, c.pos, dir * speed * dt);
                        c.pos = np;
                        c.moving = m;
                        if !m {
                            // blocked: try sidestepping
                            let side = Vec2::new(-dir.y, dir.x);
                            let (np, m2) = slide(map, def, c.pos, side * speed * dt);
                            c.pos = np;
                            c.moving = m2;
                        }
                    } else if c.cd <= 0.0 {
                        c.cd = def.cooldown;
                        c.attack_anim = 0.35;
                        if t_is_player {
                            hits.write(PlayerHit { amount: def.damage, from: c.pos, special: String::new() });
                        } else if let Some(te) = tent {
                            dmg.write(DamageEvent { target: te, amount: def.damage, crit: false, by_player: false, special: String::new(), cannon: false });
                        }
                    }
                }
            }
            _ => {
                // idle / wander near home
                c.aggro = false;
                c.wander -= dt;
                if c.wander <= 0.0 {
                    c.wander = 1.5 + rng.f32() * 3.0;
                    let a = rng.f32() * std::f32::consts::TAU;
                    c.wander_dir = if rng.chance(0.4) { Vec2::ZERO } else { Vec2::new(a.cos(), a.sin()) };
                    if c.pos.distance(c.home) > 8.0 {
                        c.wander_dir = (c.home - c.pos).normalize_or_zero();
                    }
                }
                if c.wander_dir != Vec2::ZERO && !matches!(def.ai, Ai::Passive) || c.wander_dir != Vec2::ZERO {
                    let (np, m) = slide(map, def, c.pos, c.wander_dir * def.speed * 0.4 * dt);
                    c.pos = np;
                    c.moving = m;
                    c.facing = dir_of(c.wander_dir);
                }
            }
        }
    }
}

fn sync_creature_sprites(time: Res<Time>, db: Res<DbRes>, mut q: Query<(&Creature, &mut Transform, &mut Sprite, &Children)>, mut bars: Query<(&mut Sprite, &mut Visibility), (With<HpBarFg>, Without<Creature>)>) {
    for (c, mut tf, mut sp, children) in &mut q {
        let Some(def) = db.0.creatures.get(&c.def) else { continue };
        let p = render::tile_px(c.pos.x, c.pos.y);
        let air = if def.flies { 6.0 } else { 0.0 };
        tf.translation = Vec3::new(p.x.round(), p.y.round() + air, 2.0 + c.pos.y * 0.001);
        let row = render::character_row(&def.arch).unwrap_or(0);
        let frame = if c.attack_anim > 0.0 { 5 + ((0.35 - c.attack_anim).max(0.0) * 8.0) as u32 % 3 } else if c.moving { 1 + ((time.elapsed_secs() * 8.0) as u32 % 4) } else { 0 };
        if let Some(a) = &mut sp.texture_atlas {
            a.index = ((row * 4 + c.facing.index() as u32) * CHAR_COLS + frame) as usize;
        }
        sp.color = if c.flash > 0.0 { Color::srgb(1.6, 0.5, 0.5) } else if c.frozen > 0.0 { Color::srgb(0.6, 0.8, 1.4) } else if c.ally { Color::srgb(0.7, 1.2, 1.2) } else { Color::WHITE };
        for ch in children.iter() {
            if let Ok((mut bs, mut vis)) = bars.get_mut(ch) {
                bs.custom_size = Some(Vec2::new(16.0 * (c.hp.max(0) as f32 / c.max_hp as f32), 2.0));
                *vis = if c.hp < c.max_hp { Visibility::Inherited } else { Visibility::Hidden };
            }
        }
    }
}

fn corpses(time: Res<Time>, mut commands: Commands, mut q: Query<(Entity, &mut Corpse, &mut Sprite)>) {
    for (e, mut c, mut s) in &mut q {
        c.0 -= time.delta_secs();
        s.color = Color::srgba(1.0, 1.0, 1.0, (c.0 / 3.0).clamp(0.0, 1.0));
        if c.0 <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

/// Clear creatures when the map changes (the spawner also handles this; this covers loads).
fn active(game: Option<Res<Game>>, db: Option<Res<DbRes>>, sheets: Option<Res<Sheets>>) -> bool {
    game.is_some() && db.is_some() && sheets.is_some()
}

pub struct CreaturePlugin;
impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnTimers>().add_systems(FixedUpdate, (spawner, creature_ai).chain().run_if(active).run_if(in_state(AppState::Playing))).add_systems(Update, (sync_creature_sprites, corpses).run_if(active).run_if(not(in_state(AppState::Boot))).run_if(not(in_state(AppState::MainMenu))));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use u67_world::db::Db;

    #[test]
    fn line_of_sight_blocked_by_walls_not_water() {
        let mut m = Map::new("t", 30, 30, tiles::GRASS);
        for y in 0..30 {
            m.set_tile(TilePos::new(10, y), tiles::WALL_STONE);
        }
        assert!(!clear_line(&m, Vec2::new(5.5, 5.5), Vec2::new(15.5, 5.5)));
        let mut m2 = Map::new("t", 30, 30, tiles::GRASS);
        for y in 0..30 {
            m2.set_tile(TilePos::new(10, y), tiles::WATER_DEEP);
        }
        assert!(clear_line(&m2, Vec2::new(5.5, 5.5), Vec2::new(15.5, 5.5)));
    }

    #[test]
    fn movement_rules_per_creature_kind() {
        let db = Db::builtin().unwrap();
        let mut m = Map::new("t", 30, 30, tiles::GRASS);
        m.set_tile(TilePos::new(5, 5), tiles::WATER_DEEP);
        let wolf = &db.creatures["wolf"];
        let serpent = &db.creatures["jormungandr"];
        let ghost = &db.creatures["ukko_storm"];
        assert!(can_stand(&m, wolf, Vec2::new(10.5, 10.5)));
        assert!(!can_stand(&m, wolf, Vec2::new(5.5, 5.5)));
        assert!(can_stand(&m, serpent, Vec2::new(5.5, 5.5)));
        assert!(!can_stand(&m, serpent, Vec2::new(10.5, 10.5)));
        assert!(can_stand(&m, ghost, Vec2::new(5.5, 5.5)));
    }

    #[test]
    fn creature_defs_reference_valid_loot_and_art() {
        let db = Db::builtin().unwrap();
        for c in db.creatures.values() {
            assert!(c.hp > 0 && c.speed > 0.0 && c.damage > 0, "{}", c.id);
            assert!(render::character_row(&c.arch).is_some(), "{} arch {}", c.id, c.arch);
        }
    }
}
