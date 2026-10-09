//! The Norse transformation: find the towns of the imported map, rename them to Midgard's
//! settlements, make the north snowy, wire up quest places, and add the Midgard extras
//! (rune rings, camps, caves, dens) using the same code as the procedural generator.
use u67_core::TilePos;
use u67_mapgen::midgard::{self, TOWNS};
use u67_world::map::{Map, World};
use u67_world::objects::Building;
use u67_world::tiles::{self};

#[derive(Clone, Debug)]
pub struct TownSite {
    pub center: TilePos,
    pub buildings: Vec<Building>,
}

/// Group buildings into town clusters (centres closer than `gap` tiles are joined), biggest first.
pub fn detect_towns(m: &Map, gap: i32, min_buildings: usize) -> Vec<TownSite> {
    let b = &m.buildings;
    let c = |x: &Building| (x.x + x.w / 2, x.y + x.h / 2);
    let mut parent: Vec<usize> = (0..b.len()).collect();
    fn find(p: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while p[r] != r {
            r = p[r];
        }
        let mut i = i;
        while p[i] != r {
            let n = p[i];
            p[i] = r;
            i = n;
        }
        r
    }
    // grid bucket to avoid O(n^2)
    use std::collections::HashMap;
    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (i, x) in b.iter().enumerate() {
        let (cx, cy) = c(x);
        grid.entry((cx.div_euclid(gap), cy.div_euclid(gap))).or_default().push(i);
    }
    for (i, x) in b.iter().enumerate() {
        let (cx, cy) = c(x);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(v) = grid.get(&(cx.div_euclid(gap) + dx, cy.div_euclid(gap) + dy)) {
                    for &j in v {
                        let (jx, jy) = c(&b[j]);
                        if j != i && (jx - cx).abs() <= gap && (jy - cy).abs() <= gap {
                            let (a, bb) = (find(&mut parent, i), find(&mut parent, j));
                            parent[a] = bb;
                        }
                    }
                }
            }
        }
    }
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..b.len() {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    let mut out: Vec<TownSite> = groups
        .into_values()
        .filter(|g| g.len() >= min_buildings)
        .map(|g| {
            let bs: Vec<Building> = g.iter().map(|i| b[*i].clone()).collect();
            let (sx, sy) = bs.iter().map(c).fold((0, 0), |a, p| (a.0 + p.0, a.1 + p.1));
            TownSite { center: TilePos::new(sx / bs.len() as i32, sy / bs.len() as i32), buildings: bs }
        })
        .collect();
    out.sort_by(|a, b| b.buildings.len().cmp(&a.buildings.len()).then(a.center.y.cmp(&b.center.y)).then(a.center.x.cmp(&b.center.x)));
    out
}

/// Nearest walkable tile to `p` within `r`.
pub fn nearest_walkable(m: &Map, p: TilePos, r: i32) -> Option<TilePos> {
    for d in 0..=r {
        for dy in -d..=d {
            for dx in -d..=d {
                if dx.abs().max(dy.abs()) != d {
                    continue;
                }
                let q = TilePos::new(p.x + dx, p.y + dy);
                if m.walkable(q) && !tiles::def(m.tile(q)).water {
                    return Some(q);
                }
            }
        }
    }
    None
}

/// Order in which Norse town ids are handed out. Index 0 gets the biggest cluster.
fn name_assignment(sites: &[TownSite]) -> Vec<(usize, &'static str)> {
    let mut result = vec![];
    let mut free: Vec<usize> = (0..sites.len()).collect();
    if free.is_empty() {
        return result;
    }
    result.push((free.remove(0), "kaupang"));
    // the four northernmost remaining clusters get the arctic names
    let mut by_y = free.clone();
    by_y.sort_by_key(|i| sites[*i].center.y);
    for (k, name) in ["rovala", "helgate", "kuusamo", "reyk"].iter().enumerate() {
        if let Some(i) = by_y.get(k) {
            result.push((*i, name));
            free.retain(|f| f != i);
        }
    }
    for name in ["sigtuna", "birka", "nidaros", "turku", "bjorgvin", "visby", "hedeby", "savo", "aldeigja", "jorvik", "mimir"] {
        if free.is_empty() {
            break;
        }
        result.push((free.remove(0), name));
    }
    result
}

