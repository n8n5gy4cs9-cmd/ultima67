//! Game screens: U7-style inventory (paperdoll + gumps + drag & drop), keyword dialogue,
//! list menus (shop, craft, travel, journal, spells), laulu rune singing, map, death screen.
use crate::app::{AppState, Game, Paths, WorldRes};
use crate::audio::SfxEvent;
use crate::data::*;
use crate::db_res::DbRes;
use crate::interact::UiRequest;
use crate::invops::{self, SlotRef, UseResult};
use crate::magic::{CastRequest, SingRequest};
use crate::menus::{self, Row};
use crate::npc::Npcs;
use crate::render::{self, Sheets, CHAR_COLS};
use crate::script;
use crate::ui::{EffectEvent, Toast};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use u67_world::dialogue::Session;
use u67_world::inventory::Item;
use u67_world::items::{self, Slot};

const S: f32 = 3.0; // UI pixel scale for icons

// ------------------------------------------------------------------ atlases
#[derive(Resource)]
pub struct UiAtlas {
    pub img: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub idx: std::collections::HashMap<String, usize>,
}

fn load_ui_atlas(mut commands: Commands, assets: Res<AssetServer>, paths: Res<Paths>, mut layouts: ResMut<Assets<TextureAtlasLayout>>) {
    let path = paths.assets.join("gfx/ui.json");
    let atlas: u67_assetgen::manifest::Atlas = serde_json::from_str(&std::fs::read_to_string(&path).expect("assets/gfx/ui.json missing")).expect("ui.json");
    let (w, h) = image::image_dimensions(paths.assets.join("gfx/ui.png")).unwrap_or((256, 320));
    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(w, h));
    let mut idx = std::collections::HashMap::new();
    for e in &atlas.entries {
        let i = layout.add_texture(URect::new(e.x as u32, e.y as u32, (e.x + e.w) as u32, (e.y + e.h) as u32));
        idx.insert(e.name.clone(), i);
    }
    commands.insert_resource(UiAtlas { img: assets.load("gfx/ui.png"), layout: layouts.add(layout), idx });
}

impl UiAtlas {
    pub fn node(&self, name: &str) -> ImageNode {
        ImageNode::from_atlas_image(self.img.clone(), TextureAtlas { layout: self.layout.clone(), index: *self.idx.get(name).unwrap_or(&0) })
    }
}

fn item_icon(sheets: &Sheets, id: &str) -> ImageNode {
    let i = items::ITEMS.iter().position(|d| d.id == id).unwrap_or(0);
    ImageNode::from_atlas_image(sheets.items_img.clone(), TextureAtlas { layout: sheets.items_layout.clone(), index: i })
}

fn despawn_all<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn panel_node(left: f32, top: f32, w: f32, h: f32) -> Node {
    Node { position_type: PositionType::Absolute, left: Val::Px(left), top: Val::Px(top), width: Val::Px(w), height: Val::Px(h), ..default() }
}

// ------------------------------------------------------------------ UI requests -> state
#[derive(Resource, Default, Clone, PartialEq)]
pub enum MenuKind {
    #[default]
    None,
    Shop { shop: String, selling: bool },
    Craft { station: String },
    Travel { by_ship: bool },
    Journal,
    Spells,
    Laulu,
    Map,
    Cheats,
}

#[derive(Resource, Default)]
pub struct MenuState {
    pub rows: Vec<Row>,
    pub scroll: usize,
    pub msg: String,
    pub dirty: bool,
    pub seq: Vec<u8>,
    pub map_img: Option<Handle<Image>>,
}

#[derive(Resource, Default)]
pub struct InvUi {
    pub open: Vec<Vec<usize>>,
    pub object: Option<usize>,
    pub drag: Option<(Item, SlotRef)>,
    pub dirty: bool,
    pub hover_text: String,
}

#[derive(Resource, Default)]
pub struct DialogueRt {
    pub npc_id: String,
    pub npc_name: String,
    pub arch: String,
    pub def_id: String,
    pub session: Option<Session>,
    pub log: Vec<String>,
    pub input: String,
    pub dirty: bool,
}

#[allow(clippy::too_many_arguments)]
fn handle_requests(
    mut ev: EventReader<UiRequest>,
    mut next: ResMut<NextState<AppState>>,
    mut kind: ResMut<MenuKind>,
    mut menu: ResMut<MenuState>,
    mut inv: ResMut<InvUi>,
    mut dlg: ResMut<DialogueRt>,
    mut game: ResMut<Game>,
    world: Res<WorldRes>,
    db: Res<DbRes>,
    npcs: Res<Npcs>,
    mut out_fx: EventWriter<EffectEvent>,
    mut toast: ResMut<Toast>,
    mut images: ResMut<Assets<Image>>,
) {
    let reqs: Vec<UiRequest> = ev.read().cloned().collect();
    for r in reqs.into_iter().take(1) {
        match r {
            UiRequest::Dialogue(i) => {
                let Some(n) = npcs.list.get(i) else { continue };
                let Some(def) = db.0.dialogues.get(&n.dialogue) else {
                    *toast = Toast { text: format!("{} has nothing to say.", n.name), timer: 2.0 };
                    continue;
                };
                let mut s = Session::new();
                let r = s.greet(def, &script::Ctx { d: &game.0 });
                let out = script::run(&mut game.0, &db.0, &world.0, &r.steps);
                dlg.log = vec![fill(&r.text, &game.0, &n.town)];
                dlg.log.extend(out.lines);
                for t in out.toasts {
                    *toast = Toast { text: t, timer: 3.0 };
                }
                for f in out.effects {
                    out_fx.write(EffectEvent(f));
                }
                dlg.npc_id = n.id.clone();
                dlg.npc_name = n.name.clone();
                dlg.arch = n.arch.clone();
                dlg.def_id = n.dialogue.clone();
                dlg.session = Some(s);
                dlg.input.clear();
                dlg.dirty = true;
                next.set(AppState::Dialogue);
            }
            UiRequest::Container(idx) => {
                inv.object = Some(idx);
                inv.dirty = true;
                next.set(AppState::Inventory);
            }
            UiRequest::Shop(shop) => {
                *kind = MenuKind::Shop { shop: shop.clone(), selling: false };
                menu.rows = menus::shop_rows(&game.0, &db.0, &shop, false);
                menu.scroll = 0;
                menu.msg = "Tab: switch buy/sell. Click an entry.".into();
                menu.dirty = true;
                next.set(AppState::Menu);
            }
            UiRequest::Craft(station) => {
                menu.rows = menus::craft_rows(&game.0, &db.0, &station);
                *kind = MenuKind::Craft { station };
                menu.scroll = 0;
                menu.msg = "Click a recipe to craft.".into();
                menu.dirty = true;
                next.set(AppState::Menu);
            }
            UiRequest::Travel { by_ship } => {
                menu.rows = menus::travel_rows(&game.0, by_ship);
                *kind = MenuKind::Travel { by_ship };
                menu.scroll = 0;
                menu.msg = String::new();
                menu.dirty = true;
                next.set(AppState::Menu);
            }
            UiRequest::Journal => {
                menu.rows = menus::journal_rows(&game.0, &db.0);
                *kind = MenuKind::Journal;
                menu.scroll = 0;
                menu.msg = "Esc to close.".into();
                menu.dirty = true;
                next.set(AppState::Menu);
            }
            UiRequest::Spells => {
                menu.rows = menus::spell_rows(&game.0, &db.0);
                *kind = MenuKind::Spells;
                menu.scroll = 0;
                menu.msg = "1-3 are your spell hotkeys (click a spell to cast now).".into();
                menu.dirty = true;
                next.set(AppState::Menu);
            }
            UiRequest::Laulu => {
                *kind = MenuKind::Laulu;
                menu.seq.clear();
                menu.msg = "Choose three runes in order, then the verse is sung.".into();
                menu.dirty = true;
                next.set(AppState::Menu);
            }
            UiRequest::Cheats => {
                menu.rows = menus::cheat_rows(&game.0);
                *kind = MenuKind::Cheats;
                menu.scroll = 0;
                menu.msg = "Click to toggle / run. Esc closes.".into();
                menu.dirty = true;
                next.set(AppState::Menu);
            }
            UiRequest::Map => {
                if let Some(m) = world.0.maps.get(&game.0.current_map) {
                    menu.map_img = Some(images.add(map_image(m, &game.0)));
                }
                *kind = MenuKind::Map;
                menu.dirty = true;
                next.set(AppState::Menu);
            }
        }
    }
}

