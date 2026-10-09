//! Weather: rain, snow, storms (with lightning) and northern-lights, chosen by region, time of
//! day and day number, or forced with the `weather` console command.
use crate::app::{AppState, Game};
use crate::audio::SfxEvent;
use crate::player::{MainCamera, SeatCam};
use bevy::prelude::*;
use u67_core::noise;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wx {
    Clear,
    Rain,
    Snow,
    Storm,
    Aurora,
}

pub fn parse(s: &str) -> Option<Wx> {
    Some(match s {
        "clear" => Wx::Clear,
        "rain" => Wx::Rain,
        "snow" => Wx::Snow,
        "storm" => Wx::Storm,
        "aurora" => Wx::Aurora,
        _ => return None,
    })
}

/// Automatic weather for a place. `north` = 0 (arctic) .. 1 (south) for Midgard's latitude.
pub fn auto_weather(map: &str, north: f32, day: u64, hour: u32, seed: u64) -> Wx {
    let roll = noise::hash(day as i32, hour as i32 / 4, seed ^ 0x77);
    let night = !(6..20).contains(&hour);
    match map {
        "midgard" => {
            if north < 0.25 {
                if night && roll > 0.55 { Wx::Aurora } else if roll > 0.35 { Wx::Snow } else { Wx::Clear }
            } else if north < 0.4 {
                if roll > 0.8 { Wx::Snow } else { Wx::Clear }
            } else if roll > 0.93 {
                Wx::Storm
            } else if roll > 0.78 {
                Wx::Rain
            } else {
                Wx::Clear
            }
        }
        "pohjola" => {
            if night { Wx::Aurora } else { Wx::Snow }
        }
        "jotunheimr" | "niflheimr" => Wx::Snow,
        "vanaheimr" => {
            if roll > 0.7 { Wx::Storm } else { Wx::Rain }
        }
        "ilma" => {
            if roll > 0.5 { Wx::Storm } else { Wx::Clear }
        }
        "tuonela" => {
            if roll > 0.5 { Wx::Rain } else { Wx::Clear }
        }
        _ => Wx::Clear,
    }
}

#[derive(Resource)]
pub struct WeatherState {
    pub current: Wx,
    pub flash: f32,
    pub next_flash: f32,
}
impl Default for WeatherState {
    fn default() -> Self {
        Self { current: Wx::Clear, flash: 0.0, next_flash: 6.0 }
    }
}

#[derive(Component)]
struct Particle {
    vel: Vec2,
    phase: f32,
    pos: Vec2,
}
#[derive(Component)]
struct HasParticles;
#[derive(Component)]
struct Lightning;
#[derive(Component)]
struct AuroraBand(f32);

fn current_weather(g: &crate::data::GameData, map_h: f32) -> Wx {
    if let Some(w) = parse(&g.weather) {
        return w;
    }
    let p = g.players[0].pos;
    let north = if map_h > 0.0 { p[1] / map_h } else { 1.0 };
    auto_weather(&g.current_map, north, g.clock.day(), g.clock.hour(), g.seed)
}

