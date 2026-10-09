//! Procedural Midgard: a Norse/Finnish remake of the Ultima 7 world's macro layout —
//! a big continent with fjords and lakes, snowy north, farmland south, islands, ~16 towns.
use crate::{cave, sites};
use u67_core::noise::fbm;
use u67_core::{Rng, TilePos};
use u67_world::map::{Map, World};
use u67_world::objects::Portal;
use u67_world::pathfind::find_path;
use u67_world::tiles::{self, TileId};

pub const SIZE: i32 = 768;

/// (id, display name, nx, ny in 0..1, coastal)
pub const TOWNS: &[(&str, &str, f32, f32, bool)] = &[
    ("kaupang", "Kaupang", 0.50, 0.78, true),
    ("birka", "Birka", 0.74, 0.62, true),
    ("hedeby", "Hedeby", 0.36, 0.84, true),
    ("sigtuna", "Sigtuna", 0.58, 0.50, false),
    ("nidaros", "Nidaros", 0.30, 0.36, true),
    ("bjorgvin", "Bjorgvin", 0.18, 0.58, true),
    ("visby", "Visby", 0.86, 0.80, true),
    ("turku", "Turku", 0.66, 0.38, true),
    ("savo", "Savo", 0.52, 0.30, false),
    ("kuusamo", "Kuusamo", 0.62, 0.20, false),
    ("rovala", "Rovala", 0.50, 0.10, false),
    ("aldeigja", "Aldeigja", 0.80, 0.42, true),
    ("reyk", "Reyk", 0.12, 0.14, true),
    ("helgate", "Helgate", 0.30, 0.18, false),
    ("jorvik", "Jorvik", 0.24, 0.78, true),
    ("mimir", "Mimir's Well", 0.46, 0.52, false),
];

struct Terrain {
    h: Vec<f32>,
}

fn height(x: i32, y: i32, seed: u64) -> f32 {
    let n = SIZE as f32;
    let (fx, fy) = (x as f32 / n, y as f32 / n);
    let warp = fbm(fx * 4.0, fy * 4.0, seed + 5, 3) * 0.12;
    let (dx, dy) = ((fx - 0.5 + warp - 0.06) / 0.50, (fy - 0.52) / 0.48);
    let mut mask = 1.0 - (dx * dx + dy * dy).powf(0.9);
    // islands
    for (cx, cy, r) in [(0.12f32, 0.14f32, 0.07f32), (0.88, 0.80, 0.07), (0.07, 0.62, 0.05), (0.93, 0.38, 0.04), (0.5, 0.97, 0.05)] {
        let d = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt() / r;
        mask = mask.max(1.0 - d * d * 0.9);
    }
    0.5 * fbm(fx * 9.0, fy * 9.0, seed, 5) + 0.62 * mask
}

fn classify(x: i32, y: i32, hh: f32, seed: u64) -> TileId {
    let (nx, ny) = (x as f32, y as f32);
    // wobbly latitude so biome borders are not ruler-straight
    let fy = y as f32 / SIZE as f32 + (fbm(nx / 55.0, ny / 55.0, seed + 61, 3) - 0.5) * 0.14;
    // fjords & lakes
    let ridge = (fbm(nx / 60.0, ny / 60.0, seed + 11, 3) - 0.5).abs();
    let lake = fbm(nx / 48.0, ny / 48.0, seed + 21, 3);
    if hh < 0.40 {
        return tiles::WATER_DEEP;
    }
    if hh < 0.45 {
        return tiles::WATER_SHALLOW;
    }
    if hh < 0.62 && ridge < 0.035 {
        return if ridge < 0.015 { tiles::WATER_DEEP } else { tiles::WATER_SHALLOW };
    }
    if hh > 0.60 && lake > 0.70 {
        return if lake > 0.76 { tiles::WATER_DEEP } else { tiles::WATER_SHALLOW };
    }
    let range = fbm(nx / 45.0, ny / 45.0, seed + 71, 4);
    if hh > 0.97 || (hh > 0.62 && range > 0.74) {
        return tiles::MOUNTAIN;
    }
    if hh > 0.88 || (hh > 0.58 && range > 0.68) {
        return if fy < 0.4 { tiles::SNOW } else { tiles::ROCK };
    }
    if hh < 0.475 {
        return if fy < 0.25 { tiles::SNOW } else { tiles::SAND };
    }
    let moist = fbm(nx / 40.0, ny / 40.0, seed + 31, 3);
    if fy < 0.22 {
        return if moist > 0.62 { tiles::ICE } else { tiles::SNOW };
    }
    if fy < 0.34 {
        return if moist > 0.5 { tiles::SNOW } else { tiles::FOREST_FLOOR };
    }
    if moist > 0.74 && hh < 0.6 {
        return tiles::SWAMP;
    }
    if fy > 0.72 && fbm(nx / 30.0, ny / 30.0, seed + 41, 2) > 0.58 {
        return tiles::FARMLAND;
    }
    if moist > 0.5 {
        tiles::FOREST_FLOOR
    } else {
        tiles::GRASS
    }
}