fn fill(text: &str, game: &GameData, town: &str) -> String {
    let town_name = u67_mapgen::midgard::TOWNS.iter().find(|t| t.0 == town).map_or(town, |t| t.1);
    text.replace("{name}", &game.players[0].name).replace("{town}", town_name)
}

/// Overview map: tiles of explored chunks (or everything with `reveal_map`), unexplored = black.
pub fn map_image(m: &u67_world::map::Map, d: &GameData) -> Image {
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let (w, h) = (m.width as u32, m.height as u32);
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let t = u67_core::TilePos::new(x, y);
            let (cx, cy) = t.chunk();
            let seen = d.map_revealed || d.explored.contains(&format!("{}:{}:{}", d.current_map, cx, cy));
            let i = ((y as u32 * w + x as u32) * 4) as usize;
            if seen {
                let c = u67_world::tiles::def(m.tile(t)).color;
                data[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
            } else {
                data[i..i + 4].copy_from_slice(&[10, 8, 14, 255]);
            }
        }
    }
    for o in &m.objects {
        if matches!(o.kind.as_str(), "longhouse_roof" | "bifrost_node" | "runestone") {
            let (cx, cy) = o.pos.chunk();
            if d.map_revealed || d.explored.contains(&format!("{}:{}:{}", d.current_map, cx, cy)) {
                let i = ((o.pos.y as u32 * w + o.pos.x as u32) * 4) as usize;
                let c: [u8; 4] = if o.kind == "longhouse_roof" { [120, 70, 40, 255] } else { [120, 240, 255, 255] };
                data[i..i + 4].copy_from_slice(&c);
            }
        }
    }
    Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default())
}

// ------------------------------------------------------------------ dialogue screen
#[derive(Component)]
struct DlgRoot;
#[derive(Component)]
struct KeywordBtn(String);

fn build_dialogue(mut commands: Commands, mut dlg: ResMut<DialogueRt>, game: Res<Game>, db: Res<DbRes>, sheets: Res<Sheets>, roots: Query<Entity, With<DlgRoot>>) {
    if !dlg.dirty {
        return;
    }
    dlg.dirty = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let Some(def) = db.0.dialogues.get(&dlg.def_id) else { return };
    let kws = dlg.session.as_ref().map(|s| s.visible(def, &script::Ctx { d: &game.0 })).unwrap_or_default();
    let row = render::character_row(&dlg.arch).unwrap_or(0);
    let start = dlg.log.len().saturating_sub(7);
    let text = dlg.log[start..].join("\n\n");
    let input = dlg.input.clone();
    let name = dlg.npc_name.clone();
    commands
        .spawn((DlgRoot, Node { position_type: PositionType::Absolute, left: Val::Px(0.0), bottom: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Px(300.0), padding: UiRect::all(Val::Px(14.0)), column_gap: Val::Px(16.0), ..default() }, BackgroundColor(Color::srgba(0.07, 0.05, 0.04, 0.95))))
        .with_children(|p| {
            p.spawn((
                Node { width: Val::Px(144.0), height: Val::Px(144.0), border: UiRect::all(Val::Px(3.0)), ..default() },
                BorderColor(Color::srgb(0.88, 0.7, 0.3)),
                ImageNode::from_atlas_image(sheets.chars_img.clone(), TextureAtlas { layout: sheets.chars_layout.clone(), index: ((row * 4 + 2) * CHAR_COLS) as usize }),
            ));
            p.spawn(Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, row_gap: Val::Px(8.0), ..default() }).with_children(|c| {
                c.spawn((Text::new(name), TextFont { font_size: 24.0, ..default() }, TextColor(Color::srgb(0.95, 0.78, 0.35))));
                c.spawn((Text::new(text), TextFont { font_size: 19.0, ..default() }));
                c.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(8.0), row_gap: Val::Px(6.0), ..default() }).with_children(|k| {
                    for kw in kws {
                        k.spawn((Button, KeywordBtn(kw.clone()), Node { padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)), ..default() }, BackgroundColor(Color::srgb(0.3, 0.22, 0.12)))).with_children(|b| {
                            b.spawn((Text::new(kw), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(1.0, 0.92, 0.7))));
                        });
                    }
                });
                c.spawn((Text::new(format!("> {input}_   (click a word or type + Enter; Esc to leave)")), TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(0.6, 0.85, 0.7))));
            });
        });
}

