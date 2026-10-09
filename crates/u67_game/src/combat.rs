//! Real-time combat: guns, melee, projectiles, cannonballs, explosions, damage, loot, death.
use crate::app::{AppState, Game, WorldRes};
use crate::audio::SfxEvent;
use crate::creatures::{radius, Corpse, Creature};
use crate::data::{obj_key, GroundItem};
use crate::db_res::DbRes;
use crate::input::{Intent, KeyMap};
use crate::render::{self, DirtyChunks, Sheets, CHAR_COLS};
use crate::settings::Action;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use u67_core::{Dir, Rng, TilePos};
use u67_world::combat as rules;
use u67_world::inventory::Item;
use u67_world::items::{self, Kind, Slot};
use u67_world::tiles;

#[derive(Resource, Default)]
pub struct Cursor {
    pub tile: Vec2,
    pub valid: bool,
}

#[derive(Resource, Default)]
pub struct PlayerRt {
    pub cooldown: f32,
    pub reload: Option<(f32, String)>,
    pub iframes: f32,
    pub roll_t: f32,
    pub roll_cd: f32,
    pub roll_dir: Vec2,
    pub haste: f32,
    pub protect: (i32, f32),
    pub light: f32,
    pub shake: f32,
    pub hurt_flash: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProjKind {
    Bullet,
    Cannonball,
    Magic,
    EnemyBolt,
}

#[derive(Event, Clone, Debug)]
pub struct SpawnProjectile {
    pub pos: Vec2,
    pub vel: Vec2,
    pub dmg: i32,
    pub friendly: bool,
    pub range: f32,
    pub kind: ProjKind,
    pub splash: f32,
    pub special: String,
    pub cannon: bool,
    pub target: Option<Vec2>,
    pub elem: String,
}

#[derive(Event, Clone, Debug)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: i32,
    pub crit: bool,
    pub by_player: bool,
    pub special: String,
    pub cannon: bool,
}
#[derive(Event, Clone, Debug)]
pub struct KillEvent {
    pub entity: Entity,
    pub by_player: bool,
    pub cannon: bool,
}
#[derive(Event, Clone, Debug)]
pub struct PlayerHit {
    pub amount: i32,
    pub from: Vec2,
    pub special: String,
}
#[derive(Event, Clone, Debug)]
pub struct ExplosionEvent {
    pub pos: Vec2,
    pub radius: f32,
    pub dmg: i32,
    pub friendly: bool,
    pub cannon: bool,
}

#[derive(Component)]
pub struct Projectile {
    pub pos: Vec2,
    pub vel: Vec2,
    pub dmg: i32,
    pub friendly: bool,
    pub range_left: f32,
    pub total: f32,
    pub kind: ProjKind,
    pub splash: f32,
    pub special: String,
    pub cannon: bool,
    pub target: Option<Vec2>,
}
#[derive(Component)]
pub struct Fx {
    pub t: f32,
    pub max: f32,
    pub grow: f32,
}
#[derive(Component)]
pub struct Floating(pub f32);
#[derive(Component)]
pub struct GroundSprite(pub usize);

pub fn aim_dir(dir: Vec2) -> Dir {
    if dir.x.abs() > dir.y.abs() {
        if dir.x > 0.0 { Dir::E } else { Dir::W }
    } else if dir.y > 0.0 {
        Dir::S
    } else {
        Dir::N
    }
}

pub fn player_armor(inv: &u67_world::inventory::Inventory) -> i32 {
    inv.equipped.values().filter_map(|i| items::get(&i.id)).map(|d| d.armor as i32).sum()
}

fn update_cursor(windows: Query<&Window>, cams: Query<(&Camera, &GlobalTransform), With<crate::player::MainCamera>>, mut cur: ResMut<Cursor>) {
    let (Ok(w), Ok((cam, gt))) = (windows.single(), cams.single()) else { return };
    cur.valid = false;
    if let Some(p) = w.cursor_position() {
        if let Ok(world) = cam.viewport_to_world_2d(gt, p) {
            cur.tile = Vec2::new(world.x / render::T, -world.y / render::T);
            cur.valid = true;
        }
    }
}

