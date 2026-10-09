//! Ultima 7 style keyword dialogue. The player picks (or types) keywords; NPCs answer and
//! reveal new keywords. Text may contain `{name}` (player) and `{town}` placeholders.
use crate::script::{Cond, Step};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Variant {
    #[serde(default)]
    pub when: Option<Cond>,
    pub text: String,
    #[serde(default)]
    pub then: Vec<Step>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Topic {
    pub keys: Vec<String>,
    #[serde(default)]
    pub when: Option<Cond>,
    pub text: String,
    #[serde(default)]
    pub then: Vec<Step>,
    #[serde(default)]
    pub reveals: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueDef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub greeting: Vec<Variant>,
    pub topics: Vec<Topic>,
    #[serde(default = "default_bye")]
    pub farewell: String,
}

fn default_bye() -> String {
    "Farewell.".into()
}

pub trait CondCtx {
    fn check(&self, c: &Cond) -> bool;
}

#[derive(Clone, Debug, Default)]
pub struct Reply {
    pub text: String,
    pub steps: Vec<Step>,
    pub end: bool,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub known: Vec<String>,
    pub spent: HashSet<usize>,
}

fn norm(s: &str) -> String {
    s.trim().to_lowercase()
}

/// U7 matches on the first four letters of a keyword.
fn matches_key(key: &str, input: &str) -> bool {
    let (k, i) = (norm(key), norm(input));
    if k == i {
        return true;
    }
    i.len() >= 4 && k.len() >= 4 && k[..4] == i[..4]
}

impl Session {
    pub fn new() -> Self {
        Self { known: vec!["name".into(), "job".into(), "bye".into()], spent: HashSet::new() }
    }

    pub fn greet(&mut self, def: &DialogueDef, ctx: &dyn CondCtx) -> Reply {
        for v in &def.greeting {
            if v.when.as_ref().is_none_or(|c| ctx.check(c)) {
                return Reply { text: v.text.clone(), steps: v.then.clone(), end: false };
            }
        }
        Reply { text: format!("{} nods at you.", if def.name.is_empty() { "They" } else { &def.name }), ..Default::default() }
    }

    /// Respond to a keyword. Unknown topics get a shrug.
    pub fn say(&mut self, def: &DialogueDef, input: &str, ctx: &dyn CondCtx) -> Reply {
        if matches_key("bye", input) {
            return Reply { text: def.farewell.clone(), steps: vec![], end: true };
        }
        for (i, t) in def.topics.iter().enumerate() {
            if !t.keys.iter().any(|k| matches_key(k, input)) {
                continue;
            }
            if !t.when.as_ref().is_none_or(|c| ctx.check(c)) {
                continue;
            }
            for r in &t.reveals {
                if !self.known.iter().any(|k| norm(k) == norm(r)) {
                    self.known.push(r.clone());
                }
            }
            self.spent.insert(i);
            return Reply { text: t.text.clone(), steps: t.then.clone(), end: false };
        }
        Reply { text: "I know nothing about that.".into(), ..Default::default() }
    }

    /// Keywords currently worth showing: known ones that still have a matching, enabled topic.
    pub fn visible(&self, def: &DialogueDef, ctx: &dyn CondCtx) -> Vec<String> {
        self.known
            .iter()
            .filter(|k| k.as_str() == "bye" || def.topics.iter().any(|t| t.keys.iter().any(|tk| matches_key(tk, k)) && t.when.as_ref().is_none_or(|c| ctx.check(c))))
            .cloned()
            .collect()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Ctx(Vec<&'static str>);
    impl CondCtx for Ctx {
        fn check(&self, c: &Cond) -> bool {
            matches!(c, Cond::Flag { flag } if self.0.contains(&flag.as_str()))
        }
    }
    fn def() -> DialogueDef {
        serde_json::from_str(r#"{"id":"x","name":"Eirik","greeting":[{"when":{"is":"flag","flag":"met"},"text":"Again!"},{"text":"Hail."}],
        "topics":[{"keys":["name"],"text":"I am Eirik.","reveals":["gate"]},{"keys":["job"],"text":"Skald."},
        {"keys":["gate"],"text":"It hums.","then":[{"op":"set_flag","flag":"heard_gate"}],"reveals":["Bifrost"]},
        {"keys":["Bifrost"],"when":{"is":"flag","flag":"met"},"text":"Rainbow bridge."}]}"#).unwrap()
    }
    #[test]
    fn greet_variants_and_topics() {
        let d = def();
        let mut s = Session::new();
        assert_eq!(s.greet(&d, &Ctx(vec![])).text, "Hail.");
        assert_eq!(s.greet(&d, &Ctx(vec!["met"])).text, "Again!");
        assert_eq!(s.say(&d, "NAME", &Ctx(vec![])).text, "I am Eirik.");
        assert!(s.known.contains(&"gate".to_string()));
        let r = s.say(&d, "gate", &Ctx(vec![]));
        assert_eq!(r.steps.len(), 1);
        assert!(s.known.contains(&"Bifrost".to_string()));
        // Bifrost gated by flag, and hidden from the keyword list until then
        assert!(!s.visible(&d, &Ctx(vec![])).contains(&"Bifrost".to_string()));
        assert_eq!(s.say(&d, "bifrost", &Ctx(vec![])).text, "I know nothing about that.");
        assert_eq!(s.say(&d, "bifrost", &Ctx(vec!["met"])).text, "Rainbow bridge.");
        assert!(s.say(&d, "bye", &Ctx(vec![])).end);
        // first-four-letters matching
        assert_eq!(s.say(&d, "gate", &Ctx(vec![])).text, "It hums.");
    }
}
