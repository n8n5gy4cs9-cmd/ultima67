//! Applies console/cheat `Effect`s and drives automated screenshots (`--screenshot out.png`).
use crate::app::{AppState, Cli, Game, Paths, WorldRes};
use crate::apply::Effect;
use crate::audio::SfxEvent;
use crate::combat::{Cursor, KillEvent};
use crate::creatures::{self, Creature};
use crate::db_res::DbRes;
use crate::magic::CastRequest;
use crate::render::{DirtyChunks, Sheets};
use crate::ui::{ConsoleRes, EffectEvent, Toast};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use u67_core::TilePos;
use u67_world::tiles;

#[derive(bevy::ecs::system::SystemParam)]
struct Writers<'w> {
    cast: EventWriter<'w, CastRequest>,
    sfx: EventWriter<'w, SfxEvent>,
    kills: EventWriter<'w, KillEvent>,
    exit: EventWriter<'w, AppExit>,
    ui: EventWriter<'w, crate::interact::UiRequest>,
    swap: ResMut<'w, crate::seats::SeatSwap>,
}

#[allow(clippy::too_many_arguments)]
fn handle_effects(
    mut ev: EventReader<EffectEvent>,
    mut game: ResMut<Game>,
    mut world: ResMut<WorldRes>,
    mut db: ResMut<DbRes>,
    paths: Res<Paths>,
    mut con: ResMut<ConsoleRes>,
    mut toast: ResMut<Toast>,
    mut w: Writers,
    mut commands: Commands,
    creatures_q: Query<(Entity, &Creature)>,
    sheets: Option<Res<Sheets>>,
    cursor: Res<Cursor>,
    mut dirty: ResMut<DirtyChunks>,
) {
    let mut queue: Vec<Effect> = ev.read().map(|e| e.0.clone()).collect::<Vec<_>>().into_iter().rev().collect();
    while let Some(e) = queue.pop() {
        let e = &e;
        let mut say = |s: String| {
            con.0.print(s.clone());
            *toast = Toast { text: s, timer: 2.5 };
        };
        let pp = Vec2::from(game.0.players[0].pos);
        let target = if cursor.valid { cursor.tile } else { pp + Vec2::new(2.0, 0.0) };
        match e {
            Effect::Quit => {
                w.exit.write(AppExit::Success);
            }
            Effect::Save(slot) => match crate::save::write(&paths.saves, *slot, &game.0) {
                Ok(p) => say(format!("saved: {}", p.display())),
                Err(err) => say(format!("save failed: {err}")),
            },
            Effect::Load(slot) => match crate::save::read(&paths.saves, *slot) {
                Ok(d) => {
                    world.0 = crate::app::reset_world(&paths, &d);
                    game.0 = d;
                    w.swap.0 = None;
                    say("loaded".into());
                }
                Err(err) => say(format!("load failed: {err}")),
            },
            Effect::Screenshot => {
                let dir = paths.saves.join("../screenshots");
                let _ = std::fs::create_dir_all(&dir);
                let path = dir.join(format!("shot_{}.png", game.0.clock.minutes));
                say(format!("screenshot -> {}", path.display()));
                commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            }
            Effect::Weather(w) => say(format!("weather set to {w} (visuals: TODO)")),
            Effect::Spawn { id, count } => {
                if let Some(sh) = &sheets {
                    for k in 0..*count {
                        let p = target + Vec2::new((k % 3) as f32 * 0.8, (k / 3) as f32 * 0.8);
                        creatures::spawn_by_id(&mut commands, sh, &db.0, id, p, false, None);
                    }
                    say(format!("spawned {count} x {id}"));
                }
            }
            Effect::KillAll => {
                let mut n = 0;
                for (en, c) in &creatures_q {
                    if !c.ally && c.pos.distance(pp) < 40.0 {
                        w.kills.write(KillEvent { entity: en, by_player: true, cannon: false });
                        n += 1;
                    }
                }
                say(format!("{n} creatures slain"));
            }
            Effect::SpawnCannon => {
                let t = TilePos::new(target.x.floor() as i32, target.y.floor() as i32);
                let map = game.0.current_map.clone();
                if let Some(m) = world.0.maps.get_mut(&map) {
                    if m.walkable(t) {
                        m.add_object("cannon", t);
                        game.0.placed.push((map.clone(), t.x, t.y, "cannon".into()));
                        dirty.0.insert(t.chunk());
                        say("cannon placed (E to load: 1 gunpowder + 1 cannon ball)".into());
                    } else {
                        say("cannot place a cannon there".into());
                    }
                }
            }
            Effect::SpawnShip(_) => {
                let map = game.0.current_map.clone();
                if let Some(m) = world.0.maps.get_mut(&map) {
                    let c = TilePos::new(pp.x as i32, pp.y as i32);
                    let mut found = None;
                    'o: for r in 1..12 {
                        for dy in -r..=r {
                            for dx in -r..=r {
                                let t = TilePos::new(c.x + dx, c.y + dy);
                                if tiles::def(m.tile(t)).water && !m.blocked_by_object(t) {
                                    found = Some(t);
                                    break 'o;
                                }
                            }
                        }
                    }
                    match found {
                        Some(t) => {
                            m.add_object("longship", t);
                            game.0.placed.push((map.clone(), t.x, t.y, "longship".into()));
                            dirty.0.insert(t.chunk());
                            say(format!("longship spawned at {},{}", t.x, t.y));
                        }
                        None => say("no water within 12 tiles".into()),
                    }
                }
            }
            Effect::Cast(id) => {
                w.cast.write(CastRequest(id.clone(), 0));
            }
            Effect::ReloadData => match u67_world::db::Db::load(&paths.assets) {
                Ok(d) => {
                    db.0 = d;
                    say("content reloaded".into());
                }
                Err(err) => say(format!("reload failed: {err}")),
            },
            Effect::ReloadAssets => say("assets reload on restart (hot reload: TODO T3.12)".into()),
            Effect::Splitscreen(n) => {
                let n = (*n as usize).clamp(1, crate::seats::MAX_SEATS);
                let start = game.0.players[0].pos;
                while game.0.players.len() < n {
                    let i = game.0.players.len();
                    game.0.players.push(crate::data::PlayerData::for_seat(i, [start[0] + i as f32 * 0.8, start[1]]));
                }
                game.0.players.truncate(n);
                game.0.map_dirty = true;
                say(format!("{n} local player{}. P1: WASD+mouse, P2: arrows (or a gamepad), P3/P4: gamepads.", if n == 1 { "" } else { "s" }));
            }
            Effect::Sfx(id) => {
                w.sfx.write(SfxEvent(id.clone()));
            }
            Effect::Toast(t) => *toast = Toast { text: t.clone(), timer: 3.0 },
            Effect::Sleep => say("You sleep until morning.".into()),
            Effect::Teleported => {}
            Effect::OpenShop(s) => {
                w.ui.write(crate::interact::UiRequest::Shop(s.clone()));
            }
            Effect::OpenCraft(s) => {
                w.ui.write(crate::interact::UiRequest::Craft(s.clone()));
            }
            Effect::Exec(file) => {
                let path = paths.config.join(file);
                match std::fs::read_to_string(&path) {
                    Ok(txt) => {
                        let reg = u67_console::Registry::standard();
                        for line in txt.lines() {
                            match reg.parse(line) {
                                Ok(Some(a)) => {
                                    let out = crate::apply::apply(&mut game.0, &world.0, a);
                                    for l in out.lines {
                                        con.0.print(l);
                                    }
                                    for f in out.effects {
                                        queue.push(f);
                                    }
                                }
                                Ok(None) => {}
                                Err(err) => con.0.print(format!("{line}: {err}")),
                            }
                        }
                    }
                    Err(err) => con.0.print(format!("exec {}: {err}", path.display())),
                }
            }
        }
    }
}