fn weapon_of(game: &Game) -> Option<&'static items::ItemDef> {
    game.0.players[0].inventory.equipped.get(&Slot::HandR).and_then(|i| items::get(&i.id))
}

#[allow(clippy::too_many_arguments)]
fn player_attack(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    kb: Res<ButtonInput<KeyCode>>,
    keys: Res<KeyMap>,
    intent: Res<Intent>,
    cursor: Res<Cursor>,
    mut game: ResMut<Game>,
    mut rt: ResMut<PlayerRt>,
    mut shots: EventWriter<SpawnProjectile>,
    mut dmg: EventWriter<DamageEvent>,
    mut sfx: EventWriter<SfxEvent>,
    mut toast: ResMut<crate::ui::Toast>,
    creatures: Query<(Entity, &Creature)>,
    db: Res<DbRes>,
    mut rng: Local<Option<Rng>>,
    op: Res<crate::interact::Operating>,
) {
    let rng = rng.get_or_insert_with(|| Rng::new(31));
    let dt = time.delta_secs();
    rt.cooldown = (rt.cooldown - dt).max(0.0);
    rt.iframes = (rt.iframes - dt).max(0.0);
    rt.roll_cd = (rt.roll_cd - dt).max(0.0);
    rt.haste = (rt.haste - dt).max(0.0);
    rt.light = (rt.light - dt).max(0.0);
    rt.shake = (rt.shake - dt).max(0.0);
    rt.hurt_flash = (rt.hurt_flash - dt).max(0.0);
    if rt.protect.1 > 0.0 {
        rt.protect.1 -= dt;
        if rt.protect.1 <= 0.0 {
            rt.protect.0 = 0;
        }
    }
    // reload progress
    if let Some((t, gun)) = rt.reload.clone() {
        let t = t - dt;
        if t <= 0.0 {
            rt.reload = None;
            if let Some(gs) = rules::gun_stats(&gun) {
                let inf = game.0.cheats.infinite_ammo;
                let ammo = items::get(&gun).and_then(|d| d.ammo).unwrap_or("ammo_9mm");
                let have = game.0.players[0].loaded.get(&gun).copied().unwrap_or(0);
                let need = gs.mag.saturating_sub(have);
                let take = if inf { need } else { (game.0.players[0].inventory.count(ammo) as u16).min(need) };
                if !inf {
                    game.0.players[0].inventory.remove(ammo, take as u32);
                }
                *game.0.players[0].loaded.entry(gun).or_insert(0) += take;
                sfx.write(SfxEvent("reload".into()));
            }
        } else {
            rt.reload = Some((t, gun));
        }
    }
    let pos = Vec2::from(game.0.players[0].pos);
    let aim = if cursor.valid { (cursor.tile - (pos - Vec2::new(0.0, 0.5))).normalize_or_zero() } else { {
        let (fx, fy) = game.0.players[0].facing.delta();
        Vec2::new(fx as f32, fy as f32)
    } };
    let aim = if aim == Vec2::ZERO { Vec2::Y } else { aim };
    // roll
    if keys.just_pressed(Action::Roll, &kb) && rt.roll_cd <= 0.0 {
        rt.roll_t = 0.25;
        rt.roll_cd = 0.9;
        rt.iframes = 0.4;
        rt.roll_dir = if intent.movement.length() > 0.1 { intent.movement.normalize() } else { aim };
    }
    // reload key
    if keys.just_pressed(Action::Reload, &kb) && rt.reload.is_none() {
        if let Some(w) = weapon_of(&game).filter(|w| w.kind == Kind::Gun) {
            if let Some(gs) = rules::gun_stats(w.id) {
                let have = game.0.players[0].loaded.get(w.id).copied().unwrap_or(0);
                if have < gs.mag && (game.0.cheats.infinite_ammo || game.0.players[0].inventory.count(w.ammo.unwrap_or("")) > 0) {
                    rt.reload = Some((gs.reload, w.id.to_string()));
                    *toast = crate::ui::Toast { text: "Reloading...".into(), timer: 1.0 };
                } else if have < gs.mag {
                    *toast = crate::ui::Toast { text: "Out of ammo!".into(), timer: 1.5 };
                }
            }
        }
    }
    if op.0.is_some() {
        return; // cannon operators fire the cannon instead
    }
    let wants = mouse.pressed(MouseButton::Left) || keys.pressed(Action::Attack, &kb);
    if !wants || rt.cooldown > 0.0 || rt.reload.is_some() || rt.roll_t > 0.0 {
        return;
    }
    game.0.players[0].facing = aim_dir(aim);
    let weapon = weapon_of(&game);
    let stats = game.0.players[0].stats.clone();
    let cheats = game.0.cheats.clone();
    match weapon {
        Some(w) if w.kind == Kind::Gun => {
            let Some(gs) = rules::gun_stats(w.id) else { return };
            let loaded = game.0.players[0].loaded.get(w.id).copied().unwrap_or(0);
            if loaded == 0 && !cheats.infinite_ammo {
                if game.0.players[0].inventory.count(w.ammo.unwrap_or("")) > 0 {
                    rt.reload = Some((gs.reload, w.id.to_string()));
                    *toast = crate::ui::Toast { text: "Reloading...".into(), timer: 1.0 };
                } else {
                    sfx.write(SfxEvent("ui_error".into()));
                    *toast = crate::ui::Toast { text: format!("No {} left!", w.ammo.and_then(items::get).map_or("ammo", |a| a.name)), timer: 1.5 };
                    rt.cooldown = 0.5;
                }
                return;
            }
            if !cheats.infinite_ammo {
                *game.0.players[0].loaded.entry(w.id.to_string()).or_insert(0) -= 1;
            }
            rt.cooldown = gs.cooldown * if rt.haste > 0.0 { 0.6 } else { 1.0 };
            let pellet_dmg = |rng: &mut Rng| -> (i32, bool) {
                let base = if gs.pellets > 1 { (w.damage as f32 * 1.6 / gs.pellets as f32).ceil() as i32 } else { w.damage as i32 };
                let (d, c) = rules::roll_damage(base, stats.dex, rng);
                (if cheats.one_hit { 99999 } else { d }, c)
            };
            for _ in 0..gs.pellets {
                let spread = (rng.f32() - 0.5) * gs.spread_deg.to_radians() * 2.0;
                let (s, c) = spread.sin_cos();
                let v = Vec2::new(aim.x * c - aim.y * s, aim.x * s + aim.y * c);
                let (d, _crit) = pellet_dmg(rng);
                shots.write(SpawnProjectile { pos: pos - Vec2::new(0.0, 0.5) + v * 0.6, vel: v * gs.speed, dmg: d, friendly: true, range: gs.range, kind: ProjKind::Bullet, splash: 0.0, special: gs.special.to_string(), cannon: false, target: None, elem: String::new() });
            }
            let snd = match w.id {
                "hunting_shotgun" => "gun_shotgun",
                "fjord_rifle" | "gungnir_sniper" | "vainamoinen_gun" => "gun_rifle",
                _ => "gun_pistol",
            };
            sfx.write(SfxEvent(if w.id == "vainamoinen_gun" { "kantele".into() } else { snd.into() }));
            rt.shake = 0.06;
        }
        other => {
            // melee (or fists)
            let (range, cd, arc, base) = match other {
                Some(w) => {
                    let m = rules::melee_stats(w.id).unwrap_or(rules::MeleeStats { range: 1.2, cooldown: 0.5, arc_deg: 100.0 });
                    (m.range, m.cooldown, m.arc_deg, w.damage as i32)
                }
                None => (1.0, 0.45, 100.0, 4),
            };
            rt.cooldown = cd * if rt.haste > 0.0 { 0.6 } else { 1.0 };
            sfx.write(SfxEvent("swing".into()));
            let mut hit_any = false;
            for (e, c) in &creatures {
                if c.ally {
                    continue;
                }
                let Some(def) = db.0.creatures.get(&c.def) else { continue };
                let to = c.pos - (pos - Vec2::new(0.0, 0.3));
                let reach = range + radius(def);
                if to.length() > reach {
                    continue;
                }
                let ang = to.normalize_or_zero().dot(aim).clamp(-1.0, 1.0).acos().to_degrees();
                if ang > arc / 2.0 {
                    continue;
                }
                let (d, crit) = rules::roll_damage(base, stats.str_, rng);
                dmg.write(DamageEvent { target: e, amount: if cheats.one_hit { 99999 } else { d }, crit, by_player: true, special: if other.is_some_and(|w| w.id == "mjolnir_drone") { "stun".into() } else { String::new() }, cannon: false });
                hit_any = true;
            }
            if hit_any {
                sfx.write(SfxEvent("hit_flesh".into()));
            }
        }
    }
}