pub fn buildable(m: &Map, x: i32, y: i32, r: i32) -> bool {
    for j in (-r..=r).step_by(2) {
        for i in (-r..=r).step_by(2) {
            let t = m.tile(TilePos::new(x + i, y + j));
            let d = tiles::def(t);
            if !d.walkable || d.water || t == tiles::MOUNTAIN {
                return false;
            }
        }
    }
    true
}

pub fn find_site(m: &Map, nx: f32, ny: f32, r: i32) -> Option<TilePos> {
    let (cx, cy) = ((nx * m.width as f32) as i32, (ny * m.height as f32) as i32);
    for rad in (0..120).step_by(3) {
        for k in 0..(8 + rad / 2) {
            let a = k as f32 / (8 + rad / 2) as f32 * std::f32::consts::TAU;
            let (x, y) = (cx + (a.cos() * rad as f32) as i32, cy + (a.sin() * rad as f32) as i32);
            if buildable(m, x, y, r) {
                return Some(TilePos::new(x, y));
            }
        }
    }
    None
}

pub fn build_town(m: &mut Map, id: &str, c: TilePos, coastal: bool, seed: u64) {
    let mut rng = Rng::new(seed);
    let r = 11;
    for j in -r..=r {
        for i in -r..=r {
            let q = TilePos::new(c.x + i, c.y + j);
            m.set_tile(q, if i.abs() <= 1 || j.abs() <= 1 { tiles::ROAD } else { tiles::GRASS });
        }
    }
    // houses in the four quadrants
    let spots = [(-9, -8), (3, -8), (-9, 3), (3, 3), (-4, -8), (-4, 4)];
    let mut n = 0;
    for (k, (dx, dy)) in spots.iter().enumerate() {
        if k >= 4 && rng.chance(0.5) {
            continue;
        }
        let (w, h) = (6 + rng.range(0, 2), 5 + rng.range(0, 2));
        let (x, y) = (c.x + dx, c.y + dy);
        let bid = sites::house(m, (x, y, w, h), tiles::WALL_WOOD, tiles::FLOOR_WOOD, &mut rng);
        n += 1;
        m.places.insert(format!("{id}_house_{n}"), TilePos::new(x + w / 2, y + h));
        let _ = bid;
    }
    m.add_object("well", c);
    m.add_object("signpost", TilePos::new(c.x + 2, c.y + r));
    m.add_object("forge", TilePos::new(c.x - 3, c.y - 2));
    m.add_object("campfire", TilePos::new(c.x + 3, c.y + 2));
    m.add_object("barrel", TilePos::new(c.x + 4, c.y - 2));
    m.add_object("crate", TilePos::new(c.x + 5, c.y - 2));
    m.places.insert(id.into(), TilePos::new(c.x, c.y + 2));
    m.places.insert(format!("{id}_market"), TilePos::new(c.x + 3, c.y - 1));
    m.places.insert(format!("{id}_tavern"), TilePos::new(c.x - 2, c.y + 3));
    m.places.insert(format!("{id}_forge"), TilePos::new(c.x - 3, c.y - 1));
    if coastal {
        // try a dock to the nearest water in 4 directions
        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
            for d in (r + 1)..(r + 40) {
                let q = TilePos::new(c.x + dx * d, c.y + dy * d);
                if tiles::def(m.tile(q)).water {
                    for k in 0..6 {
                        let w = TilePos::new(q.x + dx * k, q.y + dy * k);
                        m.set_tile(w, tiles::FLOOR_WOOD);
                    }
                    m.add_object("longship", TilePos::new(q.x + dx * 7 + dy, q.y + dy * 7 + dx));
                    m.places.insert(format!("{id}_dock"), q);
                    return;
                }
                if !tiles::def(m.tile(q)).walkable {
                    break;
                }
            }
        }
    }
}