fn dialogue_say(dlg: &mut DialogueRt, game: &mut Game, world: &WorldRes, db: &DbRes, word: &str, fx: &mut EventWriter<EffectEvent>, toast: &mut Toast) -> bool {
    let Some(def) = db.0.dialogues.get(&dlg.def_id) else { return true };
    let Some(mut s) = dlg.session.take() else { return true };
    let town = db.0.npcs.iter().find(|n| n.id == dlg.npc_id).map(|n| n.town.clone()).unwrap_or_default();
    let r = s.say(def, word, &script::Ctx { d: &game.0 });
    dlg.log.push(format!("> {word}"));
    dlg.log.push(fill(&r.text, &game.0, &town));
    let out = script::run(&mut game.0, &db.0, &world.0, &r.steps);
    dlg.log.extend(out.lines);
    for t in out.toasts {
        *toast = Toast { text: t, timer: 3.5 };
    }
    for f in out.effects {
        fx.write(EffectEvent(f));
    }
    dlg.session = Some(s);
    dlg.dirty = true;
    r.end
}

#[allow(clippy::too_many_arguments)]
fn dialogue_input(
    mut kev: EventReader<KeyboardInput>,
    q: Query<(&Interaction, &KeywordBtn), Changed<Interaction>>,
    mut dlg: ResMut<DialogueRt>,
    mut game: ResMut<Game>,
    world: Res<WorldRes>,
    db: Res<DbRes>,
    mut fx: EventWriter<EffectEvent>,
    mut toast: ResMut<Toast>,
    mut next: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    let mut word: Option<String> = None;
    for (i, k) in &q {
        if *i == Interaction::Pressed {
            word = Some(k.0.clone());
        }
    }
    for e in kev.read() {
        if !e.state.is_pressed() {
            continue;
        }
        match &e.logical_key {
            Key::Character(s) => {
                dlg.input.push_str(s);
                dlg.dirty = true;
            }
            Key::Space => {
                dlg.input.push(' ');
                dlg.dirty = true;
            }
            Key::Backspace => {
                dlg.input.pop();
                dlg.dirty = true;
            }
            Key::Enter => {
                if !dlg.input.trim().is_empty() {
                    word = Some(std::mem::take(&mut dlg.input));
                }
            }
            _ => {}
        }
    }
    if let Some(w) = word {
        if dialogue_say(&mut dlg, &mut game, &world, &db, &w, &mut fx, &mut toast) {
            next.set(AppState::Playing);
        } else if *state.get() == AppState::Dialogue {
            dlg.input.clear();
        }
    }
}

// ------------------------------------------------------------------ inventory screen
#[derive(Component)]
struct InvRoot;
#[derive(Component)]
struct SlotUi(SlotRef);
#[derive(Component)]
struct Ghost;
#[derive(Component)]
struct InfoText;

pub fn describe_item(it: &Item) -> String {
    let Some(d) = items::get(&it.id) else { return it.id.clone() };
    let mut s = format!("{}{}   wt {:.1}", d.name, if it.qty > 1 { format!(" x{}", it.qty) } else { String::new() }, d.weight * it.qty as f32);
    if d.damage > 0 {
        s += &format!("   dmg {}", d.damage);
    }
    if d.armor > 0 {
        s += &format!("   armor {}", d.armor);
    }
    if d.capacity > 0.0 {
        s += &format!("   holds {:.0} ({:.1} used)", d.capacity, it.contents_weight());
    }
    s += &format!("   value {}", u67_world::combat::item_value(&it.id));
    s
}

fn slot_to_equip(name: &str) -> Slot {
    match name {
        "head" => Slot::Head,
        "neck" => Slot::Neck,
        "torso" => Slot::Torso,
        "back" => Slot::Back,
        "hand_l" => Slot::HandL,
        "hand_r" => Slot::HandR,
        "ring_l" => Slot::RingL,
        "ring_r" => Slot::RingR,
        "legs" => Slot::Legs,
        "feet" => Slot::Feet,
        _ => Slot::Ammo,
    }
}

fn spawn_slot(p: &mut ChildSpawnerCommands, ui: &UiAtlas, sheets: &Sheets, left: f32, top: f32, size: f32, r: SlotRef, item: Option<&Item>) {
    p.spawn((Button, SlotUi(r), Node { position_type: PositionType::Absolute, left: Val::Px(left), top: Val::Px(top), width: Val::Px(size), height: Val::Px(size), ..default() }, ui.node("slot"))).with_children(|s| {
        if let Some(it) = item {
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(3.0), top: Val::Px(3.0), width: Val::Px(size - 6.0), height: Val::Px(size - 6.0), ..default() }, item_icon(sheets, &it.id)));
            if it.qty > 1 {
                s.spawn((Text::new(it.qty.to_string()), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), Node { position_type: PositionType::Absolute, right: Val::Px(2.0), bottom: Val::Px(0.0), ..default() }));
            }
        }
    });
}

fn grid_panel(p: &mut ChildSpawnerCommands, ui: &UiAtlas, sheets: &Sheets, left: f32, top: f32, title: &str, items_in: &[Item], mk: &dyn Fn(usize) -> SlotRef) -> f32 {
    let cols = 6usize;
    let slot = 18.0 * S;
    let n = items_in.len() + 1;
    let rows = n.div_ceil(cols).max(3);
    let w = cols as f32 * (slot + 2.0) + 20.0;
    let h = rows as f32 * (slot + 2.0) + 46.0;
    p.spawn((panel_node(left, top, w, h), BackgroundColor(Color::srgb(0.24, 0.17, 0.11)), BorderColor(Color::srgb(0.88, 0.7, 0.3)))).with_children(|g| {
        g.spawn((Text::new(title.to_string()), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(1.0, 0.92, 0.7)), Node { position_type: PositionType::Absolute, left: Val::Px(10.0), top: Val::Px(8.0), ..default() }));
        for i in 0..rows * cols {
            let (c, r) = (i % cols, i / cols);
            let item = items_in.get(i);
            // slot index: existing item, or the first empty slot (append); further empties are inert but harmless
            let sref = mk(i.min(items_in.len()));
            spawn_slot(g, ui, sheets, 10.0 + c as f32 * (slot + 2.0), 34.0 + r as f32 * (slot + 2.0), slot, sref, item);
        }
    });
    w
}

