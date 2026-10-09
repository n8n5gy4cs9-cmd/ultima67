//! World object kinds (static scenery and interactables).
use serde::{Deserialize, Serialize};
use u67_core::TilePos;

#[derive(Clone, Copy, Debug)]
pub struct ObjectDef {
    pub id: &'static str,
    pub blocking: bool,
    /// Hidden when the player is inside the same building (roofs).
    pub roof: bool,
    pub container: bool,
    pub frames: u8,
}

const fn o(id: &'static str, blocking: bool, roof: bool, container: bool, frames: u8) -> ObjectDef {
    ObjectDef { id, blocking, roof, container, frames }
}

pub const OBJECTS: &[ObjectDef] = &[
    o("pine_tree", true, false, false, 1),
    o("birch_tree", true, false, false, 1),
    o("runestone", true, false, false, 2),
    o("longhouse_roof", false, true, false, 1),
    o("door_wood", false, false, false, 2),
    o("chest", true, false, true, 2),
    o("bed", false, false, false, 1),
    o("table", true, false, false, 1),
    o("barrel", true, false, true, 1),
    o("cannon", true, false, false, 4),
    o("longship", true, false, false, 4),
    o("sled", true, false, false, 4),
    o("campfire", false, false, false, 3),
    o("forge", true, false, false, 2),
    o("well", true, false, false, 1),
    o("boulder", true, false, false, 1),
    o("bifrost_node", false, false, false, 3),
    o("crate", true, false, true, 1),
    o("signpost", true, false, false, 1),
    o("wreck", true, false, false, 1),
];

pub fn def(id: &str) -> Option<&'static ObjectDef> {
    OBJECTS.iter().find(|d| d.id == id)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldObject {
    pub kind: String,
    pub pos: TilePos,
    pub frame: u8,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Portal {
    pub pos: TilePos,
    pub target_map: String,
    pub target_pos: TilePos,
}
