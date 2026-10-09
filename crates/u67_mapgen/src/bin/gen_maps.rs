//! `cargo run --release -p u67_mapgen --bin gen_maps [out_dir]` dumps every map as .u67map for the editor/inspection.
use std::path::PathBuf;
fn main() {
    let out: PathBuf = std::env::args().nth(1).unwrap_or_else(|| "assets/maps".into()).into();
    std::fs::create_dir_all(&out).unwrap();
    let w = u67_mapgen::generate_world(u67_mapgen::DEFAULT_SEED);
    for (name, m) in &w.maps {
        let b = u67_world::mapio::to_bytes(m);
        std::fs::write(out.join(format!("{name}.u67map")), &b).unwrap();
        if std::env::var("PREVIEW").is_ok() {
            let mut img = image::RgbImage::new(m.width as u32, m.height as u32);
            for (i, t) in m.tiles.iter().enumerate() {
                let c = u67_world::tiles::def(*t).color;
                img.put_pixel((i as i32 % m.width) as u32, (i as i32 / m.width) as u32, image::Rgb(c));
            }
            for o in &m.objects {
                let c = match o.kind.as_str() { "pine_tree" => [20, 70, 40], "birch_tree" => [110, 150, 50], "runestone" | "bifrost_node" => [120, 240, 255], "longhouse_roof" | "door_wood" => [90, 50, 30], _ => continue };
                img.put_pixel(o.pos.x as u32, o.pos.y as u32, image::Rgb(c));
            }
            for (k, p) in &m.places {
                if !k.contains('_') { for d in -2..=2 { for e in -2..=2 { if (p.x + d) >= 0 && (p.y + e) >= 0 && p.x + d < m.width && p.y + e < m.height { img.put_pixel((p.x + d) as u32, (p.y + e) as u32, image::Rgb([255, 0, 0])); } } } }
            }
            img.save(out.join(format!("{name}.png"))).unwrap();
        }
        println!("{name}: {}x{} {} objects, {} places, {} KB", m.width, m.height, m.objects.len(), m.places.len(), b.len() / 1024);
    }
}
