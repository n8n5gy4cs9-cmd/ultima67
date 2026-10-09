//! Reads the pieces of an Ultima 7 install that describe the world map.
//!
//! Layout (from the open file-format knowledge shared by the Exult project):
//! * `u7map`   - 12x12 superchunks, each 16x16 little-endian u16 chunk-template ids.
//! * `u7chunks`- chunk templates, 16x16 tiles x 2 bytes: shape = b0 + 256*(b1&3), frame = (b1>>2)&0x1f.
//! * `u7ifixNN`- Flex per superchunk (hex NN = 00..8f); entry (cy*16+cx) lists the chunk's fixed
//!   objects, 4 bytes each: [tx<<4|ty][lift&15][shape lo][shape hi&3 | frame<<2].
//! * `shapes.vga` - Flex of shapes. Shapes below 0x96 are flat 8x8 terrain tiles, 64 raw bytes per frame.
//! * `palettes.flx` - entry 0: 256 RGB triplets, 6 bits per channel.
//! * `text.flx` - entries 0..0x3ff are shape names (NUL-terminated strings).
//! * `tfa.dat`  - 3 flag bytes per shape: [0]&8 solid, [0]&16 water, [1]&32 door, [2]&7/>>3 x/y dims, [0]>>5 height.
use crate::flex::Flex;
use std::path::{Path, PathBuf};

pub const FIRST_OBJ_SHAPE: usize = 0x96;
pub const SCHUNKS: usize = 12;
pub const CHUNKS: usize = SCHUNKS * 16; // 192
pub const TILES: usize = CHUNKS * 16; // 3072

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedObj {
    pub x: i32,
    pub y: i32,
    pub lift: u8,
    pub shape: u16,
    pub frame: u8,
}

#[derive(Default)]
pub struct U7Data {
    /// chunk-template id per absolute chunk, row-major (192x192)
    pub terrain: Vec<u16>,
    /// each template: 256 packed (shape, frame) tiles
    pub templates: Vec<Vec<(u16, u8)>>,
    pub fixed: Vec<FixedObj>,
    pub names: Vec<String>,
    pub tfa: Vec<[u8; 3]>,
    /// average RGB per frame for each flat shape (index = shape)
    pub flats: Vec<Vec<[u8; 3]>>,
}

impl U7Data {
    pub fn name(&self, shape: u16) -> &str {
        self.names.get(shape as usize).map_or("", |s| s.as_str())
    }
    fn flag(&self, shape: u16, byte: usize, bit: u8) -> bool {
        self.tfa.get(shape as usize).is_some_and(|t| t[byte] & (1 << bit) != 0)
    }
    pub fn solid(&self, shape: u16) -> bool {
        self.flag(shape, 0, 3)
    }
    pub fn water(&self, shape: u16) -> bool {
        self.flag(shape, 0, 4)
    }
    pub fn door(&self, shape: u16) -> bool {
        self.flag(shape, 1, 5)
    }
    /// (x tiles, y tiles, height)
    pub fn dims(&self, shape: u16) -> (u8, u8, u8) {
        self.tfa.get(shape as usize).map_or((1, 1, 0), |t| (1 + (t[2] & 7), 1 + ((t[2] >> 3) & 7), t[0] >> 5))
    }
    pub fn flat_rgb(&self, shape: u16, frame: u8) -> Option<[u8; 3]> {
        let f = self.flats.get(shape as usize)?;
        f.get(frame as usize).or(f.first()).copied()
    }
}

/// Find `name` in `dir` ignoring case (original installs use upper-case DOS names).
pub fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let want = name.to_lowercase();
    std::fs::read_dir(dir).ok()?.flatten().find(|e| e.file_name().to_string_lossy().to_lowercase() == want).map(|e| e.path())
}

fn read(dir: &Path, name: &str) -> Result<Vec<u8>, String> {
    let p = find_file(dir, name).ok_or_else(|| format!("{name} not found in {}", dir.display()))?;
    std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))
}

