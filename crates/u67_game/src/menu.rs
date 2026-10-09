//! Title menu, options, credits and the story intro.
use crate::app::{AppState, Game, HasQuicksave, Paths, SettingsRes, WorldRes};
use crate::data::{GameData, PlayerData};
use crate::player::Zoom;
use crate::seats::{MAX_SEATS};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};

#[derive(Resource)]
pub struct MenuSel {
    pub idx: usize,
    pub players: usize,
    pub opt: usize,
    pub slide: usize,
    pub return_to: AppState,
    pub msg: String,
}
impl Default for MenuSel {
    fn default() -> Self {
        Self { idx: 0, players: 1, opt: 0, slide: 0, return_to: AppState::MainMenu, msg: String::new() }
    }
}

const ITEMS: [&str; 6] = ["New Game", "Players", "Continue", "Options", "Credits", "Quit"];

#[derive(Component)]
struct MainRoot;
#[derive(Component)]
struct MainItem(usize);
#[derive(Component)]
struct OptRoot;
#[derive(Component)]
struct OptItem(usize);
#[derive(Component)]
struct CreditsRoot;
#[derive(Component)]
struct IntroRoot;
#[derive(Component)]
struct IntroText;

pub const SLIDES: [&str; 6] = [
    "The Nine Realms were never lost.\n\nThey were only... quiet.",
    "Yggdrasil's roots became the Bifrost: gates between worlds.\nFor a thousand years they slept beneath barley fields and fjords.",
    "Then came the satellites, the rifles, and Aesir Dynamics.\nSomeone pushed power through the old gates.",
    "In Kaupang, a gate began to hum.\nThe dead began to walk out of it.",
    "You are a Rune-Warden of Midgård, sworn to guard the gates,\narmed with a rune pistol and the oldest oaths.",
    "Wolves strain at Gleipnir. In the far north a witch stirs.\nBeyond the black, a singer waits with a kantele that shoots.\n\nGo.",
];

fn full_screen(bg: Color) -> (Node, BackgroundColor) {
    (Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::FlexEnd, align_items: AlignItems::Center, padding: UiRect::bottom(Val::Px(50.0)), row_gap: Val::Px(6.0), ..default() }, BackgroundColor(bg))
}

fn spawn_main(mut commands: Commands, assets: Res<AssetServer>, quick: Option<Res<HasQuicksave>>) {
    let _ = quick;
    commands
        .spawn((MainRoot, full_screen(Color::BLACK), ImageNode::new(assets.load("gfx/title.png"))))
        .with_children(|p| {
            p.spawn((Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::axes(Val::Px(40.0), Val::Px(10.0)), ..default() }, BackgroundColor(Color::srgba(0.02, 0.02, 0.08, 0.62)))).with_children(|col| {
                for (i, _) in ITEMS.iter().enumerate() {
                    col.spawn((Button, MainItem(i), Node { padding: UiRect::axes(Val::Px(18.0), Val::Px(3.0)), ..default() }, BackgroundColor(Color::NONE))).with_children(|b| {
                        b.spawn((Text::new(""), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE)));
                    });
                }
            });
            p.spawn((Text::new(u67_core::CREDIT), TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(0.75, 0.75, 0.8)), Node { margin: UiRect::top(Val::Px(10.0)), ..default() }));
            p.spawn((Text::new("Up/Down: choose   Enter: select   Left/Right: players"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgb(0.55, 0.55, 0.6))));
        });
}

/// Replace the world and game state with a fresh new game for `players` seats.
pub fn start_new_game(game: &mut Game, world: &mut WorldRes, paths: &Paths, players: usize, seed: u64) {
    let fresh = GameData::new(seed, "midgard", [0.0, 0.0]);
    world.0 = crate::app::reset_world(paths, &fresh);
    let start = world.0.maps["midgard"].places.get("start").copied().unwrap_or_default();
    let pos = [start.x as f32 + 0.5, start.y as f32 + 0.5];
    let mut g = GameData::new(seed, "midgard", pos);
    for i in 1..players.clamp(1, MAX_SEATS) {
        g.players.push(PlayerData::for_seat(i, [pos[0] + i as f32 * 0.8, pos[1]]));
    }
    game.0 = g;
}

