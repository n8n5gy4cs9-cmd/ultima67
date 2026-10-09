//! Local multiplayer ("seats"): one seat per player (1-4), each with an input device, a camera
//! viewport and runtime combat state. Game data is shared; code written for "the player" keeps
//! using `players[0]` by temporarily swapping the active seat into slot 0 (see [`with_seat`]).
use crate::data::{GameData, PlayerData};
use bevy::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Keyboard1,
    Keyboard2,
    Gamepad(Entity),
}

#[derive(Resource, Default)]
pub struct Seats {
    pub devices: Vec<Device>,
}

pub const MAX_SEATS: usize = 4;
pub const SEAT_NAMES: [&str; 4] = ["Rune-Warden", "Sigrun", "Bjorn", "Brokk"];
pub const SEAT_ARCH: [&str; 4] = ["player", "valkyrie", "berserker", "dwarf"];

/// Run `f` with seat `i` swapped into `players[0]`.
pub fn with_seat<R>(d: &mut GameData, i: usize, f: impl FnOnce(&mut GameData) -> R) -> R {
    if i == 0 || i >= d.players.len() {
        return f(d);
    }
    d.players.swap(0, i);
    let r = f(d);
    d.players.swap(0, i);
    r
}

impl PlayerData {
    /// Starting character for co-op seat `i` (1..=3).
    pub fn for_seat(i: usize, pos: [f32; 2]) -> PlayerData {
        let mut p = PlayerData::new(SEAT_NAMES[i.min(3)], pos);
        let inv = &mut p.inventory;
        inv.unequip(u67_world::items::Slot::HandR).ok(); // swap the starter pistol for the class weapon
        match i {
            1 => {
                inv.add("round_spear", 1).ok();
                inv.add("round_shield", 1).ok();
                if let Some(ix) = inv.pack.iter().position(|it| it.id == "round_spear") {
                    inv.equip(&[ix]).ok();
                }
                if let Some(ix) = inv.pack.iter().position(|it| it.id == "round_shield") {
                    inv.equip(&[ix]).ok();
                }
                p.stats.str_ += 3;
            }
            2 => {
                inv.add("viking_axe", 1).ok();
                inv.add("horned_helm", 1).ok();
                if let Some(ix) = inv.pack.iter().position(|it| it.id == "viking_axe") {
                    inv.equip(&[ix]).ok();
                }
                p.stats.str_ += 5;
            }
            _ => {
                inv.add("hunting_shotgun", 1).ok();
                inv.add("ammo_shell", 24).ok();
                if let Some(ix) = inv.pack.iter().position(|it| it.id == "hunting_shotgun") {
                    inv.equip(&[ix]).ok();
                }
                p.loaded.insert("hunting_shotgun".into(), 6);
                p.stats.dex += 2;
            }
        }
        p.stats.hp = p.stats.max_hp();
        p.inventory.max_weight = p.stats.str_ as f32 * 2.0 + 20.0;
        p
    }
}

/// Per-seat input for this frame.
#[derive(Clone, Copy, Default, Debug)]
pub struct Intent {
    pub movement: Vec2,
    pub run: bool,
    pub interact: bool,
    pub attack: bool,
    pub attack_held: bool,
    pub roll: bool,
    pub reload: bool,
    pub inventory: bool,
    pub pause: bool,
    /// Right-stick aim (zero when the seat uses the mouse or has no aim input).
    pub aim: Vec2,
    pub spells: [bool; 3],
}

#[derive(Resource, Default)]
pub struct Intents {
    pub list: Vec<Intent>,
    pub active: usize,
}
impl Deref for Intents {
    type Target = Intent;
    fn deref(&self) -> &Intent {
        &self.list[self.active.min(self.list.len().saturating_sub(1))]
    }
}
impl Intents {
    pub fn get(&self, i: usize) -> Intent {
        self.list.get(i).copied().unwrap_or_default()
    }
}

/// Runtime (not saved) combat state of one seat.
#[derive(Clone, Default)]
pub struct SeatRt {
    pub cooldown: f32,
    pub reload: Option<(f32, String)>,
    pub iframes: f32,
    pub roll_t: f32,
    pub roll_cd: f32,
    pub roll_dir: Vec2,
    pub haste: f32,
    pub protect: (i32, f32),
    pub light: f32,
    pub shake: f32,
    pub hurt_flash: f32,
    /// Downed in co-op: waiting for a revive.
    pub downed: bool,
    pub down_timer: f32,
}

/// All seats' runtime state; derefs to the active seat so `rt.cooldown` works inside seat loops.
#[derive(Resource)]
pub struct PlayerRt {
    pub list: Vec<SeatRt>,
    pub active: usize,
}
impl Default for PlayerRt {
    fn default() -> Self {
        Self { list: vec![SeatRt::default(); MAX_SEATS], active: 0 }
    }
}
impl Deref for PlayerRt {
    type Target = SeatRt;
    fn deref(&self) -> &SeatRt {
        &self.list[self.active]
    }
}
impl DerefMut for PlayerRt {
    fn deref_mut(&mut self) -> &mut SeatRt {
        &mut self.list[self.active]
    }
}

