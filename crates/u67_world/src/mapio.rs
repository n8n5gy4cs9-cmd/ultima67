//! Compact binary map format: "U67M" | u32 meta_len | meta JSON | RLE tiles (u16 id, u16 run).
use crate::map::Map;
use crate::objects::{Building, Portal, WorldObject};
use crate::tiles::TileId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use u67_core::TilePos;

#[derive(Serialize, Deserialize)]
struct Meta {
    name: String,
    width: i32,
    height: i32,
    objects: Vec<WorldObject>,
    portals: Vec<Portal>,
    #[serde(default)]
    buildings: Vec<Building>,
    places: BTreeMap<String, TilePos>,
}

pub fn to_bytes(m: &Map) -> Vec<u8> {
    let meta = Meta {
        name: m.name.clone(),
        width: m.width,
        height: m.height,
        objects: m.objects.clone(),
        portals: m.portals.clone(),
        buildings: m.buildings.clone(),
        places: m.places.clone(),
    };
    let json = serde_json::to_vec(&meta).expect("serialize map meta");
    let mut o = b"U67M".to_vec();
    o.extend((json.len() as u32).to_le_bytes());
    o.extend(json);
    let mut i = 0;
    while i < m.tiles.len() {
        let t = m.tiles[i];
        let mut run = 1usize;
        while i + run < m.tiles.len() && m.tiles[i + run] == t && run < u16::MAX as usize {
            run += 1;
        }
        o.extend(t.0.to_le_bytes());
        o.extend((run as u16).to_le_bytes());
        i += run;
    }
    o
}

pub fn from_bytes(b: &[u8]) -> Result<Map, String> {
    if b.len() < 8 || &b[..4] != b"U67M" {
        return Err("not a U67M map".into());
    }
    let ml = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
    let meta: Meta = serde_json::from_slice(b.get(8..8 + ml).ok_or("truncated meta")?).map_err(|e| e.to_string())?;
    let n = (meta.width * meta.height) as usize;
    let mut tiles = Vec::with_capacity(n);
    for ch in b[8 + ml..].chunks_exact(4) {
        let t = TileId(u16::from_le_bytes([ch[0], ch[1]]));
        let run = u16::from_le_bytes([ch[2], ch[3]]) as usize;
        tiles.extend(std::iter::repeat_n(t, run));
    }
    if tiles.len() != n {
        return Err(format!("tile count {} != {}", tiles.len(), n));
    }
    let mut map = Map::new(&meta.name, meta.width, meta.height, TileId(0));
    map.tiles = tiles;
    map.objects = meta.objects;
    map.portals = meta.portals;
    map.buildings = meta.buildings;
    map.places = meta.places;
    map.reindex();
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiles;
    #[test]
    fn roundtrip() {
        let mut m = Map::new("t", 40, 30, tiles::GRASS);
        m.set_tile(TilePos::new(3, 3), tiles::LAVA);
        m.add_object("chest", TilePos::new(5, 5));
        m.places.insert("home".into(), TilePos::new(1, 2));
        let b = to_bytes(&m);
        let r = from_bytes(&b).unwrap();
        assert_eq!(r.tiles, m.tiles);
        assert_eq!(r.objects.len(), 1);
        assert_eq!(r.places["home"], TilePos::new(1, 2));
        assert!(from_bytes(b"nope").is_err());
        assert!(b.len() < 400);
    }
}
