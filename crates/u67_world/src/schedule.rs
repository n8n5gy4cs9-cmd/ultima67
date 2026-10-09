//! NPC daily schedules: up to 8 three-hour slots, each an activity at a place.
use serde::{Deserialize, Serialize};
use u67_core::TilePos;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Sleep,
    Eat,
    Work,
    TendShop,
    Farm,
    Forge,
    Patrol,
    Pray,
    Tavern,
    Loiter,
    Wander,
    Stand,
    Dance,
    Fish,
    Guard,
    FollowLeader,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Slot3h {
    /// Slot index 0..8 (0 = 00:00-03:00, 1 = 03:00-06:00 ...).
    pub slot: u8,
    pub activity: Activity,
    pub pos: TilePos,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Schedule {
    pub entries: Vec<Slot3h>,
}

impl Schedule {
    pub fn set(&mut self, slot: u8, activity: Activity, pos: TilePos) {
        self.entries.retain(|e| e.slot != slot);
        self.entries.push(Slot3h { slot, activity, pos });
        self.entries.sort_by_key(|e| e.slot);
    }
    /// The entry in force at `hour`: the latest slot at or before it, wrapping around midnight.
    pub fn at_hour(&self, hour: u32) -> Option<&Slot3h> {
        let s = (hour / 3) as u8;
        self.entries.iter().rev().find(|e| e.slot <= s).or_else(|| self.entries.last())
    }
    /// A typical townsperson template around a home, work and tavern location.
    pub fn townsperson(home: TilePos, work: TilePos, tavern: TilePos) -> Self {
        let mut s = Self::default();
        s.set(0, Activity::Sleep, home);
        s.set(2, Activity::Eat, home);
        s.set(3, Activity::Work, work);
        s.set(5, Activity::Eat, tavern);
        s.set(6, Activity::Tavern, tavern);
        s.set(7, Activity::Sleep, home);
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picks_slot() {
        let (h, w, t) = (TilePos::new(0, 0), TilePos::new(5, 5), TilePos::new(9, 9));
        let s = Schedule::townsperson(h, w, t);
        assert_eq!(s.at_hour(2).unwrap().activity, Activity::Sleep);
        assert_eq!(s.at_hour(10).unwrap().pos, w);
        assert_eq!(s.at_hour(15).unwrap().activity, Activity::Eat);
        assert_eq!(s.at_hour(23).unwrap().activity, Activity::Sleep);
    }
    #[test]
    fn wraps_before_first_slot() {
        let mut s = Schedule::default();
        s.set(2, Activity::Work, TilePos::new(1, 1));
        assert_eq!(s.at_hour(1).unwrap().activity, Activity::Work);
    }
}
