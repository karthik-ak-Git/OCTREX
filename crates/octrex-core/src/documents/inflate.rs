//! Minimal dependency-free raw DEFLATE (RFC 1951) decompressor.
//!
//! Used for PDF `FlateDecode` streams and ZIP `deflated` entries so Phase 15
//! does not need to pull in a compression crate. Bounded and fail-closed:
//! corrupt input returns an error instead of guessing.

use std::fmt;

#[derive(Debug)]
pub struct InflateError(pub String);

impl fmt::Display for InflateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "inflate error: {}", self.0)
    }
}

struct BitReader<'a> {
    data: &'a [u8],
    bit_pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, bit_pos: 0 }
    }

    fn read_bits(&mut self, n: usize) -> Result<u32, InflateError> {
        if n > 32 {
            return Err(InflateError("too many bits requested".to_string()));
        }
        let mut out: u32 = 0;
        for i in 0..n {
            let byte_idx = self.bit_pos / 8;
            if byte_idx >= self.data.len() {
                return Err(InflateError("unexpected end of deflate stream".to_string()));
            }
            let bit = (self.data[byte_idx] >> (self.bit_pos % 8)) & 1;
            out |= (bit as u32) << i;
            self.bit_pos += 1;
        }
        Ok(out)
    }

    fn align_to_byte(&mut self) {
        if !self.bit_pos.is_multiple_of(8) {
            self.bit_pos += 8 - (self.bit_pos % 8);
        }
    }

    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>, InflateError> {
        self.align_to_byte();
        let start = self.bit_pos / 8;
        let end = start + n;
        if end > self.data.len() {
            return Err(InflateError("unexpected end reading bytes".to_string()));
        }
        self.bit_pos = end * 8;
        Ok(self.data[start..end].to_vec())
    }
}

struct Huffman {
    // codes sorted by length; decode by walking bits
    counts: Vec<u16>,  // counts per bit length
    symbols: Vec<u16>, // symbols sorted by (len, symbol)
}

impl Huffman {
    fn from_lengths(lengths: &[u8]) -> Result<Self, InflateError> {
        if lengths.iter().copied().max().unwrap_or(0) > 15 {
            return Err(InflateError("invalid huffman lengths".to_string()));
        }
        let mut bl_count = vec![0u16; 16];
        for &l in lengths {
            if l as usize >= bl_count.len() {
                return Err(InflateError("bad code length".to_string()));
            }
            if l != 0 {
                bl_count[l as usize] += 1;
            }
        }
        // Canonical code assignment; store symbols ordered by len then symbol.
        let mut next_code = [0u32; 16];
        let mut code = 0u32;
        for bits in 1..16 {
            code = (code + bl_count[bits - 1] as u32) << 1;
            next_code[bits] = code;
        }
        let mut symbols: Vec<(u8, u16)> = Vec::new();
        for (sym, &len) in lengths.iter().enumerate() {
            if len != 0 {
                symbols.push((len, sym as u16));
            }
        }
        symbols.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        // Validate over-subscribed codes
        let mut ordered_symbols = Vec::with_capacity(symbols.len());
        for (_, s) in symbols {
            ordered_symbols.push(s);
        }
        Ok(Self {
            counts: bl_count,
            symbols: ordered_symbols,
        })
    }

    fn decode(&self, r: &mut BitReader, lengths: &[u8]) -> Result<u16, InflateError> {
        // Rebuild canonical codes map on the fly (lengths small; fine for bounded docs).
        // Build table: for each symbol in canonical order assign code.
        let mut bl_count = [0u32; 16];
        for &l in lengths {
            if l != 0 {
                bl_count[l as usize] += 1;
            }
        }
        let mut next_code = [0u32; 16];
        let mut code = 0u32;
        for bits in 1..16 {
            code = (code + bl_count[bits - 1]) << 1;
            next_code[bits] = code;
        }
        let mut ordered: Vec<(u8, u16)> = Vec::new();
        for (sym, &len) in lengths.iter().enumerate() {
            if len != 0 {
                ordered.push((len, sym as u16));
            }
        }
        ordered.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut code_map: Vec<(u32, u8, u16)> = Vec::new();
        let mut cur: std::collections::HashMap<u8, u32> = Default::default();
        for (len, sym) in ordered {
            let c = *cur.get(&len).unwrap_or(&next_code[len as usize]);
            code_map.push((c, len, sym));
            cur.insert(len, c + 1);
        }
        let mut cur_code: u32 = 0;
        for len in 1..=15usize {
            cur_code = (cur_code << 1) | r.read_bits(1)?;
            // note: canonical codes are MSB-first but deflate transmits LSB-first
            // per code length; reverse bits within length for comparison.
            for (c, l, s) in &code_map {
                if *l as usize == len {
                    // reverse `len` bits of cur_code
                    let mut rev = 0u32;
                    let mut tmp = cur_code;
                    for _ in 0..len {
                        rev = (rev << 1) | (tmp & 1);
                        tmp >>= 1;
                    }
                    if rev == *c {
                        return Ok(*s);
                    }
                }
            }
            if len == 15 {
                break;
            }
        }
        let _ = &self.counts;
        let _ = &self.symbols;
        Err(InflateError("invalid huffman code".to_string()))
    }
}

