//! Ultima 7 style inventory: paperdoll slots + backpack with nested containers.
//! Items are addressed by an `ItemPath` (index chain from the pack root).
use crate::items::{self, Slot};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub qty: u32,
    /// Contents when this item is a container.
    pub contents: Vec<Item>,
}

impl Item {
    pub fn new(id: &str, qty: u32) -> Self {
        Self { id: id.into(), qty: qty.max(1), contents: vec![] }
    }
    pub fn weight(&self) -> f32 {
        let own = items::get(&self.id).map_or(1.0, |d| d.weight) * self.qty as f32;
        own + self.contents.iter().map(Item::weight).sum::<f32>()
    }
    fn capacity(&self) -> f32 {
        items::get(&self.id).map_or(0.0, |d| d.capacity)
    }
    fn stackable(&self) -> bool {
        items::get(&self.id).is_some_and(|d| d.stackable)
    }
    pub fn contents_weight(&self) -> f32 {
        self.contents.iter().map(Item::weight).sum()
    }
}

pub type ItemPath = Vec<usize>;

#[derive(Debug, PartialEq, Eq)]
pub enum InvError {
    UnknownItem,
    NoSuchItem,
    NotContainer,
    TooHeavy,
    SlotMismatch,
    SlotOccupied,
    IntoSelf,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inventory {
    pub equipped: BTreeMap<Slot, Item>,
    pub pack: Vec<Item>,
    /// Strength-derived carry limit (weight units).
    pub max_weight: f32,
}

impl Inventory {
    pub fn new(strength: u32) -> Self {
        Self { max_weight: strength as f32 * 2.0 + 20.0, ..Default::default() }
    }

    pub fn total_weight(&self) -> f32 {
        self.pack.iter().map(Item::weight).sum::<f32>() + self.equipped.values().map(Item::weight).sum::<f32>()
    }

    /// Add to the pack: stacks with an existing stack, else the first container with room, else the root.
    pub fn add(&mut self, id: &str, qty: u32) -> Result<(), InvError> {
        let def = items::get(id).ok_or(InvError::UnknownItem)?;
        let item = Item::new(id, qty);
        if self.total_weight() + item.weight() > self.max_weight {
            return Err(InvError::TooHeavy);
        }
        if def.stackable {
            if let Some(s) = self.pack.iter_mut().find(|i| i.id == id) {
                s.qty += item.qty;
                return Ok(());
            }
        }
        self.pack.push(item);
        Ok(())
    }

    pub fn count(&self, id: &str) -> u32 {
        fn walk(v: &[Item], id: &str) -> u32 {
            v.iter().map(|i| (if i.id == id { i.qty } else { 0 }) + walk(&i.contents, id)).sum()
        }
        walk(&self.pack, id) + self.equipped.values().filter(|i| i.id == id).map(|i| i.qty).sum::<u32>()
    }

    /// Remove up to `qty` of `id` from the pack tree. Returns how many were removed.
    pub fn remove(&mut self, id: &str, mut qty: u32) -> u32 {
        fn walk(v: &mut Vec<Item>, id: &str, qty: &mut u32) {
            let mut i = 0;
            while i < v.len() && *qty > 0 {
                if v[i].id == id {
                    let take = v[i].qty.min(*qty);
                    v[i].qty -= take;
                    *qty -= take;
                    if v[i].qty == 0 {
                        v.remove(i);
                        continue;
                    }
                }
                walk(&mut v[i].contents, id, qty);
                i += 1;
            }
        }
        let want = qty;
        walk(&mut self.pack, id, &mut qty);
        want - qty
    }

    fn container_mut(&mut self, path: &[usize]) -> Result<&mut Vec<Item>, InvError> {
        let mut v = &mut self.pack;
        for &i in path {
            let it = v.get_mut(i).ok_or(InvError::NoSuchItem)?;
            if it.capacity() <= 0.0 {
                return Err(InvError::NotContainer);
            }
            v = &mut it.contents;
        }
        Ok(v)
    }

    pub fn get(&self, path: &[usize]) -> Option<&Item> {
        let (last, parents) = path.split_last()?;
        let mut v = &self.pack;
        for &i in parents {
            v = &v.get(i)?.contents;
        }
        v.get(*last)
    }

    /// Detach the item at `path` (drag start).
    pub fn take(&mut self, path: &[usize]) -> Result<Item, InvError> {
        let (last, parents) = path.split_last().ok_or(InvError::NoSuchItem)?;
        let v = self.container_mut(parents)?;
        if *last >= v.len() {
            return Err(InvError::NoSuchItem);
        }
        Ok(v.remove(*last))
    }

