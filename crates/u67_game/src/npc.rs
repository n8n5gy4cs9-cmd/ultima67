//! NPC simulation: daily schedules (U7 style), pathing, party following. Pure logic first,
//! Bevy glue (sprites) at the bottom.
use crate::app::{AppState, Game, WorldRes};
use crate::db_res::DbRes;
use crate::render::{self, Sheets, CHAR_COLS};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use u67_core::{Dir, GameClock, Rng, TilePos};
use u67_world::db::Db;
use u67_world::map::{Map, World};
use u67_world::npcs::NpcDef;
use u67_world::pathfind::find_path;
use u67_world::schedule::{Activity, Schedule};

pub const TOWN_IDS: &[&str] = &["kaupang", "birka", "hedeby", "sigtuna", "nidaros", "bjorgvin", "visby", "turku", "savo", "kuusamo", "rovala", "aldeigja", "reyk", "helgate", "jorvik", "mimir"];

#[derive(Clone, Debug)]
pub struct Npc {
    pub id: String,
    pub name: String,
    pub arch: String,
    pub dialogue: String,
    pub shop: Option<String>,
    pub town: String,
    pub recruitable: bool,
    pub pos: [f32; 2],
    pub facing: Dir,
    pub schedule: Schedule,
    pub slot: i32,
    pub activity: Activity,
    pub target: Option<TilePos>,
    pub path: Vec<TilePos>,
    pub moving: bool,
    pub repath: f32,
    pub wander: f32,
    pub in_party: bool,
    pub hp: i32,
    pub max_hp: i32,
    pub attack_cd: f32,
}

#[derive(Resource, Default)]
pub struct Npcs {
    pub map: String,
    pub list: Vec<Npc>,
    /// Bumped whenever the list is rebuilt so sprite entities resync.
    pub epoch: u32,
}

const MALE: &[&str] = &["Arnbjorn", "Gunnar", "Ivar", "Leif", "Olaf", "Ragnvald", "Sigurd", "Torstein", "Ulf", "Halfdan", "Eino", "Matti", "Tapio", "Kalle"];
const FEMALE: &[&str] = &["Astrid", "Gerd", "Ingrid", "Ragna", "Sif", "Thora", "Yrsa", "Helga", "Aili", "Kaisa", "Liisa", "Sanna", "Tyyne", "Runa"];

fn place(map: &Map, name: &str, town: &str) -> Option<TilePos> {
    map.places.get(&name.replace("{town}", town)).copied()
}

fn def_to_npc(def: &NpcDef, map: &Map, party: bool) -> Option<Npc> {
    let home = place(map, &def.home, &def.town)?;
    let work = place(map, &def.work, &def.town).unwrap_or(home);
    let tavern = def.tavern.as_ref().and_then(|t| place(map, t, &def.town)).unwrap_or(work);
    let mut schedule = Schedule::townsperson(home, work, tavern);
    if let Some(a) = def.work_activity {
        for e in schedule.entries.iter_mut().filter(|e| e.activity == Activity::Work) {
            e.activity = a;
        }
    }
    Some(Npc {
        id: def.id.clone(),
        name: def.name.clone(),
        arch: def.arch.clone(),
        dialogue: def.dialogue.clone().unwrap_or_else(|| def.id.clone()),
        shop: def.shop.clone(),
        town: def.town.clone(),
        recruitable: def.recruitable,
        pos: [work.x as f32 + 0.5, work.y as f32 + 0.5],
        facing: Dir::S,
        schedule,
        slot: -1,
        activity: Activity::Stand,
        target: None,
        path: vec![],
        moving: false,
        repath: 0.0,
        wander: 0.0,
        in_party: party,
        hp: 60,
        max_hp: 60,
        attack_cd: 0.0,
    })
}

