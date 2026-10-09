//! Player entities (1-4 local seats): fixed-step movement with tile collision, portals,
//! split-screen cameras, day/night overlay.
use crate::app::{AppState, Game, SettingsRes, WorldRes};
use crate::input::KeyMap;
use crate::render::{self, Sheets};
use crate::seats::{self, Intents, PlayerRt};
use crate::settings::Action;
use bevy::prelude::*;
use bevy::render::camera::{ClearColorConfig, Viewport};
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
/// The camera of seat `.0`.
#[derive(Component)]
pub struct SeatCam(pub usize);
/// Camera of seat 0 (used for mouse aiming).
#[derive(Component)]
pub struct MainCamera;
#[derive(Component)]
pub struct Daylight(pub usize);

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

/// Effective zoom: split-screen viewports are smaller, so zoom out a little.
pub fn effective_zoom(base: f32, seats: usize) -> f32 {
    if seats >= 2 { (base * 0.67).max(1.5) } else { base }
}

#[derive(Component)]
pub struct UiCamera;

#[allow(clippy::too_many_arguments)]
fn spawn_cameras_and_players(
    mut commands: Commands,
    game: Res<Game>,
    sheets: Res<Sheets>,
    zoom: Res<Zoom>,
    players: Query<Entity, With<Player>>,
    cams: Query<Entity, With<SeatCam>>,
) {
    let n = game.0.players.len();
    if players.iter().count() == n && cams.iter().count() == n {
        return;
    }
    for e in players.iter().chain(cams.iter()) {
        commands.entity(e).despawn();
    }
    let z = effective_zoom(zoom.0, n);
    for i in 0..n {
        let mut cam = commands.spawn((
            Camera2d,
            SeatCam(i),
            Camera { order: i as isize, ..default() },
            Projection::from(OrthographicProjection { scale: 1.0 / z, ..OrthographicProjection::default_2d() }),
            Transform::from_xyz(0.0, 0.0, 999.0),
        ));
        if i == 0 {
            cam.insert(MainCamera);
        }
        cam.with_children(|c| {
            c.spawn((Daylight(i), Sprite { color: Color::srgba(0.04, 0.06, 0.22, 0.0), custom_size: Some(Vec2::splat(6000.0)), ..default() }, Transform::from_xyz(0.0, 0.0, -900.0)));
        });
    }
    for (i, p) in game.0.players.iter().enumerate() {
        let pos = Vec2::from(p.pos);
        let arch = render::character_row(seats::SEAT_ARCH[i.min(3)]).unwrap_or(0);
        let row = arch * 4 + p.facing.index() as u32;
        commands.spawn((
            Player(i),
            Motion { prev: pos, cur: pos, moving: false },
            Sprite { image: sheets.chars_img.clone(), texture_atlas: Some(TextureAtlas { layout: sheets.chars_layout.clone(), index: (row * render::CHAR_COLS) as usize }), anchor: Anchor::Custom(Vec2::new(0.0, -0.5 + 1.5 / 24.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 2.0),
        ));
    }
}

/// Lay the seat cameras out in the window (split-screen viewports).
fn layout_viewports(windows: Query<&Window>, game: Res<Game>, mut cams: Query<(&SeatCam, &mut Camera)>) {
    let Ok(w) = windows.single() else { return };
    let n = game.0.players.len();
    let rects = seats::layout(n, w.physical_width(), w.physical_height());
    for (sc, mut cam) in &mut cams {
        cam.viewport = if n <= 1 {
            None
        } else {
            rects.get(sc.0).map(|r| Viewport { physical_position: UVec2::new(r.0, r.1), physical_size: UVec2::new(r.2.max(1), r.3.max(1)), ..default() })
        };
    }
}

/// The overlay UI camera draws on top of all world cameras without clearing them.
fn ui_camera_clear(seat_cams: Query<(), With<SeatCam>>, mut ui: Query<&mut Camera, With<UiCamera>>) {
    let any = seat_cams.iter().next().is_some();
    for mut c in &mut ui {
        c.clear_color = if any { ClearColorConfig::None } else { ClearColorConfig::Default };
    }
}

#[allow(clippy::too_many_arguments)]
fn move_players(
    time: Res<Time>,
    mut game: ResMut<Game>,
    world: Res<WorldRes>,
    mut intents: ResMut<Intents>,
    mut q: Query<(&Player, &mut Motion)>,
    state: Res<State<AppState>>,
    mut rt: ResMut<PlayerRt>,
) {
    let dt = time.delta_secs();
    if *state.get() == AppState::Playing {
        game.0.clock.advance(dt);
    }
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    for (pl, mut m) in &mut q {
        m.prev = m.cur;
        m.moving = false;
        if *state.get() != AppState::Playing || pl.0 >= game.0.players.len() {
            continue;
        }
        let i = pl.0;
        intents.active = i.min(intents.list.len().saturating_sub(1));
        rt.active = i;
        let intent = intents.get(i);
        // downed players lie still (and tick toward auto-revive)
        if rt.downed {
            rt.down_timer += dt;
            continue;
        }
        // sailing: only the first seat's vehicle exists
        if i == 0 {
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
                game.0.players[0].pos = v.pos;
                game.0.players[0].facing = v.facing;
                game.0.vehicle = Some(v);
                continue;
            }
        }
        if rt.roll_t > 0.0 {
            m.cur = Vec2::from(game.0.players[i].pos);
            m.moving = true;
            continue;
        }
        let cheats = game.0.cheats.clone();
        let mut v = intent.movement;
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
            game.0.players[i].facing = dir_of(intent.movement);
        }
        game.0.players[i].pos = m.cur.into();
        // portals: the first seat to step on one takes everyone along (shared world)
        let t = TilePos::new(m.cur.x.floor() as i32, m.cur.y.floor() as i32);
        if let Some(p) = map.portal_at(t) {
            let (tm, tp) = (p.target_map.clone(), p.target_pos);
            if world.0.maps.contains_key(&tm) {
                game.0.current_map = tm;
                for (k, pd) in game.0.players.iter_mut().enumerate() {
                    pd.pos = [tp.x as f32 + 0.5 + k as f32 * 0.6, tp.y as f32 + 0.5];
                }
                game.0.map_dirty = true;
            }
        }
    }
    intents.active = 0;
    rt.active = 0;
}