pub fn parse_terrain(map: &[u8]) -> Result<Vec<u16>, String> {
    if map.len() < SCHUNKS * SCHUNKS * 512 {
        return Err(format!("u7map too small ({} bytes)", map.len()));
    }
    let mut out = vec![0u16; CHUNKS * CHUNKS];
    for sc in 0..SCHUNKS * SCHUNKS {
        let (scy, scx) = (16 * (sc / SCHUNKS), 16 * (sc % SCHUNKS));
        for i in 0..256 {
            let o = sc * 512 + i * 2;
            let id = u16::from_le_bytes([map[o], map[o + 1]]);
            let (cx, cy) = (scx + i % 16, scy + i / 16);
            out[cy * CHUNKS + cx] = id;
        }
    }
    Ok(out)
}

pub fn parse_templates(chunks: &[u8]) -> Vec<Vec<(u16, u8)>> {
    chunks.chunks_exact(512).map(|t| t.chunks_exact(2).map(|b| (b[0] as u16 + 256 * (b[1] & 3) as u16, (b[1] >> 2) & 0x1f)).collect()).collect()
}

pub fn parse_ifix(sc: usize, data: &[u8], out: &mut Vec<FixedObj>) -> Result<(), String> {
    let flex = Flex::parse(data)?;
    let (scy, scx) = (16 * (sc / SCHUNKS) as i32, 16 * (sc % SCHUNKS) as i32);
    for i in 0..flex.len().min(256) {
        let Some(e) = flex.get(i) else { continue };
        let (cx, cy) = (scx + (i % 16) as i32, scy + (i / 16) as i32);
        for o in e.chunks_exact(4) {
            out.push(FixedObj {
                x: cx * 16 + ((o[0] >> 4) & 15) as i32,
                y: cy * 16 + (o[0] & 15) as i32,
                lift: o[1] & 15,
                shape: o[2] as u16 + 256 * (o[3] & 3) as u16,
                frame: o[3] >> 2,
            });
        }
    }
    Ok(())
}

pub fn parse_names(text_flx: &[u8]) -> Result<Vec<String>, String> {
    let f = Flex::parse(text_flx)?;
    Ok((0..f.len().min(0x400)).map(|i| f.get(i).map(|b| String::from_utf8_lossy(b.split(|c| *c == 0).next().unwrap_or(&[])).to_string()).unwrap_or_default()).collect())
}

pub fn parse_flats(shapes_vga: &[u8], palette: &[[u8; 3]]) -> Result<Vec<Vec<[u8; 3]>>, String> {
    let f = Flex::parse(shapes_vga)?;
    let mut out = vec![];
    for s in 0..FIRST_OBJ_SHAPE.min(f.len()) {
        let frames = match f.get(s) {
            Some(e) => e
                .chunks_exact(64)
                .map(|px| {
                    let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);
                    for p in px {
                        let c = palette.get(*p as usize).copied().unwrap_or([0, 0, 0]);
                        r += c[0] as u32;
                        g += c[1] as u32;
                        b += c[2] as u32;
                    }
                    [(r / 64) as u8, (g / 64) as u8, (b / 64) as u8]
                })
                .collect(),
            None => vec![],
        };
        out.push(frames);
    }
    Ok(out)
}

pub fn parse_palette(entry: &[u8]) -> Vec<[u8; 3]> {
    entry.chunks_exact(3).take(256).map(|c| [(c[0] as u32 * 255 / 63) as u8, (c[1] as u32 * 255 / 63) as u8, (c[2] as u32 * 255 / 63) as u8]).collect()
}