#[allow(clippy::too_many_arguments)]
fn main_menu(
    kb: Res<ButtonInput<KeyCode>>,
    items: Query<(&Interaction, &MainItem, &Children)>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
    mut sel: ResMut<MenuSel>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    paths: Res<Paths>,
    mut game: ResMut<Game>,
    mut world: ResMut<WorldRes>,
    mut toast: ResMut<crate::ui::Toast>,
    time: Res<Time>,
) {
    let mut activate = false;
    for (i, it, _) in &items {
        match *i {
            Interaction::Hovered => sel.idx = it.0,
            Interaction::Pressed => {
                sel.idx = it.0;
                activate = true;
            }
            Interaction::None => {}
        }
    }
    if kb.just_pressed(KeyCode::ArrowDown) {
        sel.idx = (sel.idx + 1) % ITEMS.len();
    }
    if kb.just_pressed(KeyCode::ArrowUp) {
        sel.idx = (sel.idx + ITEMS.len() - 1) % ITEMS.len();
    }
    if sel.idx == 1 {
        if kb.just_pressed(KeyCode::ArrowRight) || kb.just_pressed(KeyCode::Space) {
            sel.players = sel.players % MAX_SEATS + 1;
        }
        if kb.just_pressed(KeyCode::ArrowLeft) {
            sel.players = (sel.players + MAX_SEATS - 2) % MAX_SEATS + 1;
        }
    }
    if kb.just_pressed(KeyCode::Enter) || kb.just_pressed(KeyCode::NumpadEnter) {
        activate = true;
    }
    if kb.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
    let blink = ((time.elapsed_secs() * 3.0) as u32).is_multiple_of(2);
    for (_, it, children) in &items {
        for ch in children.iter() {
            if let Ok((mut t, mut c)) = texts.get_mut(ch) {
                let selected = sel.idx == it.0;
                let label = if it.0 == 1 { format!("Players: < {} >{}", sel.players, if sel.players > 1 { "  (split-screen co-op)" } else { "" }) } else { ITEMS[it.0].to_string() };
                t.0 = if selected { format!("> {label} <") } else { format!("  {label}  ") };
                c.0 = if selected { if blink { Color::srgb(1.0, 0.85, 0.4) } else { Color::srgb(1.0, 0.95, 0.7) } } else { Color::srgb(0.85, 0.85, 0.9) };
            }
        }
    }
    if !activate {
        return;
    }
    match sel.idx {
        0 => {
            let n = sel.players;
            start_new_game(&mut game, &mut world, &paths, n, u67_mapgen::DEFAULT_SEED);
            sel.slide = 0;
            next.set(AppState::Intro);
        }
        1 => sel.players = sel.players % MAX_SEATS + 1,
        2 => match crate::save::read(&paths.saves, None) {
            Ok(d) => {
                world.0 = crate::app::reset_world(&paths, &d);
                game.0 = d;
                next.set(AppState::Playing);
            }
            Err(e) => *toast = crate::ui::Toast { text: format!("No quicksave yet ({e})"), timer: 3.0 },
        },
        3 => {
            sel.return_to = AppState::MainMenu;
            sel.opt = 0;
            next.set(AppState::Options);
        }
        4 => next.set(AppState::Credits),
        _ => {
            exit.write(AppExit::Success);
        }
    }
}

// ---------------------------------------------------------------- options
pub const OPT_LABELS: [&str; 9] = ["Master volume", "Music volume", "Effects volume", "Zoom", "Fullscreen", "Text size", "Hot-reload assets", "Colour-blind safe bars", "Back"];

pub fn opt_value(s: &crate::settings::Settings, i: usize) -> String {
    match i {
        0 => format!("{:.0}%", s.master_volume * 100.0),
        1 => format!("{:.0}%", s.music_volume * 100.0),
        2 => format!("{:.0}%", s.sfx_volume * 100.0),
        3 => format!("{:.0}x", s.zoom),
        4 => if s.fullscreen { "on" } else { "off" }.into(),
        5 => format!("{:.0}%", s.text_scale * 100.0),
        6 => if s.hot_reload { "on" } else { "off" }.into(),
        7 => if s.colorblind { "on" } else { "off" }.into(),
        _ => String::new(),
    }
}

/// Change option `i` by `dir` (-1/+1). Returns true if something changed.
pub fn opt_change(s: &mut crate::settings::Settings, i: usize, dir: i32) -> bool {
    let d = dir as f32;
    let step = |v: &mut f32, s: f32, lo: f32, hi: f32| {
        let n = (*v + s * d).clamp(lo, hi);
        let ch = (n - *v).abs() > 1e-4;
        *v = n;
        ch
    };
    match i {
        0 => step(&mut s.master_volume, 0.1, 0.0, 1.0),
        1 => step(&mut s.music_volume, 0.1, 0.0, 1.0),
        2 => step(&mut s.sfx_volume, 0.1, 0.0, 1.0),
        3 => step(&mut s.zoom, 1.0, 1.0, 8.0),
        4 => {
            s.fullscreen = !s.fullscreen;
            true
        }
        5 => step(&mut s.text_scale, 0.1, 0.8, 1.6),
        6 => {
            s.hot_reload = !s.hot_reload;
            true
        }
        7 => {
            s.colorblind = !s.colorblind;
            true
        }
        _ => false,
    }
}

