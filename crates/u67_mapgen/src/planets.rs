//! One generated map per world body. Sol system (main) + Kalevala system.
use crate::sites;
use u67_core::noise::fbm;
use u67_core::{Rng, TilePos};
use u67_world::map::Map;
use u67_world::tiles::{self, TileId};

pub struct Body {
    pub id: &'static str,
    pub name: &'static str,
    pub system: &'static str,
    pub size: i32,
    /// hotspot names to generate (each becomes a named place with a site)
    pub hotspots: &'static [(&'static str, &'static str)],
}

pub const BODIES: &[Body] = &[
    Body { id: "sol", name: "Sól", system: "sol", size: 160, hotspots: &[("solarsmidja", "bunker"), ("flare_altar", "ring"), ("molten_cache", "cache"), ("sun_wreck", "wreck")] },
    Body { id: "dvalinn", name: "Dvalinn", system: "sol", size: 200, hotspots: &[("brokkr_forge", "bunker"), ("sindri_anvil", "ring"), ("magma_cache", "cache"), ("dwarf_wreck", "wreck"), ("smith_camp", "camp")] },
    Body { id: "vanaheimr", name: "Vanaheimr", system: "sol", size: 220, hotspots: &[("freyja_grove", "ring"), ("njord_dock", "camp"), ("seidr_school", "bunker"), ("vanir_cache", "cache"), ("storm_wreck", "wreck")] },
    Body { id: "maani", name: "Máni", system: "sol", size: 200, hotspots: &[("skoll_crater", "ring"), ("lunar_fort", "bunker"), ("hati_wreck", "wreck"), ("dust_cache", "cache"), ("silent_camp", "camp")] },
    Body { id: "muspelheimr", name: "Múspelheimr", system: "sol", size: 240, hotspots: &[("surtr_bunker_7", "bunker"), ("ember_ring", "ring"), ("red_wreck", "wreck"), ("fire_cache", "cache"), ("giant_camp", "camp")] },
    Body { id: "svartalfheimr", name: "Svartálfheimr", system: "sol", size: 160, hotspots: &[("smuggler_rock", "camp"), ("miner_wreck", "wreck"), ("black_cache", "cache"), ("dark_elf_bunker", "bunker")] },
    Body { id: "asgard", name: "Ásgarðr", system: "sol", size: 240, hotspots: &[("valhalla_hall", "bunker"), ("odin_vault", "bunker"), ("gjallar_ring", "ring"), ("cloud_cache", "cache"), ("bilskirnir", "camp")] },
    Body { id: "jotunheimr", name: "Jötunheimr", system: "sol", size: 240, hotspots: &[("thrymr_keep", "bunker"), ("ice_giant_ring", "ring"), ("frozen_wreck", "wreck"), ("glacier_cache", "cache"), ("rime_camp", "camp")] },
    Body { id: "niflheimr", name: "Niflheimr", system: "sol", size: 200, hotspots: &[("hvergelmir", "ring"), ("mist_wreck", "wreck"), ("frost_cache", "cache"), ("nidhogg_roots", "bunker")] },
    Body { id: "hel", name: "Hel", system: "sol", size: 220, hotspots: &[("naglfar_wreck", "wreck"), ("helheim_hall", "bunker"), ("garm_gate", "ring"), ("bone_cache", "cache"), ("nastrond", "camp")] },
    Body { id: "ginnungagap", name: "Ginnungagap", system: "sol", size: 130, hotspots: &[("sampo_gate", "ring"), ("void_wreck", "wreck"), ("gate_cache", "cache")] },
    Body { id: "vainola", name: "Väinölä", system: "kalevala", size: 220, hotspots: &[("kantele_temple", "bunker"), ("singing_grove", "ring"), ("hero_cache", "cache")] },
    Body { id: "pohjola", name: "Pohjola", system: "kalevala", size: 220, hotspots: &[("iron_fortress", "bunker"), ("aurora_ring", "ring"), ("louhi_wreck", "wreck")] },
    Body { id: "tuonela", name: "Tuonela", system: "kalevala", size: 200, hotspots: &[("swan_river", "ring"), ("tuoni_hall", "bunker"), ("dead_cache", "cache")] },
    Body { id: "ilma", name: "Ilma", system: "kalevala", size: 200, hotspots: &[("sky_anvil", "ring"), ("ukko_peak", "bunker"), ("storm_cache", "cache")] },
    Body { id: "sampola", name: "Sampola", system: "kalevala", size: 200, hotspots: &[("sampo_forge_ruin", "bunker"), ("shard_field", "ring"), ("broken_wreck", "wreck")] },
];

