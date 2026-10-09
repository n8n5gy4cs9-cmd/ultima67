//! Builds a Midgard `Map` from U7 data: terrain tiles, objects, walls and roofs.
use crate::classify::{classify_flat, Class, Rules};
use crate::u7::{U7Data, CHUNKS, TILES};
use u67_core::TilePos;
use u67_world::map::Map;
use u67_world::objects::Building;
use u67_world::tiles::{self, TileId};

#[derive(Debug, Default)]
pub struct Report {
    pub objects: usize,
    pub walls: usize,
    pub floors: usize,
    pub roofs: usize,
    pub buildings: usize,
    pub skipped: usize,
}

/// Flat shapes are classified once per (shape, frame).
fn tile_class_table(d: &U7Data) -> impl Fn(u16, u8) -> TileId + '_ {
    move |shape, frame| match d.flat_rgb(shape, frame) {
        Some(rgb) => classify_flat(rgb, d.water(shape)),
        None => {
            // not a flat shape: whatever it is, treat solid things as rock and water things as water
            if d.water(shape) {
                tiles::WATER_SHALLOW
            } else {
                tiles::GRASS
            }
        }
    }
}

/// Import terrain + objects. The map is `TILES` x `TILES` (3072) like the original.
pub fn import(d: &U7Data, rules: &Rules) -> (Map, Report) {
    let mut m = Map::new("midgard", TILES as i32, TILES as i32, tiles::GRASS);
    let class_of = tile_class_table(d);
    // classify each template tile once
    let tpl_tiles: Vec<Vec<TileId>> = d.templates.iter().map(|t| t.iter().map(|(s, f)| class_of(*s, *f)).collect()).collect();
    for cy in 0..CHUNKS {
        for cx in 0..CHUNKS {
            let id = d.terrain[cy * CHUNKS + cx] as usize;
            let Some(tpl) = tpl_tiles.get(id) else { continue };
            for ty in 0..16 {
                for tx in 0..16 {
                    m.tiles[(cy * 16 + ty) * TILES + cx * 16 + tx] = tpl[ty * 16 + tx];
                }
            }
        }
    }
    let mut rep = Report::default();
    let mut roofs: Vec<TilePos> = vec![];
    for o in &d.fixed {
        let p = TilePos::new(o.x, o.y);
        match rules.classify(d, o.shape, o.lift, (o.x * 31 + o.y * 17) as u32) {
            Class::Object(kind) => {
                if u67_world::objects::def(&kind).is_some() {
                    // ships only make sense on water
                    if kind == "longship" && !tiles::def(m.tile(p)).water {
                        rep.skipped += 1;
                        continue;
                    }
                    m.add_object(&kind, p);
                    rep.objects += 1;
                }
            }
            Class::Wall(t) => {
                m.set_tile(p, t);
                rep.walls += 1;
            }
            Class::Floor(t) => {
                m.set_tile(p, t);
                rep.floors += 1;
            }
            Class::Roof => roofs.push(p),
            Class::Skip => rep.skipped += 1,
        }
    }
    rep.roofs = roofs.len();
    rep.buildings = group_roofs(&mut m, &roofs);
    (m, rep)
}

