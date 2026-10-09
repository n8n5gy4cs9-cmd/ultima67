//! Applies console/cheat `Effect`s and drives automated screenshots (`--screenshot out.png`).
use crate::app::{AppState, Cli, Game, Paths};
use crate::apply::Effect;
use crate::ui::{ConsoleRes, EffectEvent, Toast};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

fn handle_effects(mut ev: EventReader<EffectEvent>, mut game: ResMut<Game>, paths: Res<Paths>, mut con: ResMut<ConsoleRes>, mut toast: ResMut<Toast>, mut exit: EventWriter<AppExit>, mut commands: Commands) {
    for EffectEvent(e) in ev.read() {
        let mut say = |s: String| {
            con.0.print(s.clone());
            *toast = Toast { text: s, timer: 2.5 };
        };
        match e {
            Effect::Quit => {
                exit.write(AppExit::Success);
            }
            Effect::Save(slot) => match crate::save::write(&paths.saves, *slot, &game.0) {
                Ok(p) => say(format!("saved: {}", p.display())),
                Err(err) => say(format!("save failed: {err}")),
            },
            Effect::Load(slot) => match crate::save::read(&paths.saves, *slot) {
                Ok(d) => {
                    game.0 = d;
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
            other => say(format!("{other:?}: not implemented yet")),
        }
    }
}

#[derive(Resource, Default)]
struct ShotState {
    frame: u32,
    taken: bool,
}

fn auto_screenshot(cli: Res<Cli>, state: Res<State<AppState>>, mut st: ResMut<ShotState>, mut commands: Commands, mut exit: EventWriter<AppExit>) {
    let Some(path) = &cli.screenshot else { return };
    if *state.get() != AppState::Playing {
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

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShotState>().add_systems(Update, (handle_effects.run_if(resource_exists::<Game>), auto_screenshot));
    }
}
