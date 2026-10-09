//! Spellcasting: reagent + mana costs (pure) and effects (Bevy systems). Spells can be cast
//! from the spellbook, hotkeys, or by "singing" a 3-rune laulu sequence.
use crate::app::{AppState, Game, WorldRes};
use crate::audio::SfxEvent;
use crate::combat::{Cursor, ProjKind, SpawnProjectile};
use crate::seats::{Intents, PlayerRt};
use crate::creatures::{self, Creature};
use crate::data::GameData;
use crate::db_res::DbRes;
use crate::menus::spell_known;
use crate::render::Sheets;
use bevy::prelude::*;
use u67_core::TilePos;
use u67_world::db::Db;
use u67_world::spells::{find_by_runes, Effect as Fx, SpellDef};

#[derive(Event, Clone, Debug)]
pub struct CastRequest(pub String, pub usize);

#[derive(Event, Clone, Debug)]
pub struct SingRequest(pub Vec<u8>);

/// Mana for casting after INT/VAKI discount.
pub fn mana_cost(d: &GameData, s: &SpellDef) -> i32 {
    let disc = (d.players[0].stats.vaki as f32 * 0.01).min(0.4);
    ((s.mana as f32) * (1.0 - disc)).ceil() as i32
}

pub fn can_cast(d: &GameData, db: &Db, s: &SpellDef) -> Result<(), String> {
    if !spell_known(d, db, &s.id) {
        return Err("You do not know that spell.".into());
    }
    if d.players[0].stats.mana < mana_cost(d, s) {
        return Err("Not enough mana.".into());
    }
    if !d.spells_known.contains("*") {
        for (r, n) in &s.reagents {
            if d.players[0].inventory.count(r) < *n {
                return Err(format!("Missing reagent: {} x{}.", u67_world::items::get(r).map_or(r.as_str(), |i| i.name), n));
            }
        }
    }
    Ok(())
}

pub fn pay(d: &mut GameData, s: &SpellDef) {
    d.players[0].stats.mana -= mana_cost(d, s);
    if !d.spells_known.contains("*") {
        for (r, n) in &s.reagents {
            d.players[0].inventory.remove(r, *n);
        }
    }
}

pub fn spell_power(d: &GameData) -> f32 {
    1.0 + d.players[0].stats.int as f32 / 40.0 + d.players[0].stats.level as f32 * 0.02
}

#[allow(clippy::too_many_arguments)]
fn handle_cast(
    mut ev: EventReader<CastRequest>,
    mut sing: EventReader<SingRequest>,
    mut game: ResMut<Game>,
    world: Res<WorldRes>,
    db: Res<DbRes>,
    cursor: Res<Cursor>,
    mut rt: ResMut<PlayerRt>,
    mut shots: EventWriter<SpawnProjectile>,
    mut sfx: EventWriter<SfxEvent>,
    mut toast: ResMut<crate::ui::Toast>,
    mut creatures_q: Query<(Entity, &mut Creature)>,
    mut commands: Commands,
    sheets: Res<Sheets>,
    active: Res<crate::seats::ActiveSeat>,
) {
    let mut requests: Vec<(String, usize)> = ev.read().map(|c| (c.0.clone(), c.1.min(game.0.players.len() - 1))).collect();
    for s in sing.read() {
        match find_by_runes(&db.0.spells, &s.0) {
            Some(sp) => requests.push((sp.id.clone(), active.0.min(game.0.players.len() - 1))),
            None => {
                // wrong verse: lose a little mana
                game.0.players[0].stats.mana = (game.0.players[0].stats.mana - 2).max(0);
                *toast = crate::ui::Toast { text: "The verse falls flat... (no spell matches those runes)".into(), timer: 2.5 };
                sfx.write(SfxEvent("spell_fizzle".into()));
            }
        }
    }
    for (id, seat) in requests {
        let Some(sp) = db.0.spells.iter().find(|s| s.id == id).cloned() else { continue };
        game.0.players.swap(0, seat);
        rt.active = seat;
        'one: {
        if rt.downed {
            break 'one;
        }
        if let Err(e) = can_cast(&game.0, &db.0, &sp) {
            *toast = crate::ui::Toast { text: e, timer: 2.0 };
            sfx.write(SfxEvent("spell_fizzle".into()));
            break 'one;
        }
        let pos = Vec2::from(game.0.players[0].pos);
        let aim = if cursor.valid { (cursor.tile - (pos - Vec2::new(0.0, 0.5))).normalize_or_zero() } else { Vec2::Y };
        let aim = if aim == Vec2::ZERO { Vec2::Y } else { aim };
        let power = spell_power(&game.0);
        pay(&mut game.0, &sp);
        sfx.write(SfxEvent("spell_cast".into()));
        match &sp.effect {
            Fx::Bolt { damage, speed, range, splash, element } => {
                shots.write(SpawnProjectile { pos: pos - Vec2::new(0.0, 0.5) + aim * 0.6, vel: aim * *speed, dmg: (*damage as f32 * power) as i32, friendly: true, range: *range, kind: ProjKind::Magic, splash: *splash, special: String::new(), cannon: false, target: None, elem: element.clone() });
            }
            Fx::Heal { amount } => {
                let max = game.0.players[0].stats.max_hp();
                game.0.players[0].stats.hp = (game.0.players[0].stats.hp + (*amount as f32 * power) as i32).min(max);
            }
            Fx::Light { seconds } => rt.light = *seconds,
            Fx::Haste { seconds } => rt.haste = *seconds,
            Fx::Protect { amount, seconds } => rt.protect = (*amount, *seconds),
            Fx::Cure => game.0.players[0].stats.hp = (game.0.players[0].stats.hp + 5).min(game.0.players[0].stats.max_hp()),
            Fx::Blink { range } => {
                if let Some(map) = world.0.maps.get(&game.0.current_map) {
                    let mut dest = pos;
                    let mut d = 0.5;
                    while d <= *range {
                        let p = pos + aim * d;
                        if map.walkable(TilePos::new(p.x.floor() as i32, p.y.floor() as i32)) {
                            dest = p;
                        } else {
                            break;
                        }
                        d += 0.5;
                    }
                    game.0.players[0].pos = dest.into();
                    game.0.map_dirty = true;
                }
            }
            Fx::Summon { creature, count, seconds } => {
                for k in 0..*count {
                    let off = Vec2::new(0.8 + k as f32 * 0.6, 0.2);
                    creatures::spawn_by_id(&mut commands, &sheets, &db.0, creature, pos + off, true, Some(*seconds));
                }
            }
            Fx::Fear { radius, seconds } => {
                for (_, mut c) in &mut creatures_q {
                    if !c.ally && c.pos.distance(pos) <= *radius {
                        c.fear = *seconds;
                    }
                }
            }
            Fx::Freeze { radius, seconds } => {
                for (_, mut c) in &mut creatures_q {
                    if !c.ally && c.pos.distance(pos) <= *radius {
                        c.frozen = *seconds;
                    }
                }
            }
            Fx::Missiles { count, damage, range } => {
                let mut targets: Vec<Vec2> = creatures_q.iter().filter(|(_, c)| !c.ally && c.pos.distance(pos) <= *range).map(|(_, c)| c.pos).collect();
                targets.sort_by(|a, b| a.distance(pos).partial_cmp(&b.distance(pos)).unwrap());
                for k in 0..*count as usize {
                    let dir = targets.get(k % targets.len().max(1)).map(|t| (*t - pos).normalize_or_zero()).unwrap_or(aim);
                    let jitter = Vec2::new(-dir.y, dir.x) * ((k as f32) - (*count as f32) / 2.0) * 0.12;
                    shots.write(SpawnProjectile { pos: pos - Vec2::new(0.0, 0.5) + jitter, vel: (dir + jitter * 0.3).normalize_or_zero() * 20.0, dmg: (*damage as f32 * power) as i32, friendly: true, range: *range, kind: ProjKind::Magic, splash: 0.0, special: String::new(), cannon: false, target: None, elem: "arcane".into() });
                }
            }
            Fx::Reveal { radius } => {
                let (cx, cy) = TilePos::new(pos.x as i32, pos.y as i32).chunk();
                let r = (*radius / 16.0).ceil() as i32;
                let map = game.0.current_map.clone();
                for dy in -r..=r {
                    for dx in -r..=r {
                        game.0.explored.insert(format!("{map}:{}:{}", cx + dx, cy + dy));
                    }
                }
                *toast = crate::ui::Toast { text: "Your raven's-eye reveals the land around you.".into(), timer: 2.0 };
            }
        }
        }
        game.0.players.swap(0, seat);
        rt.active = 0;
    }
}

