//! Player entity: fixed-step movement with tile collision, portals, camera, day/night overlay.
use crate::app::{AppState, Game, SettingsRes, WorldRes};
use crate::input::{Intent, KeyMap};
use crate::render::{self, Sheets};
use crate::settings::Action;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use u67_core::{Dir, TilePos};
use u67_world::map::Map;

#[derive(Resource)]
pub struct Zoom(pub f32);

#[derive(Component)]
pub struct Player(pub usize);
#[derive(Component, Default)]
pub struct Motion {
    pub prev: Vec2,
    pub cur: Vec2,
    pub moving: bool,
}
#[derive(Component)]
pub struct MainCamera;
#[derive(Component)]
pub struct Daylight;

pub const WALK: f32 = 3.2;
pub const RUN: f32 = 5.5;
const HALF_W: f32 = 0.28;
const DEPTH: f32 = 0.3;

/// Can an entity with its feet at (x,y) (tile units) stand here?
pub fn can_stand(map: &Map, x: f32, y: f32) -> bool {
    for (dx, dy) in [(-HALF_W, 0.0), (HALF_W, 0.0), (-HALF_W, -DEPTH), (HALF_W, -DEPTH)] {
        if !map.walkable(TilePos::new((x + dx).floor() as i32, (y + dy).floor() as i32)) {
            return false;
        }
    }
    true
}

