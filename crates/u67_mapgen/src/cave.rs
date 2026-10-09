//! Cellular-automata caves for dungeons and barrows.
use crate::sites;
use u67_core::{Rng, TilePos};
use u67_world::map::Map;
use u67_world::tiles;

pub struct CaveSpec {
    pub name: String,
    pub size: i32,
    pub floor: u16,
    pub wall: u16,
    pub seed: u64,
    pub chests: i32,
}

pub fn cave(spec: &CaveSpec) -> Map {
    let n = spec.size;
    let mut rng = Rng::new(spec.seed);
    let mut wall = vec![false; (n * n) as usize];
    let at = |x: i32, y: i32| (y * n + x) as usize;
    for y in 0..n {
        for x in 0..n {
            wall[at(x, y)] = x < 2 || y < 2 || x >= n - 2 || y >= n - 2 || rng.chance(0.46);
        }
    }
    for _ in 0..5 {
        let old = wall.clone();
        for y in 1..n - 1 {
            for x in 1..n - 1 {
                let mut c = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if old[at(x + dx, y + dy)] {
                            c += 1;
                        }
                    }
                }
                wall[at(x, y)] = c >= 5;
            }
        }
    }
    // entrance column at bottom-centre, carve a corridor up
    let ex = n / 2;
    for y in (n - 12)..n - 1 {
        for dx in -1..=1 {
            wall[at(ex + dx, y)] = false;
        }
    }
    // keep only the region connected to the entrance
    let start = (ex, n - 3);
    let mut seen = vec![false; (n * n) as usize];
    let mut stack = vec![start];
    seen[at(start.0, start.1)] = true;
    let mut far = (start, 0);
    while let Some((x, y)) = stack.pop() {
        let d = (y - start.1).abs() + (x - start.0).abs();
        if d > far.1 {
            far = ((x, y), d);
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx > 0 && ny > 0 && nx < n - 1 && ny < n - 1 && !wall[at(nx, ny)] && !seen[at(nx, ny)] {
                seen[at(nx, ny)] = true;
                stack.push((nx, ny));
            }
        }
    }
    let mut m = Map::new(&spec.name, n, n, tiles::TileId(spec.wall));
    let mut floors = vec![];
    for y in 0..n {
        for x in 0..n {
            if seen[at(x, y)] {
                m.set_tile(TilePos::new(x, y), tiles::TileId(spec.floor));
                floors.push(TilePos::new(x, y));
            }
        }
    }
    m.places.insert("entrance".into(), TilePos::new(start.0, start.1));
    m.places.insert("boss".into(), TilePos::new(far.0 .0, far.0 .1));
    for _ in 0..spec.chests {
        let q = *rng.pick(&floors);
        if q.manhattan(TilePos::new(start.0, start.1)) > 6 {
            sites::cache(&mut m, q, &mut rng);
        }
    }
    for _ in 0..(n / 4) {
        let q = *rng.pick(&floors);
        if q.manhattan(TilePos::new(start.0, start.1)) > 6 {
            m.add_object("boulder", q);
        }
    }
    sites::camp(&mut m, TilePos::new(far.0 .0, far.0 .1), &mut rng);
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use u67_world::pathfind::find_path;
    #[test]
    fn cave_connected_to_entrance_and_boss() {
        let m = cave(&CaveSpec { name: "c".into(), size: 80, floor: tiles::ROCK.0, wall: tiles::MOUNTAIN.0, seed: 3, chests: 5 });
        let e = m.places["entrance"];
        let b = m.places["boss"];
        assert!(m.walkable(e));
        assert!(find_path(&m, e, b, 200_000).is_some());
    }
}