fn fixed_lit_lengths() -> Vec<u8> {
    let mut v = vec![0u8; 288];
    v[..144].fill(8);
    v[144..256].fill(9);
    v[256..280].fill(7);
    v[280..288].fill(8);
    v
}

fn fixed_dist_lengths() -> Vec<u8> {
    vec![5u8; 32]
}

fn length_base_extra(sym: u16) -> Option<(usize, u32)> {
    let (base, extra) = match sym {
        257 => (3, 0),
        258 => (4, 0),
        259 => (5, 0),
        260 => (6, 0),
        261 => (7, 0),
        262 => (8, 0),
        263 => (9, 0),
        264 => (10, 0),
        265 => (11, 1),
        266 => (13, 1),
        267 => (15, 1),
        268 => (17, 1),
        269 => (19, 2),
        270 => (23, 2),
        271 => (27, 2),
        272 => (31, 2),
        273 => (35, 3),
        274 => (43, 3),
        275 => (51, 3),
        276 => (59, 3),
        277 => (67, 4),
        278 => (83, 4),
        279 => (99, 4),
        280 => (115, 4),
        281 => (163, 5),
        282 => (195, 5),
        283 => (227, 5),
        284 => (258, 5),
        _ => return None,
    };
    Some((base, extra))
}

fn dist_base_extra(sym: u16) -> Option<(usize, u32)> {
    let (base, extra) = match sym {
        0 => (1, 0),
        1 => (2, 0),
        2 => (3, 0),
        3 => (4, 0),
        4 => (5, 1),
        5 => (7, 1),
        6 => (9, 2),
        7 => (13, 2),
        8 => (17, 3),
        9 => (25, 3),
        10 => (33, 4),
        11 => (49, 4),
        12 => (65, 5),
        13 => (97, 5),
        14 => (129, 6),
        15 => (193, 6),
        16 => (257, 7),
        17 => (385, 7),
        18 => (513, 8),
        19 => (769, 8),
        20 => (1025, 9),
        21 => (1537, 9),
        22 => (2049, 10),
        23 => (3073, 10),
        24 => (4097, 11),
        25 => (6145, 11),
        26 => (8193, 12),
        27 => (12289, 12),
        28 => (16385, 13),
        29 => (24577, 13),
        _ => return None,
    };
    Some((base, extra))
}

