//! World rendering: terrain baked per chunk into one texture, objects as sprites, roofs, animation.
use crate::app::{AppState, Game, Paths, SettingsRes, WorldRes};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::Anchor;
use std::collections::{HashMap, HashSet};
use u67_core::{noise, CHUNK};
use u67_world::map::Map;
use u67_world::objects::{self, WorldObject};
use u67_world::tiles;

pub const T: f32 = 16.0;
pub const OBJ_CELL: u32 = 32;
pub const CHAR_CELL: u32 = 24;
pub const CHAR_COLS: u32 = 11;

pub fn tile_px(x: f32, y: f32) -> Vec2 {
    Vec2::new(x * T, -y * T)
}

pub struct TerrainPx {
    pub w: u32,
    pub data: Vec<u8>,
}

#[derive(Resource)]
pub struct Sheets {
    pub terrain_path: std::path::PathBuf,
    pub terrain_mtime: Option<std::time::SystemTime>,
    pub objects_img: Handle<Image>,
    pub objects_layout: Handle<TextureAtlasLayout>,
    pub chars_img: Handle<Image>,
    pub chars_layout: Handle<TextureAtlasLayout>,
    pub items_img: Handle<Image>,
    pub items_layout: Handle<TextureAtlasLayout>,
    pub terrain: TerrainPx,
}

/// Chunks whose terrain/objects changed (door opened, wall destroyed...) and must be rebuilt.
#[derive(Resource, Default)]
pub struct DirtyChunks(pub HashSet<(i32, i32)>);

#[derive(Resource, Default)]
struct Loaded {
    map: String,
    chunks: HashMap<(i32, i32), Vec<Entity>>,
}

#[derive(Resource, Default)]
pub struct ChunkIndex {
    pub map: String,
    pub by_chunk: HashMap<(i32, i32), Vec<usize>>,
}

#[derive(Component)]
pub struct Roof(pub u16);
#[derive(Component)]
pub struct ObjRef(pub usize);
#[derive(Component)]
pub struct Flicker {
    pub row: u32,
    pub frames: u32,
}

pub fn character_row(name: &str) -> Option<u32> {
    u67_assetgen::characters::names().iter().position(|n| *n == name).map(|i| i as u32)
}

pub fn load_sheets(mut commands: Commands, assets: Res<AssetServer>, paths: Res<Paths>, mut layouts: ResMut<Assets<TextureAtlasLayout>>) {
    let rows = (objects::OBJECTS.len() + 1) as u32;
    let objects_layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(OBJ_CELL), 4, rows, None, None));
    let nchars = u67_assetgen::characters::names().len() as u32;
    let chars_layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(CHAR_CELL), CHAR_COLS, nchars * 4, None, None));
    let nitems = u67_world::items::ITEMS.len() as u32;
    let items_layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(16), 8, nitems.div_ceil(8), None, None));
    let img = image::open(paths.assets.join("gfx/terrain.png")).expect("assets/gfx/terrain.png missing - run: cargo run -p u67_assetgen").to_rgba8();
    let terrain_path = paths.assets.join("gfx/terrain.png");
    let terrain_mtime = std::fs::metadata(&terrain_path).and_then(|m| m.modified()).ok();
    commands.insert_resource(Sheets {
        terrain_path,
        terrain_mtime,
        objects_img: assets.load("gfx/objects.png"),
        objects_layout,
        chars_img: assets.load("gfx/characters.png"),
        chars_layout,
        items_img: assets.load("gfx/items.png"),
        items_layout,
        terrain: TerrainPx { w: img.width(), data: img.into_raw() },
    });
}

/// RGBA pixels (256x256) for one 16x16-tile chunk.
pub fn chunk_rgba(map: &Map, cx: i32, cy: i32, sheet: &TerrainPx) -> Vec<u8> {
    let side = (CHUNK as u32 * 16) as usize;
    let mut out = vec![0u8; side * side * 4];
    for ty in 0..CHUNK {
        for tx in 0..CHUNK {
            let (x, y) = (cx * CHUNK + tx, cy * CHUNK + ty);
            let id = map.tile(u67_core::TilePos::new(x, y)).0 as u32;
            let var = (noise::hash(x, y, 9) * 4.0) as u32 % 4;
            for row in 0..16u32 {
                let src = (((id * 16 + row) * sheet.w + var * 16) * 4) as usize;
                let dst = ((ty as usize * 16 + row as usize) * side + tx as usize * 16) * 4;
                if src + 64 <= sheet.data.len() {
                    out[dst..dst + 64].copy_from_slice(&sheet.data[src..src + 64]);
                }
            }
        }
    }
    out
}