#[allow(clippy::too_many_arguments)]
fn build_inventory(mut commands: Commands, mut inv: ResMut<InvUi>, game: Res<Game>, world: Res<WorldRes>, ui: Res<UiAtlas>, sheets: Res<Sheets>, roots: Query<Entity, With<InvRoot>>) {
    if !inv.dirty {
        return;
    }
    inv.dirty = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let p0 = &game.0.players[0];
    let open = inv.open.clone();
    let object = inv.object;
    let map = world.0.maps.get(&game.0.current_map);
    commands.spawn((InvRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.82)))).with_children(|root| {
        root.spawn((Text::new("INVENTORY    drag items (left click)  |  right click: use / open / equip  |  shift-click: quick move  |  click outside: drop  |  I / Esc: close"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(0.8, 0.8, 0.7)), Node { position_type: PositionType::Absolute, left: Val::Px(20.0), top: Val::Px(10.0), ..default() }));
        // paperdoll
        let (pl, pt) = (24.0, 50.0);
        root.spawn((panel_node(pl, pt, 80.0 * S, 96.0 * S), ui.node("paperdoll"))).with_children(|doll| {
            for (name, x, y) in u67_assetgen::ui::PAPERDOLL_SLOTS {
                let slot = slot_to_equip(name);
                spawn_slot(doll, &ui, &sheets, *x as f32 * S, *y as f32 * S, 18.0 * S, SlotRef::Paper(slot), p0.inventory.equipped.get(&slot));
            }
        });
        // stats
        let st = &p0.stats;
        root.spawn((
            Text::new(format!(
                "{}\nLevel {}   XP {}\nHP {}/{}   Mana {}/{}\nSTR {}  DEX {}  INT {}  VAKI {}\nWeight {:.1}/{:.1}\nSilver {}",
                p0.name, st.level, st.xp, st.hp, st.max_hp(), st.mana, st.max_mana(), st.str_, st.dex, st.int, st.vaki, p0.inventory.total_weight(), p0.inventory.max_weight, p0.inventory.count("silver")
            )),
            TextFont { font_size: 17.0, ..default() },
            Node { position_type: PositionType::Absolute, left: Val::Px(24.0), top: Val::Px(50.0 + 96.0 * S + 12.0), ..default() },
        ));
        // pack root + opened containers
        let mut x = 24.0 + 80.0 * S + 24.0;
        let w = grid_panel(root, &ui, &sheets, x, 50.0, "Carried", &p0.inventory.pack, &|i| SlotRef::Pack { container: vec![], index: i });
        x += w + 16.0;
        let mut top = 50.0;
        for path in &open {
            if let Some(it) = p0.inventory.get(path) {
                if items::get(&it.id).is_some_and(|d| d.capacity > 0.0) {
                    let name = items::get(&it.id).map_or("Container", |d| d.name);
                    let pth = path.clone();
                    let w = grid_panel(root, &ui, &sheets, x, top, name, &it.contents, &move |i| SlotRef::Pack { container: pth.clone(), index: i });
                    top += 40.0 + (it.contents.len() + 1).div_ceil(6).max(3) as f32 * (18.0 * S + 2.0) + 12.0;
                    if top > 500.0 {
                        top = 50.0;
                        x += w + 16.0;
                    }
                }
            }
        }
        if let (Some(oi), Some(m)) = (object, map) {
            if let Some(o) = m.objects.get(oi) {
                let contents = o.contents.clone().unwrap_or_default();
                grid_panel(root, &ui, &sheets, x.max(24.0 + 80.0 * S + 24.0 + 6.0 * 56.0 + 60.0), 50.0, &format!("{} (in the world)", o.kind), &contents, &move |i| SlotRef::Obj { idx: oi, index: i });
            }
        }
        root.spawn((InfoText, Text::new(""), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(0.7, 1.0, 0.8)), Node { position_type: PositionType::Absolute, left: Val::Px(24.0), bottom: Val::Px(14.0), ..default() }));
    });
}

fn slot_item<'a>(game: &'a GameData, world: &'a u67_world::map::World, s: &SlotRef) -> Option<&'a Item> {
    match s {
        SlotRef::Paper(sl) => game.players[0].inventory.equipped.get(sl),
        SlotRef::Pack { container, index } => {
            let mut p = container.clone();
            p.push(*index);
            game.players[0].inventory.get(&p)
        }
        SlotRef::Obj { idx, index } => world.maps.get(&game.current_map)?.objects.get(*idx)?.contents.as_ref()?.get(*index),
    }
}

