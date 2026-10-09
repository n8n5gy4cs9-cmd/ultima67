//! Ultima 7 -> Midgard importer. Reads the user's own legally-owned U7 files (never shipped
//! with the game) and converts the world map into a Midgard `.u67map` the game loads from
//! `assets/maps/`. Format knowledge: see `docs/formats/u7.md`.
pub mod classify;
pub mod flex;
pub mod import;
pub mod norse;
pub mod u7;

use std::path::Path;
use u67_world::map::World;

/// Full pipeline: load U7 data from `dir`, import, transform, return the world (Midgard + caves).
pub fn run(dir: &Path, rules: &classify::Rules, seed: u64) -> Result<(World, Vec<String>), String> {
    let d = u7::load(dir)?;
    let (mut m, rep) = import::import(&d, rules);
    let mut log = vec![format!("imported {} fixed objects: {rep:?}", d.fixed.len())];
    let mut world = World::default();
    log.extend(norse::transform(&mut m, &mut world, seed));
    world.insert(m);
    Ok((world, log))
}