/// Terrain palette per body: (low, mid, high, accent, highest-wall).
fn palette(id: &str) -> [TileId; 5] {
    use tiles::*;
    match id {
        "sol" => [LAVA, ASH, FLOOR_STONE, LAVA, MOUNTAIN],
        "dvalinn" => [LAVA, ROCK, ASH, ROCK, MOUNTAIN],
        "vanaheimr" => [WATER_SHALLOW, FOREST_FLOOR, GRASS, SWAMP, MOUNTAIN],
        "maani" => [REGOLITH, REGOLITH, ROCK, ROCK, MOUNTAIN],
        "muspelheimr" => [MARS_DUST, MARS_DUST, ASH, LAVA, MOUNTAIN],
        "svartalfheimr" => [VOID, ROCK, REGOLITH, ROCK, MOUNTAIN],
        "asgard" => [GAS, GAS, FLOOR_STONE, BIFROST, WALL_STONE],
        "jotunheimr" => [SNOW, ICE, SNOW, ROCK, MOUNTAIN],
        "niflheimr" => [WATER_SHALLOW, ICE, SNOW, ICE, MOUNTAIN],
        "hel" => [SWAMP, ASH, FLOOR_STONE, ASH, MOUNTAIN],
        "ginnungagap" => [VOID, FLOOR_STONE, BIFROST, FLOOR_STONE, MOUNTAIN],
        "vainola" => [WATER_SHALLOW, GRASS, FOREST_FLOOR, GRASS, MOUNTAIN],
        "pohjola" => [ICE, SNOW, AURORA_ICE, SNOW, MOUNTAIN],
        "tuonela" => [WATER_DEEP, SWAMP, ASH, SWAMP, MOUNTAIN],
        "ilma" => [GAS, GAS, ROCK, BIFROST, MOUNTAIN],
        _ => [ASH, REGOLITH, ROCK, ASH, MOUNTAIN],
    }
}

fn site_tile_ok(m: &Map, p: TilePos, r: i32) -> bool {
    for j in (-r..=r).step_by(2) {
        for i in (-r..=r).step_by(2) {
            let q = TilePos::new(p.x + i, p.y + j);
            let d = tiles::def(m.tile(q));
            if !d.walkable || d.hazard || (d.water && !m.in_bounds(q)) {
                return false;
            }
        }
    }
    true
}

