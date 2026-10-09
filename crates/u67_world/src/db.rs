//! The content database, loaded from JSON in `assets/data/` (embedded fallbacks for tests/builds).
use crate::crafting::Recipe;
use crate::dialogue::DialogueDef;
use crate::loot::LootTable;
use crate::npcs::{CreatureDef, NpcDef, ShopDef, SpawnDef};
use crate::quests::QuestDef;
use crate::script::{Cond, Step};
use crate::spells::SpellDef;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::Path;

macro_rules! embedded {
    ($name:literal) => {
        include_str!(concat!("../../../assets/data/", $name))
    };
}

#[derive(Default, Clone)]
pub struct Db {
    pub quests: Vec<QuestDef>,
    pub dialogues: HashMap<String, DialogueDef>,
    pub npcs: Vec<NpcDef>,
    pub shops: HashMap<String, ShopDef>,
    pub creatures: HashMap<String, CreatureDef>,
    pub spawns: Vec<SpawnDef>,
    pub loot: HashMap<String, LootTable>,
    pub spells: Vec<SpellDef>,
    pub recipes: Vec<Recipe>,
    pub signs: HashMap<String, Vec<String>>,
}

fn parse<T: DeserializeOwned>(name: &str, s: &str) -> Result<Vec<T>, String> {
    serde_json::from_str(s).map_err(|e| format!("{name}: {e}"))
}

impl Db {
    fn from_sources(get: &dyn Fn(&str, &str) -> String) -> Result<Db, String> {
        let quests = parse("quests.json", &get("quests.json", embedded!("quests.json")))?;
        let dialogues = parse::<DialogueDef>("dialogue.json", &get("dialogue.json", embedded!("dialogue.json")))?.into_iter().map(|d| (d.id.clone(), d)).collect();
        let npcs = parse("npcs.json", &get("npcs.json", embedded!("npcs.json")))?;
        let shops = parse::<ShopDef>("shops.json", &get("shops.json", embedded!("shops.json")))?.into_iter().map(|d| (d.id.clone(), d)).collect();
        let creatures = parse::<CreatureDef>("creatures.json", &get("creatures.json", embedded!("creatures.json")))?.into_iter().map(|d| (d.id.clone(), d)).collect();
        let spawns = parse("spawns.json", &get("spawns.json", embedded!("spawns.json")))?;
        let loot = parse::<LootTable>("loot.json", &get("loot.json", embedded!("loot.json")))?.into_iter().map(|d| (d.id.clone(), d)).collect();
        let spells = parse("spells.json", &get("spells.json", embedded!("spells.json")))?;
        let recipes = parse("recipes.json", &get("recipes.json", embedded!("recipes.json")))?;
        let signs: Vec<(String, Vec<String>)> = serde_json::from_str(&get("lore.json", embedded!("lore.json"))).map_err(|e| format!("lore.json: {e}"))?;
        Ok(Db { quests, dialogues, npcs, shops, creatures, spawns, loot, spells, recipes, signs: signs.into_iter().collect() })
    }

    /// Built-in content only.
    pub fn builtin() -> Result<Db, String> {
        Self::from_sources(&|_, emb| emb.to_string())
    }

    /// Content from `dir` where files exist (modding), else the embedded version.
    pub fn load(dir: &Path) -> Result<Db, String> {
        Self::from_sources(&|name, emb| std::fs::read_to_string(dir.join("data").join(name)).unwrap_or_else(|_| emb.to_string()))
    }

    pub fn quest(&self, id: &str) -> Option<&QuestDef> {
        self.quests.iter().find(|q| q.id == id)
    }

