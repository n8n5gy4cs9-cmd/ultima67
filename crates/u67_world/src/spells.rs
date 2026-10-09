//! Spells: U7-style reagent + circle magic, plus Kalevala "laulu" (sung) casting by rune sequence.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Effect {
    /// Projectile that damages.
    Bolt {
        damage: i32,
        speed: f32,
        range: f32,
        #[serde(default)]
        splash: f32,
        #[serde(default)]
        element: String,
    },
    Heal {
        amount: i32,
    },
    Light {
        seconds: f32,
    },
    Haste {
        seconds: f32,
    },
    Protect {
        amount: i32,
        seconds: f32,
    },
    Blink {
        range: f32,
    },
    Summon {
        creature: String,
        count: u32,
        seconds: f32,
    },
    Fear {
        radius: f32,
        seconds: f32,
    },
    Freeze {
        radius: f32,
        seconds: f32,
    },
    Missiles {
        count: u32,
        damage: i32,
        range: f32,
    },
    Reveal {
        radius: f32,
    },
    Cure,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpellDef {
    pub id: String,
    pub name: String,
    pub circle: u32,
    pub mana: i32,
    #[serde(default)]
    pub reagents: Vec<(String, u32)>,
    /// laulu: sequence of rune indices (0..24, see ui.json rune_N) that casts this spell
    pub runes: Vec<u8>,
    pub effect: Effect,
    pub desc: String,
    /// learned by default?
    #[serde(default)]
    pub known: bool,
}

pub fn find_by_runes<'a>(spells: &'a [SpellDef], seq: &[u8]) -> Option<&'a SpellDef> {
    spells.iter().find(|s| s.runes == seq)
}
