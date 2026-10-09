use std::path::PathBuf;
use u67_assetgen::{canvas::Canvas, characters, manifest::*, objects, terrain};

fn main() -> Result<(), String> {
    let root: PathBuf = std::env::args().nth(1).unwrap_or_else(|| "assets".into()).into();
    let mut m = Manifest::default();
    let mut a = Atlas::new("gfx/terrain.png");
    let s: Canvas = terrain::sheet(&mut a);
    s.save(&root.join("gfx/terrain.png"))?;
    a.save(&root.join("gfx/terrain.json"))?;
    m.add("terrain", "sheet", "gfx/terrain.png", "terrain tiles", "16x16 x4 variants");
    let mut a = Atlas::new("gfx/objects.png");
    objects::sheet(&mut a).save(&root.join("gfx/objects.png"))?;
    a.save(&root.join("gfx/objects.json"))?;
    m.add("objects", "sheet", "gfx/objects.png", "objects", "32x32 cells");
    let mut a = Atlas::new("gfx/characters.png");
    characters::sheet(&mut a).save(&root.join("gfx/characters.png"))?;
    a.save(&root.join("gfx/characters.json"))?;
    m.add("characters", "sheet", "gfx/characters.png", "characters + creatures", "24x24 cells; 11 cols: idle,walk1-4,attack1-3,die1-3; 4 rows per archetype N,E,S,W");
    m.save(&root)?;
    Ok(())
}