/// Build the NPC list for `map_name`: named NPCs of that map, generic townsfolk and guards (Midgard),
/// plus party members from anywhere (placed beside the player).
pub fn build_npcs(db: &Db, world: &World, map_name: &str, player: [f32; 2], party: &[String], seed: u64) -> Vec<Npc> {
    let mut out = vec![];
    let Some(map) = world.maps.get(map_name) else { return out };
    for def in &db.npcs {
        let dm = def.map.as_deref().unwrap_or("midgard");
        let is_party = party.contains(&def.id);
        if dm == map_name && !is_party {
            if let Some(n) = def_to_npc(def, map, false) {
                out.push(n);
            }
        } else if is_party {
            // party members follow the player to any map
            let home_map = world.maps.get(dm).unwrap_or(map);
            if let Some(mut n) = def_to_npc(def, home_map, true) {
                n.pos = [player[0] + 1.0, player[1] + 0.5];
                out.push(n);
            }
        }
    }
    if map_name == "midgard" {
        let mut rng = Rng::new(seed ^ 0xA11CE);
        for town in TOWN_IDS {
            let Some(center) = map.places.get(*town).copied() else { continue };
            for k in 0..3 {
                let female = rng.chance(0.5);
                let name = if female { *rng.pick(FEMALE) } else { *rng.pick(MALE) };
                let home = map.places.get(&format!("{town}_house_{}", 1 + k + (rng.range(0, 2)))).copied().unwrap_or(center);
                let opts = [map.places.get(&format!("{town}_market")).copied().unwrap_or(center), map.places.get(&format!("{town}_forge")).copied().unwrap_or(center), center];
                let work = *rng.pick(&opts);
                let tavern = map.places.get(&format!("{town}_tavern")).copied().unwrap_or(center);
                let mut schedule = Schedule::townsperson(home, work, tavern);
                for e in schedule.entries.iter_mut().filter(|e| e.activity == Activity::Work) {
                    e.activity = *rng.pick(&[Activity::Wander, Activity::Loiter, Activity::Farm, Activity::Fish]);
                }
                out.push(Npc {
                    id: format!("{town}_villager_{k}"),
                    name: name.into(),
                    arch: if female { "villager_f" } else { "villager_m" }.into(),
                    dialogue: "villager".into(),
                    shop: None,
                    town: town.to_string(),
                    recruitable: false,
                    pos: [work.x as f32 + 0.5, work.y as f32 + 0.5],
                    facing: Dir::S,
                    schedule,
                    slot: -1,
                    activity: Activity::Stand,
                    target: None,
                    path: vec![],
                    moving: false,
                    repath: 0.0,
                    wander: 0.0,
                    in_party: false,
                    hp: 40,
                    max_hp: 40,
                    attack_cd: 0.0,
                });
            }
            for k in 0..2 {
                let post = TilePos::new(center.x + if k == 0 { -9 } else { 9 }, center.y + 10);
                let mut schedule = Schedule::default();
                schedule.set(0, Activity::Guard, post);
                out.push(Npc {
                    id: format!("{town}_guard_{k}"),
                    name: "Warden Guard".into(),
                    arch: "warden_guard".into(),
                    dialogue: "guard".into(),
                    shop: None,
                    town: town.to_string(),
                    recruitable: false,
                    pos: [post.x as f32 + 0.5, post.y as f32 + 0.5],
                    facing: Dir::S,
                    schedule,
                    slot: -1,
                    activity: Activity::Guard,
                    target: None,
                    path: vec![],
                    moving: false,
                    repath: 0.0,
                    wander: 0.0,
                    in_party: false,
                    hp: 90,
                    max_hp: 90,
                    attack_cd: 0.0,
                });
            }
        }
    }
    out
}

const WALK_SPEED: f32 = 1.7;
const FOLLOW_SPEED: f32 = 3.6;

fn tile_of(p: [f32; 2]) -> TilePos {
    TilePos::new(p[0].floor() as i32, p[1].floor() as i32)
}

fn walk_path(n: &mut Npc, dt: f32, speed: f32) {
    n.moving = false;
    let Some(next) = n.path.first().copied() else { return };
    let tgt = [next.x as f32 + 0.5, next.y as f32 + 0.5];
    let (dx, dy) = (tgt[0] - n.pos[0], tgt[1] - n.pos[1]);
    let dist = (dx * dx + dy * dy).sqrt();
    let step = speed * dt;
    if dist <= step {
        n.pos = tgt;
        n.path.remove(0);
    } else {
        n.pos[0] += dx / dist * step;
        n.pos[1] += dy / dist * step;
    }
    n.moving = true;
    n.facing = if dx.abs() > dy.abs() { if dx > 0.0 { Dir::E } else { Dir::W } } else if dy > 0.0 { Dir::S } else { Dir::N };
}