fn spawn_projectiles(mut ev: EventReader<SpawnProjectile>, mut commands: Commands) {
    for s in ev.read() {
        let angle = s.vel.y.atan2(s.vel.x);
        let (color, size) = match s.kind {
            ProjKind::Bullet => (Color::srgb(1.0, 0.92, 0.5), Vec2::new(5.0, 1.5)),
            ProjKind::Cannonball => (Color::srgb(0.12, 0.12, 0.14), Vec2::new(6.0, 6.0)),
            ProjKind::Magic => (match s.elem.as_str() {
                "fire" => Color::srgb(1.0, 0.5, 0.15),
                "lightning" => Color::srgb(0.7, 0.9, 1.0),
                _ => Color::srgb(0.7, 0.5, 1.0),
            }, Vec2::new(7.0, 4.0)),
            ProjKind::EnemyBolt => (if s.elem == "frost" { Color::srgb(0.5, 0.95, 1.0) } else { Color::srgb(1.0, 0.35, 0.3) }, Vec2::new(6.0, 3.0)),
        };
        let p = render::tile_px(s.pos.x, s.pos.y);
        commands.spawn((
            Projectile { pos: s.pos, vel: s.vel, dmg: s.dmg, friendly: s.friendly, range_left: s.range, total: s.range.max(0.01), kind: s.kind, splash: s.splash, special: s.special.clone(), cannon: s.cannon, target: s.target },
            Sprite { color, custom_size: Some(size), ..default() },
            Transform::from_xyz(p.x, p.y, 3.0).with_rotation(Quat::from_rotation_z(-angle)),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn step_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Game>,
    world: Res<WorldRes>,
    db: Res<DbRes>,
    rt: Res<PlayerRt>,
    mut q: Query<(Entity, &mut Projectile, &mut Transform)>,
    creatures: Query<(Entity, &Creature)>,
    mut dmg: EventWriter<DamageEvent>,
    mut hits: EventWriter<PlayerHit>,
    mut boom: EventWriter<ExplosionEvent>,
) {
    let dt = time.delta_secs();
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    let player = Vec2::from(game.0.players[0].pos) - Vec2::new(0.0, 0.5);
    for (e, mut p, mut tf) in &mut q {
        let step = p.vel * dt;
        p.pos += step;
        p.range_left -= step.length();
        let prog = 1.0 - (p.range_left / p.total).clamp(0.0, 1.0);
        let px = render::tile_px(p.pos.x, p.pos.y);
        let arc = if p.kind == ProjKind::Cannonball { (prog * std::f32::consts::PI).sin() * 14.0 } else { 0.0 };
        tf.translation = Vec3::new(px.x, px.y + arc, 3.0 + p.pos.y * 0.001);
        let mut done = false;
        let mut hit_pos = p.pos;
        if p.kind == ProjKind::Cannonball {
            if p.range_left <= 0.0 {
                done = true;
                if let Some(t) = p.target {
                    hit_pos = t;
                }
                boom.write(ExplosionEvent { pos: hit_pos, radius: 2.6, dmg: p.dmg, friendly: p.friendly, cannon: true });
            }
        } else {
            let t = TilePos::new(p.pos.x.floor() as i32, p.pos.y.floor() as i32);
            let td = tiles::def(map.tile(t));
            if !map.in_bounds(t) || (!td.walkable && !td.water) || map.blocked_by_object(t) {
                done = true;
            } else if p.friendly {
                for (ce, c) in &creatures {
                    if c.ally {
                        continue;
                    }
                    let Some(def) = db.0.creatures.get(&c.def) else { continue };
                    if (c.pos - Vec2::new(0.0, 0.4 * def.scale.min(2.0))).distance(p.pos) < radius(def) + 0.2 || c.pos.distance(p.pos) < radius(def) + 0.2 {
                        dmg.write(DamageEvent { target: ce, amount: p.dmg, crit: false, by_player: true, special: p.special.clone(), cannon: p.cannon });
                        done = true;
                        break;
                    }
                }
            } else if p.pos.distance(player) < 0.42 && rt.iframes <= 0.0 {
                hits.write(PlayerHit { amount: p.dmg, from: p.pos - p.vel, special: p.special.clone() });
                done = true;
            }
            if !done && p.range_left <= 0.0 {
                done = true;
            }
            if done && p.splash > 0.0 {
                boom.write(ExplosionEvent { pos: p.pos, radius: p.splash, dmg: p.dmg, friendly: p.friendly, cannon: false });
            }
        }
        if done {
            commands.entity(e).despawn();
        }
    }
}

fn float_text(commands: &mut Commands, pos: Vec2, text: String, color: Color) {
    let p = render::tile_px(pos.x, pos.y);
    commands.spawn((Floating(0.9), Text2d::new(text), TextFont { font_size: 10.0, ..default() }, TextColor(color), Transform::from_xyz(p.x, p.y + 16.0, 20.0)));
}

#[allow(clippy::too_many_arguments)]
fn apply_damage(mut ev: EventReader<DamageEvent>, mut commands: Commands, mut q: Query<&mut Creature>, db: Res<DbRes>, mut kills: EventWriter<KillEvent>, mut sfx: EventWriter<SfxEvent>) {
    for d in ev.read() {
        let Ok(mut c) = q.get_mut(d.target) else { continue };
        if c.hp <= 0 {
            continue;
        }
        let Some(def) = db.0.creatures.get(&c.def) else { continue };
        c.hp -= d.amount;
        c.flash = 0.12;
        c.aggro = true;
        float_text(&mut commands, c.pos, format!("{}{}", d.amount, if d.crit { "!" } else { "" }), if d.crit { Color::srgb(1.0, 0.8, 0.2) } else { Color::WHITE });
        let boss = def.ai == u67_world::npcs::Ai::Boss;
        match d.special.as_str() {
            "stun" if !boss => c.stun = 1.8,
            "stun" => c.stun = 0.4,
            "flare" if !boss => c.fear = 3.0,
            _ => {}
        }
        sfx.write(SfxEvent("hit_flesh".into()));
        if c.hp <= 0 {
            kills.write(KillEvent { entity: d.target, by_player: d.by_player, cannon: d.cannon });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_kills(mut ev: EventReader<KillEvent>, mut commands: Commands, mut game: ResMut<Game>, db: Res<DbRes>, q: Query<(&Creature, &Transform, &Sprite)>, sheets: Res<Sheets>, mut toast: ResMut<crate::ui::Toast>, mut sfx: EventWriter<SfxEvent>, mut rng: Local<Option<Rng>>) {
    let rng = rng.get_or_insert_with(|| Rng::new(555));
    for k in ev.read() {
        let Ok((c, tf, _)) = q.get(k.entity) else { continue };
        let Some(def) = db.0.creatures.get(&c.def) else { continue };
        let map = game.0.current_map.clone();
        if k.by_player && !c.ally {
            *game.0.counters.entry(format!("kills_{}", c.def)).or_insert(0) += 1;
            *game.0.counters.entry("kills_total".into()).or_insert(0) += 1;
            if k.cannon {
                *game.0.counters.entry("cannon_kills".into()).or_insert(0) += 1;
            }
            let lvl_gap = game.0.players[0].stats.level as i32 - 1;
            let xp = ((def.xp as f32) * (1.0 - (lvl_gap as f32 * 0.03).min(0.5))).round().max(1.0) as u32;
            let g = game.0.players[0].stats.add_xp(xp);
            toast.text = format!("{} defeated. +{xp} XP{}", def.name, if g > 0 { " - LEVEL UP!" } else { "" });
            toast.timer = 3.0;
            if g > 0 {
                sfx.write(SfxEvent("level_up".into()));
            }
            if let Some(key) = &c.unique_key {
                game.0.flags.insert(key.clone());
            }
            if let Some(l) = def.loot.as_ref().and_then(|l| db.0.loot.get(l)) {
                for (item, qty) in l.roll(rng) {
                    let jitter = Vec2::new(rng.f32() - 0.5, rng.f32() - 0.5) * 0.9;
                    let pos = c.pos + jitter;
                    game.0.ground.push(GroundItem { map: map.clone(), pos: [pos.x, pos.y], item: Item::new(&item, qty) });
                }
            }
        }
        sfx.write(SfxEvent("death".into()));
        // corpse
        let row = render::character_row(&def.arch).unwrap_or(0);
        commands.spawn((
            Corpse(8.0),
            Sprite { image: sheets.chars_img.clone(), texture_atlas: Some(TextureAtlas { layout: sheets.chars_layout.clone(), index: ((row * 4 + c.facing.index() as u32) * CHAR_COLS + 10) as usize }), anchor: Anchor::Custom(Vec2::new(0.0, -0.5 + 1.5 / 24.0)), ..default() },
            Transform::from_translation(Vec3::new(tf.translation.x, tf.translation.y, 1.5)).with_scale(Vec3::splat(def.scale)),
        ));
        commands.entity(k.entity).despawn();
    }
}

fn apply_player_hit(mut ev: EventReader<PlayerHit>, mut game: ResMut<Game>, mut rt: ResMut<PlayerRt>, mut sfx: EventWriter<SfxEvent>, mut next: ResMut<NextState<AppState>>) {
    for h in ev.read() {
        if rt.iframes > 0.0 || game.0.cheats.god {
            continue;
        }
        let armor = player_armor(&game.0.players[0].inventory) + rt.protect.0;
        let d = rules::after_armor(h.amount, armor);
        game.0.players[0].stats.hp -= d;
        rt.iframes = 0.35;
        rt.hurt_flash = 0.25;
        rt.shake = 0.12;
        sfx.write(SfxEvent("hit_flesh".into()));
        if game.0.players[0].stats.hp <= 0 {
            game.0.players[0].stats.hp = 0;
            next.set(AppState::Dead);
            sfx.write(SfxEvent("death".into()));
        }
    }
}

const DESTRUCTIBLE: &[&str] = &["crate", "barrel", "door_wood", "door_open", "signpost", "table", "bed", "pine_tree", "birch_tree"];

#[allow(clippy::too_many_arguments)]
fn explosions(
    mut ev: EventReader<ExplosionEvent>,
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut world: ResMut<WorldRes>,
    mut dirty: ResMut<DirtyChunks>,
    creatures: Query<(Entity, &Creature)>,
    mut dmg: EventWriter<DamageEvent>,
    mut hits: EventWriter<PlayerHit>,
    mut sfx: EventWriter<SfxEvent>,
    rt: Res<PlayerRt>,
) {
    for b in ev.read() {
        // visual
        let p = render::tile_px(b.pos.x, b.pos.y);
        commands.spawn((Fx { t: 0.0, max: 0.45, grow: b.radius * render::T * 2.2 }, Sprite { color: Color::srgba(1.0, 0.6, 0.2, 0.85), custom_size: Some(Vec2::splat(6.0)), ..default() }, Transform::from_xyz(p.x, p.y, 15.0)));
        sfx.write(SfxEvent(if b.cannon { "explosion".into() } else { "hit_metal".into() }));
        for (e, c) in &creatures {
            let d = c.pos.distance(b.pos);
            if d <= b.radius + 0.3 && b.friendly != c.ally {
                let amt = (b.dmg as f32 * (1.0 - (d / (b.radius + 0.3)) * 0.6)).round() as i32;
                dmg.write(DamageEvent { target: e, amount: amt.max(1), crit: false, by_player: b.friendly, special: String::new(), cannon: b.cannon });
            }
        }
        let pp = Vec2::from(game.0.players[0].pos);
        let d = pp.distance(b.pos);
        if d <= b.radius && rt.iframes <= 0.0 {
            hits.write(PlayerHit { amount: (b.dmg as f32 * if b.friendly { 0.4 } else { 1.0 }) as i32, from: b.pos, special: String::new() });
        }
        if !b.cannon {
            continue;
        }
        // destroy scenery (cannons only)
        let map_name = game.0.current_map.clone();
        let Some(map) = world.0.maps.get_mut(&map_name) else { continue };
        let mut remove: Vec<usize> = vec![];
        for (i, o) in map.objects.iter().enumerate() {
            if DESTRUCTIBLE.contains(&o.kind.as_str()) && Vec2::new(o.pos.x as f32 + 0.5, o.pos.y as f32 + 0.5).distance(b.pos) <= b.radius {
                remove.push(i);
            }
        }
        for &i in remove.iter().rev() {
            if let Some(o) = map.remove_object(i) {
                let key = obj_key(&map_name, o.pos.x, o.pos.y, &o.kind);
                game.0.obj_state.entry(key).or_default().removed = true;
                dirty.0.insert(o.pos.chunk());
                if let Some(items) = o.contents {
                    for it in items {
                        game.0.ground.push(GroundItem { map: map_name.clone(), pos: [o.pos.x as f32 + 0.5, o.pos.y as f32 + 0.5], item: it });
                    }
                }
            }
        }
        let r = b.radius.ceil() as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                let t = TilePos::new(b.pos.x.floor() as i32 + dx, b.pos.y.floor() as i32 + dy);
                if map.tile(t) == tiles::WALL_WOOD && Vec2::new(t.x as f32 + 0.5, t.y as f32 + 0.5).distance(b.pos) <= b.radius - 0.4 {
                    map.set_tile(t, tiles::FLOOR_WOOD);
                    game.0.tile_edits.push((map_name.clone(), t.x, t.y, tiles::FLOOR_WOOD.0));
                    dirty.0.insert(t.chunk());
                }
            }
        }
    }
}

fn fx_update(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Fx, &mut Sprite)>, mut f: Query<(Entity, &mut Floating, &mut Transform, &mut TextColor)>) {
    let dt = time.delta_secs();
    for (e, mut fx, mut s) in &mut q {
        fx.t += dt;
        let k = (fx.t / fx.max).min(1.0);
        s.custom_size = Some(Vec2::splat(6.0 + fx.grow * k));
        s.color = Color::srgba(1.0, 0.6 - 0.4 * k, 0.2, 0.85 * (1.0 - k));
        if fx.t >= fx.max {
            commands.entity(e).despawn();
        }
    }
    for (e, mut fl, mut tf, mut c) in &mut f {
        fl.0 -= dt;
        tf.translation.y += 22.0 * dt;
        c.0 = c.0.with_alpha((fl.0 / 0.5).clamp(0.0, 1.0));
        if fl.0 <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

// ---------------------------------------------------------------- ground items

#[derive(Resource, Default)]
struct GroundSync {
    map: String,
    n: usize,
}

fn item_index(id: &str) -> usize {
    items::ITEMS.iter().position(|d| d.id == id).unwrap_or(0)
}

fn sync_ground(mut commands: Commands, game: Res<Game>, sheets: Res<Sheets>, mut st: ResMut<GroundSync>, q: Query<Entity, With<GroundSprite>>) {
    let here: Vec<(usize, &GroundItem)> = game.0.ground.iter().enumerate().filter(|(_, g)| g.map == game.0.current_map).collect();
    if st.map == game.0.current_map && st.n == game.0.ground.len() {
        return;
    }
    st.map = game.0.current_map.clone();
    st.n = game.0.ground.len();
    for e in &q {
        commands.entity(e).despawn();
    }
    for (i, g) in here {
        let p = render::tile_px(g.pos[0], g.pos[1]);
        commands.spawn((GroundSprite(i), Sprite { image: sheets.items_img.clone(), texture_atlas: Some(TextureAtlas { layout: sheets.items_layout.clone(), index: item_index(&g.item.id) }), anchor: Anchor::BottomCenter, ..default() }, Transform::from_xyz(p.x, p.y - 4.0, 1.2 + g.pos[1] * 0.001)));
    }
}

/// Try to pick up ground item `idx`; returns a message.
pub fn pickup(game: &mut Game, idx: usize) -> String {
    if idx >= game.0.ground.len() {
        return String::new();
    }
    let g = game.0.ground[idx].clone();
    let name = items::get(&g.item.id).map_or(g.item.id.as_str(), |d| d.name);
    let res = game.0.players[0].inventory.add(&g.item.id, g.item.qty);
    match res {
        Ok(()) => {
            // containers keep their contents
            if !g.item.contents.is_empty() {
                if let Some(last) = game.0.players[0].inventory.pack.last_mut() {
                    last.contents = g.item.contents.clone();
                }
            }
            game.0.ground.remove(idx);
            format!("Picked up {} x{}", name, g.item.qty)
        }
        Err(_) => format!("Too heavy: {name}"),
    }
}

fn auto_pickup(mut game: ResMut<Game>, mut toast: ResMut<crate::ui::Toast>, mut sfx: EventWriter<SfxEvent>) {
    let p = game.0.players[0].pos;
    let map = game.0.current_map.clone();
    let mut i = 0;
    while i < game.0.ground.len() {
        let g = &game.0.ground[i];
        let near = g.map == map && (g.pos[0] - p[0]).powi(2) + (g.pos[1] - p[1]).powi(2) < 0.8 * 0.8;
        let auto = matches!(items::get(&g.item.id).map(|d| d.kind), Some(Kind::Ammo)) || g.item.id == "silver" || g.item.id == "krediitti";
        if near && auto {
            let msg = pickup(&mut game, i);
            if !msg.is_empty() && !msg.starts_with("Too") {
                *toast = crate::ui::Toast { text: msg, timer: 1.2 };
                sfx.write(SfxEvent("pickup".into()));
                continue;
            }
        }
        i += 1;
    }
}

fn roll_move(mut game: ResMut<Game>, world: Res<WorldRes>, mut rt: ResMut<PlayerRt>, time: Res<Time>) {
    if rt.roll_t <= 0.0 {
        return;
    }
    let dt = time.delta_secs();
    rt.roll_t -= dt;
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    let cur = Vec2::from(game.0.players[0].pos);
    let np = crate::player::step(map, cur, rt.roll_dir * 9.0 * dt, game.0.cheats.noclip);
    game.0.players[0].pos = np.into();
}

fn game_ready(game: Option<Res<Game>>, db: Option<Res<DbRes>>, sheets: Option<Res<Sheets>>) -> bool {
    game.is_some() && db.is_some() && sheets.is_some()
}

pub struct CombatPlugin;
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cursor>()
            .init_resource::<PlayerRt>()
            .init_resource::<GroundSync>()
            .init_resource::<DirtyChunks>()
            .add_event::<SpawnProjectile>()
            .add_event::<DamageEvent>()
            .add_event::<KillEvent>()
            .add_event::<PlayerHit>()
            .add_event::<ExplosionEvent>()
            .add_systems(Update, (update_cursor, player_attack).chain().run_if(game_ready).run_if(in_state(AppState::Playing)))
            .add_systems(Update, (fx_update, sync_ground).run_if(game_ready).run_if(not(in_state(AppState::Boot))).run_if(not(in_state(AppState::MainMenu))))
            .add_systems(
                FixedUpdate,
                (roll_move, spawn_projectiles, step_projectiles, explosions, apply_damage, handle_kills, apply_player_hit, auto_pickup).chain().run_if(game_ready).run_if(in_state(AppState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aim_directions() {
        assert_eq!(aim_dir(Vec2::new(1.0, 0.3)), Dir::E);
        assert_eq!(aim_dir(Vec2::new(-0.2, 1.0)), Dir::S);
    }
    #[test]
    fn armor_sums_equipped() {
        let mut inv = u67_world::inventory::Inventory::new(20);
        inv.add("kevlar_vest", 1).unwrap();
        inv.add("horned_helm", 1).unwrap();
        inv.equip(&[0]).unwrap();
        inv.equip(&[0]).unwrap();
        assert_eq!(player_armor(&inv), 9);
    }
    #[test]
    fn ground_pickup_respects_weight() {
        let mut g = Game(crate::data::GameData::new(1, "midgard", [0.0, 0.0]));
        g.0.players[0].inventory.pack.clear();
        g.0.players[0].inventory.max_weight = 3.0;
        g.0.ground.push(GroundItem { map: "midgard".into(), pos: [0.0, 0.0], item: Item::new("bear_axe", 1) });
        g.0.ground.push(GroundItem { map: "midgard".into(), pos: [0.0, 0.0], item: Item::new("silver", 5) });
        assert!(pickup(&mut g, 0).starts_with("Too heavy"));
        assert_eq!(g.0.ground.len(), 2);
        assert!(pickup(&mut g, 1).starts_with("Picked up"));
        assert_eq!(g.0.ground.len(), 1);
    }
}