#[allow(clippy::too_many_arguments)]
fn inventory_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    kb: Res<ButtonInput<KeyCode>>,
    slots: Query<(&SlotUi, &Interaction)>,
    mut inv: ResMut<InvUi>,
    mut game: ResMut<Game>,
    mut world: ResMut<WorldRes>,
    mut toast: ResMut<Toast>,
    mut sfx: EventWriter<SfxEvent>,
    mut info: Query<&mut Text, With<InfoText>>,
    mut ghost: Query<(&mut Node, &mut Visibility), With<Ghost>>,
    windows: Query<&Window>,
) {
    let hovered: Option<SlotRef> = slots.iter().find(|(_, i)| **i != Interaction::None).map(|(s, _)| s.0.clone());
    // info line
    if let Ok(mut t) = info.single_mut() {
        t.0 = match (&inv.drag, &hovered) {
            (Some((it, _)), _) => format!("Holding: {}", describe_item(it)),
            (None, Some(h)) => slot_item(&game.0, &world.0, h).map_or(String::new(), describe_item),
            _ => String::new(),
        };
    }
    // ghost follows the cursor
    if let (Ok((mut n, mut v)), Ok(w)) = (ghost.single_mut(), windows.single()) {
        if let Some(c) = w.cursor_position() {
            n.left = Val::Px(c.x - 24.0);
            n.top = Val::Px(c.y - 24.0);
        }
        *v = if inv.drag.is_some() { Visibility::Inherited } else { Visibility::Hidden };
    }
    let shift = kb.pressed(KeyCode::ShiftLeft) || kb.pressed(KeyCode::ShiftRight);
    let touched_obj = |s: &SlotRef| if let SlotRef::Obj { idx, .. } = s { Some(*idx) } else { None };
    // right click
    if mouse.just_pressed(MouseButton::Right) {
        if let Some((it, from)) = inv.drag.take() {
            let _ = invops::put(&mut game.0, &mut world.0, &from, it.clone()).or_else(|(_, back)| {
                game.0.players[0].inventory.pack.push(back);
                Ok::<_, ()>(None)
            });
            inv.dirty = true;
            return;
        }
        if let Some(h) = &hovered {
            match h {
                SlotRef::Pack { container, index } => {
                    let mut p = container.clone();
                    p.push(*index);
                    match invops::use_item(&mut game.0, &p) {
                        UseResult::Msg(m) => *toast = Toast { text: m, timer: 2.5 },
                        UseResult::Equipped(n) => *toast = Toast { text: format!("Equipped {n}"), timer: 1.5 },
                        UseResult::OpenContainer(path) => {
                            if !inv.open.contains(&path) {
                                inv.open.push(path);
                            }
                        }
                        UseResult::Nothing => {}
                    }
                    inv.dirty = true;
                }
                SlotRef::Paper(sl) => {
                    // unequip to pack
                    let _ = game.0.players[0].inventory.unequip(*sl);
                    inv.dirty = true;
                }
                SlotRef::Obj { idx, index } => {
                    let r = invops::move_item(&mut game.0, &mut world.0, h, &SlotRef::Pack { container: vec![], index: 0 });
                    let _ = (idx, index);
                    if r.is_err() {
                        *toast = Toast { text: "You cannot carry that.".into(), timer: 1.5 };
                    }
                    invops::persist_obj(&mut game.0, &world.0, touched_obj(h).unwrap());
                    inv.dirty = true;
                }
            }
        }
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    match (inv.drag.take(), hovered) {
        // pick up
        (None, Some(h)) => {
            if shift {
                // quick move
                let to = match &h {
                    SlotRef::Obj { .. } => SlotRef::Pack { container: vec![], index: 0 },
                    SlotRef::Pack { .. } => match inv.object {
                        Some(oi) => SlotRef::Obj { idx: oi, index: 0 },
                        None => {
                            let mut p = vec![];
                            if let SlotRef::Pack { container, index } = &h {
                                p = container.clone();
                                p.push(*index);
                            }
                            let _ = invops::use_item(&mut game.0, &p);
                            inv.dirty = true;
                            return;
                        }
                    },
                    SlotRef::Paper(_) => SlotRef::Pack { container: vec![], index: 0 },
                };
                if let Err(e) = invops::move_item(&mut game.0, &mut world.0, &h, &to) {
                    *toast = Toast { text: format!("Cannot: {e:?}"), timer: 1.5 };
                }
                for s in [&h, &to] {
                    if let Some(i) = touched_obj(s) {
                        invops::persist_obj(&mut game.0, &world.0, i);
                    }
                }
                inv.dirty = true;
                return;
            }
            if let Some(it) = invops::take(&mut game.0, &mut world.0, &h) {
                sfx.write(SfxEvent("pickup".into()));
                inv.drag = Some((it, h));
                inv.dirty = true;
            }
        }
        // drop onto a slot
        (Some((it, from)), Some(to)) => {
            // dropping a container onto itself/its contents is refused by put()
            match invops::put(&mut game.0, &mut world.0, &to, it) {
                Ok(replaced) => {
                    sfx.write(SfxEvent("drop".into()));
                    if let Some(r) = replaced {
                        // swap: displaced item goes into the origin slot (or the pack)
                        if invops::put(&mut game.0, &mut world.0, &from, r.clone()).is_err() {
                            game.0.players[0].inventory.pack.push(r);
                        }
                    }
                }
                Err((e, back)) => {
                    *toast = Toast { text: match e {
                        u67_world::inventory::InvError::TooHeavy => "That will not fit / is too heavy.".into(),
                        u67_world::inventory::InvError::SlotMismatch => "That does not go there.".into(),
                        other => format!("Cannot: {other:?}"),
                    }, timer: 1.8 };
                    sfx.write(SfxEvent("ui_error".into()));
                    if invops::put(&mut game.0, &mut world.0, &from, back.clone()).is_err() {
                        game.0.players[0].inventory.pack.push(back);
                    }
                }
            }
            for s in [&from, &to] {
                if let Some(i) = touched_obj(s) {
                    invops::persist_obj(&mut game.0, &world.0, i);
                }
            }
            inv.dirty = true;
        }
        // click outside any slot while holding: drop on the ground
        (Some((it, from)), None) => {
            let p = game.0.players[0].pos;
            let name = items::get(&it.id).map_or(it.id.clone(), |d| d.name.to_string());
            let cur_map = game.0.current_map.clone();
            game.0.ground.push(GroundItem { map: cur_map, pos: [p[0] + 0.4, p[1] - 0.2], item: it });
            let _ = from;
            *toast = Toast { text: format!("Dropped {name}"), timer: 1.5 };
            sfx.write(SfxEvent("drop".into()));
            inv.dirty = true;
        }
        (None, None) => {}
    }
}

fn enter_inventory(mut inv: ResMut<InvUi>, mut commands: Commands, ui: Res<UiAtlas>, sheets: Res<Sheets>, mut game: ResMut<Game>) {
    inv.dirty = true;
    inv.drag = None;
    if inv.open.is_empty() {
        // open the first container item in the pack by default (U7 opens the backpack)
        if let Some(i) = game.0.players[0].inventory.pack.iter().position(|it| it.id == "backpack") {
            inv.open.push(vec![i]);
        }
    }
    // keep paths valid
    let valid: Vec<Vec<usize>> = inv.open.iter().filter(|p| game.0.players[0].inventory.get(p).is_some_and(|it| items::get(&it.id).is_some_and(|d| d.capacity > 0.0))).cloned().collect();
    inv.open = valid;
    let _ = &mut game;
    let id = game.0.players[0].inventory.pack.first().map(|i| i.id.clone()).unwrap_or_else(|| "rope".into());
    commands.spawn((Ghost, Node { position_type: PositionType::Absolute, width: Val::Px(48.0), height: Val::Px(48.0), ..default() }, item_icon(&sheets, &id), Visibility::Hidden, ZIndex(100)));
    let _ = &ui;
}

fn exit_inventory(mut inv: ResMut<InvUi>, mut game: ResMut<Game>, mut world: ResMut<WorldRes>) {
    // put a held item back
    if let Some((it, from)) = inv.drag.take() {
        if invops::put(&mut game.0, &mut world.0, &from, it.clone()).is_err() {
            game.0.players[0].inventory.pack.push(it);
        }
    }
    inv.object = None;
}

fn refresh_ghost_icon(inv: Res<InvUi>, sheets: Res<Sheets>, mut g: Query<&mut ImageNode, With<Ghost>>) {
    if let (Some((it, _)), Ok(mut n)) = (&inv.drag, g.single_mut()) {
        *n = item_icon(&sheets, &it.id);
    }
}

// ------------------------------------------------------------------ list menus
#[derive(Component)]
struct MenuRoot;
#[derive(Component)]
struct MenuRow(String);
#[derive(Component)]
struct RuneBtn(u8);

const VISIBLE_ROWS: usize = 15;

#[allow(clippy::too_many_arguments)]
fn build_menu(mut commands: Commands, mut menu: ResMut<MenuState>, kind: Res<MenuKind>, ui: Res<UiAtlas>, roots: Query<Entity, With<MenuRoot>>, game: Res<Game>) {
    if !menu.dirty {
        return;
    }
    menu.dirty = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let kind = kind.clone();
    let rows: Vec<Row> = menu.rows.iter().skip(menu.scroll).take(VISIBLE_ROWS).cloned().collect();
    let msg = menu.msg.clone();
    let seq = menu.seq.clone();
    let map_img = menu.map_img.clone();
    let more = menu.rows.len() > menu.scroll + VISIBLE_ROWS;
    commands
        .spawn((MenuRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.85))))
        .with_children(|root| {
            root.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(18.0)), row_gap: Val::Px(4.0), min_width: Val::Px(760.0), ..default() }, BackgroundColor(Color::srgb(0.17, 0.12, 0.08)), BorderColor(Color::srgb(0.88, 0.7, 0.3)))).with_children(|panel| {
                match &kind {
                    MenuKind::Laulu => {
                        panel.spawn((Text::new("SING A LAULU - choose 3 runes"), TextFont { font_size: 26.0, ..default() }, TextColor(Color::srgb(0.95, 0.78, 0.35))));
                        panel.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(8.0), row_gap: Val::Px(8.0), max_width: Val::Px(560.0), ..default() }).with_children(|g| {
                            for i in 0..24u8 {
                                g.spawn((Button, RuneBtn(i), Node { width: Val::Px(56.0), height: Val::Px(72.0), ..default() }, ui.node(&format!("rune_{i}")), BackgroundColor(Color::srgb(0.1, 0.1, 0.2))));
                            }
                        });
                        panel.spawn((Text::new(format!("Verse: {}", seq.iter().map(|r| format!("[{r}]")).collect::<Vec<_>>().join(" "))), TextFont { font_size: 24.0, ..default() }, TextColor(Color::srgb(0.5, 1.0, 1.0))));
                    }
                    MenuKind::Map => {
                        if let Some(img) = map_img {
                            panel.spawn((Node { width: Val::Px(640.0), height: Val::Px(640.0), ..default() }, ImageNode::new(img)));
                        }
                        panel.spawn((Text::new(format!("MAP - {}   (you: tile {:.0},{:.0})   Esc to close", game.0.current_map, game.0.players[0].pos[0], game.0.players[0].pos[1])), TextFont { font_size: 18.0, ..default() }));
                    }
                    _ => {
                        for r in rows {
                            if r.header {
                                panel.spawn((Text::new(r.label), TextFont { font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.78, 0.35))));
                            } else {
                                let col = if r.enabled { Color::srgb(0.95, 0.92, 0.8) } else { Color::srgb(0.5, 0.48, 0.44) };
                                panel.spawn((Button, MenuRow(if r.enabled { r.tag.clone() } else { String::new() }), Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)), ..default() }, BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.04)))).with_children(|b| {
                                    b.spawn((Text::new(r.label), TextFont { font_size: 18.0, ..default() }, TextColor(col)));
                                });
                            }
                        }
                        if more {
                            panel.spawn((Text::new("... scroll with the mouse wheel / PageDown ..."), TextFont { font_size: 15.0, ..default() }, TextColor(Color::srgb(0.6, 0.6, 0.6))));
                        }
                    }
                }
                panel.spawn((Text::new(msg), TextFont { font_size: 17.0, ..default() }, TextColor(Color::srgb(0.6, 1.0, 0.8))));
            });
        });
}