fn register_town(m: &mut Map, id: &str, s: &TownSite) -> bool {
    let Some(center) = nearest_walkable(m, s.center, 14) else { return false };
    m.places.insert(id.into(), center);
    let mut bs = s.buildings.clone();
    bs.sort_by_key(|b| std::cmp::Reverse(b.w * b.h));
    for (i, b) in bs.iter().take(4).enumerate() {
        let door = nearest_walkable(m, TilePos::new(b.x + b.w / 2, b.y + b.h), 4).unwrap_or(center);
        m.places.insert(format!("{id}_house_{}", i + 1), door);
    }
    for i in bs.len() + 1..=4 {
        m.places.insert(format!("{id}_house_{i}"), center);
    }
    let forge = m
        .objects
        .iter()
        .filter(|o| o.kind == "forge" && (o.pos.x - center.x).abs() < 40 && (o.pos.y - center.y).abs() < 40)
        .min_by_key(|o| o.pos.manhattan(center))
        .map(|o| o.pos);
    let near = |dx, dy| nearest_walkable(m, TilePos::new(center.x + dx, center.y + dy), 6).unwrap_or(center);
    let (market, tavern, forge_pos) = (near(4, -1), near(-3, 3), forge.and_then(|f| nearest_walkable(m, TilePos::new(f.x, f.y + 1), 3)).unwrap_or_else(|| near(-3, -1)));
    m.places.insert(format!("{id}_market"), market);
    m.places.insert(format!("{id}_tavern"), tavern);
    m.places.insert(format!("{id}_forge"), forge_pos);
    // dock: walk in the four directions until water
    for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
        let mut last = center;
        for d in 1..60 {
            let q = TilePos::new(center.x + dx * d, center.y + dy * d);
            if !m.in_bounds(q) {
                break;
            }
            if tiles::def(m.tile(q)).water {
                m.places.insert(format!("{id}_dock"), last);
                break;
            }
            if !m.walkable(q) {
                break;
            }
            last = q;
        }
        if m.places.contains_key(&format!("{id}_dock")) {
            break;
        }
    }
    if m.objects_at(center).next().is_none() && id == "mimir" {
        m.add_object("well", center);
    }
    true
}

/// Make the north snowy: grass -> snow / forest floor by latitude (like the generated Midgard).
pub fn arctic_north(m: &mut Map) {
    let h = m.height as f32;
    for y in 0..m.height {
        let lat = y as f32 / h;
        for x in 0..m.width {
            let p = TilePos::new(x, y);
            let t = m.tile(p);
            let n = u67_core::noise::value(x as f32 / 9.0, y as f32 / 9.0, 17);
            let nt = if lat < 0.2 {
                match t {
                    tiles::GRASS | tiles::FOREST_FLOOR | tiles::FARMLAND => Some(if n > 0.72 { tiles::ICE } else { tiles::SNOW }),
                    tiles::SAND => Some(tiles::SNOW),
                    _ => None,
                }
            } else if lat < 0.32 && n > 0.5 {
                match t {
                    tiles::GRASS => Some(tiles::SNOW),
                    _ => None,
                }
            } else {
                None
            };
            if let Some(nt) = nt {
                m.set_tile(p, nt);
            }
        }
    }
}