/// After a teleport/load, snap player entities to the saved positions.
fn sync_after_teleport(mut game: ResMut<Game>, mut q: Query<(&Player, &mut Motion)>) {
    if !game.0.map_dirty {
        // external movement (roll, knockback) writes game positions; keep the entity in step
        for (pl, mut m) in &mut q {
            let Some(pd) = game.0.players.get(pl.0) else { continue };
            let p = Vec2::from(pd.pos);
            if (p - m.cur).length() > 1e-4 && m.moving {
                m.cur = p;
            }
        }
        return;
    }
    game.0.map_dirty = false;
    for (pl, mut m) in &mut q {
        let Some(pd) = game.0.players.get(pl.0) else { continue };
        let p = Vec2::from(pd.pos);
        m.prev = p;
        m.cur = p;
    }
}

fn render_players(time: Res<Time<Fixed>>, game: Res<Game>, rt: Res<PlayerRt>, mut q: Query<(&Player, &Motion, &mut Transform, &mut Sprite)>, wall: Res<Time>) {
    let alpha = time.overstep_fraction();
    for (pl, m, mut tf, mut sp) in &mut q {
        let Some(pd) = game.0.players.get(pl.0) else { continue };
        let arch = render::character_row(seats::SEAT_ARCH[pl.0.min(3)]).unwrap_or(0);
        let p = m.prev.lerp(m.cur, alpha);
        let px = render::tile_px(p.x, p.y);
        tf.translation = Vec3::new(px.x.round(), px.y.round(), 2.0 + p.y * 0.001);
        let down = rt.list.get(pl.0).is_some_and(|r| r.downed);
        let frame = if down { 10 } else if m.moving { 1 + ((wall.elapsed_secs() * 9.0) as u32 % 4) } else { 0 };
        let row = arch * 4 + pd.facing.index() as u32;
        sp.color = if down { Color::srgb(0.6, 0.6, 0.7) } else { Color::WHITE };
        if let Some(a) = &mut sp.texture_atlas {
            a.index = (row * render::CHAR_COLS + frame) as usize;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn camera_follow(game: Res<Game>, rt: Res<PlayerRt>, time: Res<Time>, zoom: Res<Zoom>, players: Query<(&Player, &Transform), Without<SeatCam>>, mut cams: Query<(&SeatCam, &mut Transform, &mut Projection)>) {
    let n = game.0.players.len();
    let z = effective_zoom(zoom.0, n);
    for (sc, mut c, mut proj) in &mut cams {
        let Some((_, p)) = players.iter().find(|(pl, _)| pl.0 == sc.0) else { continue };
        let shake_amt = rt.list.get(sc.0).map_or(0.0, |r| r.shake);
        let shake = if shake_amt > 0.0 { Vec2::new((time.elapsed_secs() * 90.0).sin(), (time.elapsed_secs() * 70.0).cos()) * shake_amt * 10.0 } else { Vec2::ZERO };
        let snap = |v: f32| (v * z).round() / z;
        c.translation.x = snap(p.translation.x + shake.x);
        c.translation.y = snap(p.translation.y + 8.0 + shake.y);
        if let Projection::Orthographic(o) = &mut *proj {
            let want = 1.0 / z;
            if (o.scale - want).abs() > 1e-5 {
                o.scale = want;
            }
        }
    }
}

fn zoom_keys(kb: Res<ButtonInput<KeyCode>>, keys: Res<KeyMap>, mut zoom: ResMut<Zoom>) {
    if keys.just_pressed(Action::ZoomIn, &kb) {
        zoom.0 = (zoom.0 + 1.0).min(8.0);
    }
    if keys.just_pressed(Action::ZoomOut, &kb) {
        zoom.0 = (zoom.0 - 1.0).max(1.0);
    }
}

fn daylight(game: Res<Game>, rt: Res<PlayerRt>, mut q: Query<(&Daylight, &mut Sprite)>) {
    let underground = matches!(game.0.current_map.as_str(), "mimir_depths" | "barrow_1" | "barrow_2" | "fenrir_den" | "troll_cave" | "hel_gate_crypt");
    for (d, mut s) in &mut q {
        let seat = rt.list.get(d.0).cloned().unwrap_or_default();
        let mut a = (1.0 - game.0.clock.daylight()) * 0.62;
        if underground {
            let lit = seat.light > 0.0 || game.0.players.get(d.0).is_some_and(|p| p.inventory.count("torch") > 0);
            a = if lit { 0.36 } else { 0.62 };
        } else if seat.light > 0.0 {
            a *= 0.6;
        }
        s.color = if seat.hurt_flash > 0.0 { Color::srgba(0.7, 0.0, 0.0, 0.35 * (seat.hurt_flash / 0.25)) } else { Color::srgba(0.04, 0.06, 0.22, a) };
    }
}

fn game_active(game: Option<Res<Game>>, sheets: Option<Res<Sheets>>, state: Res<State<AppState>>) -> bool {
    game.is_some() && sheets.is_some() && !matches!(state.get(), AppState::Boot | AppState::MainMenu)
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        let z = app.world().get_resource::<crate::app::Cli>().and_then(|c| c.zoom).unwrap_or_else(|| app.world().resource::<SettingsRes>().0.zoom);
        app.insert_resource(Zoom(z.clamp(1.0, 8.0)))
            .init_resource::<Intents>()
            .init_resource::<seats::Seats>()
            .init_resource::<PlayerRt>()
            .init_resource::<seats::Operating>()
            .init_resource::<seats::ActiveSeat>()
            .init_resource::<seats::SeatSwap>()
            .add_systems(Update, ui_camera_clear)
            .add_systems(Update, (spawn_cameras_and_players, layout_viewports, sync_after_teleport, render_players, camera_follow, zoom_keys, daylight).chain().run_if(game_active))
            .add_systems(FixedUpdate, move_players.run_if(game_active).run_if(resource_exists::<WorldRes>));
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
        let p = step(&m, Vec2::new(4.6, 2.5), Vec2::new(0.5, 0.2), false);
        assert!(p.x < 4.8 && (p.y - 2.7).abs() < 1e-4, "{p}");
        assert_eq!(step(&m, Vec2::new(4.6, 2.5), Vec2::new(1.0, 0.0), true).x, 5.6);
        assert!(!can_stand(&m, 0.1, 2.5));
    }
    #[test]
    fn facing() {
        assert_eq!(dir_of(Vec2::new(1.0, 0.2)), Dir::E);
        assert_eq!(dir_of(Vec2::new(0.0, -1.0)), Dir::N);
    }
    #[test]
    fn splitscreen_zooms_out_a_little() {
        assert_eq!(effective_zoom(3.0, 1), 3.0);
        assert!(effective_zoom(3.0, 2) < 3.0 && effective_zoom(3.0, 2) >= 1.5);
        assert_eq!(effective_zoom(1.0, 4), 1.5);
    }
}