pub fn generate(b: &Body, seed: u64) -> Map {
    let n = b.size;
    let pal = palette(b.id);
    let s = seed ^ (b.id.len() as u64 * 7919) ^ (b.id.as_bytes()[0] as u64 * 104729);
    let mut m = Map::new(b.id, n, n, pal[1]);
    let free_space = b.id == "svartalfheimr";
    for y in 0..n {
        for x in 0..n {
            let (fx, fy) = (x as f32 / n as f32, y as f32 / n as f32);
            let edge = (1.0 - ((fx - 0.5).abs() * 2.0).max((fy - 0.5).abs() * 2.0)).clamp(0.0, 1.0);
            let hgt = fbm(x as f32 / 38.0, y as f32 / 38.0, s, 4) * (0.55 + 0.6 * edge.sqrt());
            let acc = fbm(x as f32 / 20.0, y as f32 / 20.0, s + 3, 3);
            let t = if free_space {
                // asteroid islands in the void
                if fbm(x as f32 / 14.0, y as f32 / 14.0, s, 3) > 0.52 { if acc > 0.6 { pal[2] } else { pal[1] } } else { pal[0] }
            } else if hgt < 0.30 {
                pal[0]
            } else if hgt < 0.50 {
                if acc > 0.66 { pal[3] } else { pal[1] }
            } else if hgt < 0.64 {
                if acc > 0.7 { pal[3] } else { pal[2] }
            } else {
                pal[4]
            };
            m.set_tile(TilePos::new(x, y), t);
        }
    }
    let mut rng = Rng::new(s + 17);
    // scatter scenery
    for y in 0..n {
        for x in 0..n {
            let q = TilePos::new(x, y);
            let t = m.tile(q);
            if !tiles::def(t).walkable || tiles::def(t).hazard {
                continue;
            }
            if rng.chance(0.012) {
                let kind = match b.system {
                    "kalevala" if t == tiles::GRASS || t == tiles::FOREST_FLOOR => "birch_tree",
                    _ if t == tiles::FOREST_FLOOR || t == tiles::GRASS => "pine_tree",
                    _ if t == tiles::SNOW || t == tiles::AURORA_ICE => "pine_tree",
                    _ => "boulder",
                };
                m.add_object(kind, q);
            }
        }
    }
    // landing site near the middle: a Bifrost node and the player's ship
    let mut landing = TilePos::new(n / 2, n / 2);
    'find: for rad in (0..n / 2).step_by(2) {
        for k in 0..16 {
            let a = k as f32 / 16.0 * std::f32::consts::TAU;
            let q = TilePos::new(n / 2 + (a.cos() * rad as f32) as i32, n / 2 + (a.sin() * rad as f32) as i32);
            if site_tile_ok(&m, q, 4) {
                landing = q;
                break 'find;
            }
        }
    }
    for j in -2..=2 {
        for i in -2..=2 {
            let q = TilePos::new(landing.x + i, landing.y + j);
            m.set_tile(q, tiles::FLOOR_STONE);
            m.objects.retain(|o| o.pos != q);
        }
    }
    m.add_object("bifrost_node", landing);
    m.add_object("longship", TilePos::new(landing.x + 3, landing.y));
    m.places.insert("landing".into(), TilePos::new(landing.x, landing.y + 1));
    // hotspots
    for (name, kind) in b.hotspots {
        let mut tries = 0;
        let site = loop {
            let q = TilePos::new(rng.range(12, n - 12), rng.range(12, n - 12));
            tries += 1;
            if site_tile_ok(&m, q, 6) && q.manhattan(landing) > 20 || tries > 4000 {
                break q;
            }
        };
        match *kind {
            "bunker" => sites::bunker(&mut m, site.x - 4, site.y - 3, &mut rng),
            "ring" => sites::rune_ring(&mut m, site, 4, 6),
            "wreck" => sites::wreck_site(&mut m, site, &mut rng),
            "cache" => sites::cache(&mut m, site, &mut rng),
            _ => sites::camp(&mut m, site, &mut rng),
        }
        m.places.insert(name.to_string(), site);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_match_design() {
        assert_eq!(BODIES.iter().filter(|b| b.system == "kalevala").count(), 5);
        assert!(BODIES.iter().filter(|b| b.system == "sol").map(|b| b.hotspots.len()).sum::<usize>() >= 40);
    }
    #[test]
    fn every_body_generates_with_landing_and_hotspots() {
        for b in BODIES {
            let m = generate(b, 67);
            assert!(m.walkable(m.places["landing"]), "{} landing", b.id);
            for (h, _) in b.hotspots {
                assert!(m.places.contains_key(*h), "{} {}", b.id, h);
            }
        }
    }
}