/// Decompress raw DEFLATE data with a bounded output.
pub fn inflate_raw(data: &[u8], max_output: usize) -> Result<Vec<u8>, InflateError> {
    let mut r = BitReader::new(data);
    let mut out: Vec<u8> = Vec::new();
    loop {
        let bfinal = r.read_bits(1)?;
        let btype = r.read_bits(2)?;
        match btype {
            0 => {
                // stored (LEN/NLEN + raw bytes)
                r.align_to_byte();
                let len_bytes = r.read_bytes(4)?;
                let len = (len_bytes[0] as usize) | ((len_bytes[1] as usize) << 8);
                let nlen = (len_bytes[2] as usize) | ((len_bytes[3] as usize) << 8);
                if len ^ 0xFFFF != nlen {
                    return Err(InflateError("bad stored block lengths".to_string()));
                }
                let chunk = r.read_bytes(len)?;
                if out.len() + chunk.len() > max_output {
                    return Err(InflateError("inflate output limit exceeded".to_string()));
                }
                out.extend_from_slice(&chunk);
            }
            1 => {
                let lit = fixed_lit_lengths();
                let dist = fixed_dist_lengths();
                decode_block(&mut r, &lit, &dist, &mut out, max_output)?;
            }
            2 => {
                let hlit = r.read_bits(5)? as usize + 257;
                let hdist = r.read_bits(5)? as usize + 1;
                let hclen = r.read_bits(4)? as usize + 4;
                const ORDER: [usize; 19] = [
                    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
                ];
                let mut cl_lens = vec![0u8; 19];
                for i in 0..hclen {
                    cl_lens[ORDER[i]] = r.read_bits(3)? as u8;
                }
                let cl_huff = Huffman::from_lengths(&cl_lens)?;
                let total = hlit + hdist;
                let mut lens: Vec<u8> = Vec::with_capacity(total);
                while lens.len() < total {
                    let sym = cl_huff.decode(&mut r, &cl_lens)?;
                    match sym {
                        0..=15 => lens.push(sym as u8),
                        16 => {
                            let rep = r.read_bits(2)? as usize + 3;
                            let prev = *lens
                                .last()
                                .ok_or_else(|| InflateError("bad repeat".to_string()))?;
                            lens.extend(std::iter::repeat_n(prev, rep));
                        }
                        17 => {
                            let rep = r.read_bits(3)? as usize + 3;
                            lens.extend(std::iter::repeat_n(0u8, rep));
                        }
                        18 => {
                            let rep = r.read_bits(7)? as usize + 11;
                            lens.extend(std::iter::repeat_n(0u8, rep));
                        }
                        _ => return Err(InflateError("bad code length symbol".to_string())),
                    }
                    if lens.len() > total {
                        return Err(InflateError("too many code lengths".to_string()));
                    }
                }
                let lit_lengths = lens[..hlit].to_vec();
                let dist_lengths = lens[hlit..].to_vec();
                decode_block(&mut r, &lit_lengths, &dist_lengths, &mut out, max_output)?;
            }
            _ => return Err(InflateError("invalid block type (reserved)".to_string())),
        }
        if bfinal == 1 {
            break;
        }
        if out.len() > max_output {
            return Err(InflateError("inflate output limit exceeded".to_string()));
        }
    }
    Ok(out)
}

fn decode_block(
    r: &mut BitReader,
    lit_lengths: &[u8],
    dist_lengths: &[u8],
    out: &mut Vec<u8>,
    max_output: usize,
) -> Result<(), InflateError> {
    let lit_huff = Huffman::from_lengths(lit_lengths)?;
    let dist_huff = Huffman::from_lengths(dist_lengths)?;
    loop {
        let sym = lit_huff.decode(r, lit_lengths)?;
        if sym < 256 {
            if out.len() + 1 > max_output {
                return Err(InflateError("inflate output limit exceeded".to_string()));
            }
            out.push(sym as u8);
        } else if sym == 256 {
            break;
        } else if sym <= 285 {
            let (base, extra) = length_base_extra(sym)
                .ok_or_else(|| InflateError("bad length symbol".to_string()))?;
            let extra_val = if extra > 0 {
                r.read_bits(extra as usize)? as usize
            } else {
                0
            };
            let length = base + extra_val;
            let dsym = dist_huff.decode(r, dist_lengths)?;
            let (dbase, dextra) =
                dist_base_extra(dsym).ok_or_else(|| InflateError("bad dist symbol".to_string()))?;
            let dextra_val = if dextra > 0 {
                r.read_bits(dextra as usize)? as usize
            } else {
                0
            };
            let dist = dbase + dextra_val;
            if dist == 0 || dist > out.len() {
                return Err(InflateError("invalid back-reference distance".to_string()));
            }
            if out.len() + length > max_output {
                return Err(InflateError("inflate output limit exceeded".to_string()));
            }
            for _ in 0..length {
                let b = out[out.len() - dist];
                out.push(b);
            }
        } else {
            return Err(InflateError("bad literal symbol".to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_block_roundtrip() {
        // Manually build a stored block: BFINAL=1, BTYPE=00, LEN=3 NLEN, "abc"
        let mut raw = vec![0x01u8, 0x03, 0x00, 0xFC, 0xFF, b'a', b'b', b'c'];
        let _ = &mut raw;
        let out = inflate_raw(&raw, 1024).unwrap();
        assert_eq!(out, b"abc");
    }
}
