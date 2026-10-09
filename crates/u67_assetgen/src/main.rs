//! `cargo run -p u67_assetgen [assets_dir] [assets_md_path]`
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let root: PathBuf = std::env::args().nth(1).unwrap_or_else(|| "assets".into()).into();
    let md: PathBuf = std::env::args().nth(2).unwrap_or_else(|| "ASSETS.md".into()).into();
    let m = u67_assetgen::generate_all(&root)?;
    m.save(&root)?;
    std::fs::write(&md, u67_assetgen::docgen::assets_md(&m)).map_err(|e| e.to_string())?;
    println!("generated {} assets -> {} (+ {})", m.entries.len(), root.display(), md.display());
    Ok(())
}