fn spawn_object(commands: &mut Commands, sheets: &Sheets, map: &Map, idx: usize, o: &WorldObject) -> Option<Entity> {
    let row = objects::OBJECTS.iter().position(|d| d.id == o.kind)? as u32;
    let def = &objects::OBJECTS[row as usize];
    let mut row = row;
    if o.kind == "pine_tree" && matches!(map.tile(o.pos), tiles::SNOW | tiles::ICE | tiles::AURORA_ICE) {
        row = objects::OBJECTS.len() as u32;
    }
    let frame = (o.frame as u32).min(def.frames as u32 - 1);
    let atlas = TextureAtlas { layout: sheets.objects_layout.clone(), index: (row * 4 + frame) as usize };
    let (x, y) = (o.pos.x as f32, o.pos.y as f32);
    let (anchor, pos, z) = if def.roof { (Anchor::TopLeft, tile_px(x, y), 5.0 + y * 0.001) } else { (Anchor::BottomCenter, tile_px(x + 0.5, y + 1.0), 1.0 + y * 0.001) };
    let mut e = commands.spawn((
        Sprite { image: sheets.objects_img.clone(), texture_atlas: Some(atlas), anchor, ..default() },
        Transform::from_xyz(pos.x, pos.y, z),
        ObjRef(idx),
    ));
    if def.roof {
        e.insert(Roof(o.group));
    }
    if def.frames > 1 && matches!(o.kind.as_str(), "campfire" | "bifrost_node" | "forge") {
        e.insert(Flicker { row, frames: if o.kind == "forge" { 2 } else { def.frames as u32 } });
    }
    Some(e.id())
}

fn game_active(game: Option<Res<Game>>, state: Res<State<AppState>>) -> bool {
    game.is_some() && !matches!(state.get(), AppState::Boot | AppState::MainMenu)
}

