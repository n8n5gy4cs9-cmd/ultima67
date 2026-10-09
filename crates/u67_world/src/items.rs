//! Static item definitions (data table). Ids are used by console `give`, saves and assets.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Slot {
    Head,
    Neck,
    Torso,
    Legs,
    Feet,
    HandL,
    HandR,
    Ammo,
    RingL,
    RingR,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Melee,
    Gun,
    Ammo,
    Armor,
    Food,
    Tool,
    Relic,
    Container,
    Reagent,
    Misc,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: Kind,
    /// Weight in "stone" units (like U7: 1.0 = about 1 kg).
    pub weight: f32,
    pub slot: Option<Slot>,
    pub stackable: bool,
    /// Max total weight a container can hold (0 = not a container).
    pub capacity: f32,
    pub damage: u16,
    pub armor: u16,
    /// Ammo item id that a gun uses.
    pub ammo: Option<&'static str>,
}

const fn it(id: &'static str, name: &'static str, kind: Kind, weight: f32) -> ItemDef {
    ItemDef { id, name, kind, weight, slot: None, stackable: false, capacity: 0.0, damage: 0, armor: 0, ammo: None }
}
const fn gun(id: &'static str, name: &'static str, w: f32, dmg: u16, ammo: &'static str) -> ItemDef {
    ItemDef { slot: Some(Slot::HandR), damage: dmg, ammo: Some(ammo), ..it(id, name, Kind::Gun, w) }
}
const fn melee(id: &'static str, name: &'static str, w: f32, dmg: u16) -> ItemDef {
    ItemDef { slot: Some(Slot::HandR), damage: dmg, ..it(id, name, Kind::Melee, w) }
}
const fn armor(id: &'static str, name: &'static str, w: f32, slot: Slot, a: u16) -> ItemDef {
    ItemDef { slot: Some(slot), armor: a, ..it(id, name, Kind::Armor, w) }
}
const fn stack(id: &'static str, name: &'static str, kind: Kind, w: f32) -> ItemDef {
    ItemDef { stackable: true, ..it(id, name, kind, w) }
}
const fn cont(id: &'static str, name: &'static str, w: f32, cap: f32) -> ItemDef {
    ItemDef { capacity: cap, ..it(id, name, Kind::Container, w) }
}

pub const ITEMS: &[ItemDef] = &[
    // guns
    gun("rune_pistol", "Rune Pistol", 1.5, 12, "ammo_9mm"),
    gun("hunting_shotgun", "Hunting Shotgun", 3.5, 30, "ammo_shell"),
    gun("fjord_rifle", "Fjord Rifle", 4.0, 22, "ammo_762"),
    gun("skald_smg", "Skald SMG", 3.0, 9, "ammo_9mm"),
    gun("gungnir_sniper", "Gungnir Sniper", 5.5, 60, "ammo_762"),
    gun("vainamoinen_gun", "Väinämöinen's Kantele-Rifle", 4.5, 48, "ammo_762"),
    gun("flare_gun", "Flare Gun", 1.0, 5, "ammo_flare"),
    // melee
    melee("viking_axe", "Viking Axe", 3.0, 14),
    melee("round_spear", "Spear", 2.5, 12),
    melee("long_sword", "Long Sword", 3.0, 15),
    melee("bear_axe", "Bear Axe", 5.0, 26),
    melee("mjolnir_drone", "Mjölnir Drone Hammer", 6.0, 40),
    melee("puukko", "Puukko Knife", 0.5, 6),
    // ammo
    stack("ammo_9mm", "9mm Rounds", Kind::Ammo, 0.02),
    stack("ammo_762", "7.62 Rounds", Kind::Ammo, 0.03),
    stack("ammo_shell", "Shotgun Shells", Kind::Ammo, 0.04),
    stack("ammo_flare", "Flares", Kind::Ammo, 0.05),
    stack("cannon_ball", "Cannon Ball", Kind::Ammo, 2.0),
    stack("gunpowder", "Gunpowder", Kind::Ammo, 0.5),
    // armor
    armor("horned_helm", "Horned Helm", 1.5, Slot::Head, 3),
    armor("kevlar_vest", "Kevlar Vest", 4.0, Slot::Torso, 6),
    armor("wolf_cloak", "Wolf Cloak", 2.0, Slot::Back, 2),
    armor("norse_greaves", "Greaves", 2.5, Slot::Legs, 3),
    armor("hiking_boots", "Hiking Boots", 1.0, Slot::Feet, 1),
    armor("round_shield", "Round Shield", 3.0, Slot::HandL, 4),
    ItemDef { slot: Some(Slot::RingL), ..it("draupnir_ring", "Draupnir Ring", Kind::Relic, 0.1) },
    ItemDef { slot: Some(Slot::Neck), ..it("thor_amulet", "Thor's Amulet", Kind::Relic, 0.2) },
    // food/tools
    stack("mead", "Mead", Kind::Food, 0.5),
    stack("rye_bread", "Rye Bread", Kind::Food, 0.3),
    stack("smoked_fish", "Smoked Fish", Kind::Food, 0.4),
    stack("medkit", "Medkit", Kind::Food, 0.8),
    it("torch", "Torch", Kind::Tool, 0.5),
    it("lockpick", "Lockpick", Kind::Tool, 0.1),
    it("rope", "Rope", Kind::Tool, 1.0),
    it("vacuum_suit", "Vacuum Suit", Kind::Tool, 6.0),
    it("rune_key", "Rune Key", Kind::Tool, 0.1),
    stack("silver", "Silver", Kind::Misc, 0.01),
    // relics
    it("sampo_shard_1", "Sampo Shard I", Kind::Relic, 1.0),
    it("sampo_shard_2", "Sampo Shard II", Kind::Relic, 1.0),
    it("sampo_shard_3", "Sampo Shard III", Kind::Relic, 1.0),
    it("sampo_shard_4", "Sampo Shard IV", Kind::Relic, 1.0),
    it("sampo_shard_5", "Sampo Shard V", Kind::Relic, 1.0),
    it("gleipnir_chain", "Gleipnir Chain", Kind::Relic, 3.0),
    it("odin_eye", "Odin's Eye", Kind::Relic, 0.3),
    it("skidbladnir_ship", "Skíðblaðnir (folded)", Kind::Relic, 2.0),
    // reagents
    stack("reagent_birch_ash", "Birch Ash", Kind::Reagent, 0.1),
    stack("reagent_raven_feather", "Raven Feather", Kind::Reagent, 0.05),
    stack("reagent_amber", "Amber", Kind::Reagent, 0.1),
    stack("reagent_fly_agaric", "Fly Agaric", Kind::Reagent, 0.1),
    stack("reagent_bog_iron", "Bog Iron", Kind::Reagent, 0.3),
    // containers
    cont("backpack", "Backpack", 1.0, 40.0),
    cont("chest", "Chest", 10.0, 200.0),
    cont("pouch", "Pouch", 0.2, 8.0),
    cont("ammo_box", "Ammo Box", 0.5, 6.0),
];

pub fn get(id: &str) -> Option<&'static ItemDef> {
    ITEMS.iter().find(|d| d.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_unique_and_ammo_valid() {
        for (i, a) in ITEMS.iter().enumerate() {
            assert!(ITEMS[i + 1..].iter().all(|b| b.id != a.id), "dup {}", a.id);
            if let Some(am) = a.ammo {
                assert_eq!(get(am).unwrap().kind, Kind::Ammo);
            }
        }
        assert!(get("vainamoinen_gun").is_some());
    }
}
