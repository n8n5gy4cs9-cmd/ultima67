//! `cargo run --release -p u67_import --bin import_u7 -- <U7 STATIC dir> [--out assets/maps] [--rules rules.json] [--seed N] [--dump-rules file.json]`
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut dir: Option<PathBuf> = None;
    let mut out = PathBuf::from("assets/maps");
    let mut rules_path: Option<PathBuf> = None;
    let mut seed = u67_mapgen::DEFAULT_SEED;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out = args.next().expect("--out DIR").into(),
            "--rules" => rules_path = args.next().map(Into::into),
            "--seed" => seed = args.next().and_then(|s| s.parse().ok()).expect("--seed N"),
            "--dump-rules" => {
                let p = args.next().expect("--dump-rules FILE");
                std::fs::write(&p, serde_json::to_string_pretty(&u67_import::classify::Rules::default()).unwrap()).unwrap();
                println!("wrote default rules to {p}");
                return;
            }
            other => dir = Some(other.into()),
        }
    }
    let dir = dir.or_else(|| Some(PathBuf::from("assets/original/u7"))).unwrap();
    let rules = match rules_path.or_else(|| Some(PathBuf::from("assets/original/u7_rules.json")).filter(|p| p.exists())) {
        Some(p) => serde_json::from_str(&std::fs::read_to_string(&p).expect("read rules")).expect("parse rules"),
        None => u67_import::classify::Rules::default(),
    };
    match u67_import::run(&dir, &rules, seed) {
        Ok((world, log)) => {
            std::fs::create_dir_all(&out).expect("create out dir");
            for l in &log {
                println!("{l}");
            }
            for (name, m) in &world.maps {
                let b = u67_world::mapio::to_bytes(m);
                std::fs::write(out.join(format!("{name}.u67map")), &b).expect("write map");
                println!("wrote {name}.u67map ({} KB)", b.len() / 1024);
            }
            println!("Done. Start the game; it loads these maps instead of generating Midgard.");
        }
        Err(e) => {
            eprintln!(
                "import failed: {e}\nPut your Ultima 7 STATIC files (u7map, u7chunks, u7ifix*, shapes.vga, palettes.flx, text.flx, tfa.dat) in {} .",
                dir.display()
            );
            std::process::exit(1);
        }
    }
}
