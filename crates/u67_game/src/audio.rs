//! Sound effects (events) and context-driven music.
use crate::app::{AppState, Game, SettingsRes};
use crate::creatures::Creature;
use crate::db_res::DbRes;
use bevy::audio::Volume;
use bevy::prelude::*;
use u67_core::TilePos;

#[derive(Event, Clone, Debug)]
pub struct SfxEvent(pub String);

#[derive(Component)]
struct MusicTrack(String);

#[derive(Resource, Default)]
struct MusicState {
    clock: f32,
}

fn play_sfx(mut ev: EventReader<SfxEvent>, mut commands: Commands, assets: Res<AssetServer>, settings: Res<SettingsRes>) {
    let mut count = 0;
    for SfxEvent(id) in ev.read() {
        count += 1;
        if count > 6 {
            break; // avoid piling up identical sounds in one frame
        }
        let v = settings.0.master_volume * settings.0.sfx_volume;
        commands.spawn((AudioPlayer::new(assets.load(format!("sfx/{id}.wav"))), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(v))));
    }
}

/// Music for the current situation.
pub fn pick_track(map: &str, in_town: bool, combat: bool, boss: bool, state: AppState) -> &'static str {
    if matches!(state, AppState::MainMenu | AppState::Boot) {
        return "menu";
    }
    if boss {
        return "boss";
    }
    if combat {
        return "combat";
    }
    match map {
        "midgard" => {
            if in_town {
                "midgard_town"
            } else {
                "midgard_wilds"
            }
        }
        "mimir_depths" | "barrow_1" | "barrow_2" | "fenrir_den" | "troll_cave" => "dungeon",
        "hel_gate_crypt" | "hel" | "niflheimr" | "tuonela" => "hel",
        "vainola" | "pohjola" | "ilma" | "sampola" => "kalevala",
        _ => "space",
    }
}

fn music_system(time: Res<Time>, mut st: ResMut<MusicState>, state: Res<State<AppState>>, game: Option<Res<Game>>, db: Option<Res<DbRes>>, world: Option<Res<crate::app::WorldRes>>, creatures: Query<&Creature>, cur: Query<(Entity, &MusicTrack)>, mut commands: Commands, assets: Res<AssetServer>, settings: Res<SettingsRes>) {
    st.clock -= time.delta_secs();
    if st.clock > 0.0 {
        return;
    }
    st.clock = 1.0;
    let (mut in_town, mut combat, mut boss, mut map_name) = (false, false, false, String::new());
    if let (Some(g), Some(w), Some(db)) = (&game, &world, &db) {
        map_name = g.0.current_map.clone();
        let p = g.0.players[0].pos;
        if let Some(m) = w.0.maps.get(&map_name) {
            in_town = crate::npc::TOWN_IDS.iter().filter_map(|t| m.places.get(*t)).any(|c| (c.x as f32 - p[0]).abs() < 16.0 && (c.y as f32 - p[1]).abs() < 16.0) && m.building_at(TilePos::new(0, 0)).is_none();
        }
        for c in &creatures {
            if c.aggro && !c.ally && c.pos.distance(Vec2::from(p)) < 14.0 {
                combat = true;
                if db.0.creatures.get(&c.def).is_some_and(|d| d.ai == u67_world::npcs::Ai::Boss) {
                    boss = true;
                }
            }
        }
    }
    let want = pick_track(&map_name, in_town, combat, boss, *state.get());
    if cur.iter().any(|(_, t)| t.0 == want) {
        return;
    }
    for (e, _) in &cur {
        commands.entity(e).despawn();
    }
    let v = settings.0.master_volume * settings.0.music_volume;
    commands.spawn((MusicTrack(want.to_string()), AudioPlayer::new(assets.load(format!("music/{want}.wav"))), PlaybackSettings::LOOP.with_volume(Volume::Linear(v))));
}

pub struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SfxEvent>().init_resource::<MusicState>().add_systems(Update, (play_sfx, music_system));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn music_follows_context() {
        let p = AppState::Playing;
        assert_eq!(pick_track("midgard", true, false, false, p), "midgard_town");
        assert_eq!(pick_track("midgard", false, false, false, p), "midgard_wilds");
        assert_eq!(pick_track("midgard", false, true, false, p), "combat");
        assert_eq!(pick_track("maani", false, true, true, p), "boss");
        assert_eq!(pick_track("barrow_1", false, false, false, p), "dungeon");
        assert_eq!(pick_track("pohjola", false, false, false, p), "kalevala");
        assert_eq!(pick_track("hel", false, false, false, p), "hel");
        assert_eq!(pick_track("dvalinn", false, false, false, p), "space");
        assert_eq!(pick_track("", false, false, false, AppState::MainMenu), "menu");
    }
}