#[allow(clippy::too_many_arguments)]
fn stream_chunks(
    mut commands: Commands,
    game: Res<Game>,
    world: Res<WorldRes>,
    sheets: Res<Sheets>,
    mut loaded: ResMut<Loaded>,
    mut index: ResMut<ChunkIndex>,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    settings: Res<SettingsRes>,
    zoom: Res<crate::player::Zoom>,
    mut dirty: ResMut<DirtyChunks>,
) {
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    if loaded.map != game.0.current_map {
        for (_, es) in loaded.chunks.drain() {
            for e in es {
                commands.entity(e).despawn();
            }
        }
        loaded.map = game.0.current_map.clone();
        index.map = loaded.map.clone();
        index.by_chunk.clear();
        for (i, o) in map.objects.iter().enumerate() {
            index.by_chunk.entry(o.pos.chunk()).or_default().push(i);
        }
    }
    let _ = &settings;
    if !dirty.0.is_empty() {
        for k in dirty.0.drain().collect::<Vec<_>>() {
            if let Some(es) = loaded.chunks.remove(&k) {
                for e in es {
                    commands.entity(e).despawn();
                }
            }
            let list: Vec<usize> = map.objects.iter().enumerate().filter(|(_, o)| o.pos.chunk() == k).map(|(i, _)| i).collect();
            index.by_chunk.insert(k, list);
        }
    }
    let win = windows.single().map(|w| w.width().max(w.height())).unwrap_or(1280.0);
    let half_tiles = win / 2.0 / (zoom.0 * T);
    let r = (half_tiles / CHUNK as f32).ceil() as i32 + 1;
    let (mw, mh) = map.chunks();
    let mut want: HashSet<(i32, i32)> = HashSet::new();
    for p in &game.0.players {
        let (pcx, pcy) = u67_core::TilePos::new(p.pos[0] as i32, p.pos[1] as i32).chunk();
        for cy in (pcy - r).max(0)..=(pcy + r).min(mh - 1) {
            for cx in (pcx - r).max(0)..=(pcx + r).min(mw - 1) {
                want.insert((cx, cy));
            }
        }
    }
    let stale: Vec<_> = loaded.chunks.keys().filter(|k| !want.contains(k)).copied().collect();
    for k in stale {
        if let Some(es) = loaded.chunks.remove(&k) {
            for e in es {
                commands.entity(e).despawn();
            }
        }
    }
    let side = CHUNK as u32 * 16;
    let mut budget = 6; // chunks built per frame, to avoid hitches
    for &(cx, cy) in &want {
        if loaded.chunks.contains_key(&(cx, cy)) || budget == 0 {
            continue;
        }
        budget -= 1;
        let img = Image::new(
            Extent3d { width: side, height: side, depth_or_array_layers: 1 },
            TextureDimension::D2,
            chunk_rgba(map, cx, cy, &sheets.terrain),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let p = tile_px((cx * CHUNK) as f32, (cy * CHUNK) as f32);
        let mut ents = vec![commands.spawn((Sprite { image: images.add(img), anchor: Anchor::TopLeft, ..default() }, Transform::from_xyz(p.x, p.y, 0.0))).id()];
        if let Some(list) = index.by_chunk.get(&(cx, cy)) {
            for &i in list {
                if let Some(e) = spawn_object(&mut commands, &sheets, map, i, &map.objects[i]) {
                    ents.push(e);
                }
            }
        }
        loaded.chunks.insert((cx, cy), ents);
    }
}

/// Dev hot-reload for the terrain sheet (it is baked into chunk textures, so Bevy's own watcher can't help).
fn watch_terrain(time: Res<Time>, mut acc: Local<f32>, mut sheets: ResMut<Sheets>, settings: Res<SettingsRes>, loaded: Res<Loaded>, mut dirty: ResMut<DirtyChunks>) {
    *acc += time.delta_secs();
    if *acc < 1.5 || !settings.0.hot_reload {
        return;
    }
    *acc = 0.0;
    let m = std::fs::metadata(&sheets.terrain_path).and_then(|m| m.modified()).ok();
    if m.is_some() && m != sheets.terrain_mtime {
        if let Ok(img) = image::open(&sheets.terrain_path) {
            let img = img.to_rgba8();
            sheets.terrain = TerrainPx { w: img.width(), data: img.into_raw() };
            sheets.terrain_mtime = m;
            dirty.0.extend(loaded.chunks.keys().copied());
            info!("terrain.png reloaded");
        }
    }
}

fn hide_roofs(game: Res<Game>, world: Res<WorldRes>, mut roofs: Query<(&Roof, &mut Visibility)>) {
    let Some(map) = world.0.maps.get(&game.0.current_map) else { return };
    let inside: HashSet<u16> = game.0.players.iter().filter_map(|p| map.building_at(u67_core::TilePos::new(p.pos[0] as i32, p.pos[1] as i32)).map(|b| b.id)).collect();
    for (r, mut v) in &mut roofs {
        let want = if inside.contains(&r.0) { Visibility::Hidden } else { Visibility::Inherited };
        if *v != want {
            *v = want;
        }
    }
}

fn animate_objects(time: Res<Time>, mut q: Query<(&Flicker, &mut Sprite)>) {
    let t = (time.elapsed_secs() * 6.0) as u32;
    for (f, mut s) in &mut q {
        if let Some(a) = &mut s.texture_atlas {
            a.index = (f.row * 4 + t % f.frames) as usize;
        }
    }
}

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DirtyChunks>()
            .init_resource::<Loaded>()
            .init_resource::<ChunkIndex>()
            .add_systems(Startup, load_sheets)
            .add_systems(Update, (stream_chunks, hide_roofs, animate_objects, watch_terrain).run_if(game_active));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunk_pixels_come_from_sheet() {
        // fake sheet: 64px wide, tile row id filled with a distinct colour
        let ntiles = tiles::TILES.len() as u32;
        let mut data = vec![0u8; (64 * 16 * ntiles * 4) as usize];
        for id in 0..ntiles {
            for y in 0..16 {
                for x in 0..64 {
                    let i = (((id * 16 + y) * 64 + x) * 4) as usize;
                    data[i..i + 4].copy_from_slice(&[id as u8, 1, 2, 255]);
                }
            }
        }
        let sheet = TerrainPx { w: 64, data };
        let mut m = Map::new("t", 16, 16, tiles::GRASS);
        m.set_tile(u67_core::TilePos::new(2, 3), tiles::LAVA);
        let px = chunk_rgba(&m, 0, 0, &sheet);
        let at = |x: usize, y: usize| px[(y * 256 + x) * 4];
        assert_eq!(at(0, 0), tiles::GRASS.0 as u8);
        assert_eq!(at(2 * 16 + 5, 3 * 16 + 7), tiles::LAVA.0 as u8);
        // out-of-map tiles render as VOID
        let px = chunk_rgba(&m, 1, 0, &sheet);
        assert_eq!(px[0], tiles::VOID.0 as u8);
    }
}
