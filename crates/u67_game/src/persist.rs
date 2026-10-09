//! World persistence: replay saved object/tile changes onto a freshly generated world.
use crate::data::*;
use u67_core::TilePos;
use u67_world::map::World;
use u67_world::tiles::TileId;

pub fn original_kind(kind: &str) -> &str {
    match kind {
        "door_open" => "door_wood",
        k => k,
    }
}

/// Record a change to the object at index `idx` of `map`.
pub fn record(d: &mut GameData, world: &World, map: &str, idx: usize, f: impl FnOnce(&mut ObjState)) {
    let Some(o) = world.maps.get(map).and_then(|m| m.objects.get(idx)) else { return };
    let key = obj_key(map, o.pos.x, o.pos.y, original_kind(&o.kind));
    f(d.obj_state.entry(key).or_default());
}

/// Apply everything in `d` (object states, tile edits, placed objects) to `world`.
pub fn apply_overrides(world: &mut World, d: &GameData) {
    for (key, st) in &d.obj_state {
        let mut it = key.splitn(4, ':');
        let (Some(map), Some(x), Some(y), Some(kind)) = (it.next(), it.next(), it.next(), it.next()) else { continue };
        let (Ok(x), Ok(y)) = (x.parse::<i32>(), y.parse::<i32>()) else { continue };
        let Some(m) = world.maps.get_mut(map) else { continue };
        let Some(idx) = m.objects.iter().position(|o| o.pos == TilePos::new(x, y) && original_kind(&o.kind) == kind) else { continue };
        if st.removed {
            m.remove_object(idx);
            continue;
        }
        if let Some(k) = &st.kind {
            m.set_object_kind(idx, k);
        }
        if let Some(f) = st.frame {
            m.objects[idx].frame = f;
        }
        if st.contents.is_some() {
            m.objects[idx].contents = st.contents.clone();
        }
    }
    for (map, x, y, t) in &d.tile_edits {
        if let Some(m) = world.maps.get_mut(map) {
            m.set_tile(TilePos::new(*x, *y), TileId(*t));
        }
    }
    for (map, x, y, kind) in &d.placed {
        if let Some(m) = world.maps.get_mut(map) {
            if !m.objects.iter().any(|o| o.pos == TilePos::new(*x, *y) && &o.kind == kind) {
                m.add_object(kind, TilePos::new(*x, *y));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use u67_world::inventory::Item;
    use u67_world::tiles;

    #[test]
    fn overrides_roundtrip() {
        let mut w = u67_mapgen::generate_world(67);
        let mut d = GameData::new(67, "midgard", [0.0, 0.0]);
        let m = &w.maps["midgard"];
        let door = m.objects.iter().position(|o| o.kind == "door_wood").unwrap();
        let chest = m.objects.iter().position(|o| o.kind == "chest").unwrap();
        let boulder = m.objects.iter().position(|o| o.kind == "boulder").unwrap();
        // simulate gameplay: open door, loot chest, destroy boulder, wall edit, dock a ship
        w.maps.get_mut("midgard").unwrap().set_object_kind(door, "door_open");
        record(&mut d, &w, "midgard", door, |s| s.kind = Some("door_open".into()));
        w.maps.get_mut("midgard").unwrap().objects[chest].contents = Some(vec![Item::new("rope", 1)]);
        record(&mut d, &w, "midgard", chest, |s| s.contents = Some(vec![Item::new("rope", 1)]));
        record(&mut d, &w, "midgard", boulder, |s| s.removed = true);
        d.tile_edits.push(("midgard".into(), 3, 3, tiles::LAVA.0));
        d.placed.push(("midgard".into(), 5, 5, "longship".into()));

        // "reload": fresh world + overrides
        let mut fresh = u67_mapgen::generate_world(67);
        apply_overrides(&mut fresh, &d);
        let fm = &fresh.maps["midgard"];
        assert_eq!(fm.objects[door].kind, "door_open");
        assert!(!fm.blocked_by_object(fm.objects[door].pos));
        assert_eq!(fm.objects[chest].contents.as_ref().unwrap()[0].id, "rope");
        assert_eq!(fm.objects[boulder].kind, "removed");
        assert_eq!(fm.tile(TilePos::new(3, 3)), tiles::LAVA);
        assert!(fm.objects.iter().any(|o| o.kind == "longship" && o.pos == TilePos::new(5, 5)));
        // applying twice is harmless
        apply_overrides(&mut fresh, &d);
        assert_eq!(fresh.maps["midgard"].objects.iter().filter(|o| o.kind == "longship" && o.pos == TilePos::new(5, 5)).count(), 1);
    }
}
