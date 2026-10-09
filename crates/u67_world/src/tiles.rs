//! Terrain tile table. Ids are stable (saved in maps); append only.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TileId(pub u16);

#[derive(Clone, Copy, Debug)]
pub struct TileDef {
    pub name: &'static str,
    pub walkable: bool,
    pub water: bool,
    pub hazard: bool,
    /// Base colour (RGB) used by the placeholder tileset generator and minimap.
    pub color: [u8; 3],
}

macro_rules! tiles {
    ($($konst:ident = $id:expr, $name:expr, $walk:expr, $water:expr, $haz:expr, $col:expr;)*) => {
        $(pub const $konst: TileId = TileId($id);)*
        pub const TILES: &[TileDef] = &[
            $(TileDef { name: $name, walkable: $walk, water: $water, hazard: $haz, color: $col },)*
        ];
    };
}

tiles! {
    VOID = 0, "void", false, false, false, [8, 8, 16];
    GRASS = 1, "grass", true, false, false, [70, 140, 60];
    FOREST_FLOOR = 2, "forest_floor", true, false, false, [40, 100, 50];
    SNOW = 3, "snow", true, false, false, [225, 235, 245];
    ICE = 4, "ice", true, false, false, [160, 210, 235];
    ROCK = 5, "rock", true, false, false, [110, 110, 120];
    MOUNTAIN = 6, "mountain", false, false, false, [80, 80, 95];
    SAND = 7, "sand", true, false, false, [214, 190, 130];
    WATER_SHALLOW = 8, "water_shallow", true, true, false, [70, 140, 200];
    WATER_DEEP = 9, "water_deep", false, true, false, [30, 70, 150];
    LAVA = 10, "lava", true, false, true, [230, 90, 20];
    REGOLITH = 11, "regolith", true, false, false, [150, 150, 150];
    GAS = 12, "gas", true, false, true, [200, 160, 110];
    ROAD = 13, "road", true, false, false, [150, 125, 95];
    FLOOR_WOOD = 14, "floor_wood", true, false, false, [150, 105, 60];
    FLOOR_STONE = 15, "floor_stone", true, false, false, [135, 135, 140];
    WALL_WOOD = 16, "wall_wood", false, false, false, [95, 60, 35];
    WALL_STONE = 17, "wall_stone", false, false, false, [100, 100, 108];
    SWAMP = 18, "swamp", true, false, false, [70, 90, 60];
    FARMLAND = 19, "farmland", true, false, false, [110, 80, 50];
    MARS_DUST = 20, "mars_dust", true, false, false, [190, 90, 60];
    ASH = 21, "ash", true, false, false, [60, 55, 60];
    AURORA_ICE = 22, "aurora_ice", true, false, false, [130, 230, 200];
    BIFROST = 23, "bifrost", true, false, false, [240, 120, 220];
}

pub fn def(t: TileId) -> &'static TileDef {
    TILES.get(t.0 as usize).unwrap_or(&TILES[0])
}

pub fn by_name(n: &str) -> Option<TileId> {
    TILES.iter().position(|d| d.name == n).map(|i| TileId(i as u16))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn table_matches_ids() {
        assert_eq!(def(LAVA).name, "lava");
        assert_eq!(by_name("bifrost"), Some(BIFROST));
        assert_eq!(TILES.len(), 24);
    }
}