#[derive(Resource, Default)]
struct ShotState {
    frame: u32,
    taken: bool,
}

fn run_cli_cmds(cli: Res<Cli>, state: Res<State<AppState>>, mut game: ResMut<Game>, world: Res<WorldRes>, mut fx: EventWriter<EffectEvent>, mut done: Local<u32>, mut ui: EventWriter<crate::interact::UiRequest>, mut next: ResMut<NextState<AppState>>, npcs: Res<crate::npc::Npcs>) {
    if (cli.cmds.is_empty() && cli.ui.is_none()) || *state.get() != AppState::Playing {
        return;
    }
    *done += 1;
    if *done == 40 {
        use crate::interact::UiRequest as U;
        match cli.ui.as_deref() {
            Some("inventory") => next.set(AppState::Inventory),
            Some("journal") => {
                ui.write(U::Journal);
            }
            Some("cheats") => {
                ui.write(U::Cheats);
            }
            Some("map") => {
                ui.write(U::Map);
            }
            Some("travel") => {
                ui.write(U::Travel { by_ship: true });
            }
            Some("spells") => {
                ui.write(U::Spells);
            }
            Some("laulu") => {
                ui.write(U::Laulu);
            }
            Some("dialogue") => {
                let p = game.0.players[0].pos;
                if let Some(i) = crate::npc::npc_near(&npcs.list, p, 40.0) {
                    ui.write(U::Dialogue(i));
                }
            }
            Some(s) if s.starts_with("shop:") => {
                ui.write(U::Shop(s[5..].to_string()));
            }
            Some(s) if s.starts_with("craft:") => {
                ui.write(U::Craft(s[6..].to_string()));
            }
            _ => {}
        }
    }
    if *done != 8 {
        return;
    }
    let reg = u67_console::Registry::standard();
    for line in &cli.cmds {
        match reg.parse(line) {
            Ok(Some(a)) => {
                let out = crate::apply::apply(&mut game.0, &world.0, a);
                for l in out.lines {
                    info!("cmd> {l}");
                }
                for f in out.effects {
                    fx.write(EffectEvent(f));
                }
            }
            Ok(None) => {}
            Err(e) => warn!("cmd '{line}': {e}"),
        }
    }
}