fn lay_roads(m: &mut Map, towns: &[(String, TilePos)]) {
    // connect each town to its nearest already-connected town (Prim-like)
    let mut connected = vec![0usize];
    let mut rest: Vec<usize> = (1..towns.len()).collect();
    while !rest.is_empty() {
        let mut best = (0usize, 0usize, i32::MAX);
        for &a in &connected {
            for (ri, &b) in rest.iter().enumerate() {
                let d = towns[a].1.manhattan(towns[b].1);
                if d < best.2 {
                    best = (a, ri, d);
                }
            }
        }
        let b = rest.remove(best.1);
        if let Some(path) = find_path(m, towns[best.0].1, towns[b].1, 400_000) {
            for q in path {
                let t = m.tile(q);
                if tiles::def(t).water {
                    m.set_tile(q, tiles::FLOOR_WOOD); // bridge
                } else if t != tiles::ROAD && t != tiles::FLOOR_WOOD && t != tiles::WALL_WOOD {
                    m.set_tile(q, tiles::ROAD);
                }
            }
        }
        connected.push(b);
    }
}

/// Rune rings, camps, wolf dens, the Kaupang gate, the world-serpent's nest, and the cave
/// dungeons with their portals. Works on any Midgard-shaped map (generated or imported).
pub fn populate_extras(m: &mut Map, world: &mut World, seed: u64) {
    // hotspots: rune rings, camps, caves
    let spot = |m: &Map, rng: &mut Rng, nx: f32, ny: f32, r: i32| find_site(m, nx + (rng.f32() - 0.5) * 0.03, ny + (rng.f32() - 0.5) * 0.03, r);
    let mut srng = Rng::new(seed + 99);
    let mid = m.places.get("mimir").copied();
    if let Some(mc) = mid {
        sites::rune_ring(m, TilePos::new(mc.x, mc.y - 8), 5, 8);
        m.places.insert("mimir_well".into(), mc);
    }
    for k in 0..10 {
        let (nx, ny) = (0.2 + srng.f32() * 0.65, 0.15 + srng.f32() * 0.7);
        if let Some(c) = spot(m, &mut srng, nx, ny, 6) {
            sites::rune_ring(m, c, 4, 6);
            m.places.insert(format!("rune_ring_{}", k + 1), c);
        }
    }
    for k in 0..6 {
        let (nx, ny) = (0.2 + srng.f32() * 0.65, 0.12 + srng.f32() * 0.75);
        if let Some(c) = spot(m, &mut srng, nx, ny, 4) {
            sites::camp(m, c, &mut srng);
            m.places.insert(format!("camp_{}", k + 1), c);
        }
    }
    for k in 0..4 {
        let (nx, ny) = (0.25 + srng.f32() * 0.5, 0.12 + srng.f32() * 0.25);
        if let Some(c) = spot(m, &mut srng, nx, ny, 3) {
            m.places.insert(format!("spawn_wolf_{}", k + 1), c);
        }
    }
    // the Kaupang gate (main quest) and the world-serpent's nest
    if let Some(c) = find_site(m, 0.51, 0.69, 8) {
        sites::rune_ring(m, c, 5, 8);
        m.places.insert("kaupang_gate".into(), c);
    }
    'nest: for gy in (0..m.height - 12).step_by(6).rev() {
        for gx in (m.width / 3..m.width * 2 / 3).step_by(6) {
            let ok = (0..9).all(|k| m.tile(TilePos::new(gx + k, gy + k)) == tiles::WATER_DEEP && m.tile(TilePos::new(gx + k, gy)) == tiles::WATER_DEEP);
            if ok {
                m.places.insert("jormungandr_nest".into(), TilePos::new(gx + 4, gy + 4));
                break 'nest;
            }
        }
    }
    // dungeon entrances (portals) -> cave maps
    let caves = [
        ("mimir_depths", 0.46f32, 0.46f32, tiles::ROCK, tiles::MOUNTAIN, 120),
        ("barrow_1", 0.62, 0.70, tiles::ROCK, tiles::MOUNTAIN, 80),
        ("barrow_2", 0.28, 0.62, tiles::ROCK, tiles::MOUNTAIN, 80),
        ("fenrir_den", 0.40, 0.10, tiles::ICE, tiles::MOUNTAIN, 110),
        ("troll_cave", 0.72, 0.28, tiles::ROCK, tiles::MOUNTAIN, 90),
        ("hel_gate_crypt", 0.34, 0.24, tiles::ASH, tiles::MOUNTAIN, 100),
    ];
    for (i, (name, nx, ny, floor, wall, size)) in caves.iter().enumerate() {
        if let Some(c) = find_site(m, *nx, *ny, 3) {
            let cv = cave::cave(&cave::CaveSpec { name: name.to_string(), size: *size, floor: floor.0, wall: wall.0, seed: seed + 700 + i as u64, chests: 6 + i as i32 });
            let door = TilePos::new(c.x, c.y);
            m.set_tile(door, tiles::FLOOR_STONE);
            m.add_object("boulder", TilePos::new(c.x - 1, c.y));
            m.add_object("boulder", TilePos::new(c.x + 1, c.y));
            let entrance = cv.places["entrance"];
            m.portals.push(Portal { pos: door, target_map: name.to_string(), target_pos: entrance });
            let mut cv = cv;
            cv.portals.push(Portal { pos: TilePos::new(entrance.x, entrance.y + 1), target_map: "midgard".into(), target_pos: TilePos::new(door.x, door.y + 1) });
            cv.set_tile(TilePos::new(entrance.x, entrance.y + 1), tiles::FLOOR_STONE);
            m.places.insert(name.to_string(), TilePos::new(door.x, door.y + 1));
            world.insert(cv);
        }
    }
    // player start: south of Kaupang's market
    if let Some(k) = m.places.get("kaupang").copied() {
        m.places.insert("start".into(), k);
    }
}

