//! Inventory interactions used by the GUI: drag/drop between paperdoll, packs and world
//! containers; using items. Pure and unit-tested.
use crate::data::*;
use crate::persist;
use u67_world::combat as rules;
use u67_world::inventory::{InvError, Item};
use u67_world::items::{self, Kind, Slot};
use u67_world::map::World;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotRef {
    Paper(Slot),
    /// container path inside the pack (empty = backpack root) + index in that container
    Pack {
        container: Vec<usize>,
        index: usize,
    },
    /// world container object idx + index
    Obj {
        idx: usize,
        index: usize,
    },
}

fn obj_items<'a>(d: &GameData, world: &'a mut World, idx: usize) -> Option<&'a mut Vec<Item>> {
    world.maps.get_mut(&d.current_map)?.objects.get_mut(idx)?.contents.as_mut()
}

/// Take the item out of a slot (start of a drag).
pub fn take(d: &mut GameData, world: &mut World, s: &SlotRef) -> Option<Item> {
    match s {
        SlotRef::Paper(slot) => d.players[0].inventory.equipped.remove(slot),
        SlotRef::Pack { container, index } => {
            let mut path = container.clone();
            path.push(*index);
            d.players[0].inventory.take(&path).ok()
        }
        SlotRef::Obj { idx, index } => {
            let v = obj_items(d, world, *idx)?;
            if *index < v.len() {
                Some(v.remove(*index))
            } else {
                None
            }
        }
    }
}

/// Put an item into a slot. Returns `Ok(replaced)` (an item that was displaced, if any) or gives the item back.
pub fn put(d: &mut GameData, world: &mut World, s: &SlotRef, item: Item) -> Result<Option<Item>, (InvError, Item)> {
    match s {
        SlotRef::Paper(slot) => {
            let def = items::get(&item.id);
            let ok = def.is_some_and(|df| df.slot == Some(*slot) || (matches!((df.slot, slot), (Some(Slot::RingL), Slot::RingR))));
            if !ok {
                return Err((InvError::SlotMismatch, item));
            }
            Ok(d.players[0].inventory.equipped.insert(*slot, item))
        }
        SlotRef::Pack { container, .. } => {
            let (backpack, item) = (d.players[0].inventory.pack.len(), item);
            let _ = backpack;
            d.players[0].inventory.put(container, item).map(|_| None)
        }
        SlotRef::Obj { idx, .. } => match obj_items(d, world, *idx) {
            Some(v) => {
                if let Some(st) = v.iter_mut().find(|i| i.id == item.id && items::get(&i.id).is_some_and(|x| x.stackable)) {
                    st.qty += item.qty;
                } else {
                    v.push(item);
                }
                Ok(None)
            }
            None => Err((InvError::NotContainer, item)),
        },
    }
}