/// Advance one NPC. `player` is the leader position for party members.
pub fn update_npc(n: &mut Npc, map: &Map, clock: &GameClock, dt: f32, player: [f32; 2], rng: &mut Rng) {
    n.attack_cd = (n.attack_cd - dt).max(0.0);
    if n.in_party {
        let d2 = (player[0] - n.pos[0]).powi(2) + (player[1] - n.pos[1]).powi(2);
        if d2 > 14.0 * 14.0 {
            n.pos = [player[0] + 1.0, player[1] + 0.5];
            n.path.clear();
        } else if d2 > 2.6 * 2.6 {
            n.repath -= dt;
            if n.repath <= 0.0 || n.path.is_empty() {
                n.repath = 0.6;
                n.path = find_path(map, tile_of(n.pos), tile_of(player), 1500).unwrap_or_default();
                if n.path.len() > 1 {
                    n.path.pop(); // stop beside the leader
                }
            }
            walk_path(n, dt, FOLLOW_SPEED);
        } else {
            n.moving = false;
            n.path.clear();
        }
        return;
    }
    let hour = clock.hour();
    let slot = (hour / 3) as i32;
    if slot != n.slot {
        n.slot = slot;
        if let Some(e) = n.schedule.at_hour(hour) {
            n.activity = e.activity;
            n.target = Some(e.pos);
            n.path.clear();
            let far = (player[0] - n.pos[0]).abs() + (player[1] - n.pos[1]).abs() > 60.0;
            if far {
                n.pos = [e.pos.x as f32 + 0.5, e.pos.y as f32 + 0.5]; // U7: off-screen NPCs just appear at their post
                n.target = None;
            }
        }
    }
    if let Some(t) = n.target {
        if tile_of(n.pos) == t {
            n.target = None;
        } else if n.path.is_empty() {
            n.path = find_path(map, tile_of(n.pos), t, 4000).unwrap_or_default();
            if n.path.is_empty() {
                n.target = None;
            }
        }
    }
    if !n.path.is_empty() {
        walk_path(n, dt, WALK_SPEED);
        return;
    }
    n.moving = false;
    if matches!(n.activity, Activity::Wander | Activity::Loiter | Activity::Farm | Activity::Fish) {
        n.wander -= dt;
        if n.wander <= 0.0 {
            n.wander = 2.0 + rng.f32() * 4.0;
            let d = *rng.pick(&Dir::ALL);
            let t = tile_of(n.pos).step(d);
            if map.walkable(t) && t.manhattan(n.schedule.at_hour(hour).map_or(t, |e| e.pos)) <= 4 {
                n.path = vec![t];
            }
        }
    }
}

/// Which NPC (index) can the player talk to near `pos`?
pub fn npc_near(npcs: &[Npc], pos: [f32; 2], reach: f32) -> Option<usize> {
    npcs.iter().enumerate().filter(|(_, n)| (n.pos[0] - pos[0]).powi(2) + (n.pos[1] - pos[1]).powi(2) <= reach * reach).min_by(|(_, a), (_, b)| {
        let da = (a.pos[0] - pos[0]).powi(2) + (a.pos[1] - pos[1]).powi(2);
        let db = (b.pos[0] - pos[0]).powi(2) + (b.pos[1] - pos[1]).powi(2);
        da.partial_cmp(&db).unwrap()
    }).map(|(i, _)| i)
}

// ------------------------------------------------------------------ Bevy glue

#[derive(Component)]
pub struct NpcSprite(pub String);

fn rebuild_npcs(mut npcs: ResMut<Npcs>, game: Res<Game>, world: Res<WorldRes>, db: Res<DbRes>) {
    let party_changed = npcs.list.iter().filter(|n| n.in_party).map(|n| n.id.clone()).collect::<std::collections::BTreeSet<_>>() != game.0.party.iter().cloned().collect();
    if npcs.map == game.0.current_map && !party_changed {
        return;
    }
    let old: Vec<Npc> = std::mem::take(&mut npcs.list);
    let mut fresh = build_npcs(&db.0, &world.0, &game.0.current_map, game.0.players[0].pos, &game.0.party, game.0.seed);
    // keep positions of NPCs that persist across a party change on the same map
    if npcs.map == game.0.current_map {
        for n in &mut fresh {
            if let Some(o) = old.iter().find(|o| o.id == n.id) {
                n.pos = o.pos;
            }
        }
    }
    npcs.map = game.0.current_map.clone();
    npcs.list = fresh;
    npcs.epoch += 1;
}