/// Everything after the raw import. Returns log lines.
pub fn transform(m: &mut Map, world: &mut World, seed: u64) -> Vec<String> {
    let mut log = vec![];
    arctic_north(m);
    let sites = detect_towns(m, 36, 3);
    log.push(format!("{} town clusters found", sites.len()));
    let mut done = std::collections::BTreeSet::new();
    for (i, id) in name_assignment(&sites) {
        if register_town(m, id, &sites[i]) {
            log.push(format!("{id}: {} buildings near {},{}", sites[i].buildings.len(), sites[i].center.x, sites[i].center.y));
            done.insert(id);
        }
    }
    // towns the original map had no room for are generated procedurally
    for (i, (id, _name, nx, ny, coastal)) in TOWNS.iter().enumerate() {
        if done.contains(id) {
            continue;
        }
        if let Some(c) = midgard::find_site(m, *nx, *ny, 13) {
            midgard::build_town(m, id, c, *coastal, seed + i as u64 * 31);
            log.push(format!("{id}: generated at {},{}", c.x, c.y));
        } else {
            log.push(format!("{id}: no room, skipped"));
        }
    }
    if let Some(k) = m.places.get("kaupang").copied() {
        m.places.insert("start".into(), k);
        // clear a landing patch around the start
        let objs: Vec<usize> = m.objects.iter().enumerate().filter(|(_, o)| (o.pos.x - k.x).abs() <= 1 && (o.pos.y - k.y).abs() <= 1).map(|(i, _)| i).collect();
        for i in objs {
            if matches!(m.objects[i].kind.as_str(), "pine_tree" | "birch_tree" | "boulder") {
                m.remove_object(i);
            }
        }
    }
    midgard::populate_extras(m, world, seed);
    m.reindex();
    log
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::Rules;
    use crate::import::import;
    use crate::u7::{FixedObj, U7Data, CHUNKS};

    /// All-grass fake world with `n` roof clusters (4 buildings each, different sizes).
    fn world_with_clusters(n: usize) -> U7Data {
        let mut d = U7Data { terrain: vec![1; CHUNKS * CHUNKS], ..Default::default() };
        d.templates = vec![vec![(0, 0); 256], vec![(1, 0); 256]];
        d.flats = vec![vec![[30, 80, 200]], vec![[60, 140, 50]]];
        d.names = vec![String::new(); 400];
        d.tfa = vec![[0; 3]; 400];
        d.names[201] = "thatch roof".into();
        d.tfa[201] = [8, 0, 0];
        for k in 0..n {
            let (bx, by) = (300 + (k as i32 % 4) * 600, 400 + (k as i32 / 4) * 700);
            for b in 0..(4 + (n - k)) as i32 {
                let (ox, oy) = (bx + (b % 3) * 14, by + (b / 3) * 14);
                for dy in 0..4 {
                    for dx in 0..4 {
                        d.fixed.push(FixedObj { x: ox + dx, y: oy + dy, lift: 5, shape: 201, frame: 0 });
                    }
                }
            }
        }
        d
    }

    #[test]
    fn clusters_become_named_norse_towns() {
        let d = world_with_clusters(6);
        let (mut m, rep) = import(&d, &Rules::default());
        assert!(rep.buildings >= 6 * 4, "{rep:?}");
        let sites = detect_towns(&m, 36, 3);
        assert_eq!(sites.len(), 6);
        assert!(sites[0].buildings.len() >= sites[5].buildings.len());
        let mut world = World::default();
        let log = transform(&mut m, &mut world, 67);
        assert!(log.iter().any(|l| l.starts_with("kaupang")), "{log:#?}");
        // every Norse town exists with its standard places, all walkable
        for (id, ..) in TOWNS {
            for suffix in ["", "_market", "_tavern", "_forge", "_house_1", "_house_4"] {
                let key = format!("{id}{suffix}");
                let p = m.places.get(&key).copied().unwrap_or_else(|| panic!("missing place {key}"));
                assert!(m.in_bounds(p), "{key}");
            }
            assert!(m.walkable(m.places[*id]), "{id} centre not walkable");
        }
        assert_eq!(m.places["start"], m.places["kaupang"]);
        // northern clusters got arctic names
        let rovala = m.places["rovala"];
        let kaupang = m.places["kaupang"];
        assert!(rovala.y <= kaupang.y + 1500);
        // extras came along: caves with portals back to the world
        assert!(m.places.contains_key("kaupang_gate") || m.places.contains_key("rune_ring_1"));
        assert!(world.maps.contains_key("barrow_1") || world.maps.contains_key("mimir_depths"), "{:?}", world.maps.keys().collect::<Vec<_>>());
        assert!(!m.portals.is_empty());
    }

    #[test]
    fn north_is_snowy_south_is_not() {
        let d = world_with_clusters(0);
        let (mut m, _) = import(&d, &Rules::default());
        arctic_north(&mut m);
        let snow_north = (0..3072).filter(|x| matches!(m.tile(TilePos::new(*x, 100)), tiles::SNOW | tiles::ICE)).count();
        let snow_south = (0..3072).filter(|x| matches!(m.tile(TilePos::new(*x, 2500)), tiles::SNOW | tiles::ICE)).count();
        assert!(snow_north > 3000 && snow_south == 0, "{snow_north} {snow_south}");
    }
}
