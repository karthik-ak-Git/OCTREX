//! Minimal dependency-free ZIP reader for OOXML (DOCX/XLSX).
//!
//! Supports `stored` (method 0) and `deflated` (method 8) entries with
//! bounded output. Fails closed on corrupt archives, encryption,
//! multi-disk, or directory-traversal entry names.

use crate::documents::inflate::inflate_raw;

#[derive(Debug)]
pub struct ZipError(pub String);

impl std::fmt::Display for ZipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "zip error: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ZipEntry {
    pub name: String,
    pub method: u16,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub data_offset: usize,
}

fn read_u16_le(b: &[u8], off: usize) -> Option<u16> {
    if off + 2 > b.len() {
        return None;
    }
    Some(u16::from_le_bytes([b[off], b[off + 1]]))
}

fn read_u32_le(b: &[u8], off: usize) -> Option<u32> {
    if off + 4 > b.len() {
        return None;
    }
    Some(u32::from_le_bytes([
        b[off],
        b[off + 1],
        b[off + 2],
        b[off + 3],
    ]))
}

/// List local-file-header entries. Bounded scan; ignores central directory.
pub fn list_entries(data: &[u8]) -> Result<Vec<ZipEntry>, ZipError> {
    if data.len() < 4 || &data[0..2] != b"PK" {
        return Err(ZipError("not a ZIP archive (missing PK magic)".to_string()));
    }
    let mut entries = Vec::new();
    let mut off = 0usize;
    let mut guard = 0usize;
    while off + 30 <= data.len() && guard < 4096 {
        guard += 1;
        let sig = read_u32_le(data, off).unwrap_or(0);
        if sig == 0x04034b50 {
            let method = read_u16_le(data, off + 8).unwrap_or(99);
            let flags = read_u16_le(data, off + 6).unwrap_or(0);
            let comp_size = read_u32_le(data, off + 18).unwrap_or(0) as usize;
            let name_len = read_u16_le(data, off + 26).unwrap_or(0) as usize;
            let extra_len = read_u16_le(data, off + 28).unwrap_or(0) as usize;
            if flags & 0x0001 != 0 {
                return Err(ZipError(
                    "encrypted ZIP entries are not supported".to_string(),
                ));
            }
            if flags & 0x0008 != 0 {
                // data-descriptor flag: size in header unreliable; fail closed
                return Err(ZipError(
                    "ZIP data-descriptor entries are not supported".to_string(),
                ));
            }
            let name_start = off + 30;
            let name_end = name_start.saturating_add(name_len);
            let data_start = name_end.saturating_add(extra_len);
            if name_end > data.len() || data_start > data.len() {
                return Err(ZipError("truncated ZIP entry header".to_string()));
            }
            let name_bytes = &data[name_start..name_end];
            let name = String::from_utf8_lossy(name_bytes).to_string();
            if name.contains("..") || name.starts_with('/') || name.contains(":\\") {
                return Err(ZipError(format!("unsafe ZIP entry name: {}", name)));
            }
            let data_end = data_start.saturating_add(comp_size);
            if data_end > data.len() {
                return Err(ZipError("truncated ZIP entry data".to_string()));
            }
            entries.push(ZipEntry {
                name,
                method,
                compressed_size: comp_size as u32,
                uncompressed_size: read_u32_le(data, off + 22).unwrap_or(0),
                data_offset: data_start,
            });
            off = data_end;
        } else if sig == 0x02014b50 || sig == 0x06054b50 {
            break; // central directory / EOCD
        } else {
            break;
        }
        if entries.len() > 1024 {
            return Err(ZipError("too many ZIP entries".to_string()));
        }
    }
    if entries.is_empty() {
        return Err(ZipError("no readable ZIP entries".to_string()));
    }
    Ok(entries)
}

/// Extract one entry by exact name with bounded decompressed output.
pub fn extract_file(data: &[u8], name: &str, max_output: usize) -> Result<Vec<u8>, ZipError> {
    let entries = list_entries(data)?;
    let e = entries
        .iter()
        .find(|x| x.name == name)
        .ok_or_else(|| ZipError(format!("ZIP entry not found: {}", name)))?;
    let start = e.data_offset;
    let end = start + e.compressed_size as usize;
    if end > data.len() {
        return Err(ZipError("truncated entry".to_string()));
    }
    let raw = &data[start..end];
    match e.method {
        0 => {
            if raw.len() > max_output {
                return Err(ZipError("entry exceeds output bound".to_string()));
            }
            Ok(raw.to_vec())
        }
        8 => inflate_raw(raw, max_output)
            .map_err(|e| ZipError(format!("deflate failed for {}: {}", name, e.0))),
        m => Err(ZipError(format!("unsupported compression method {}", m))),
    }
}

/// Extract all entries matching a prefix (e.g. `xl/worksheets/`), bounded.
pub fn extract_prefix(
    data: &[u8],
    prefix: &str,
    max_files: usize,
    max_output_each: usize,
) -> Result<Vec<(String, Vec<u8>)>, ZipError> {
    let entries = list_entries(data)?;
    let mut out = Vec::new();
    for e in entries.iter().filter(|x| x.name.starts_with(prefix)) {
        if e.name.ends_with('/') {
            continue;
        }
        if out.len() >= max_files {
            break;
        }
        let bytes = extract_file(data, &e.name, max_output_each)?;
        out.push((e.name.clone(), bytes));
    }
    Ok(out)
}

pub fn is_zip_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 4
        && &bytes[0..2] == b"PK"
        && (bytes[2] == 0x03 || bytes[2] == 0x01)
        && bytes[3] == 0x04
        || bytes.len() >= 2 && &bytes[0..2] == b"PK"
}