fn simulate_npcs(time: Res<Time>, game: Res<Game>, world: Res<WorldRes>, mut npcs: ResMut<Npcs>, mut local: Local<Option<Rng>>) {
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    let rng = local.get_or_insert_with(|| Rng::new(99));
    let player = game.0.players[0].pos;
    let dt = time.delta_secs();
    let clock = game.0.clock;
    for n in &mut npcs.list {
        update_npc(n, map, &clock, dt, player, rng);
    }
}

#[derive(Resource, Default)]
struct SpawnedEpoch(u32);

fn sync_npc_sprites(mut commands: Commands, npcs: Res<Npcs>, sheets: Res<Sheets>, mut epoch: ResMut<SpawnedEpoch>, existing: Query<Entity, With<NpcSprite>>, mut q: Query<(&NpcSprite, &mut Transform, &mut Sprite)>, time: Res<Time>) {
    if epoch.0 != npcs.epoch {
        epoch.0 = npcs.epoch;
        for e in &existing {
            commands.entity(e).despawn();
        }
        for n in &npcs.list {
            let row = render::character_row(&n.arch).unwrap_or(0);
            commands.spawn((
                NpcSprite(n.id.clone()),
                Sprite { image: sheets.chars_img.clone(), texture_atlas: Some(TextureAtlas { layout: sheets.chars_layout.clone(), index: (row * 4 * CHAR_COLS + 2 * CHAR_COLS) as usize }), anchor: Anchor::Custom(Vec2::new(0.0, -0.5 + 1.5 / 24.0)), ..default() },
                Transform::from_xyz(0.0, 0.0, 2.0),
            ));
        }
        return;
    }
    for (tag, mut tf, mut sp) in &mut q {
        let Some(n) = npcs.list.iter().find(|n| n.id == tag.0) else { continue };
        let p = render::tile_px(n.pos[0], n.pos[1]);
        tf.translation = Vec3::new(p.x.round(), p.y.round(), 2.0 + n.pos[1] * 0.001);
        let row = render::character_row(&n.arch).unwrap_or(0);
        let frame = if n.moving { 1 + ((time.elapsed_secs() * 8.0) as u32 % 4) } else { 0 };
        if let Some(a) = &mut sp.texture_atlas {
            a.index = ((row * 4 + n.facing.index() as u32) * CHAR_COLS + frame) as usize;
        }
    }
}

fn active(game: Option<Res<Game>>, db: Option<Res<DbRes>>, state: Res<State<AppState>>) -> bool {
    game.is_some() && db.is_some() && !matches!(state.get(), AppState::Boot | AppState::MainMenu)
}

/// Party members shoot/strike the nearest hostile creature in range.
fn party_combat(time: Res<Time>, game: Res<Game>, mut npcs: ResMut<Npcs>, creatures: Query<(Entity, &crate::creatures::Creature)>, mut dmg: EventWriter<crate::combat::DamageEvent>, mut sfx: EventWriter<crate::audio::SfxEvent>) {
    let dt = time.delta_secs();
    let lvl = game.0.players[0].stats.level as f32;
    for n in npcs.list.iter_mut().filter(|n| n.in_party) {
        n.attack_cd -= dt;
        if n.attack_cd > 0.0 {
            continue;
        }
        let me = Vec2::new(n.pos[0], n.pos[1]);
        let best = creatures.iter().filter(|(_, c)| !c.ally && c.pos.distance(me) < 9.0).min_by(|a, b| a.1.pos.distance(me).partial_cmp(&b.1.pos.distance(me)).unwrap());
        if let Some((e, _)) = best {
            n.attack_cd = 1.1;
            dmg.write(crate::combat::DamageEvent { target: e, amount: (7.0 + lvl * 1.5) as i32, crit: false, by_player: true, special: String::new(), cannon: false });
            sfx.write(crate::audio::SfxEvent(if n.arch == "valkyrie" || n.arch == "berserker" { "swing" } else { "gun_pistol" }.into()));
        }
    }
}