    /// Cross-reference check; returns human-readable problems (empty = OK).
    pub fn validate(&self, places: &dyn Fn(&str) -> bool) -> Vec<String> {
        let mut bad = vec![];
        let item = |id: &str, ctx: &str, bad: &mut Vec<String>| {
            if crate::items::get(id).is_none() {
                bad.push(format!("{ctx}: unknown item '{id}'"));
            }
        };
        fn walk_steps(db: &Db, steps: &[Step], ctx: &str, bad: &mut Vec<String>, places: &dyn Fn(&str) -> bool) {
            for s in steps {
                match s {
                    Step::Give { item, .. } | Step::Take { item, .. } if crate::items::get(item).is_none() => bad.push(format!("{ctx}: unknown item '{item}'")),
                    Step::StartQuest { quest } | Step::CompleteQuest { quest } if db.quest(quest).is_none() => bad.push(format!("{ctx}: unknown quest '{quest}'")),
                    Step::Shop { shop } if !db.shops.contains_key(shop) => bad.push(format!("{ctx}: unknown shop '{shop}'")),
                    Step::Spawn { id, .. } if !db.creatures.contains_key(id) => bad.push(format!("{ctx}: unknown creature '{id}'")),
                    Step::Learn { spell } if !db.spells.iter().any(|x| &x.id == spell) => bad.push(format!("{ctx}: unknown spell '{spell}'")),
                    Step::Teleport { place } if !places(place) => bad.push(format!("{ctx}: unknown place '{place}'")),
                    Step::If { cond, then, els } => {
                        walk_cond(db, cond, ctx, bad);
                        walk_steps(db, then, ctx, bad, places);
                        walk_steps(db, els, ctx, bad, places);
                    }
                    _ => {}
                }
            }
        }
        fn walk_cond(db: &Db, c: &Cond, ctx: &str, bad: &mut Vec<String>) {
            match c {
                Cond::HasItem { item, .. } if crate::items::get(item).is_none() => bad.push(format!("{ctx}: unknown item '{item}'")),
                Cond::Quest { quest, state } => {
                    if db.quest(quest).is_none() {
                        bad.push(format!("{ctx}: unknown quest '{quest}'"));
                    }
                    if !["none", "active", "done"].contains(&state.as_str()) {
                        bad.push(format!("{ctx}: bad quest state '{state}'"));
                    }
                }
                Cond::And { all } => all.iter().for_each(|c| walk_cond(db, c, ctx, bad)),
                Cond::Or { any } => any.iter().for_each(|c| walk_cond(db, c, ctx, bad)),
                Cond::Not { cond } => walk_cond(db, cond, ctx, bad),
                _ => {}
            }
        }
        for q in &self.quests {
            let ctx = format!("quest {}", q.id);
            if q.steps.is_empty() {
                bad.push(format!("{ctx}: no steps"));
            }
            for s in &q.steps {
                walk_cond(self, &s.when, &ctx, &mut bad);
            }
            if let Some(c) = &q.start_when {
                walk_cond(self, c, &ctx, &mut bad);
            }
            walk_steps(self, &q.reward, &ctx, &mut bad, places);
        }
        for d in self.dialogues.values() {
            let ctx = format!("dialogue {}", d.id);
            for v in &d.greeting {
                if let Some(c) = &v.when {
                    walk_cond(self, c, &ctx, &mut bad);
                }
                walk_steps(self, &v.then, &ctx, &mut bad, places);
            }
            for t in &d.topics {
                if let Some(c) = &t.when {
                    walk_cond(self, c, &ctx, &mut bad);
                }
                walk_steps(self, &t.then, &ctx, &mut bad, places);
            }
        }
        for n in &self.npcs {
            if let Some(d) = &n.dialogue {
                if !self.dialogues.contains_key(d) {
                    bad.push(format!("npc {}: unknown dialogue '{d}'", n.id));
                }
            }
            if let Some(s) = &n.shop {
                if !self.shops.contains_key(s) {
                    bad.push(format!("npc {}: unknown shop '{s}'", n.id));
                }
            }
            if !u67_assetgen_names_contains(&n.arch) {
                bad.push(format!("npc {}: unknown sprite archetype '{}'", n.id, n.arch));
            }
        }
        for s in self.shops.values() {
            for e in &s.stock {
                item(&e.item, &format!("shop {}", s.id), &mut bad);
            }
        }
        for c in self.creatures.values() {
            if let Some(l) = &c.loot {
                if !self.loot.contains_key(l) {
                    bad.push(format!("creature {}: unknown loot '{l}'", c.id));
                }
            }
            if !u67_assetgen_names_contains(&c.arch) {
                bad.push(format!("creature {}: unknown sprite archetype '{}'", c.id, c.arch));
            }
        }
        for s in &self.spawns {
            if !self.creatures.contains_key(&s.creature) {
                bad.push(format!("spawn {}@{}: unknown creature '{}'", s.creature, s.place, s.creature));
            }
        }
        for t in self.loot.values() {
            for e in &t.entries {
                item(&e.item, &format!("loot {}", t.id), &mut bad);
            }
        }
        for s in &self.spells {
            for (r, _) in &s.reagents {
                item(r, &format!("spell {}", s.id), &mut bad);
            }
        }
        for r in &self.recipes {
            for (i, _) in &r.inputs {
                item(i, &format!("recipe {}", r.id), &mut bad);
            }
            item(&r.output.0, &format!("recipe {}", r.id), &mut bad);
        }
        bad
    }
}

/// Character archetype names (kept here to avoid depending on the asset generator crate).
pub const ARCHETYPES: &[&str] = &[
    "player",
    "valkyrie",
    "dwarf",
    "singer",
    "volva",
    "berserker",
    "villager_m",
    "villager_f",
    "merc",
    "draugr",
    "troll",
    "louhi",
    "jotun",
    "skald",
    "jarl",
    "warden_guard",
    "wolf",
    "fenrir",
    "ice_wolf",
];
fn u67_assetgen_names_contains(n: &str) -> bool {
    ARCHETYPES.contains(&n)
}