/// Connected components of roof tiles (gaps of 1 allowed) become buildings; roof objects are placed on
/// every second tile (our roof sprite covers 2x2 tiles) so they hide together when the player is inside.
fn group_roofs(m: &mut Map, roofs: &[TilePos]) -> usize {
    use std::collections::{HashMap, HashSet};
    let set: HashSet<TilePos> = roofs.iter().copied().collect();
    let mut seen: HashSet<TilePos> = HashSet::new();
    let mut ids: HashMap<TilePos, u16> = HashMap::new();
    let mut next: u16 = m.buildings.len() as u16 + 1;
    let mut starts: Vec<TilePos> = roofs.to_vec();
    starts.sort_by_key(|p| (p.y, p.x));
    for s in starts {
        if seen.contains(&s) {
            continue;
        }
        let (mut minx, mut miny, mut maxx, mut maxy) = (s.x, s.y, s.x, s.y);
        let mut stack = vec![s];
        seen.insert(s);
        let id = next;
        next = next.saturating_add(1);
        let mut members = vec![];
        while let Some(p) = stack.pop() {
            members.push(p);
            minx = minx.min(p.x);
            maxx = maxx.max(p.x);
            miny = miny.min(p.y);
            maxy = maxy.max(p.y);
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let q = TilePos::new(p.x + dx, p.y + dy);
                    if set.contains(&q) && seen.insert(q) {
                        stack.push(q);
                    }
                }
            }
        }
        for p in members {
            ids.insert(p, id);
        }
        m.buildings.push(Building { id, x: minx - 1, y: miny - 1, w: maxx - minx + 3, h: maxy - miny + 3 });
    }
    // roof sprites on a 2x2 lattice anchored per building
    for (p, id) in ids {
        let b = m.buildings.iter().find(|b| b.id == id).cloned().unwrap();
        if (p.x - b.x).rem_euclid(2) == 1 && (p.y - b.y).rem_euclid(2) == 1 {
            m.add_object_in("longhouse_roof", p, id);
        }
    }
    m.buildings.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::u7::FixedObj;

    /// A tiny fake U7 world: templates 0 = water, 1 = grass; chunk (0,0) grass, rest water.
    pub fn fake() -> U7Data {
        let mut d = U7Data { terrain: vec![0; CHUNKS * CHUNKS], ..Default::default() };
        d.templates = vec![vec![(0, 0); 256], vec![(1, 0); 256]];
        d.terrain[0] = 1;
        d.flats = vec![vec![[30, 80, 200]], vec![[60, 140, 50]]];
        d.names = vec![String::new(); 400];
        d.tfa = vec![[0; 3]; 400];
        d.names[200] = "oak tree".into();
        d.names[201] = "thatch roof".into();
        d.tfa[201] = [8, 0, 0];
        d.names[202] = "stone wall".into();
        d
    }

    #[test]
    fn terrain_objects_walls_and_roofs() {
        let mut d = fake();
        d.fixed = vec![
            FixedObj { x: 3, y: 3, lift: 0, shape: 200, frame: 0 },
            FixedObj { x: 5, y: 5, lift: 0, shape: 202, frame: 0 },
            // a 4x4 roof block
            FixedObj { x: 8, y: 8, lift: 5, shape: 201, frame: 0 },
            FixedObj { x: 9, y: 8, lift: 5, shape: 201, frame: 0 },
            FixedObj { x: 8, y: 9, lift: 5, shape: 201, frame: 0 },
            FixedObj { x: 9, y: 9, lift: 5, shape: 201, frame: 0 },
            // an unrelated roof far away
            FixedObj { x: 100, y: 100, lift: 5, shape: 201, frame: 0 },
        ];
        let (m, rep) = import(&d, &Rules::default());
        assert_eq!(m.width, 3072);
        assert_eq!(m.tile(TilePos::new(0, 0)), tiles::GRASS);
        assert_eq!(m.tile(TilePos::new(16, 0)), tiles::WATER_SHALLOW);
        assert_eq!(m.tile(TilePos::new(5, 5)), tiles::WALL_STONE);
        assert_eq!(rep.objects, 1);
        assert_eq!((rep.walls, rep.roofs, rep.buildings), (1, 5, 2));
        assert!(m.objects.iter().any(|o| o.kind == "pine_tree" || o.kind == "birch_tree"));
        assert!(m.objects.iter().any(|o| o.kind == "longhouse_roof"));
        assert!(m.building_at(TilePos::new(9, 9)).is_some());
        assert!(m.building_at(TilePos::new(50, 50)).is_none());
        // trees block movement, walls too
        assert!(!m.walkable(TilePos::new(5, 5)));
    }
}