/// Which cannon (object index) each seat is operating.
#[derive(Resource)]
pub struct Operating {
    pub list: Vec<Option<usize>>,
    pub active: usize,
}
impl Default for Operating {
    fn default() -> Self {
        Self { list: vec![None; MAX_SEATS], active: 0 }
    }
}
impl Operating {
    pub fn get(&self) -> Option<usize> {
        self.list[self.active]
    }
    pub fn set(&mut self, v: Option<usize>) {
        self.list[self.active] = v;
    }
}

/// The seat that opened the current modal screen (inventory, dialogue, menu).
#[derive(Resource, Default)]
pub struct ActiveSeat(pub usize);

/// Viewport rectangles (x, y, w, h) in physical pixels for `n` seats in a `w` x `h` window.
pub fn layout(n: usize, w: u32, h: u32) -> Vec<(u32, u32, u32, u32)> {
    match n {
        0 | 1 => vec![(0, 0, w, h)],
        2 => vec![(0, 0, w / 2, h), (w / 2, 0, w - w / 2, h)],
        3 => vec![(0, 0, w / 2, h / 2), (w / 2, 0, w - w / 2, h / 2), (w / 4, h / 2, w / 2, h - h / 2)],
        _ => vec![(0, 0, w / 2, h / 2), (w / 2, 0, w - w / 2, h / 2), (0, h / 2, w / 2, h - h / 2), (w / 2, h / 2, w - w / 2, h - h / 2)],
    }
}

/// Assign devices to seats: seat 0 keyboard+mouse; further seats prefer gamepads, then the second keyboard.
pub fn assign_devices(n: usize, pads: &[Entity]) -> Vec<Device> {
    let mut out = vec![Device::Keyboard1];
    let mut pad_iter = pads.iter();
    let mut kb2_free = true;
    for _ in 1..n {
        if let Some(p) = pad_iter.next() {
            out.push(Device::Gamepad(*p));
        } else if kb2_free {
            kb2_free = false;
            out.push(Device::Keyboard2);
        } else {
            out.push(Device::Keyboard2); // nobody left: shares (documented limitation)
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swapping_seats_is_symmetric() {
        let mut d = GameData::new(1, "midgard", [5.0, 5.0]);
        d.players.push(PlayerData::for_seat(1, [6.0, 5.0]));
        d.players.push(PlayerData::for_seat(2, [7.0, 5.0]));
        let name = with_seat(&mut d, 2, |d| {
            d.players[0].stats.hp -= 5;
            d.players[0].name.clone()
        });
        assert_eq!(name, "Bjorn");
        assert_eq!(d.players[0].name, "Rune-Warden");
        assert_eq!(d.players[2].stats.hp, d.players[2].stats.max_hp() - 5);
        assert_eq!(d.players[1].name, "Sigrun");
    }

    #[test]
    fn viewports_tile_the_window_without_overlap() {
        for n in 1..=4 {
            let l = layout(n, 1281, 721);
            assert_eq!(l.len(), n);
            for (i, a) in l.iter().enumerate() {
                assert!(a.0 + a.2 <= 1281 && a.1 + a.3 <= 721);
                for b in &l[i + 1..] {
                    let overlap = a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3;
                    assert!(!overlap, "{n}: {a:?} vs {b:?}");
                }
            }
        }
    }

    #[test]
    fn coop_characters_have_distinct_kits() {
        for i in 1..=3 {
            let p = PlayerData::for_seat(i, [0.0, 0.0]);
            assert!(p.inventory.equipped.contains_key(&u67_world::items::Slot::HandR), "seat {i}");
            assert!(p.stats.hp > 0);
        }
        assert_ne!(
            PlayerData::for_seat(1, [0.0; 2]).inventory.equipped[&u67_world::items::Slot::HandR].id,
            PlayerData::for_seat(2, [0.0; 2]).inventory.equipped[&u67_world::items::Slot::HandR].id
        );
    }

    #[test]
    fn device_assignment_prefers_pads() {
        let a = Entity::from_raw(1);
        let b = Entity::from_raw(2);
        assert_eq!(assign_devices(1, &[a]), vec![Device::Keyboard1]);
        assert_eq!(assign_devices(2, &[]), vec![Device::Keyboard1, Device::Keyboard2]);
        assert_eq!(assign_devices(3, &[a, b]), vec![Device::Keyboard1, Device::Gamepad(a), Device::Gamepad(b)]);
    }
}

/// Which seat (if any) is currently swapped into `players[0]` for a modal screen.
#[derive(Resource, Default)]
pub struct SeatSwap(pub Option<usize>);

impl SeatSwap {
    pub fn swap_out(&mut self, d: &mut GameData) {
        if let Some(s) = self.0.take() {
            if s < d.players.len() {
                d.players.swap(0, s);
            }
        }
    }
    pub fn swap_in(&mut self, d: &mut GameData, seat: usize) {
        self.swap_out(d);
        if seat != 0 && seat < d.players.len() {
            d.players.swap(0, seat);
            self.0 = Some(seat);
        }
    }
}
