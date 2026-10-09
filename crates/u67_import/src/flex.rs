//! Reader (and a test-only writer) for the "Flex" archive format used by Ultima 7:
//! 80-byte title, u32 magic 0xffff1a00 at 0x50, u32 entry count at 0x54, then at 0x80
//! `count` pairs of (offset: u32, length: u32), all little-endian.
pub const MAGIC: u32 = 0xffff_1a00;

pub struct Flex<'a> {
    data: &'a [u8],
    index: Vec<(usize, usize)>,
}

fn u32_at(d: &[u8], o: usize) -> Option<u32> {
    d.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()))
}

impl<'a> Flex<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self, String> {
        if u32_at(data, 0x50) != Some(MAGIC) {
            return Err("not a Flex file (bad magic)".into());
        }
        let count = u32_at(data, 0x54).ok_or("truncated header")? as usize;
        let mut index = Vec::with_capacity(count);
        for i in 0..count {
            let off = u32_at(data, 0x80 + i * 8).ok_or("truncated index")? as usize;
            let len = u32_at(data, 0x84 + i * 8).ok_or("truncated index")? as usize;
            if off != 0 && off.checked_add(len).is_none_or(|e| e > data.len()) {
                return Err(format!("entry {i} out of range"));
            }
            index.push((off, len));
        }
        Ok(Self { data, index })
    }
    pub fn len(&self) -> usize {
        self.index.len()
    }
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
    /// Entry bytes; `None` when the slot is empty or out of range.
    pub fn get(&self, i: usize) -> Option<&'a [u8]> {
        let (off, len) = *self.index.get(i)?;
        if off == 0 || len == 0 {
            return None;
        }
        self.data.get(off..off + len)
    }
}

/// Build a Flex archive (used by tests and by the synthetic-data generator).
pub fn build(entries: &[Vec<u8>]) -> Vec<u8> {
    let mut out = vec![0u8; 0x80 + entries.len() * 8];
    out[..8].copy_from_slice(b"u67test\0");
    out[0x50..0x54].copy_from_slice(&MAGIC.to_le_bytes());
    out[0x54..0x58].copy_from_slice(&(entries.len() as u32).to_le_bytes());
    out[0x58..0x5c].copy_from_slice(&0xccu32.to_le_bytes());
    for (i, e) in entries.iter().enumerate() {
        if e.is_empty() {
            continue;
        }
        let off = out.len() as u32;
        out.extend_from_slice(e);
        out[0x80 + i * 8..0x84 + i * 8].copy_from_slice(&off.to_le_bytes());
        out[0x84 + i * 8..0x88 + i * 8].copy_from_slice(&(e.len() as u32).to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_and_errors() {
        let b = build(&[b"hello".to_vec(), vec![], b"world!".to_vec()]);
        let f = Flex::parse(&b).unwrap();
        assert_eq!(f.len(), 3);
        assert_eq!(f.get(0), Some(&b"hello"[..]));
        assert_eq!(f.get(1), None);
        assert_eq!(f.get(2), Some(&b"world!"[..]));
        assert_eq!(f.get(9), None);
        assert!(Flex::parse(b"nope").is_err());
        let mut bad = b.clone();
        bad[0x84] = 0xff; // length beyond the file
        bad[0x85] = 0xff;
        assert!(Flex::parse(&bad).is_err());
    }
}
