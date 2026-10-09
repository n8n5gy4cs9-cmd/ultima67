//! Procedural world generation: Midgard, caves, and every planet map.
pub mod cave;
pub mod midgard;
pub mod planets;
pub mod sites;

use u67_world::map::World;

pub const DEFAULT_SEED: u64 = 67;

/// Build the whole game world deterministically from `seed`.
pub fn generate_world(seed: u64) -> World {
    let mut w = World::default();
    let m = midgard::generate(seed, &mut w);
    w.insert(m);
    for b in planets::BODIES {
        w.insert(planets::generate(b, seed));
    }
    w
}
