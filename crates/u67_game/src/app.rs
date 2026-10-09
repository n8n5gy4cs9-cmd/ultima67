//! App plumbing: resources, states, startup/boot, top-level plugin.
use crate::data::GameData;
use crate::input::{self, KeyMap};
use crate::settings::{Action, Settings};
use bevy::prelude::*;
use std::path::PathBuf;
use u67_core::platform;
use u67_world::map::World;

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum AppState {
    #[default]
    Boot,
    MainMenu,
    Playing,
    Paused,
    Inventory,
    Dialogue,
    Console,
    Menu,
    Dead,
    Options,
    Credits,
    Intro,
}

#[derive(Resource)]
pub struct SettingsRes(pub Settings);
#[derive(Resource)]
pub struct WorldRes(pub World);
#[derive(Resource)]
pub struct Game(pub GameData);
#[derive(Resource, Clone)]
pub struct Paths {
    pub assets: PathBuf,
    pub saves: PathBuf,
    pub config: PathBuf,
}

/// Command-line options (also used for automated screenshots/tests).
#[derive(Resource, Default, Clone, Debug)]
pub struct Cli {
    pub skip_menu: bool,
    pub screenshot: Option<String>,
    pub tp: Option<String>,
    pub time: Option<(u32, u32)>,
    pub frames: u32,
    pub zoom: Option<f32>,
    /// console commands (separated by ';') to run once the game starts
    pub cmds: Vec<String>,
    /// open a screen at startup for visual checks: inventory|journal|cheats|map|travel|spells|laulu|dialogue|shop:<id>|craft:<station>|chest
    pub ui: Option<String>,
}

impl Cli {
    pub fn parse(args: impl Iterator<Item = String>) -> Self {
        let mut c = Cli { frames: 90, ..Default::default() };
        let a: Vec<String> = args.collect();
        let mut i = 0;
        while i < a.len() {
            let next = a.get(i + 1).cloned();
            match a[i].as_str() {
                "--skip-menu" => c.skip_menu = true,
                "--screenshot" => {
                    c.screenshot = next;
                    c.skip_menu = true;
                    i += 1;
                }
                "--tp" => {
                    c.tp = next;
                    i += 1;
                }
                "--time" => {
                    c.time = next.and_then(|t| t.split_once(':').and_then(|(h, m)| Some((h.parse().ok()?, m.parse().ok()?))));
                    i += 1;
                }
                "--frames" => {
                    c.frames = next.and_then(|n| n.parse().ok()).unwrap_or(90);
                    i += 1;
                }
                "--ui" => {
                    c.ui = next;
                    i += 1;
                }
                "--cmd" => {
                    c.cmds = next.map(|s| s.split(';').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()).unwrap_or_default();
                    i += 1;
                }
                "--zoom" => {
                    c.zoom = next.and_then(|n| n.parse().ok());
                    i += 1;
                }
                _ => {}
            }
            i += 1;
        }
        c
    }
}

pub fn asset_root() -> PathBuf {
    if let Ok(p) = std::env::var("U67_ASSETS") {
        return p.into();
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            for cand in [d.join("assets"), d.join("../Resources/assets")] {
                if cand.join("manifest.json").exists() {
                    return cand;
                }
            }
        }
    }
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    if dev.join("manifest.json").exists() {
        return dev.canonicalize().unwrap_or(dev);
    }
    "assets".into()
}

/// The generated world, with any `.u67map` files in `<assets>/maps` laid over it (hand-edited or
/// imported maps replace the generated map of the same name).
pub fn build_world(assets: &std::path::Path, seed: u64) -> World {
    let mut w = u67_mapgen::generate_world(seed);
    let dir = assets.join("maps");
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            if e.path().extension().is_some_and(|x| x == "u67map") {
                match std::fs::read(e.path()).map_err(|e| e.to_string()).and_then(|b| u67_world::mapio::from_bytes(&b)) {
                    Ok(m) => {
                        info!("loaded map override {}", m.name);
                        w.insert(m);
                    }
                    Err(err) => warn!("bad map {}: {err}", e.path().display()),
                }
            }
        }
    }
    w
}

#[derive(Component)]
struct BootText;

#[derive(Resource, Default)]
struct BootFrames(u32);

/// Fresh world from the seed with all of `data`'s saved changes applied (used after loading a save).
pub fn reset_world(paths: &Paths, data: &GameData) -> World {
    let mut w = build_world(&paths.assets, data.seed);
    crate::persist::apply_overrides(&mut w, data);
    w
}

fn boot_start(mut commands: Commands) {
    commands.spawn((Camera2d, crate::player::UiCamera, IsDefaultUiCamera, Camera { order: 100, ..default() }));
    commands.spawn((
        BootText,
        Text::new("Generating Midgård..."),
        TextFont { font_size: 32.0, ..default() },
        Node { position_type: PositionType::Absolute, left: Val::Px(40.0), top: Val::Px(40.0), ..default() },
    ));
}