pub struct NpcPlugin;
impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Npcs>().init_resource::<SpawnedEpoch>().add_systems(Update, (rebuild_npcs, simulate_npcs.run_if(in_state(AppState::Playing)), sync_npc_sprites).chain().run_if(active)).add_systems(FixedUpdate, party_combat.run_if(active).run_if(in_state(AppState::Playing)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use u67_world::tiles;

    fn tiny() -> (Map, Npc) {
        let mut m = Map::new("t", 60, 60, tiles::GRASS);
        m.places.insert("home".into(), TilePos::new(5, 5));
        m.places.insert("work".into(), TilePos::new(20, 5));
        let def = NpcDef { id: "x".into(), name: "X".into(), arch: "villager_m".into(), town: "t".into(), home: "home".into(), work: "work".into(), tavern: None, work_activity: Some(Activity::Forge), dialogue: None, shop: None, recruitable: false, map: None };
        let n = def_to_npc(&def, &m, false).unwrap();
        (m, n)
    }

    #[test]
    fn follows_schedule_over_the_day() {
        let (m, mut n) = tiny();
        let mut clock = GameClock::default();
        let mut rng = Rng::new(1);
        clock.set_hm(1, 0); // sleep slot: home. Player far away -> teleports home.
        update_npc(&mut n, &m, &clock, 0.1, [500.0, 500.0], &mut rng);
        assert_eq!(tile_of(n.pos), TilePos::new(5, 5));
        // at 10:00 the work slot starts; player nearby, so the NPC walks there
        clock.set_hm(10, 0);
        for _ in 0..4000 {
            update_npc(&mut n, &m, &clock, 0.1, [10.0, 10.0], &mut rng);
        }
        assert_eq!(tile_of(n.pos), TilePos::new(20, 5));
        assert_eq!(n.activity, Activity::Forge);
    }

    #[test]
    fn party_member_follows_and_teleports() {
        let (m, mut n) = tiny();
        n.in_party = true;
        n.pos = [5.5, 5.5];
        let clock = GameClock::default();
        let mut rng = Rng::new(1);
        for _ in 0..600 {
            update_npc(&mut n, &m, &clock, 0.05, [14.5, 8.5], &mut rng);
        }
        let d = ((n.pos[0] - 14.5).powi(2) + (n.pos[1] - 8.5).powi(2)).sqrt();
        assert!(d < 3.2, "dist {d}");
        // far away: snaps next to the leader
        update_npc(&mut n, &m, &clock, 0.05, [55.0, 55.0], &mut rng);
        assert!((n.pos[0] - 56.0).abs() < 0.1);
    }

    #[test]
    fn builds_named_generic_and_party_npcs() {
        let db = Db::builtin().unwrap();
        let w = u67_mapgen::generate_world(67);
        let list = build_npcs(&db, &w, "midgard", [100.0, 100.0], &[], 67);
        assert!(list.iter().any(|n| n.id == "eirikr"));
        assert!(list.iter().filter(|n| n.id.contains("villager")).count() >= 30);
        assert!(list.iter().any(|n| n.arch == "warden_guard"));
        let on_moon = build_npcs(&db, &w, "maani", [50.0, 50.0], &["sigrun".to_string()], 67);
        assert!(on_moon.iter().any(|n| n.id == "sigrun" && n.in_party));
        assert!(!on_moon.iter().any(|n| n.id == "eirikr"));
        for n in &list {
            assert!(db.dialogues.contains_key(&n.dialogue), "{}: {}", n.id, n.dialogue);
        }
    }

    #[test]
    fn finds_nearest_npc() {
        let (_, n) = tiny();
        let mut a = n.clone();
        a.pos = [10.0, 10.0];
        let mut b = n;
        b.pos = [10.5, 10.0];
        assert_eq!(npc_near(&[a, b], [10.6, 10.0], 1.5), Some(1));
        assert_eq!(npc_near(&[], [0.0, 0.0], 1.0), None);
    }
}