    /// Drop `item` into the container at `into` (empty path = pack root). On error the item is returned.
    pub fn put(&mut self, into: &[usize], item: Item) -> Result<(), (InvError, Item)> {
        let root = into.is_empty();
        let cap_ok = {
            let w = item.weight();
            if root {
                true
            } else {
                match self.get(into) {
                    Some(c) if c.capacity() > 0.0 => c.contents_weight() + w <= c.capacity(),
                    Some(_) => return Err((InvError::NotContainer, item)),
                    None => return Err((InvError::NoSuchItem, item)),
                }
            }
        };
        if !cap_ok {
            return Err((InvError::TooHeavy, item));
        }
        match self.container_mut(into) {
            Ok(v) => {
                if item.stackable() {
                    if let Some(s) = v.iter_mut().find(|i| i.id == item.id && i.contents.is_empty()) {
                        s.qty += item.qty;
                        return Ok(());
                    }
                }
                v.push(item);
                Ok(())
            }
            Err(e) => Err((e, item)),
        }
    }

    /// Drag-and-drop move between containers. A container cannot be dropped into itself.
    pub fn move_item(&mut self, from: &[usize], into: &[usize]) -> Result<(), InvError> {
        if into.len() >= from.len() && &into[..from.len()] == from {
            return Err(InvError::IntoSelf);
        }
        // Adjust `into` if removal of `from` shifts sibling indices at the same depth.
        let mut into_adj = into.to_vec();
        let d = from.len() - 1;
        if into_adj.len() > d && into_adj[..d] == from[..d] && into_adj[d] > from[d] {
            into_adj[d] -= 1;
        }
        let item = self.take(from)?;
        match self.put(&into_adj, item) {
            Ok(()) => Ok(()),
            Err((e, item)) => {
                let (last, parents) = from.split_last().unwrap();
                let v = self.container_mut(parents).unwrap();
                v.insert((*last).min(v.len()), item);
                Err(e)
            }
        }
    }

    pub fn equip(&mut self, path: &[usize]) -> Result<(), InvError> {
        let id = self.get(path).ok_or(InvError::NoSuchItem)?.id.clone();
        let slot = items::get(&id).and_then(|d| d.slot).ok_or(InvError::SlotMismatch)?;
        let slot = if slot == Slot::RingL && self.equipped.contains_key(&Slot::RingL) { Slot::RingR } else { slot };
        if self.equipped.contains_key(&slot) {
            return Err(InvError::SlotOccupied);
        }
        let item = self.take(path)?;
        self.equipped.insert(slot, item);
        Ok(())
    }

    pub fn unequip(&mut self, slot: Slot) -> Result<(), InvError> {
        let item = self.equipped.remove(&slot).ok_or(InvError::NoSuchItem)?;
        self.pack.push(item);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn inv() -> Inventory {
        Inventory::new(20)
    }
    #[test]
    fn stack_and_count() {
        let mut i = inv();
        i.add("ammo_9mm", 10).unwrap();
        i.add("ammo_9mm", 5).unwrap();
        assert_eq!(i.pack.len(), 1);
        assert_eq!(i.count("ammo_9mm"), 15);
        assert_eq!(i.remove("ammo_9mm", 20), 15);
        assert!(i.pack.is_empty());
    }
    #[test]
    fn nested_container_drag() {
        let mut i = inv();
        i.add("backpack", 1).unwrap(); // [0]
        i.add("pouch", 1).unwrap(); // [1]
        i.add("rope", 1).unwrap(); // [2]
        i.move_item(&[2], &[0]).unwrap(); // rope -> backpack
        i.move_item(&[1], &[0]).unwrap(); // pouch -> backpack (index shift)
        assert_eq!(i.pack.len(), 1);
        assert_eq!(i.pack[0].contents.len(), 2);
        assert_eq!(i.move_item(&[0], &[0, 1]), Err(InvError::IntoSelf));
        assert_eq!(i.count("rope"), 1);
    }
    #[test]
    fn container_capacity() {
        let mut i = Inventory::new(100);
        i.add("pouch", 1).unwrap();
        i.add("mjolnir_drone", 1).unwrap(); // 6, pouch holds 8
        i.add("bear_axe", 1).unwrap(); // 5
        i.move_item(&[1], &[0]).unwrap();
        assert_eq!(i.move_item(&[1], &[0]), Err(InvError::TooHeavy));
        assert_eq!(i.pack.len(), 2); // restored
    }
    #[test]
    fn weight_limit_and_equip() {
        let mut i = Inventory::new(0); // limit 20
        assert_eq!(i.add("bear_axe", 5), Err(InvError::TooHeavy));
        i.add("rune_pistol", 1).unwrap();
        i.equip(&[0]).unwrap();
        assert!(i.equipped.contains_key(&Slot::HandR));
        i.unequip(Slot::HandR).unwrap();
        assert_eq!(i.pack.len(), 1);
        assert_eq!(i.add("nonsense", 1), Err(InvError::UnknownItem));
    }
}