fn spawn_options(mut commands: Commands) {
    commands.spawn((OptRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: Val::Px(8.0), ..default() }, BackgroundColor(Color::srgba(0.03, 0.02, 0.06, 0.97)))).with_children(|p| {
        p.spawn((Text::new("OPTIONS"), TextFont { font_size: 48.0, ..default() }, TextColor(Color::srgb(0.95, 0.78, 0.35))));
        for i in 0..OPT_LABELS.len() {
            p.spawn((Button, OptItem(i), Node { padding: UiRect::axes(Val::Px(18.0), Val::Px(3.0)), ..default() })).with_children(|b| {
                b.spawn((Text::new(""), TextFont { font_size: 24.0, ..default() }));
            });
        }
        p.spawn((Text::new("Up/Down choose - Left/Right change - Esc back.   Keys: edit settings.ron (rebind keys & gamepad buttons)"), TextFont { font_size: 15.0, ..default() }, TextColor(Color::srgb(0.6, 0.6, 0.65))));
    });
}

#[allow(clippy::too_many_arguments)]
fn options_input(
    kb: Res<ButtonInput<KeyCode>>,
    items: Query<(&Interaction, &OptItem, &Children)>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
    mut sel: ResMut<MenuSel>,
    mut settings: ResMut<SettingsRes>,
    mut next: ResMut<NextState<AppState>>,
    paths: Res<Paths>,
    mut zoom: ResMut<Zoom>,
    mut ui_scale: ResMut<UiScale>,
    mut windows: Query<&mut Window>,
) {
    let n = OPT_LABELS.len();
    let mut dir = 0;
    let mut enter = false;
    for (i, it, _) in &items {
        match *i {
            Interaction::Hovered => sel.opt = it.0,
            Interaction::Pressed => {
                sel.opt = it.0;
                enter = true;
                dir = 1;
            }
            Interaction::None => {}
        }
    }
    if kb.just_pressed(KeyCode::ArrowDown) {
        sel.opt = (sel.opt + 1) % n;
    }
    if kb.just_pressed(KeyCode::ArrowUp) {
        sel.opt = (sel.opt + n - 1) % n;
    }
    if kb.just_pressed(KeyCode::ArrowRight) {
        dir = 1;
    }
    if kb.just_pressed(KeyCode::ArrowLeft) {
        dir = -1;
    }
    if kb.just_pressed(KeyCode::Enter) {
        enter = true;
        dir = 1;
    }
    let back = kb.just_pressed(KeyCode::Escape) || (enter && sel.opt == n - 1);
    if dir != 0 && sel.opt < n - 1 && opt_change(&mut settings.0, sel.opt, dir) {
        zoom.0 = settings.0.zoom;
        ui_scale.0 = settings.0.text_scale;
        if let Ok(mut w) = windows.single_mut() {
            w.mode = if settings.0.fullscreen { WindowMode::BorderlessFullscreen(MonitorSelection::Primary) } else { WindowMode::Windowed };
        }
    }
    for (_, it, children) in &items {
        for ch in children.iter() {
            if let Ok((mut t, mut c)) = texts.get_mut(ch) {
                let v = opt_value(&settings.0, it.0);
                let selected = sel.opt == it.0;
                t.0 = if v.is_empty() { format!("{}{}", if selected { "> " } else { "  " }, OPT_LABELS[it.0]) } else { format!("{}{:<24} < {} >", if selected { "> " } else { "  " }, OPT_LABELS[it.0], v) };
                c.0 = if selected { Color::srgb(1.0, 0.9, 0.5) } else { Color::srgb(0.85, 0.85, 0.9) };
            }
        }
    }
    if back {
        let _ = settings.0.save(&paths.config);
        next.set(sel.return_to);
    }
}

// ---------------------------------------------------------------- credits
fn spawn_credits(mut commands: Commands) {
    commands.spawn((CreditsRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: Val::Px(10.0), ..default() }, BackgroundColor(Color::srgba(0.03, 0.02, 0.06, 0.97)))).with_children(|p| {
        p.spawn((Text::new("ULTIMA 67"), TextFont { font_size: 56.0, ..default() }, TextColor(Color::srgb(0.95, 0.78, 0.35))));
        p.spawn((Text::new(u67_core::CREDIT), TextFont { font_size: 26.0, ..default() }));
        p.spawn((Text::new("Inspired by Ultima VII and Ultima VI (Origin Systems / Richard Garriott) and Nox (Westwood).\nEngine ideas and techniques learned from the Exult, Nuvie and OpenNox projects (GPL).\nBuilt with Rust and the Bevy engine. UI font: DejaVu Sans.\nAll sprites, sound effects, music and maps in this repository are generated placeholders.\n\nKalevala and the Poetic Edda belong to everyone.\n\n[Esc] back"), TextFont { font_size: 19.0, ..default() }, TextColor(Color::srgb(0.8, 0.8, 0.85)), TextLayout::new_with_justify(JustifyText::Center)));
    });
}

