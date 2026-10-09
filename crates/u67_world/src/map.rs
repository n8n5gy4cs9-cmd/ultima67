//! A map is a tile grid + objects + portals. Interiors/planets are separate maps.
use crate::objects::{self, Building, Portal, WorldObject};
use crate::tiles::{self, TileId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use u67_core::{TilePos, CHUNK};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TileId>,
    pub objects: Vec<WorldObject>,
    pub portals: Vec<Portal>,
    #[serde(default)]
    pub buildings: Vec<Building>,
    /// Named places for `tp <name>`.
    pub places: BTreeMap<String, TilePos>,
}

impl Map {
    pub fn new(name: &str, width: i32, height: i32, fill: TileId) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            tiles: vec![fill; (width * height) as usize],
            objects: vec![],
            portals: vec![],
            buildings: vec![],
            places: BTreeMap::new(),
        }
    }
    pub fn in_bounds(&self, p: TilePos) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.width && p.y < self.height
    }
    fn idx(&self, p: TilePos) -> usize {
        (p.y * self.width + p.x) as usize
    }
    pub fn tile(&self, p: TilePos) -> TileId {
        if self.in_bounds(p) { self.tiles[self.idx(p)] } else { tiles::VOID }
    }
    pub fn set_tile(&mut self, p: TilePos, t: TileId) {
        if self.in_bounds(p) {
            let i = self.idx(p);
            self.tiles[i] = t;
        }
    }
    pub fn add_object(&mut self, kind: &str, p: TilePos) {
        debug_assert!(objects::def(kind).is_some(), "unknown object {kind}");
        self.objects.push(WorldObject { kind: kind.into(), pos: p, frame: 0, locked: false, group: 0 });
    }
    pub fn add_object_in(&mut self, kind: &str, p: TilePos, group: u16) {
        debug_assert!(objects::def(kind).is_some(), "unknown object {kind}");
        self.objects.push(WorldObject { kind: kind.into(), pos: p, frame: 0, locked: false, group });
    }
    pub fn building_at(&self, p: TilePos) -> Option<&Building> {
        self.buildings.iter().find(|b| b.contains(p))
    }
    pub fn objects_at(&self, p: TilePos) -> impl Iterator<Item = &WorldObject> {
        self.objects.iter().filter(move |o| o.pos == p)
    }
    pub fn blocked_by_object(&self, p: TilePos) -> bool {
        self.objects_at(p).any(|o| objects::def(&o.kind).is_some_and(|d| d.blocking))
    }
    pub fn walkable(&self, p: TilePos) -> bool {
        self.in_bounds(p) && tiles::def(self.tile(p)).walkable && !self.blocked_by_object(p)
    }
    pub fn portal_at(&self, p: TilePos) -> Option<&Portal> {
        self.portals.iter().find(|q| q.pos == p)
    }
    pub fn chunks(&self) -> (i32, i32) {
        ((self.width + CHUNK - 1) / CHUNK, (self.height + CHUNK - 1) / CHUNK)
    }
}

/// All maps of the game, by name.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct World {
    pub maps: BTreeMap<String, Map>,
}

impl World {
    pub fn insert(&mut self, m: Map) {
        self.maps.insert(m.name.clone(), m);
    }
    pub fn find_place(&self, name: &str) -> Option<(&str, TilePos)> {
        self.maps.values().find_map(|m| m.places.get(name).map(|p| (m.name.as_str(), *p)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn walkability() {
        let mut m = Map::new("t", 32, 32, tiles::GRASS);
        m.set_tile(TilePos::new(1, 1), tiles::WATER_DEEP);
        m.add_object("pine_tree", TilePos::new(2, 2));
        assert!(m.walkable(TilePos::new(0, 0)));
        assert!(!m.walkable(TilePos::new(1, 1)));
        assert!(!m.walkable(TilePos::new(2, 2)));
        assert!(!m.walkable(TilePos::new(-1, 0)));
        assert_eq!(m.chunks(), (2, 2));
    }
}