fn auto_screenshot(cli: Res<Cli>, state: Res<State<AppState>>, mut st: ResMut<ShotState>, mut commands: Commands, mut exit: EventWriter<AppExit>) {
    let Some(path) = &cli.screenshot else { return };
    let in_ui = cli.ui.is_some() && !matches!(state.get(), AppState::Boot | AppState::MainMenu);
    if *state.get() != AppState::Playing && !in_ui {
        return;
    }
    st.frame += 1;
    if st.frame == cli.frames && !st.taken {
        st.taken = true;
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone()));
    }
    if st.frame > cli.frames + 20 {
        exit.write(AppExit::Success);
    }
}

fn ready(game: Option<Res<Game>>, db: Option<Res<DbRes>>) -> bool {
    game.is_some() && db.is_some()
}

#[derive(Component)]
struct ColliderViz;

fn draw_colliders(time: Res<Time>, game: Res<Game>, world: Res<WorldRes>, mut commands: Commands, existing: Query<Entity, With<ColliderViz>>, mut timer: Local<f32>) {
    *timer -= time.delta_secs();
    if *timer > 0.0 {
        return;
    }
    *timer = 0.25;
    for e in &existing {
        commands.entity(e).despawn();
    }
    if !game.0.cheats.debug_colliders {
        return;
    }
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    let p = game.0.players[0].pos;
    let (cx, cy) = (p[0] as i32, p[1] as i32);
    for dy in -16..=16 {
        for dx in -26..=26 {
            let t = TilePos::new(cx + dx, cy + dy);
            if map.in_bounds(t) && !map.walkable(t) {
                let px = crate::render::tile_px(t.x as f32, t.y as f32);
                commands.spawn((ColliderViz, Sprite { color: Color::srgba(1.0, 0.1, 0.1, 0.35), custom_size: Some(Vec2::splat(15.0)), anchor: bevy::sprite::Anchor::TopLeft, ..default() }, Transform::from_xyz(px.x, px.y, 50.0)));
            }
        }
    }
}

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShotState>().add_systems(Update, (handle_effects.run_if(ready), draw_colliders.run_if(ready), run_cli_cmds.run_if(ready), auto_screenshot));
    }
}