fn credits_input(kb: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, mut next: ResMut<NextState<AppState>>) {
    if kb.just_pressed(KeyCode::Escape) || kb.just_pressed(KeyCode::Enter) || mouse.just_pressed(MouseButton::Left) {
        next.set(AppState::MainMenu);
    }
}

// ---------------------------------------------------------------- intro
fn spawn_intro(mut commands: Commands, assets: Res<AssetServer>, mut sel: ResMut<MenuSel>) {
    sel.slide = 0;
    commands.spawn((IntroRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::BLACK), ImageNode::new(assets.load("gfx/title.png")).with_color(Color::srgba(0.35, 0.35, 0.45, 1.0)))).with_children(|p| {
        p.spawn((IntroText, Text::new(SLIDES[0]), TextFont { font_size: 34.0, ..default() }, TextColor(Color::srgb(0.95, 0.92, 0.8)), TextLayout::new_with_justify(JustifyText::Center)));
        p.spawn((Text::new("[Space] continue    [Esc] skip"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(0.6, 0.6, 0.65)), Node { position_type: PositionType::Absolute, bottom: Val::Px(20.0), ..default() }));
    });
}

fn intro_input(kb: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, mut sel: ResMut<MenuSel>, mut next: ResMut<NextState<AppState>>, mut q: Query<&mut Text, With<IntroText>>) {
    if kb.just_pressed(KeyCode::Escape) {
        next.set(AppState::Playing);
        return;
    }
    if kb.just_pressed(KeyCode::Space) || kb.just_pressed(KeyCode::Enter) || mouse.just_pressed(MouseButton::Left) {
        sel.slide += 1;
        if sel.slide >= SLIDES.len() {
            next.set(AppState::Playing);
            return;
        }
    }
    if let Ok(mut t) = q.single_mut() {
        t.0 = SLIDES[sel.slide.min(SLIDES.len() - 1)].to_string();
    }
}

fn despawn_all<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn apply_settings(settings: Res<SettingsRes>, mut ui_scale: ResMut<UiScale>) {
    ui_scale.0 = settings.0.text_scale;
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        use AppState::*;
        app.init_resource::<MenuSel>()
            .add_systems(Startup, apply_settings)
            .add_systems(OnEnter(MainMenu), spawn_main)
            .add_systems(OnExit(MainMenu), despawn_all::<MainRoot>)
            .add_systems(Update, main_menu.run_if(in_state(MainMenu)).run_if(resource_exists::<Game>))
            .add_systems(OnEnter(Options), spawn_options)
            .add_systems(OnExit(Options), despawn_all::<OptRoot>)
            .add_systems(Update, options_input.run_if(in_state(Options)))
            .add_systems(OnEnter(Credits), spawn_credits)
            .add_systems(OnExit(Credits), despawn_all::<CreditsRoot>)
            .add_systems(Update, credits_input.run_if(in_state(Credits)))
            .add_systems(OnEnter(Intro), spawn_intro)
            .add_systems(OnExit(Intro), despawn_all::<IntroRoot>)
            .add_systems(Update, intro_input.run_if(in_state(Intro)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn option_changes_clamp_and_toggle() {
        let mut s = Settings::default();
        assert!(opt_change(&mut s, 0, -1));
        assert!((s.master_volume - 0.7).abs() < 1e-5);
        s.master_volume = 1.0;
        assert!(!opt_change(&mut s, 0, 1), "already at max");
        assert!(opt_change(&mut s, 4, 1) && s.fullscreen);
        s.zoom = 8.0;
        assert!(!opt_change(&mut s, 3, 1));
        assert!(opt_change(&mut s, 5, 1) && (s.text_scale - 1.1).abs() < 1e-5);
        assert!(opt_change(&mut s, 7, 1) && s.colorblind);
        assert_eq!(opt_value(&s, 3), "8x");
        assert!(!opt_change(&mut s, 8, 1));
    }

    #[test]
    fn story_has_slides_and_new_game_seats() {
        assert_eq!(SLIDES.len(), 6);
        let paths = Paths { assets: crate::app::asset_root(), saves: std::env::temp_dir(), config: std::env::temp_dir() };
        let mut game = Game(GameData::new(1, "midgard", [0.0, 0.0]));
        let mut world = WorldRes(u67_world::map::World::default());
        start_new_game(&mut game, &mut world, &paths, 3, 67);
        assert_eq!(game.0.players.len(), 3);
        assert!(world.0.maps.contains_key("midgard"));
        assert_eq!(game.0.current_map, "midgard");
    }
}
