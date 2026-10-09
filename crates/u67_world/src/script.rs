//! Data-driven scripting ("usecode lite"): steps and conditions authored in JSON.
//! Executed by the game against its state (see `u67_game::script`).
use serde::{Deserialize, Serialize};

fn one() -> u32 {
    1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Step {
    Say { text: String },
    Toast { text: String },
    SetFlag { flag: String },
    ClearFlag { flag: String },
    Inc { counter: String },
    Give { item: String, #[serde(default = "one")] count: u32 },
    Take { item: String, #[serde(default = "one")] count: u32 },
    Money { amount: i64 },
    Xp { amount: u32 },
    Heal,
    StartQuest { quest: String },
    CompleteQuest { quest: String },
    JoinParty { npc: String },
    LeaveParty { npc: String },
    Teleport { place: String },
    Travel { body: String },
    Sfx { id: String },
    Spawn { id: String, #[serde(default = "one")] count: u32 },
    Shop { shop: String },
    Craft { station: String },
    Sleep,
    UnlockTravel { node: String },
    Learn { spell: String },
    If { cond: Cond, then: Vec<Step>, #[serde(default)] els: Vec<Step> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "is", rename_all = "snake_case")]
pub enum Cond {
    Flag { flag: String },
    NotFlag { flag: String },
    HasItem { item: String, #[serde(default = "one")] count: u32 },
    Counter { counter: String, min: u32 },
    /// state: "none" | "active" | "done"
    Quest { quest: String, state: String },
    MinLevel { level: u32 },
    InMap { map: String },
    Night,
    Day,
    InParty { npc: String },
    And { all: Vec<Cond> },
    Or { any: Vec<Cond> },
    Not { cond: Box<Cond> },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_roundtrip() {
        let j = r#"[{"op":"give","item":"rope"},{"op":"if","cond":{"is":"and","all":[{"is":"flag","flag":"a"},{"is":"not","cond":{"is":"night"}}]},"then":[{"op":"say","text":"hi"}]}]"#;
        let v: Vec<Step> = serde_json::from_str(j).unwrap();
        assert_eq!(v[0], Step::Give { item: "rope".into(), count: 1 });
        assert!(matches!(v[1], Step::If { .. }));
        let back = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<Vec<Step>>(&back).unwrap(), v);
    }
}
