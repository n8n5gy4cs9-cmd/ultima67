//! UI: main menu, HUD, pause menu, console overlay, inventory placeholder, toasts.
use crate::app::{AppState, Cli, Game, HasQuicksave, Paths, WorldRes};
use crate::apply;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use u67_console::Console;

#[derive(Resource, Default)]
pub struct ConsoleRes(pub Console);

#[derive(Resource, Default)]
pub struct Toast {
    pub text: String,
    pub timer: f32,
}

#[derive(Event, Clone, Debug)]
pub struct EffectEvent(pub apply::Effect);

#[derive(Component)]
struct Hud;
#[derive(Component)]
struct ToastText;
#[derive(Component)]
struct PauseRoot;
#[derive(Component)]
struct ConsoleRoot;
#[derive(Component)]
struct ConsoleText;

fn text_node(left: f32, top: f32) -> Node {
    Node { position_type: PositionType::Absolute, left: Val::Px(left), top: Val::Px(top), ..default() }
}

fn overlay(alpha: f32) -> (Node, BackgroundColor) {
    (Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: Val::Px(12.0), ..default() }, BackgroundColor(Color::srgba(0.03, 0.02, 0.05, alpha)))
}

fn despawn_all<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn spawn_hud(mut commands: Commands, existing: Query<(), With<Hud>>) {
    if existing.iter().next().is_some() {
        return;
    }
    commands.spawn((Hud, Text::new(""), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(1.0, 0.95, 0.8)), text_node(10.0, 8.0)));
    commands.spawn((ToastText, Text::new(""), TextFont { font_size: 26.0, ..default() }, TextColor(Color::srgb(0.6, 1.0, 0.8)), text_node(10.0, 60.0)));
}

fn update_hud(game: Res<Game>, diag: Res<bevy::diagnostic::DiagnosticsStore>, mut hud: Query<&mut Text, (With<Hud>, Without<ToastText>)>, mut toast_q: Query<&mut Text, (With<ToastText>, Without<Hud>)>, mut toast: ResMut<Toast>, time: Res<Time>) {
    let g = &game.0;
    if g.players.len() > 1 {
        if let Ok(mut t) = hud.single_mut() {
            t.0 = format!("{}  {:02}:{:02}  day {}   |  SPLIT-SCREEN x{}: P1 WASD+mouse | P2 arrows . / , ; ' | pads: stick, RT, A interact, B roll, X reload, Y inventory", g.current_map, g.clock.hour(), g.clock.minute(), g.clock.day() + 1, g.players.len());
        }
        return;
    }
    let p = &g.players[0];
    let fps = diag.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS).and_then(|d| d.smoothed()).unwrap_or(0.0);
    let mut s = format!("{}  {:02}:{:02}  day {}  HP {}/{}  Lv {}", g.current_map, g.clock.hour(), g.clock.minute(), g.clock.day() + 1, p.stats.hp, p.stats.max_hp(), p.stats.level);
    if g.cheats.fps {
        s += &format!("  {fps:.0} fps  @{:.1},{:.1}", p.pos[0], p.pos[1]);
    }
    let on: Vec<_> = u67_console::Cheat::ALL.iter().filter(|c| g.cheats.get(**c) && !matches!(c, u67_console::Cheat::Fps)).map(|c| c.command()).collect();
    if !on.is_empty() {
        s += &format!("  [cheats: {}]", on.join(","));
    }
    if let Ok(mut t) = hud.single_mut() {
        t.0 = s;
    }
    toast.timer -= time.delta_secs();
    if let Ok(mut t) = toast_q.single_mut() {
        t.0 = if toast.timer > 0.0 { toast.text.clone() } else { String::new() };
    }
}

fn spawn_pause(mut commands: Commands) {
    commands.spawn((PauseRoot, overlay(0.55))).with_children(|p| {
        p.spawn((Text::new("PAUSED"), TextFont { font_size: 56.0, ..default() }));
        p.spawn((Text::new("[Esc] resume   [F5] quick save   [F9] quick load   [O] options   [M] main menu   [Q] quit"), TextFont { font_size: 22.0, ..default() }));
    });
}