fn hotkeys(intents: Res<Intents>, game: Res<Game>, mut req: EventWriter<CastRequest>) {
    for seat in 0..game.0.players.len() {
        for (i, pressed) in intents.get(seat).spells.iter().enumerate() {
            if *pressed {
                if let Some(s) = game.0.players[seat].spell_slots.get(i) {
                    req.write(CastRequest(s.clone(), seat));
                }
            }
        }
    }
}

fn ready(game: Option<Res<Game>>, db: Option<Res<DbRes>>, sheets: Option<Res<Sheets>>) -> bool {
    game.is_some() && db.is_some() && sheets.is_some()
}

pub struct MagicPlugin;
impl Plugin for MagicPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastRequest>().add_event::<SingRequest>().add_systems(Update, (hotkeys.run_if(in_state(AppState::Playing)), handle_cast).run_if(ready));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn costs_and_reagents() {
        let db = Db::builtin().unwrap();
        let mut d = GameData::new(1, "midgard", [0.0, 0.0]);
        let valo = db.spells.iter().find(|s| s.id == "valo").unwrap().clone();
        let bolt = db.spells.iter().find(|s| s.id == "ukon_nuoli").unwrap().clone();
        assert!(can_cast(&d, &db, &valo).unwrap_err().contains("reagent"));
        d.players[0].inventory.add("reagent_birch_ash", 2).unwrap();
        assert!(can_cast(&d, &db, &valo).is_ok());
        let mana = d.players[0].stats.mana;
        pay(&mut d, &valo);
        assert!(d.players[0].stats.mana < mana);
        assert_eq!(d.players[0].inventory.count("reagent_birch_ash"), 1);
        // unknown spell
        let blink = db.spells.iter().find(|s| s.id == "siirto").unwrap();
        assert!(can_cast(&d, &db, blink).unwrap_err().contains("do not know"));
        // spells_all removes reagent needs
        d.spells_known.insert("*".into());
        assert!(can_cast(&d, &db, &bolt).is_ok());
        d.players[0].stats.mana = 0;
        assert!(can_cast(&d, &db, &bolt).unwrap_err().contains("mana"));
    }

    #[test]
    fn rune_sequences_unique_and_findable() {
        let db = Db::builtin().unwrap();
        for (i, a) in db.spells.iter().enumerate() {
            assert_eq!(a.runes.len(), 3);
            assert!(a.runes.iter().all(|r| *r < 24));
            for b in &db.spells[i + 1..] {
                assert_ne!(a.runes, b.runes, "{} vs {}", a.id, b.id);
            }
        }
        assert_eq!(find_by_runes(&db.spells, &[0, 4, 8]).unwrap().id, "valo");
        assert!(find_by_runes(&db.spells, &[1, 1, 1]).is_none());
    }
}
