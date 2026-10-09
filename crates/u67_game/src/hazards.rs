//! Environmental hazards on other worlds: vacuum, heat, cold, lava, toxic gas.
use crate::app::{AppState, Game, WorldRes};
use crate::seats::PlayerRt;
use crate::ui::Toast;
use bevy::prelude::*;
use u67_core::TilePos;
use u67_world::inventory::Inventory;
use u67_world::items::Slot;
use u67_world::tiles::{self, TileDef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Env {
    Normal,
    Vacuum,
    Heat,
    Cold,
    Gas,
}

pub fn env_of(map: &str) -> Env {
    match map {
        "maani" | "svartalfheimr" | "ginnungagap" => Env::Vacuum,
        "sol" | "dvalinn" | "muspelheimr" => Env::Heat,
        "jotunheimr" | "niflheimr" | "pohjola" | "fenrir_den" => Env::Cold,
        "asgard" | "ilma" => Env::Gas,
        _ => Env::Normal,
    }
}

/// Damage per second and a warning message for standing on `tile` in `map` with `inv`.
pub fn hazard(map: &str, tile: &TileDef, inv: &Inventory) -> (f32, &'static str) {
    let mut dps = 0.0f32;
    let mut msg = "";
    if tile.name == "lava" {
        dps += 12.0;
        msg = "The lava burns you!";
    }
    match env_of(map) {
        Env::Vacuum if inv.count("vacuum_suit") == 0 => {
            dps += 3.0;
            msg = "No air! Carry a vacuum suit.";
        }
        Env::Heat if tile.name != "floor_stone" && inv.equipped.get(&Slot::Back).is_none_or(|i| i.id != "wolf_cloak") && inv.count("vacuum_suit") == 0 => {
            dps += 1.0;
            msg = "The heat is withering. A vacuum suit insulates you.";
        }
        Env::Cold if inv.equipped.get(&Slot::Back).is_none_or(|i| i.id != "wolf_cloak") && inv.count("vacuum_suit") == 0 => {
            dps += 1.2;
            msg = "Freezing! Wear a wolf cloak (or a suit).";
        }
        Env::Gas if tile.name == "gas" => {
            dps += 4.0;
            msg = "Toxic gas! Stay on the platforms.";
        }
        _ => {}
    }
    (dps, msg)
}

#[allow(clippy::too_many_arguments)]
fn tick(time: Res<Time>, mut acc: Local<f32>, mut game: ResMut<Game>, world: Res<WorldRes>, mut rts: ResMut<PlayerRt>, mut toast: ResMut<Toast>, mut next: ResMut<NextState<AppState>>) {
    *acc += time.delta_secs();
    if *acc < 1.0 {
        return;
    }
    *acc = 0.0;
    if game.0.cheats.god {
        return;
    }
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    let map_name = game.0.current_map.clone();
    for i in 0..game.0.players.len() {
        if rts.list[i].downed || rts.list[i].roll_t > 0.0 || (i == 0 && game.0.vehicle.is_some()) {
            continue;
        }
        let p = game.0.players[i].pos;
        let t = map.tile(TilePos::new(p[0].floor() as i32, p[1].floor() as i32));
        let (dps, msg) = hazard(&map_name, tiles::def(t), &game.0.players[i].inventory);
        if dps > 0.0 {
            game.0.players[i].stats.hp -= dps.ceil() as i32;
            *toast = Toast { text: msg.into(), timer: 2.0 };
            if game.0.players[i].stats.hp <= 0 {
                game.0.players[i].stats.hp = 0;
                rts.list[i].downed = true;
                rts.list[i].down_timer = 0.0;
                if (0..game.0.players.len()).all(|k| rts.list[k].downed) {
                    next.set(AppState::Dead);
                }
            }
        }
    }
}

pub struct HazardPlugin;
impl Plugin for HazardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tick.run_if(resource_exists::<Game>).run_if(resource_exists::<WorldRes>).run_if(in_state(AppState::Playing)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vacuum_needs_a_suit_cold_needs_a_cloak_lava_always_burns() {
        let mut inv = Inventory::new(20);
        let grass = tiles::def(tiles::GRASS);
        assert_eq!(hazard("midgard", grass, &inv).0, 0.0);
        assert!(hazard("maani", tiles::def(tiles::REGOLITH), &inv).0 >= 3.0);
        inv.add("vacuum_suit", 1).unwrap();
        assert_eq!(hazard("maani", tiles::def(tiles::REGOLITH), &inv).0, 0.0);
        assert!(hazard("maani", tiles::def(tiles::LAVA), &inv).0 >= 12.0);
        let mut inv2 = Inventory::new(20);
        assert!(hazard("jotunheimr", tiles::def(tiles::SNOW), &inv2).0 > 0.0);
        inv2.add("wolf_cloak", 1).unwrap();
        inv2.equip(&[0]).unwrap();
        assert_eq!(hazard("jotunheimr", tiles::def(tiles::SNOW), &inv2).0, 0.0);
        assert!(hazard("asgard", tiles::def(tiles::GAS), &inv2).0 > 0.0);
    }
}