fn pause_input(kb: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>, mut next: ResMut<NextState<AppState>>, mut sel: ResMut<crate::menu::MenuSel>) {
    if kb.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
    if kb.just_pressed(KeyCode::KeyO) {
        sel.return_to = AppState::Paused;
        sel.opt = 0;
        next.set(AppState::Options);
    }
    if kb.just_pressed(KeyCode::KeyM) {
        sel.idx = 0;
        next.set(AppState::MainMenu);
    }
}

fn spawn_console(mut commands: Commands) {
    commands.spawn((ConsoleRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(46.0), padding: UiRect::all(Val::Px(10.0)), ..default() }, BackgroundColor(Color::srgba(0.02, 0.03, 0.08, 0.88)))).with_children(|p| {
        p.spawn((ConsoleText, Text::new(""), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(0.75, 1.0, 0.85))));
    });
}

fn console_input(mut ev: EventReader<KeyboardInput>, mut con: ResMut<ConsoleRes>, mut game: ResMut<Game>, world: Res<WorldRes>, mut fx: EventWriter<EffectEvent>) {
    for e in ev.read() {
        if !e.state.is_pressed() {
            continue;
        }
        match &e.logical_key {
            Key::Character(s) => {
                if !s.contains('`') && !s.contains('§') {
                    con.0.input.push_str(s);
                }
            }
            Key::Space => con.0.input.push(' '),
            Key::Backspace => {
                con.0.input.pop();
            }
            Key::Tab => con.0.tab(),
            Key::ArrowUp => con.0.history_prev(),
            Key::ArrowDown => con.0.history_next(),
            Key::Enter => {
                if let Some(action) = con.0.submit() {
                    let out = apply::apply(&mut game.0, &world.0, action);
                    for l in out.lines {
                        con.0.print(l);
                    }
                    for f in out.effects {
                        fx.write(EffectEvent(f));
                    }
                }
            }
            _ => {}
        }
    }
}

fn console_view(con: Res<ConsoleRes>, time: Res<Time>, mut q: Query<&mut Text, With<ConsoleText>>) {
    let Ok(mut t) = q.single_mut() else { return };
    let n = con.0.scrollback.len();
    let mut s = con.0.scrollback[n.saturating_sub(15)..].join("\n");
    let cursor = if (time.elapsed_secs() * 2.0) as u32 % 2 == 0 { "_" } else { " " };
    s += &format!("\n> {}{cursor}", con.0.input);
    t.0 = s;
}

fn quick_keys(kb: Res<ButtonInput<KeyCode>>, keys: Res<crate::input::KeyMap>, mut fx: EventWriter<EffectEvent>) {
    use crate::settings::Action;
    if keys.just_pressed(Action::QuickSave, &kb) {
        fx.write(EffectEvent(apply::Effect::Save(None)));
    }
    if keys.just_pressed(Action::QuickLoad, &kb) {
        fx.write(EffectEvent(apply::Effect::Load(None)));
    }
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        use AppState::*;
        app.init_resource::<ConsoleRes>()
            .init_resource::<Toast>()
            .add_event::<EffectEvent>()
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            .add_systems(OnEnter(Playing), spawn_hud)
            .add_systems(Update, update_hud.run_if(resource_exists::<Game>).run_if(not(in_state(Boot))).run_if(not(in_state(MainMenu))))
            .add_systems(OnEnter(Paused), spawn_pause)
            .add_systems(OnExit(Paused), despawn_all::<PauseRoot>)
            .add_systems(Update, (pause_input, quick_keys).run_if(in_state(Paused)))
            .add_systems(Update, quick_keys.run_if(in_state(Playing)))
            .add_systems(OnEnter(Console), spawn_console)
            .add_systems(OnExit(Console), despawn_all::<ConsoleRoot>)
            .add_systems(Update, (console_input, console_view).chain().run_if(in_state(Console)))
;
        let _ = Cli::default;
    }
}
