//! Reusable hand-built structures: houses, rune rings, bunkers, wreck sites, caches.
use u67_core::{Rng, TilePos};
use u67_world::map::Map;
use u67_world::objects::Building;
use u67_world::tiles::{self, TileId};

fn p(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

/// Build a wooden house with footprint (x,y,w,h). Door in the south wall. Returns the building id.
pub fn house(m: &mut Map, x: i32, y: i32, w: i32, h: i32, wall: TileId, floor: TileId, furnish: &mut Rng) -> u16 {
    let id = m.buildings.len() as u16 + 1;
    for j in 0..h {
        for i in 0..w {
            let edge = i == 0 || j == 0 || i == w - 1 || j == h - 1;
            m.set_tile(p(x + i, y + j), if edge { wall } else { floor });
        }
    }
    let door = p(x + w / 2, y + h - 1);
    m.set_tile(door, floor);
    m.add_object_in("door_wood", door, id);
    // roofs cover the interior in 2x2 chunks (roof sprite is 32px = 2 tiles)
    for j in (0..h).step_by(2) {
        for i in (0..w).step_by(2) {
            m.add_object_in("longhouse_roof", p(x + i, y + j), id);
        }
    }
    // furniture
    let inside = |i: i32, j: i32| p(x + 1 + i.rem_euclid(w - 2), y + 1 + j.rem_euclid(h - 2));
    m.add_object("bed", inside(0, 0));
    m.add_object("table", inside(w / 2, h / 2 - 1));
    if furnish.chance(0.7) {
        m.add_object("chest", inside(w - 3, 0));
    }
    if furnish.chance(0.5) {
        m.add_object("barrel", inside(w - 3, h - 3));
    }
    m.buildings.push(Building { id, x, y, w, h });
    id
}

/// Ring of runestones around a bifrost node.
pub fn rune_ring(m: &mut Map, c: TilePos, r: i32, stones: i32) {
    m.set_tile(c, tiles::FLOOR_STONE);
    m.add_object("bifrost_node", c);
    for k in 0..stones {
        let a = k as f32 / stones as f32 * std::f32::consts::TAU;
        let q = p(c.x + (a.cos() * r as f32).round() as i32, c.y + (a.sin() * r as f32).round() as i32);
        if m.walkable(q) {
            m.add_object("runestone", q);
        }
    }
}

/// Small stone bunker with a chest and forge.
pub fn bunker(m: &mut Map, x: i32, y: i32, rng: &mut Rng) {
    house(m, x, y, 9, 7, tiles::WALL_STONE, tiles::FLOOR_STONE, rng);
    m.add_object("forge", p(x + 2, y + 2));
    m.add_object("chest", p(x + 6, y + 2));
}

pub fn wreck_site(m: &mut Map, c: TilePos, rng: &mut Rng) {
    m.add_object("wreck", c);
    for _ in 0..4 {
        let q = p(c.x + rng.range(-3, 4), c.y + rng.range(-3, 4));
        if m.walkable(q) {
            m.add_object(if rng.chance(0.5) { "crate" } else { "barrel" }, q);
        }
    }
    let q = p(c.x + 2, c.y + 1);
    if m.walkable(q) {
        m.add_object("chest", q);
    }
}

pub fn cache(m: &mut Map, c: TilePos, rng: &mut Rng) {
    m.add_object("chest", c);
    for d in [(-1, 0), (1, 0), (0, -1)] {
        let q = p(c.x + d.0, c.y + d.1);
        if m.walkable(q) && rng.chance(0.8) {
            m.add_object("boulder", q);
        }
    }
}

/// Camp with fire and crates.
pub fn camp(m: &mut Map, c: TilePos, rng: &mut Rng) {
    m.add_object("campfire", c);
    for d in [(-2, 1), (2, 0), (0, 2)] {
        let q = p(c.x + d.0, c.y + d.1);
        if m.walkable(q) && rng.chance(0.7) {
            m.add_object(if rng.chance(0.5) { "crate" } else { "barrel" }, q);
        }
    }
}
