//! A* on a map (8-directional, no corner cutting).
use crate::map::Map;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use u67_core::TilePos;

const DIRS: [(i32, i32); 8] = [(0, -1), (1, 0), (0, 1), (-1, 0), (1, -1), (1, 1), (-1, 1), (-1, -1)];

/// Returns the path excluding `from`, including `to`. `limit` caps explored nodes.
pub fn find_path(map: &Map, from: TilePos, to: TilePos, limit: usize) -> Option<Vec<TilePos>> {
    if from == to {
        return Some(vec![]);
    }
    if !map.walkable(to) {
        return None;
    }
    let h = |p: TilePos| (p.x - to.x).abs().max((p.y - to.y).abs()) as u32;
    let mut open = BinaryHeap::new();
    let mut g: HashMap<TilePos, u32> = HashMap::new();
    let mut came: HashMap<TilePos, TilePos> = HashMap::new();
    g.insert(from, 0);
    open.push(Reverse((h(from), from.x, from.y)));
    let mut explored = 0;
    while let Some(Reverse((_, x, y))) = open.pop() {
        let cur = TilePos::new(x, y);
        if cur == to {
            let mut path = vec![cur];
            let mut c = cur;
            while let Some(&p) = came.get(&c) {
                if p == from {
                    break;
                }
                path.push(p);
                c = p;
            }
            path.reverse();
            return Some(path);
        }
        explored += 1;
        if explored > limit {
            return None;
        }
        let gc = g[&cur];
        for (i, (dx, dy)) in DIRS.iter().enumerate() {
            let n = TilePos::new(cur.x + dx, cur.y + dy);
            if !map.walkable(n) {
                continue;
            }
            if i >= 4 && (!map.walkable(TilePos::new(cur.x + dx, cur.y)) || !map.walkable(TilePos::new(cur.x, cur.y + dy))) {
                continue;
            }
            let ng = gc + if i < 4 { 10 } else { 14 };
            if g.get(&n).is_none_or(|&old| ng < old) {
                g.insert(n, ng);
                came.insert(n, cur);
                open.push(Reverse((ng + h(n) * 10, n.x, n.y)));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiles;
    #[test]
    fn routes_around_wall() {
        let mut m = Map::new("t", 10, 10, tiles::GRASS);
        for y in 0..9 {
            m.set_tile(TilePos::new(5, y), tiles::WALL_STONE);
        }
        let p = find_path(&m, TilePos::new(2, 2), TilePos::new(8, 2), 5000).unwrap();
        assert_eq!(*p.last().unwrap(), TilePos::new(8, 2));
        assert!(p.iter().any(|t| t.y == 9));
        assert!(p.iter().all(|t| m.walkable(*t)));
    }
    #[test]
    fn unreachable() {
        let mut m = Map::new("t", 10, 10, tiles::GRASS);
        for y in 0..10 {
            m.set_tile(TilePos::new(5, y), tiles::WALL_STONE);
        }
        assert!(find_path(&m, TilePos::new(2, 2), TilePos::new(8, 2), 5000).is_none());
    }
}
