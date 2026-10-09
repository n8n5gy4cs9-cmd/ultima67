//! Turns Ultima 7 shapes into Midgard tiles and objects.
//! Terrain: by the average colour of the flat tile (+ the game's own water flag).
//! Objects: by name keywords (from text.flx) and tfa flags; rules are data (`Rules`), so you can
//! tweak them with `assets/original/u7_rules.json`.
use crate::u7::U7Data;
use serde::{Deserialize, Serialize};
use u67_world::tiles::{self, TileId};

fn rgb_to_hsv(c: [u8; 3]) -> (f32, f32, f32) {
    let (r, g, b) = (c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let d = mx - mn;
    let h = if d == 0.0 {
        0.0
    } else if mx == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if mx == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    ((h + 360.0) % 360.0, if mx == 0.0 { 0.0 } else { d / mx }, mx)
}

/// Terrain class from a flat tile's average colour.
pub fn classify_flat(rgb: [u8; 3], water_flag: bool) -> TileId {
    let (h, s, v) = rgb_to_hsv(rgb);
    if water_flag || (h > 180.0 && h < 265.0 && s > 0.25 && v > 0.15) {
        return if v < 0.45 { tiles::WATER_DEEP } else { tiles::WATER_SHALLOW };
    }
    if v < 0.14 {
        return tiles::MOUNTAIN;
    }
    if s < 0.18 {
        return if v > 0.82 {
            tiles::SNOW
        } else if v > 0.35 {
            tiles::ROCK
        } else {
            tiles::MOUNTAIN
        };
    }
    if s > 0.7 && v > 0.75 && !(30.0..330.0).contains(&h) {
        return tiles::LAVA;
    }
    match h {
        h if (65.0..175.0).contains(&h) => {
            if v < 0.34 {
                tiles::FOREST_FLOOR
            } else if s > 0.55 && v < 0.5 {
                tiles::SWAMP
            } else {
                tiles::GRASS
            }
        }
        h if (38.0..65.0).contains(&h) => {
            if v > 0.6 {
                tiles::SAND
            } else {
                tiles::FARMLAND
            }
        }
        h if (12.0..38.0).contains(&h) => {
            if v > 0.55 && s < 0.5 {
                tiles::SAND
            } else {
                tiles::ROAD
            }
        }
        h if !(12.0..330.0).contains(&h) => {
            if s > 0.7 && v > 0.75 {
                tiles::LAVA
            } else {
                tiles::MARS_DUST
            }
        }
        _ => tiles::ROCK,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "do", rename_all = "snake_case")]
pub enum Action {
    /// place an object kind (our `u67_world::objects` id)
    Object {
        kind: String,
    },
    /// two kinds, picked per position (variety)
    Either {
        a: String,
        b: String,
    },
    /// replace the tile with a wall / floor tile
    Wall {
        tile: String,
    },
    Floor {
        tile: String,
    },
    Roof,
    Skip,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    /// any of these (lower-case) substrings in the shape name
    pub name_contains: Vec<String>,
    #[serde(flatten)]
    pub action: Action,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rules {
    pub named: Vec<Rule>,
    /// solid, unnamed shapes at least this tall (tfa height) become wall tiles
    pub wall_min_height: u8,
    /// objects lifted at least this high are roofs
    pub roof_min_lift: u8,
}

impl Default for Rules {
    fn default() -> Self {
        let r = |names: &[&str], action: Action| Rule { name_contains: names.iter().map(|s| s.to_string()).collect(), action };
        let obj = |k: &str| Action::Object { kind: k.into() };
        Self {
            named: vec![
                r(&["roof"], Action::Roof),
                r(&["tree", "yew", "willow", "oak", "fir "], Action::Either { a: "pine_tree".into(), b: "birch_tree".into() }),
                r(&["boulder", "rock"], obj("boulder")),
                r(&["chest"], obj("chest")),
                r(&["barrel", "keg"], obj("barrel")),
                r(&["crate", "box"], obj("crate")),
                r(&["bed", "bedroll"], obj("bed")),
                r(&["table", "desk"], obj("table")),
                r(&["well", "fountain"], obj("well")),
                r(&["sign"], obj("signpost")),
                r(&["forge", "furnace", "anvil"], obj("forge")),
                r(&["campfire", "fire pit", "fireplace", "brazier", "stove"], obj("campfire")),
                r(&["cannon"], obj("cannon")),
                r(&["ship", "boat", "skiff"], obj("longship")),
                r(&["door", "gate", "portcullis"], obj("door_wood")),
                r(&["wooden wall", "wood wall", "palisade", "fence"], Action::Wall { tile: "wall_wood".into() }),
                r(&["wall", "pillar", "column", "battlement"], Action::Wall { tile: "wall_stone".into() }),
                r(&["floor", "carpet", "rug", "flagstone"], Action::Floor { tile: "floor_wood".into() }),
                r(&["window", "chair", "bench", "lamp", "torch", "candle", "flower", "bush", "grass", "weed", "sack", "bucket", "pot", "plant"], Action::Skip),
            ],
            wall_min_height: 3,
            roof_min_lift: 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Class {
    Object(String),
    Wall(TileId),
    Floor(TileId),
    Roof,
    Skip,
}

impl Rules {
    /// Classify one fixed object. `variant` (any deterministic number) picks between `Either` options.
    pub fn classify(&self, d: &U7Data, shape: u16, lift: u8, variant: u32) -> Class {
        let name = d.name(shape).to_lowercase();
        if d.door(shape) {
            return Class::Object("door_wood".into());
        }
        if !name.is_empty() {
            for rule in &self.named {
                if rule.name_contains.iter().any(|n| name.contains(n.as_str())) {
                    return match &rule.action {
                        Action::Object { kind } => Class::Object(kind.clone()),
                        Action::Either { a, b } => Class::Object(if variant.is_multiple_of(3) { b.clone() } else { a.clone() }),
                        Action::Wall { tile } => Class::Wall(tiles::by_name(tile).unwrap_or(tiles::WALL_STONE)),
                        Action::Floor { tile } => Class::Floor(tiles::by_name(tile).unwrap_or(tiles::FLOOR_WOOD)),
                        Action::Roof => Class::Roof,
                        Action::Skip => Class::Skip,
                    };
                }
            }
        }
        if lift >= self.roof_min_lift && d.solid(shape) {
            return Class::Roof;
        }
        if name.is_empty() && d.solid(shape) && d.dims(shape).2 >= self.wall_min_height && lift < self.roof_min_lift {
            return Class::Wall(tiles::WALL_STONE);
        }
        Class::Skip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_colours_map_to_sensible_tiles() {
        assert_eq!(classify_flat([30, 90, 200], false), tiles::WATER_SHALLOW);
        assert_eq!(classify_flat([10, 30, 90], false), tiles::WATER_DEEP);
        assert_eq!(classify_flat([60, 140, 50], false), tiles::GRASS);
        assert_eq!(classify_flat([20, 70, 30], false), tiles::FOREST_FLOOR);
        assert_eq!(classify_flat([240, 240, 245], false), tiles::SNOW);
        assert_eq!(classify_flat([120, 120, 125], false), tiles::ROCK);
        assert_eq!(classify_flat([220, 190, 120], false), tiles::SAND);
        assert_eq!(classify_flat([130, 90, 50], false), tiles::ROAD);
        assert_eq!(classify_flat([250, 70, 20], false), tiles::LAVA);
        assert_eq!(classify_flat([0, 0, 0], false), tiles::MOUNTAIN);
        // the game's own water flag always wins
        assert_eq!(classify_flat([60, 140, 50], true), tiles::WATER_SHALLOW);
    }

    #[test]
    fn name_rules_and_flags() {
        let mut d = U7Data { names: vec![String::new(); 400], tfa: vec![[0; 3]; 400], ..Default::default() };
        d.names[200] = "Oak Tree".into();
        d.names[201] = "wooden chest".into();
        d.names[202] = "stone wall".into();
        d.names[203] = "thatch roof".into();
        d.names[204] = "rug".into();
        d.names[205] = "window".into();
        d.tfa[210] = [8 | (4 << 5), 0, 0]; // unnamed, solid, 4 high -> wall
        d.tfa[211] = [8, 32, 0]; // unnamed door
        d.tfa[212] = [8 | (1 << 5), 0, 0]; // unnamed solid, low -> skip
        let r = Rules::default();
        assert!(matches!(r.classify(&d, 200, 0, 1), Class::Object(k) if k == "pine_tree"));
        assert!(matches!(r.classify(&d, 200, 0, 3), Class::Object(k) if k == "birch_tree"));
        assert_eq!(r.classify(&d, 201, 0, 0), Class::Object("chest".into()));
        assert_eq!(r.classify(&d, 202, 0, 0), Class::Wall(tiles::WALL_STONE));
        assert_eq!(r.classify(&d, 203, 5, 0), Class::Roof);
        assert_eq!(r.classify(&d, 204, 0, 0), Class::Floor(tiles::FLOOR_WOOD));
        assert_eq!(r.classify(&d, 205, 0, 0), Class::Skip);
        assert_eq!(r.classify(&d, 210, 0, 0), Class::Wall(tiles::WALL_STONE));
        assert_eq!(r.classify(&d, 211, 0, 0), Class::Object("door_wood".into()));
        assert_eq!(r.classify(&d, 212, 0, 0), Class::Skip);
        // high solid unnamed object = roof
        assert_eq!(r.classify(&d, 212, 6, 0), Class::Roof);
    }

    #[test]
    fn rules_roundtrip_as_json() {
        let r = Rules::default();
        let j = serde_json::to_string_pretty(&r).unwrap();
        let back: Rules = serde_json::from_str(&j).unwrap();
        assert_eq!(back.named.len(), r.named.len());
        assert_eq!(back.wall_min_height, 3);
    }
}
