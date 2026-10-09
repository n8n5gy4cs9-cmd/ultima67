//! Weighted loot tables.
use serde::{Deserialize, Serialize};
use u67_core::Rng;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LootEntry {
    pub item: String,
    pub weight: u32,
    #[serde(default = "one")]
    pub min: u32,
    #[serde(default = "one")]
    pub max: u32,
}
fn one() -> u32 {
    1
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LootTable {
    pub id: String,
    /// number of rolls (inclusive range)
    pub rolls: (u32, u32),
    /// chance that a roll yields nothing (0..1)
    #[serde(default)]
    pub nothing: f32,
    pub entries: Vec<LootEntry>,
}

impl LootTable {
    pub fn roll(&self, rng: &mut Rng) -> Vec<(String, u32)> {
        let total: u32 = self.entries.iter().map(|e| e.weight).sum();
        let mut out: Vec<(String, u32)> = vec![];
        if total == 0 {
            return out;
        }
        let n = rng.range(self.rolls.0 as i32, self.rolls.1 as i32 + 1);
        for _ in 0..n {
            if rng.chance(self.nothing) {
                continue;
            }
            let mut pick = rng.range(0, total as i32) as u32;
            for e in &self.entries {
                if pick < e.weight {
                    let q = rng.range(e.min as i32, e.max.max(e.min) as i32 + 1) as u32;
                    match out.iter_mut().find(|(i, _)| *i == e.item) {
                        Some(x) => x.1 += q,
                        None => out.push((e.item.clone(), q)),
                    }
                    break;
                }
                pick -= e.weight;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rolls_respect_weights_and_ranges() {
        let t = LootTable { id: "t".into(), rolls: (3, 3), nothing: 0.0, entries: vec![LootEntry { item: "silver".into(), weight: 1, min: 5, max: 9 }] };
        let r = t.roll(&mut Rng::new(1));
        assert_eq!(r.len(), 1);
        assert!((15..=27).contains(&r[0].1));
        let empty = LootTable { entries: vec![], ..t.clone() };
        assert!(empty.roll(&mut Rng::new(1)).is_empty());
    }
}