/// Load everything from `dir` (the STATIC folder, or the install root containing it).
pub fn load(dir: &Path) -> Result<U7Data, String> {
    let dir = if find_file(dir, "u7map").is_some() {
        dir.to_path_buf()
    } else {
        find_file(dir, "static").ok_or_else(|| format!("no u7map or STATIC folder in {}", dir.display()))?
    };
    let mut d = U7Data { terrain: parse_terrain(&read(&dir, "u7map")?)?, templates: parse_templates(&read(&dir, "u7chunks")?), ..Default::default() };
    for sc in 0..SCHUNKS * SCHUNKS {
        match read(&dir, &format!("u7ifix{sc:02x}")) {
            Ok(b) => parse_ifix(sc, &b, &mut d.fixed)?,
            Err(_) => continue, // some superchunks have no fixed objects
        }
    }
    d.names = read(&dir, "text.flx").and_then(|b| parse_names(&b)).unwrap_or_default();
    d.tfa = read(&dir, "tfa.dat").map(|b| b.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()).unwrap_or_default();
    let pal_bytes = read(&dir, "palettes.flx")?;
    let pal = Flex::parse(&pal_bytes)?.get(0).map(parse_palette).ok_or("palettes.flx entry 0 missing")?;
    d.flats = parse_flats(&read(&dir, "shapes.vga")?, &pal)?;
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terrain_and_templates_decode() {
        let mut map = vec![0u8; 144 * 512];
        // superchunk 13 (row 1, col 1), chunk index 17 (cy=1,cx=1) -> template 5
        map[13 * 512 + 17 * 2] = 5;
        let t = parse_terrain(&map).unwrap();
        assert_eq!(t[(16 + 1) * CHUNKS + (16 + 1)], 5);
        assert_eq!(t[0], 0);
        assert!(parse_terrain(&[0; 10]).is_err());
        // shape 0x3ff frame 7 packs to b0=0xff, b1=3 | 7<<2
        let tpl = parse_templates(&[0xff, 3 | (7 << 2)].repeat(256));
        assert_eq!(tpl[0][0], (0x3ff, 7));
        assert_eq!(tpl[0].len(), 256);
    }
    #[test]
    fn ifix_objects_have_absolute_coordinates() {
        // chunk (cx=2,cy=1) of superchunk 1 -> absolute chunk (18,1); object at tx=3,ty=4, lift 5, shape 0x1ab, frame 2
        let mut entries = vec![vec![]; 256];
        entries[16 + 2] = vec![(3 << 4) | 4, 5, 0xab, 1 | (2 << 2)];
        let data = crate::flex::build(&entries);
        let mut out = vec![];
        parse_ifix(1, &data, &mut out).unwrap();
        assert_eq!(out, vec![FixedObj { x: 18 * 16 + 3, y: 16 + 4, lift: 5, shape: 0x1ab, frame: 2 }]);
    }
    #[test]
    fn names_flags_and_flats() {
        let mut names: Vec<Vec<u8>> = vec![vec![]; 0x400];
        names[0x97] = b"oak tree\0junk".to_vec();
        let n = parse_names(&crate::flex::build(&names)).unwrap();
        assert_eq!(n[0x97], "oak tree");
        let mut d = U7Data { tfa: vec![[0; 3]; 200], ..Default::default() };
        d.tfa[0x97] = [8 | (3 << 5), 0, 1 | (2 << 3)];
        d.tfa[0x98] = [16, 32, 0];
        assert!(d.solid(0x97) && !d.water(0x97) && d.water(0x98) && d.door(0x98));
        assert_eq!(d.dims(0x97), (2, 3, 3));
        let pal: Vec<[u8; 3]> = (0..256).map(|i| [i as u8, 0, 0]).collect();
        let mut shapes = vec![vec![]; 0x96];
        shapes[1] = [10u8; 64].iter().chain([20u8; 64].iter()).copied().collect();
        let flats = parse_flats(&crate::flex::build(&shapes), &pal).unwrap();
        assert_eq!(flats[1], vec![[10, 0, 0], [20, 0, 0]]);
        assert!(flats[0].is_empty());
        assert_eq!(parse_palette(&[63, 0, 31])[0], [255, 0, 125]);
    }
}