/// Complete a drag: take from `from`, put into `to`; if `to` is occupied on the paperdoll the old
/// item swaps into `from`'s pack. On failure the item returns to where it came from.
pub fn move_item(d: &mut GameData, world: &mut World, from: &SlotRef, to: &SlotRef) -> Result<(), InvError> {
    if from == to {
        return Ok(());
    }
    let Some(item) = take(d, world, from) else { return Err(InvError::NoSuchItem) };
    // dropping a container into itself / its own contents is not allowed
    if let (SlotRef::Pack { container: fc, index: fi }, SlotRef::Pack { container: tc, .. }) = (from, to) {
        let mut own = fc.clone();
        own.push(*fi);
        if tc.len() >= own.len() && tc[..own.len()] == own[..] {
            let _ = put(d, world, from, item);
            return Err(InvError::IntoSelf);
        }
    }
    // when the source index precedes the target in the same container, indexes shift; we only use `container` for packs
    match put(d, world, to, item) {
        Ok(None) => Ok(()),
        Ok(Some(replaced)) => {
            // swapped out an equipped item: put it where the dragged item came from, else into the pack root
            if put(d, world, from, replaced.clone()).is_err() {
                d.players[0].inventory.pack.push(replaced);
            }
            Ok(())
        }
        Err((e, back)) => {
            if put(d, world, from, back.clone()).is_err() {
                d.players[0].inventory.pack.push(back);
            }
            Err(e)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UseResult {
    Msg(String),
    OpenContainer(Vec<usize>),
    Equipped(String),
    Nothing,
}

/// Right-click on a pack item: eat/heal, open containers, equip gear.
pub fn use_item(d: &mut GameData, path: &[usize]) -> UseResult {
    let Some(it) = d.players[0].inventory.get(path).cloned() else { return UseResult::Nothing };
    let Some(def) = items::get(&it.id) else { return UseResult::Nothing };
    if def.capacity > 0.0 {
        return UseResult::OpenContainer(path.to_vec());
    }
    if let Some(h) = rules::heal_amount(def.id) {
        let max = d.players[0].stats.max_hp();
        if d.players[0].stats.hp >= max && def.id != "mead" {
            return UseResult::Msg("You are already at full health.".into());
        }
        d.players[0].stats.hp = (d.players[0].stats.hp + h).min(max);
        let p = path.to_vec();
        if let Some(i) = d.players[0].inventory.get(&p).map(|i| i.qty) {
            if i > 1 {
                let mut q = d.players[0].inventory.take(&p).unwrap();
                q.qty -= 1;
                let _ = d.players[0].inventory.put(&p[..p.len() - 1], q);
            } else {
                let _ = d.players[0].inventory.take(&p);
            }
        }
        return UseResult::Msg(format!("You use {}. (+{h} HP)", def.name));
    }
    if def.slot.is_some() {
        return match d.players[0].inventory.equip(path) {
            Ok(()) => UseResult::Equipped(def.name.to_string()),
            Err(InvError::SlotOccupied) => {
                // swap with the occupant (equip only fails for items that have a slot)
                let Some(slot) = def.slot else { return UseResult::Nothing };
                let new = d.players[0].inventory.take(path).unwrap();
                if let Some(old) = d.players[0].inventory.equipped.insert(slot, new) {
                    d.players[0].inventory.pack.push(old);
                }
                UseResult::Equipped(def.name.to_string())
            }
            Err(e) => UseResult::Msg(format!("Cannot equip: {e:?}")),
        };
    }
    match def.kind {
        Kind::Tool if def.id == "torch" => UseResult::Msg("You raise the torch. (Light helps in caves.)".into()),
        _ => UseResult::Msg(format!("{} - weight {:.1}", def.name, def.weight)),
    }
}

/// Dump a container item's contents into the world next to the player (drop).
pub fn drop_to_ground(d: &mut GameData, world: &mut World, from: &SlotRef) -> Option<String> {
    let it = take(d, world, from)?;
    let name = items::get(&it.id).map_or(it.id.clone(), |x| x.name.to_string());
    let p = d.players[0].pos;
    d.ground.push(GroundItem { map: d.current_map.clone(), pos: [p[0] + 0.3, p[1] - 0.2], item: it });
    Some(format!("Dropped {name}"))
}

/// Persist the contents of world container `idx` after a drag touched it.
pub fn persist_obj(d: &mut GameData, world: &World, idx: usize) {
    let map = d.current_map.clone();
    let c = world.maps.get(&map).and_then(|m| m.objects.get(idx)).and_then(|o| o.contents.clone());
    persist::record(d, world, &map, idx, |s| s.contents = c);
}

#[cfg(test)]
mod tests {
    use super::*;
    use u67_world::tiles;

    fn setup() -> (GameData, World) {
        let mut w = World::default();
        let mut m = u67_world::map::Map::new("midgard", 20, 20, tiles::GRASS);
        m.add_object("chest", u67_core::TilePos::new(3, 3));
        m.objects[0].contents = Some(vec![Item::new("silver", 50), Item::new("medkit", 1)]);
        w.insert(m);
        (GameData::new(1, "midgard", [3.5, 4.5]), w)
    }

    #[test]
    fn loot_from_chest_into_pack() {
        let (mut d, mut w) = setup();
        move_item(&mut d, &mut w, &SlotRef::Obj { idx: 0, index: 1 }, &SlotRef::Pack { container: vec![], index: 99 }).unwrap();
        assert_eq!(d.players[0].inventory.count("medkit"), 1);
        assert_eq!(w.maps["midgard"].objects[0].contents.as_ref().unwrap().len(), 1);
        // stacking silver
        let before = d.players[0].inventory.count("silver");
        move_item(&mut d, &mut w, &SlotRef::Obj { idx: 0, index: 0 }, &SlotRef::Pack { container: vec![], index: 0 }).unwrap();
        assert_eq!(d.players[0].inventory.count("silver"), before + 50);
        assert!(w.maps["midgard"].objects[0].contents.as_ref().unwrap().is_empty());
    }

    #[test]
    fn equip_by_drag_swaps_and_rejects_wrong_slot() {
        let (mut d, mut w) = setup();
        d.players[0].inventory.add("kevlar_vest", 1).unwrap();
        d.players[0].inventory.add("horned_helm", 1).unwrap();
        let vest = d.players[0].inventory.pack.iter().position(|i| i.id == "kevlar_vest").unwrap();
        // wrong slot: item returns
        let r = move_item(&mut d, &mut w, &SlotRef::Pack { container: vec![], index: vest }, &SlotRef::Paper(Slot::Head));
        assert_eq!(r, Err(InvError::SlotMismatch));
        assert_eq!(d.players[0].inventory.count("kevlar_vest"), 1);
        // a refused drop puts the item back at the end of the pack, so look its index up again
        let vest = d.players[0].inventory.pack.iter().position(|i| i.id == "kevlar_vest").unwrap();
        move_item(&mut d, &mut w, &SlotRef::Pack { container: vec![], index: vest }, &SlotRef::Paper(Slot::Torso)).unwrap();
        assert!(d.players[0].inventory.equipped.contains_key(&Slot::Torso));
        // equip a second torso item: swap back to pack
        d.players[0].inventory.add("wolf_cloak", 1).unwrap();
        assert_eq!(d.players[0].inventory.count("kevlar_vest"), 1);
    }

    #[test]
    fn cannot_put_container_in_itself() {
        let (mut d, mut w) = setup();
        d.players[0].inventory.add("pouch", 1).unwrap();
        let pi = d.players[0].inventory.pack.iter().position(|i| i.id == "pouch").unwrap();
        let r = move_item(&mut d, &mut w, &SlotRef::Pack { container: vec![], index: pi }, &SlotRef::Pack { container: vec![pi], index: 0 });
        assert_eq!(r, Err(InvError::IntoSelf));
        assert_eq!(d.players[0].inventory.count("pouch"), 1);
    }

    #[test]
    fn use_items() {
        let (mut d, _) = setup();
        d.players[0].stats.hp = 10;
        let bread = d.players[0].inventory.pack.iter().position(|i| i.id == "rye_bread").unwrap();
        let before = d.players[0].inventory.count("rye_bread");
        assert!(matches!(use_item(&mut d, &[bread]), UseResult::Msg(m) if m.contains("+6 HP")));
        assert_eq!(d.players[0].stats.hp, 16);
        assert_eq!(d.players[0].inventory.count("rye_bread"), before - 1);
        let bp = d.players[0].inventory.pack.iter().position(|i| i.id == "backpack").unwrap();
        assert_eq!(use_item(&mut d, &[bp]), UseResult::OpenContainer(vec![bp]));
        // the starting pistol is already equipped; equipping an axe swaps it back into the pack
        assert_eq!(d.players[0].inventory.equipped[&Slot::HandR].id, "rune_pistol");
        d.players[0].inventory.add("viking_axe", 1).unwrap();
        let axe = d.players[0].inventory.pack.iter().position(|i| i.id == "viking_axe").unwrap();
        assert_eq!(use_item(&mut d, &[axe]), UseResult::Equipped("Viking Axe".into()));
        assert_eq!(d.players[0].inventory.equipped[&Slot::HandR].id, "viking_axe");
        assert_eq!(d.players[0].inventory.count("rune_pistol"), 1);
    }

    #[test]
    fn drop_creates_ground_item() {
        let (mut d, mut w) = setup();
        let i = d.players[0].inventory.pack.iter().position(|i| i.id == "torch").unwrap();
        let msg = drop_to_ground(&mut d, &mut w, &SlotRef::Pack { container: vec![], index: i }).unwrap();
        assert!(msg.contains("Torch"));
        assert_eq!(d.ground.len(), 1);
    }
}