#[allow(clippy::too_many_arguments)]
fn menu_input(
    rows: Query<(&Interaction, &MenuRow), Changed<Interaction>>,
    runes: Query<(&Interaction, &RuneBtn), Changed<Interaction>>,
    kb: Res<ButtonInput<KeyCode>>,
    mut wheel: EventReader<MouseWheel>,
    mut menu: ResMut<MenuState>,
    mut kind: ResMut<MenuKind>,
    mut game: ResMut<Game>,
    world: Res<WorldRes>,
    db: Res<DbRes>,
    mut cast: EventWriter<CastRequest>,
    mut sing: EventWriter<SingRequest>,
    mut next: ResMut<NextState<AppState>>,
    mut sfx: EventWriter<SfxEvent>,
    mut toast: ResMut<Toast>,
    mut fx: EventWriter<EffectEvent>,
) {
    for w in wheel.read() {
        let d = if w.y > 0.0 { -2i32 } else { 2 };
        menu.scroll = (menu.scroll as i32 + d).clamp(0, menu.rows.len().saturating_sub(VISIBLE_ROWS) as i32) as usize;
        menu.dirty = true;
    }
    if kb.just_pressed(KeyCode::PageDown) {
        menu.scroll = (menu.scroll + 8).min(menu.rows.len().saturating_sub(VISIBLE_ROWS));
        menu.dirty = true;
    }
    if kb.just_pressed(KeyCode::PageUp) {
        menu.scroll = menu.scroll.saturating_sub(8);
        menu.dirty = true;
    }
    if let MenuKind::Shop { shop, selling } = kind.clone() {
        if kb.just_pressed(KeyCode::Tab) {
            *kind = MenuKind::Shop { shop: shop.clone(), selling: !selling };
            menu.rows = menus::shop_rows(&game.0, &db.0, &shop, !selling);
            menu.scroll = 0;
            menu.dirty = true;
            return;
        }
    }
    for (i, r) in &runes {
        if *i == Interaction::Pressed {
            menu.seq.push(r.0);
            sfx.write(SfxEvent("ui_click".into()));
            menu.dirty = true;
            if menu.seq.len() == 3 {
                sing.write(SingRequest(menu.seq.clone()));
                menu.seq.clear();
                next.set(AppState::Playing);
            }
        }
    }
    for (i, r) in &rows {
        if *i != Interaction::Pressed || r.0.is_empty() {
            continue;
        }
        sfx.write(SfxEvent("ui_click".into()));
        let Some((verb, arg)) = r.0.split_once(':') else { continue };
        let k = kind.clone();
        let result: Result<String, String> = match (verb, &k) {
            ("buy", MenuKind::Shop { shop, .. }) => menus::buy(&mut game.0, &db.0, shop, arg),
            ("sell", MenuKind::Shop { shop, .. }) => menus::sell(&mut game.0, &db.0, shop, arg),
            ("craft", MenuKind::Craft { .. }) => menus::craft(&mut game.0, &db.0, arg),
            ("go", MenuKind::Travel { by_ship }) => match menus::travel(&mut game.0, &world.0, &db.0, arg, *by_ship) {
                Ok(m) => {
                    next.set(AppState::Playing);
                    sfx.write(SfxEvent("bifrost".into()));
                    Ok(m)
                }
                Err(e) => Err(e),
            },
            ("cmd", MenuKind::Cheats) => {
                let reg = u67_console::Registry::standard();
                match reg.parse(arg) {
                    Ok(Some(a)) => {
                        let out = crate::apply::apply(&mut game.0, &world.0, a);
                        for f in out.effects {
                            fx.write(EffectEvent(f));
                        }
                        Ok(out.lines.join("  "))
                    }
                    Ok(None) => Ok(String::new()),
                    Err(e) => Err(e),
                }
            }
            ("cast", MenuKind::Spells) => {
                cast.write(CastRequest(arg.to_string()));
                next.set(AppState::Playing);
                Ok(String::new())
            }
            _ => Ok(String::new()),
        };
        match result {
            Ok(m) => {
                if !m.is_empty() {
                    toast.text = m.clone();
                    toast.timer = 3.0;
                    menu.msg = m;
                }
            }
            Err(e) => {
                menu.msg = e;
                sfx.write(SfxEvent("ui_error".into()));
            }
        }
        // refresh rows
        match &k {
            MenuKind::Shop { shop, selling } => menu.rows = menus::shop_rows(&game.0, &db.0, shop, *selling),
            MenuKind::Craft { station } => menu.rows = menus::craft_rows(&game.0, &db.0, station),
            MenuKind::Travel { by_ship } => menu.rows = menus::travel_rows(&game.0, *by_ship),
            MenuKind::Cheats => menu.rows = menus::cheat_rows(&game.0),
            _ => {}
        }
        menu.dirty = true;
    }
}

