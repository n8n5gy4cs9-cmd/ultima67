//! Quest definitions. A quest auto-advances: each step has a `when` condition; when the
//! current step's condition holds the quest moves on; after the last step it completes and
//! pays out `reward`.
use crate::script::{Cond, Step};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestStep {
    pub text: String,
    pub when: Cond,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Main,
    Side,
    Optional,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestDef {
    pub id: String,
    pub title: String,
    pub desc: String,
    pub category: Category,
    pub steps: Vec<QuestStep>,
    #[serde(default)]
    pub reward: Vec<Step>,
    /// If set, the quest starts automatically as soon as this holds.
    #[serde(default)]
    pub start_when: Option<Cond>,
}