/// Move with per-axis sliding collision.
pub fn step(map: &Map, pos: Vec2, delta: Vec2, noclip: bool) -> Vec2 {
    if noclip {
        return pos + delta;
    }
    let mut p = pos;
    if can_stand(map, p.x + delta.x, p.y) {
        p.x += delta.x;
    }
    if can_stand(map, p.x, p.y + delta.y) {
        p.y += delta.y;
    }
    p
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

fn spawn_camera_and_player(mut commands: Commands, game: Res<Game>, sheets: Res<Sheets>, zoom: Res<Zoom>, existing: Query<(), With<Player>>, cams: Query<(), With<MainCamera>>, old_cams: Query<Entity, (With<Camera2d>, Without<MainCamera>)>) {
    if existing.iter().next().is_some() {
        return;
    }
    for e in &old_cams {
        commands.entity(e).despawn();
    }
    if cams.iter().next().is_none() {
        commands
            .spawn((Camera2d, MainCamera, Projection::from(OrthographicProjection { scale: 1.0 / zoom.0, ..OrthographicProjection::default_2d() }), Transform::from_xyz(0.0, 0.0, 999.0)))
            .with_children(|c| {
                c.spawn((Daylight, Sprite { color: Color::srgba(0.04, 0.06, 0.22, 0.0), custom_size: Some(Vec2::splat(6000.0)), ..default() }, Transform::from_xyz(0.0, 0.0, -900.0)));
            });
    }
    let arch = render::character_row("player").unwrap_or(0);
    for (i, p) in game.0.players.iter().enumerate() {
        let pos = Vec2::from(p.pos);
        let row = arch * 4 + p.facing.index() as u32;
        commands.spawn((
            Player(i),
            Motion { prev: pos, cur: pos, moving: false },
            Sprite { image: sheets.chars_img.clone(), texture_atlas: Some(TextureAtlas { layout: sheets.chars_layout.clone(), index: (row * render::CHAR_COLS) as usize }), anchor: Anchor::Custom(Vec2::new(0.0, -0.5 + 1.5 / 24.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 2.0),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn move_players(time: Res<Time>, mut game: ResMut<Game>, world: Res<WorldRes>, intent: Res<Intent>, mut q: Query<(&Player, &mut Motion)>, state: Res<State<AppState>>, rt: Res<crate::combat::PlayerRt>) {
    let dt = time.delta_secs();
    if *state.get() == AppState::Playing {
        game.0.clock.advance(dt);
    }
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    for (pl, mut m) in &mut q {
        m.prev = m.cur;
        m.moving = false;
        if *state.get() != AppState::Playing {
            continue;
        }
        // sailing
        if let Some(mut v) = game.0.vehicle.clone() {
            let mv = intent.movement;
            if mv.length() > 0.01 {
                let speed = if intent.run { 8.0 } else { 5.5 };
                let np = crate::interact::boat_step(map, Vec2::from(v.pos), mv * speed * dt);
                m.moving = (np - Vec2::from(v.pos)).length() > 1e-4;
                v.pos = np.into();
                v.facing = dir_of(mv);
            }
            m.cur = Vec2::from(v.pos);
            game.0.players[pl.0].pos = v.pos;
            game.0.players[pl.0].facing = v.facing;
            game.0.vehicle = Some(v);
            continue;
        }
        if rt.roll_t > 0.0 {
            // dodge roll moves the player (see combat::roll_move)
            m.cur = Vec2::from(game.0.players[pl.0].pos);
            m.moving = true;
            continue;
        }
        let cheats = game.0.cheats.clone();
        let mut v = if pl.0 == 0 { intent.movement } else { Vec2::ZERO };
        let mut speed = if intent.run { RUN } else { WALK };
        if cheats.fast {
            speed *= 3.0;
        }
        if rt.haste > 0.0 {
            speed *= 1.5;
        }
        if v.length() > 0.01 {
            v = v.normalize_or_zero() * intent.movement.length() * speed * dt;
            let np = step(map, m.cur, v, cheats.noclip);
            m.moving = (np - m.cur).length() > 1e-4;
            m.cur = np;
            if !rt.reload.is_some() || true {
                game.0.players[pl.0].facing = dir_of(intent.movement);
            }
        }
        game.0.players[pl.0].pos = m.cur.into();
        // portals
        let t = TilePos::new(m.cur.x.floor() as i32, m.cur.y.floor() as i32);
        if let Some(p) = map.portal_at(t) {
            let (tm, tp) = (p.target_map.clone(), p.target_pos);
            if world.0.maps.contains_key(&tm) {
                game.0.current_map = tm;
                game.0.players[pl.0].pos = [tp.x as f32 + 0.5, tp.y as f32 + 0.5];
                game.0.map_dirty = true;
            }
        }
    }
}

/// After a teleport/load, snap player entities to the saved positions.
fn sync_after_teleport(mut game: ResMut<Game>, mut q: Query<(&Player, &mut Motion)>) {
    if !game.0.map_dirty {
        // external movement (roll, knockback) writes game positions; keep the entity in step
        for (pl, mut m) in &mut q {
            let p = Vec2::from(game.0.players[pl.0].pos);
            if (p - m.cur).length() > 1e-4 && m.moving {
                m.cur = p;
            }
        }
        return;
    }
    game.0.map_dirty = false;
    for (pl, mut m) in &mut q {
        let p = Vec2::from(game.0.players[pl.0].pos);
        m.prev = p;
        m.cur = p;
    }
}

fn render_players(time: Res<Time<Fixed>>, game: Res<Game>, mut q: Query<(&Player, &Motion, &mut Transform, &mut Sprite)>, wall: Res<Time>) {
    let alpha = time.overstep_fraction();
    let arch = render::character_row("player").unwrap_or(0);
    for (pl, m, mut tf, mut sp) in &mut q {
        let p = m.prev.lerp(m.cur, alpha);
        let px = render::tile_px(p.x, p.y);
        tf.translation = Vec3::new(px.x.round(), px.y.round(), 2.0 + p.y * 0.001);
        let facing = game.0.players[pl.0].facing;
        let frame = if m.moving { 1 + ((wall.elapsed_secs() * 9.0) as u32 % 4) } else { 0 };
        let row = arch * 4 + facing.index() as u32;
        if let Some(a) = &mut sp.texture_atlas {
            a.index = (row * render::CHAR_COLS + frame) as usize;
        }
    }
}

fn camera_follow(game: Res<Game>, rt: Res<crate::combat::PlayerRt>, time: Res<Time>, zoom: Res<Zoom>, players: Query<&Transform, (With<Player>, Without<MainCamera>)>, mut cam: Query<&mut Transform, With<MainCamera>>) {
    let (Ok(mut c), Some(p)) = (cam.single_mut(), players.iter().next()) else { return };
    let _ = &game;
    let snap = |v: f32| (v * zoom.0).round() / zoom.0;
    let shake = if rt.shake > 0.0 { Vec2::new((time.elapsed_secs() * 90.0).sin(), (time.elapsed_secs() * 70.0).cos()) * rt.shake * 10.0 } else { Vec2::ZERO };
    c.translation.x = snap(p.translation.x + shake.x);
    c.translation.y = snap(p.translation.y + 8.0 + shake.y);
}

fn zoom_keys(kb: Res<ButtonInput<KeyCode>>, keys: Res<KeyMap>, mut zoom: ResMut<Zoom>, mut proj: Query<&mut Projection, With<MainCamera>>, cli: Res<crate::app::Cli>) {
    let _ = &cli;
    let mut changed = false;
    if keys.just_pressed(Action::ZoomIn, &kb) {
        zoom.0 = (zoom.0 + 1.0).min(8.0);
        changed = true;
    }
    if keys.just_pressed(Action::ZoomOut, &kb) {
        zoom.0 = (zoom.0 - 1.0).max(1.0);
        changed = true;
    }
    if changed {
        if let Ok(mut p) = proj.single_mut() {
            if let Projection::Orthographic(o) = &mut *p {
                o.scale = 1.0 / zoom.0;
            }
        }
    }
}

fn daylight(game: Res<Game>, rt: Res<crate::combat::PlayerRt>, mut q: Query<&mut Sprite, With<Daylight>>) {
    let underground = matches!(game.0.current_map.as_str(), "mimir_depths" | "barrow_1" | "barrow_2" | "fenrir_den" | "troll_cave" | "hel_gate_crypt");
    let mut a = (1.0 - game.0.clock.daylight()) * 0.62;
    if underground {
        let lit = rt.light > 0.0 || game.0.players[0].inventory.count("torch") > 0;
        a = if lit { 0.36 } else { 0.62 };
    } else if rt.light > 0.0 {
        a *= 0.6;
    }
    let hurt = rt.hurt_flash;
    for mut s in &mut q {
        s.color = if hurt > 0.0 { Color::srgba(0.7, 0.0, 0.0, 0.35 * (hurt / 0.25)) } else { Color::srgba(0.04, 0.06, 0.22, a) };
    }
}

fn game_active(game: Option<Res<Game>>, state: Res<State<AppState>>) -> bool {
    game.is_some() && !matches!(state.get(), AppState::Boot | AppState::MainMenu)
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        let z = app.world().get_resource::<crate::app::Cli>().and_then(|c| c.zoom).unwrap_or_else(|| app.world().resource::<SettingsRes>().0.zoom);
        app.insert_resource(Zoom(z.clamp(1.0, 8.0)))
            .add_systems(Update, (spawn_camera_and_player, sync_after_teleport, render_players, camera_follow, zoom_keys, daylight).chain().run_if(game_active))
            .add_systems(FixedUpdate, move_players.run_if(game_active));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use u67_world::tiles;
    #[test]
    fn collision_and_sliding() {
        let mut m = Map::new("t", 10, 10, tiles::GRASS);
        for y in 0..10 {
            m.set_tile(TilePos::new(5, y), tiles::WALL_STONE);
        }
        assert!(can_stand(&m, 2.5, 2.5));
        assert!(!can_stand(&m, 5.5, 2.5));
        // walking into the wall stops at it; sliding along y still works
        let p = step(&m, Vec2::new(4.6, 2.5), Vec2::new(0.5, 0.2), false);
        assert!(p.x < 4.8 && (p.y - 2.7).abs() < 1e-4, "{p}");
        // noclip passes through
        assert_eq!(step(&m, Vec2::new(4.6, 2.5), Vec2::new(1.0, 0.0), true).x, 5.6);
        // map edge blocks
        assert!(!can_stand(&m, 0.1, 2.5));
    }
    #[test]
    fn facing() {
        assert_eq!(dir_of(Vec2::new(1.0, 0.2)), Dir::E);
        assert_eq!(dir_of(Vec2::new(0.0, -1.0)), Dir::N);
    }
}