// ------------------------------------------------------------------ death screen
#[derive(Component)]
struct DeadRoot;

fn spawn_dead(mut commands: Commands) {
    commands.spawn((DeadRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: Val::Px(14.0), ..default() }, BackgroundColor(Color::srgba(0.25, 0.0, 0.0, 0.7)))).with_children(|p| {
        p.spawn((Text::new("YOU HAVE FALLEN"), TextFont { font_size: 64.0, ..default() }, TextColor(Color::srgb(1.0, 0.8, 0.7))));
        p.spawn((Text::new("[Enter] rise again at Kaupang      [L] load quicksave"), TextFont { font_size: 26.0, ..default() }));
    });
}

fn dead_input(kb: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>, world: Res<WorldRes>, paths: Res<Paths>, mut next: ResMut<NextState<AppState>>, mut toast: ResMut<Toast>) {
    if kb.just_pressed(KeyCode::Enter) {
        let s = world.0.maps["midgard"].places.get("start").copied().unwrap_or_default();
        let g = &mut game.0;
        g.current_map = "midgard".into();
        g.players[0].pos = [s.x as f32 + 0.5, s.y as f32 + 0.5];
        g.players[0].stats.hp = g.players[0].stats.max_hp() / 2;
        let lost = g.players[0].inventory.count("silver") / 10;
        g.players[0].inventory.remove("silver", lost);
        g.vehicle = None;
        g.map_dirty = true;
        *toast = Toast { text: format!("You wake in Kaupang, lighter by {lost} silver."), timer: 4.0 };
        next.set(AppState::Playing);
    } else if kb.just_pressed(KeyCode::KeyL) {
        match crate::save::read(&paths.saves, None) {
            Ok(d) => {
                game.0 = d;
                next.set(AppState::Playing);
            }
            Err(e) => *toast = Toast { text: format!("No quicksave: {e}"), timer: 3.0 },
        }
    }
}

// ------------------------------------------------------------------ HUD extras
#[derive(Component)]
struct Hud2;

fn spawn_hud2(mut commands: Commands, q: Query<(), With<Hud2>>) {
    if q.iter().next().is_some() {
        return;
    }
    commands.spawn((Hud2, Text::new(""), TextFont { font_size: 17.0, ..default() }, TextColor(Color::srgb(1.0, 0.95, 0.8)), Node { position_type: PositionType::Absolute, right: Val::Px(12.0), top: Val::Px(8.0), ..default() }));
}