fn boot_work(
    mut frames: ResMut<BootFrames>,
    mut commands: Commands,
    paths: Res<Paths>,
    cli: Res<Cli>,
    mut next: ResMut<NextState<AppState>>,
    texts: Query<Entity, With<BootText>>,
) {
    frames.0 += 1;
    if frames.0 < 3 {
        return; // let the loading text render first
    }
    let seed = u67_mapgen::DEFAULT_SEED;
    let world = build_world(&paths.assets, seed);
    let start = world.maps["midgard"].places.get("start").copied().unwrap_or_default();
    let mut data = GameData::new(seed, "midgard", [start.x as f32 + 0.5, start.y as f32 + 0.5]);
    if let Ok(saved) = crate::save::read(&paths.saves, None) {
        if !cli.skip_menu {
            commands.insert_resource(HasQuicksave(true));
            let _ = saved;
        }
    }
    if let Some(t) = &cli.tp {
        if let Some((m, p)) = world.find_place(t) {
            data.current_map = m.to_string();
            data.players[0].pos = [p.x as f32 + 0.5, p.y as f32 + 0.5];
        }
    }
    if let Some((h, m)) = cli.time {
        data.clock.set_hm(h, m);
    }
    let mut world = world;
    crate::persist::apply_overrides(&mut world, &data);
    match u67_world::db::Db::load(&paths.assets) {
        Ok(db) => {
            let problems = db.validate(&|p| world.find_place(p).is_some());
            for p in &problems {
                warn!("content: {p}");
            }
            commands.insert_resource(crate::db_res::DbRes(db));
        }
        Err(e) => panic!("content database failed to load: {e}"),
    }
    commands.insert_resource(WorldRes(world));
    commands.insert_resource(Game(data));
    for e in &texts {
        commands.entity(e).despawn();
    }
    next.set(if cli.skip_menu { AppState::Playing } else { AppState::MainMenu });
}

#[derive(Resource)]
pub struct HasQuicksave(pub bool);

fn state_hotkeys(
    kb: Res<ButtonInput<KeyCode>>,
    keys: Res<KeyMap>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
    intents: Res<crate::seats::Intents>,
    mut active: ResMut<crate::seats::ActiveSeat>,
    mut swap: ResMut<crate::seats::SeatSwap>,
    game: Option<ResMut<Game>>,
) {
    use AppState::*;
    let s = *state.get();
    let pause = intents.list.iter().any(|i| i.pause) || keys.just_pressed(Action::Pause, &kb);
    let inv_seat = intents.list.iter().position(|i| i.inventory);
    let inv = inv_seat.is_some();
    let con = keys.just_pressed(Action::Console, &kb);
    if s == Playing {
        if let (Some(seat), Some(mut g)) = (inv_seat, game) {
            active.0 = seat;
            swap.swap_in(&mut g.0, seat);
        }
    }
    match s {
        Playing => {
            if pause {
                next.set(Paused);
            } else if inv {
                next.set(Inventory);
            } else if con {
                next.set(Console);
            }
        }
        Paused if pause => next.set(Playing),
        Inventory if inv || pause => next.set(Playing),
        Console if pause || con => next.set(Playing),
        Dialogue if pause => next.set(Playing),
        Menu if pause => next.set(Playing),
        _ => {}
    }
}

pub struct U67Plugin;

impl Plugin for U67Plugin {
    fn build(&self, app: &mut App) {
        // Latin-Extended capable default font (Väinämöinen, Jörmungandr, runes...)
        if let Some(mut fonts) = app.world_mut().get_resource_mut::<Assets<Font>>() {
            if let Ok(f) = Font::try_from_bytes(include_bytes!("../../../assets/fonts/UI.ttf").to_vec()) {
                fonts.insert(&Handle::<Font>::default(), f);
            }
        }
        let paths = app.world().resource::<Paths>().clone();
        let settings = Settings::load_or_create(&paths.config);
        app.insert_resource(KeyMap::from_settings(&settings))
            .insert_resource(SettingsRes(settings))
            .init_resource::<BootFrames>()
            .insert_resource(Time::<Fixed>::from_hz(30.0))
            .init_state::<AppState>()
            .add_systems(OnEnter(AppState::Boot), boot_start)
            .add_systems(Update, boot_work.run_if(in_state(AppState::Boot)))
            .add_systems(Update, (input::read_intent, state_hotkeys).run_if(not(in_state(AppState::Boot))))
            .add_plugins((
                crate::render::RenderPlugin,
                crate::player::PlayerPlugin,
                crate::ui::UiPlugin,
                crate::debug::DebugPlugin,
                crate::npc::NpcPlugin,
                crate::creatures::CreaturePlugin,
                crate::combat::CombatPlugin,
                crate::interact::InteractPlugin,
                crate::magic::MagicPlugin,
                crate::audio::AudioPlugin,
                crate::gui::GuiPlugin,
                crate::hazards::HazardPlugin,
                crate::weather::WeatherPlugin,
                crate::menu::MenuPlugin,
            ));
    }
}

pub fn run() {
    let assets = asset_root();
    let paths = Paths { assets: assets.clone(), saves: platform::save_dir(), config: platform::config_dir() };
    let cli = Cli::parse(std::env::args().skip(1));
    let settings_probe = Settings::load_or_create(&paths.config);
    let mut app = App::new();
    app.insert_resource(paths)
        .insert_resource(cli)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin { file_path: assets.to_string_lossy().into_owned(), watch_for_changes_override: Some(settings_probe.hot_reload), ..default() })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("{} - {}", u67_core::GAME_NAME, u67_core::CREDIT),
                        resolution: (1280.0_f32, 720.0_f32).into(),
                        present_mode: if settings_probe.vsync { bevy::window::PresentMode::AutoVsync } else { bevy::window::PresentMode::AutoNoVsync },
                        mode: if settings_probe.fullscreen {
                            bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                        } else {
                            bevy::window::WindowMode::Windowed
                        },
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(U67Plugin)
        .run();
}