fn spawn_particles(mut commands: Commands, cams: Query<Entity, (With<SeatCam>, Without<HasParticles>)>) {
    for cam in &cams {
        commands.entity(cam).insert(HasParticles).with_children(|c| {
            for i in 0..140 {
                let r = |k: u32| noise::hash(i, k as i32, 91);
                c.spawn((Particle { vel: Vec2::ZERO, phase: r(1) * 6.28, pos: Vec2::new((r(2) - 0.5) * 2.0, (r(3) - 0.5) * 2.0) }, Sprite { color: Color::NONE, custom_size: Some(Vec2::new(1.0, 4.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 400.0), Visibility::Hidden));
            }
            c.spawn((Lightning, Sprite { color: Color::NONE, custom_size: Some(Vec2::splat(6000.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 500.0)));
            for k in 0..3 {
                c.spawn((AuroraBand(k as f32), Sprite { color: Color::NONE, custom_size: Some(Vec2::new(2400.0, 60.0 + k as f32 * 20.0)), ..default() }, Transform::from_xyz(0.0, 220.0 - k as f32 * 36.0, 300.0)));
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn update_weather(
    time: Res<Time>,
    game: Res<Game>,
    world: Res<crate::app::WorldRes>,
    mut st: ResMut<WeatherState>,
    windows: Query<&Window>,
    cam_scale: Query<&Projection, With<MainCamera>>,
    mut particles: Query<(&mut Particle, &mut Transform, &mut Sprite, &mut Visibility), Without<Lightning>>,
    mut light: Query<&mut Sprite, (With<Lightning>, Without<Particle>, Without<AuroraBand>)>,
    mut bands: Query<(&AuroraBand, &mut Sprite), (Without<Particle>, Without<Lightning>)>,
    mut sfx: EventWriter<SfxEvent>,
) {
    let dt = time.delta_secs();
    let map_h = world.0.maps.get(&game.0.current_map).map_or(0.0, |m| m.height as f32);
    let underground = matches!(game.0.current_map.as_str(), "mimir_depths" | "barrow_1" | "barrow_2" | "fenrir_den" | "troll_cave" | "hel_gate_crypt");
    let w = if underground { Wx::Clear } else { current_weather(&game.0, map_h) };
    st.current = w;
    let scale = cam_scale.single().ok().map_or(0.33, |p| if let Projection::Orthographic(o) = p { o.scale } else { 0.33 });
    let (ww, wh) = windows.single().map_or((1280.0, 720.0), |w| (w.width(), w.height()));
    let (hx, hy) = (ww * 0.5 * scale * 1.1, wh * 0.5 * scale * 1.1);
    let t = time.elapsed_secs();
    for (mut p, mut tf, mut sp, mut vis) in &mut particles {
        let (vel, color, size, on) = match w {
            Wx::Rain => (Vec2::new(-40.0, -230.0), Color::srgba(0.7, 0.8, 1.0, 0.55), Vec2::new(1.0, 5.0), true),
            Wx::Storm => (Vec2::new(-90.0, -300.0), Color::srgba(0.7, 0.8, 1.0, 0.6), Vec2::new(1.0, 6.0), true),
            Wx::Snow => (Vec2::new(0.0, -28.0 - (p.phase * 3.0).sin().abs() * 14.0), Color::srgba(1.0, 1.0, 1.0, 0.85), Vec2::new(2.0, 2.0), true),
            _ => (Vec2::ZERO, Color::NONE, Vec2::ONE, false),
        };
        *vis = if on { Visibility::Inherited } else { Visibility::Hidden };
        if !on {
            continue;
        }
        p.vel = vel;
        let sway = if w == Wx::Snow { (t * 1.3 + p.phase).sin() * 8.0 } else { 0.0 };
        let v = p.vel;
        p.pos.x += (v.x + sway) * dt / hx;
        p.pos.y += v.y * dt / hy;
        if p.pos.y < -1.0 {
            p.pos.y += 2.0;
        }
        if p.pos.x < -1.0 {
            p.pos.x += 2.0;
        }
        if p.pos.x > 1.0 {
            p.pos.x -= 2.0;
        }
        tf.translation.x = p.pos.x * hx;
        tf.translation.y = p.pos.y * hy;
        sp.color = color;
        sp.custom_size = Some(size);
    }
    // lightning
    st.flash = (st.flash - dt * 3.0).max(0.0);
    if w == Wx::Storm {
        st.next_flash -= dt;
        if st.next_flash <= 0.0 {
            st.next_flash = 5.0 + noise::hash((t * 10.0) as i32, 1, 5) * 9.0;
            st.flash = 1.0;
            sfx.write(SfxEvent("explosion".into()));
        }
    }
    for mut s in &mut light {
        s.color = Color::srgba(0.9, 0.95, 1.0, st.flash * 0.55);
    }
    // aurora bands
    for (b, mut s) in &mut bands {
        let a = if w == Wx::Aurora { 0.10 + 0.09 * (t * 0.7 + b.0 * 1.7).sin().abs() } else { 0.0 };
        s.color = Color::srgba(0.3 + 0.2 * (t * 0.3 + b.0).sin().abs(), 1.0, 0.7 - b.0 * 0.15, a);
    }
}

fn active(game: Option<Res<Game>>, world: Option<Res<crate::app::WorldRes>>, state: Res<State<AppState>>) -> bool {
    game.is_some() && world.is_some() && !matches!(state.get(), AppState::Boot | AppState::MainMenu)
}

pub struct WeatherPlugin;
impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherState>().add_systems(Update, (spawn_particles, update_weather).chain().run_if(active).run_if(not(in_state(AppState::Paused))));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regional_weather() {
        assert_eq!(auto_weather("pohjola", 1.0, 3, 23, 1), Wx::Aurora);
        assert_eq!(auto_weather("pohjola", 1.0, 3, 12, 1), Wx::Snow);
        assert_eq!(auto_weather("jotunheimr", 1.0, 3, 12, 1), Wx::Snow);
        assert_eq!(auto_weather("maani", 1.0, 3, 12, 1), Wx::Clear);
        // deterministic
        assert_eq!(auto_weather("midgard", 0.8, 5, 9, 67), auto_weather("midgard", 0.8, 5, 9, 67));
        // the south sees rain sometimes, the far north never rain
        let mut rain = 0;
        for day in 0..200 {
            if matches!(auto_weather("midgard", 0.8, day, 12, 67), Wx::Rain | Wx::Storm) {
                rain += 1;
            }
            assert!(!matches!(auto_weather("midgard", 0.1, day, 12, 67), Wx::Rain | Wx::Storm));
        }
        assert!((10..90).contains(&rain), "{rain}");
        assert_eq!(parse("storm"), Some(Wx::Storm));
        assert_eq!(parse("auto"), None);
    }
}