pub fn generate(seed: u64, world: &mut World) -> Map {
    let mut m = Map::new("midgard", SIZE, SIZE, tiles::WATER_DEEP);
    let t = Terrain { h: (0..SIZE * SIZE).map(|i| height(i % SIZE, i / SIZE, seed)).collect() };
    for y in 0..SIZE {
        for x in 0..SIZE {
            m.set_tile(TilePos::new(x, y), classify(x, y, t.h[(y * SIZE + x) as usize], seed));
        }
    }
    // towns (placed before roads/trees so they are never blocked)
    let mut placed: Vec<(String, TilePos)> = vec![];
    for (i, (id, _name, nx, ny, coastal)) in TOWNS.iter().enumerate() {
        if let Some(c) = find_site(&m, *nx, *ny, 13) {
            build_town(&mut m, id, c, *coastal, seed + i as u64 * 31);
            placed.push((id.to_string(), TilePos::new(c.x, c.y + 12)));
        }
    }
    // keep road endpoints walkable: roads start at town south edge
    lay_roads(&mut m, &placed);
    let town_zones: Vec<TilePos> = placed.iter().map(|(_, p)| TilePos::new(p.x, p.y - 12)).collect();
    // trees & scenery
    let mut rng = Rng::new(seed ^ 0xF0F0);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let q = TilePos::new(x, y);
            let tile = m.tile(q);
            if tile == tiles::ROAD || town_zones.iter().any(|z| (z.x - x).abs() < 14 && (z.y - y).abs() < 14) {
                continue;
            }
            let dens = fbm(x as f32 / 25.0, y as f32 / 25.0, seed + 51, 3);
            let fy = y as f32 / SIZE as f32;
            match tile {
                tiles::FOREST_FLOOR if dens > 0.45 && rng.chance(0.30) => m.add_object(if fy > 0.55 && rng.chance(0.5) { "birch_tree" } else { "pine_tree" }, q),
                tiles::GRASS if dens > 0.60 && rng.chance(0.10) => m.add_object("birch_tree", q),
                tiles::SNOW if dens > 0.55 && rng.chance(0.07) => m.add_object("pine_tree", q),
                tiles::ROCK if rng.chance(0.03) => m.add_object("boulder", q),
                tiles::GRASS | tiles::FOREST_FLOOR if rng.chance(0.0006) => m.add_object("boulder", q),
                _ => {}
            }
        }
    }
    populate_extras(&mut m, world, seed);
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn midgard_has_towns_land_and_start() {
        let mut w = World::default();
        let m = generate(67, &mut w);
        for (id, ..) in TOWNS {
            assert!(m.places.contains_key(*id), "missing town {id}");
        }
        let land = m.tiles.iter().filter(|t| tiles::def(**t).walkable && !tiles::def(**t).water).count();
        let frac = land as f32 / m.tiles.len() as f32;
        assert!((0.25..0.8).contains(&frac), "land fraction {frac}");
        assert!(m.walkable(m.places["start"]));
        assert!(!w.maps.is_empty());
    }
    #[test]
    fn towns_are_connected_by_land() {
        let mut w = World::default();
        let m = generate(67, &mut w);
        let a = m.places["kaupang"];
        let b = m.places["sigtuna"];
        assert!(find_path(&m, a, b, 600_000).is_some());
    }
}