fn update_hud2(game: Res<Game>, db: Res<DbRes>, npcs: Res<Npcs>, rt: Res<crate::combat::PlayerRt>, op: Res<crate::interact::Operating>, cursor: Res<crate::combat::Cursor>, creatures: Query<&crate::creatures::Creature>, world: Res<WorldRes>, mut q: Query<&mut Text, With<Hud2>>) {
    let Ok(mut t) = q.single_mut() else { return };
    let p = &game.0.players[0];
    let mut s = String::new();
    if let Some(w) = p.inventory.equipped.get(&Slot::HandR).and_then(|i| items::get(&i.id)) {
        if let Some(gs) = u67_world::combat::gun_stats(w.id) {
            let loaded = p.loaded.get(w.id).copied().unwrap_or(0);
            let reserve = p.inventory.count(w.ammo.unwrap_or(""));
            s += &format!("{}  {}/{}  (+{})", w.name, if game.0.cheats.infinite_ammo { "inf".into() } else { loaded.to_string() }, gs.mag, if game.0.cheats.infinite_ammo { "inf".into() } else { reserve.to_string() });
            if rt.reload.is_some() {
                s += "  RELOADING";
            }
        } else {
            s += w.name;
        }
    } else {
        s += "Unarmed";
    }
    s += &format!("\nMana {}/{}   [1-3] spells  [B] book  [L] sing", p.stats.mana, p.stats.max_mana());
    if op.0.is_some() {
        s += "\nCANNON: aim + click to fire (E to leave)";
    }
    if game.0.vehicle.is_some() {
        s += "\nSAILING: M star map, click = cannon, E = disembark";
    }
    for n in npcs.list.iter().filter(|n| n.in_party) {
        s += &format!("\n+ {}", n.name);
    }
    if let Some((q, st)) = db.0.quests.iter().filter(|q| q.category == u67_world::quests::Category::Main).find_map(|q| game.0.quests.get(&q.id).filter(|p| p.state == QuestState::Active).map(|p| (q, p.step))) {
        s += &format!("\n\nQUEST: {}\n{}", q.title, q.steps.get(st as usize).map_or("", |x| x.text.as_str()));
    }
    if game.0.cheats.inspect && cursor.valid {
        let c = cursor.tile;
        let mut best: Option<(f32, String)> = None;
        for cr in creatures.iter() {
            let d = cr.pos.distance(c);
            if d < 1.5 && best.as_ref().is_none_or(|b| d < b.0) {
                best = Some((d, format!("creature {} hp {}/{} stun {:.1} ai-aggro {}", cr.def, cr.hp, cr.max_hp, cr.stun, cr.aggro)));
            }
        }
        for n in &npcs.list {
            let d = Vec2::new(n.pos[0], n.pos[1]).distance(c);
            if d < 1.5 && best.as_ref().is_none_or(|b| d < b.0) {
                best = Some((d, format!("npc {} ({}) {:?} at {:.1},{:.1}", n.id, n.name, n.activity, n.pos[0], n.pos[1])));
            }
        }
        let t = u67_core::TilePos::new(c.x.floor() as i32, c.y.floor() as i32);
        if let Some(m) = world.0.maps.get(&game.0.current_map) {
            s += &format!("\n\nINSPECT tile {},{} = {}  walkable={}", t.x, t.y, u67_world::tiles::def(m.tile(t)).name, m.walkable(t));
            for o in m.objects_at(t) {
                s += &format!("\n object {} frame {} group {}", o.kind, o.frame, o.group);
            }
        }
        if let Some((_, b)) = best {
            s += &format!("\n{b}");
        }
    }
    s += "\n\n[I] inventory  [J] journal  [M] map  [E] interact  [F2] cheats";
    t.0 = s;
}

// ------------------------------------------------------------------ plugin
fn ready(game: Option<Res<Game>>, db: Option<Res<DbRes>>, sheets: Option<Res<Sheets>>, ui: Option<Res<UiAtlas>>) -> bool {
    game.is_some() && db.is_some() && sheets.is_some() && ui.is_some()
}

/// Periodic game-logic ticks: quests + visits (twice a second).
fn progress_tick(time: Res<Time>, mut acc: Local<f32>, mut game: ResMut<Game>, world: Res<WorldRes>, db: Res<DbRes>, mut toast: ResMut<Toast>, mut fx: EventWriter<EffectEvent>) {
    *acc += time.delta_secs();
    if *acc < 0.5 {
        return;
    }
    *acc = 0.0;
    let cap = game.0.players[0].stats.str_ as f32 * 2.0 + 20.0;
    game.0.players[0].inventory.max_weight = cap;
    let visits = script::track_visits(&mut game.0, &world.0);
    let out = script::tick_quests(&mut game.0, &db.0, &world.0);
    let mut lines = visits;
    lines.extend(out.toasts);
    if let Some(l) = lines.last() {
        *toast = Toast { text: l.clone(), timer: 4.0 };
    }
    for e in out.effects {
        fx.write(EffectEvent(e));
    }
}

pub struct GuiPlugin;
impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        use AppState::*;
        app.init_resource::<MenuKind>()
            .init_resource::<MenuState>()
            .init_resource::<InvUi>()
            .init_resource::<DialogueRt>()
            .add_systems(Startup, load_ui_atlas)
            .add_systems(Update, handle_requests.run_if(ready).run_if(in_state(Playing).or(in_state(Dialogue))))
            .add_systems(OnEnter(Dialogue), |mut d: ResMut<DialogueRt>| d.dirty = true)
            .add_systems(Update, (build_dialogue, dialogue_input).chain().run_if(ready).run_if(in_state(Dialogue)))
            .add_systems(OnExit(Dialogue), despawn_all::<DlgRoot>)
            .add_systems(OnEnter(Inventory), enter_inventory.run_if(resource_exists::<UiAtlas>))
            .add_systems(Update, (inventory_mouse, build_inventory, refresh_ghost_icon).chain().run_if(ready).run_if(in_state(Inventory)))
            .add_systems(OnExit(Inventory), (exit_inventory, despawn_all::<InvRoot>, despawn_all::<Ghost>))
            .add_systems(OnEnter(Menu), |mut m: ResMut<MenuState>| m.dirty = true)
            .add_systems(Update, (menu_input, build_menu).chain().run_if(ready).run_if(in_state(Menu)))
            .add_systems(OnExit(Menu), despawn_all::<MenuRoot>)
            .add_systems(OnEnter(Dead), spawn_dead)
            .add_systems(Update, dead_input.run_if(ready).run_if(in_state(Dead)))
            .add_systems(OnExit(Dead), despawn_all::<DeadRoot>)
            .add_systems(OnEnter(Playing), spawn_hud2)
            .add_systems(Update, (update_hud2, progress_tick).run_if(ready).run_if(in_state(Playing)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_descriptions() {
        let mut bp = Item::new("backpack", 1);
        bp.contents.push(Item::new("rope", 2));
        let d = describe_item(&bp);
        assert!(d.contains("Backpack") && d.contains("holds 40"));
        assert!(describe_item(&Item::new("viking_axe", 1)).contains("dmg 14"));
        assert!(describe_item(&Item::new("ammo_9mm", 30)).contains("x30"));
    }

    #[test]
    fn paperdoll_slot_names_map_to_all_equip_slots() {
        let mut seen = std::collections::BTreeSet::new();
        for (n, ..) in u67_assetgen::ui::PAPERDOLL_SLOTS {
            seen.insert(slot_to_equip(n));
        }
        assert_eq!(seen.len(), 11);
    }

    #[test]
    fn map_image_hides_unexplored() {
        let w = u67_mapgen::generate_world(67);
        let m = &w.maps["maani"];
        let mut d = GameData::new(1, "maani", [20.0, 20.0]);
        let img = map_image(m, &d);
        let px = img.data.as_ref().unwrap();
        assert_eq!(&px[0..3], &[10, 8, 14]);
        d.map_revealed = true;
        let img2 = map_image(m, &d);
        assert_ne!(&img2.data.as_ref().unwrap()[0..3], &[10, 8, 14]);
        d.map_revealed = false;
        d.explored.insert("maani:0:0".into());
        assert_ne!(&map_image(m, &d).data.as_ref().unwrap()[0..3], &[10, 8, 14]);
    }
}
