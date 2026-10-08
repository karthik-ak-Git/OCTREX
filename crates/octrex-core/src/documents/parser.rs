//! Safe format-specific document parsers (dependency-free except serde_json/regex).
//!
//! Never executes document content. Unsupported formats return a structured
//! `UnsupportedFormat` outcome — never a guess.

use crate::documents::types::{
    DocumentBlock, DocumentBlockKind, DocumentFormat, DocumentPage, DocumentParseWarning,
    DocumentSection, DocumentSecurityFinding,
};
use crate::documents::zip as zipmod;

const MAX_BLOCKS: usize = 5000;
const MAX_FULL_CHARS: usize = 500_000;
const MAX_ZIP_OUTPUT: usize = 20 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: Option<String>,
    pub sections: Vec<DocumentSection>,
    pub pages: Vec<DocumentPage>,
    pub blocks: Vec<DocumentBlock>,
    pub full_text: String,
    pub warnings: Vec<DocumentParseWarning>,
}

#[derive(Debug, Clone)]
pub enum ParseOutcome {
    Parsed(ParsedDocument),
    UnsupportedFormat {
        extension: String,
        reason: String,
    },
    Failed {
        reason: String,
        warnings: Vec<DocumentParseWarning>,
    },
}

fn warn(code: &str, msg: impl Into<String>, loc: Option<String>) -> DocumentParseWarning {
    DocumentParseWarning {
        code: code.to_string(),
        message: msg.into(),
        location: loc,
    }
}

fn truncate_text(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...[truncated {} chars]", &s[..max], s.len() - max)
    }
}

#[allow(clippy::too_many_arguments)]
fn push_block(
    blocks: &mut Vec<DocumentBlock>,
    full: &mut String,
    kind: DocumentBlockKind,
    text: String,
    page: Option<u32>,
    section_id: Option<String>,
    line_start: Option<u32>,
    line_end: Option<u32>,
    source_ref: String,
) {
    if blocks.len() >= MAX_BLOCKS {
        return;
    }
    if text.trim().is_empty() {
        return;
    }
    let t = truncate_text(&text, 20_000);
    if full.len() < MAX_FULL_CHARS {
        if !full.is_empty() {
            full.push('\n');
        }
        let room = MAX_FULL_CHARS - full.len();
        full.push_str(&t[..t.len().min(room)]);
    }
    let id = format!("blk-{}", blocks.len() + 1);
    blocks.push(DocumentBlock {
        id,
        kind,
        text: t,
        page,
        section_id,
        line_start,
        line_end,
        source_ref,
    });
}

/// Scan text for prompt-injection / credential / policy-override signals.
/// Returns findings WITHOUT echoing secret values.
pub fn scan_text_for_findings(
    text: &str,
    location: Option<String>,
) -> Vec<DocumentSecurityFinding> {
    let mut out = Vec::new();
    let lower = text.to_lowercase();
    let injection_markers = [
        "ignore all previous instructions",
        "ignore previous instructions",
        "ignore all octrex policies",
        "send this file to the cloud",
        "send this document to online",
        "disable security",
        "bypass security",
        "use this credential",
        "always approve this operation",
        "cloud use is approved",
        "always permitted",
        "cloud access is always",
    ];
    for m in injection_markers {
        if lower.contains(m) {
            out.push(DocumentSecurityFinding {
                category: "PROMPT_INJECTION".to_string(),
                severity: "WARNING".to_string(),
                summary: format!(
                    "Document contains instruction-like text ('{}'); treated as untrusted content, never as policy",
                    m
                ),
                location: location.clone(),
            });
            break;
        }
    }
    // Credential-like signals (no values echoed).
    let cred_markers = [
        "api_key",
        "api-key",
        "bearer ",
        "-----begin",
        "password",
        "aws_secret_access_key",
        "sk-",
    ];
    for m in cred_markers {
        if lower.contains(m) {
            out.push(DocumentSecurityFinding {
                category: "CREDENTIAL".to_string(),
                severity: "CRITICAL".to_string(),
                summary:
                    "Credential-like secret indicator detected; values redacted from logs/UI/audit"
                        .to_string(),
                location: location.clone(),
            });
            break;
        }
    }
    if lower.contains("top secret") || lower.contains("do not distribute") {
        out.push(DocumentSecurityFinding {
            category: "SENSITIVE_PATTERN".to_string(),
            severity: "HIGH".to_string(),
            summary: "Restricted-distribution marker detected".to_string(),
            location: location.clone(),
        });
    }
    out
}

pub fn parse_bytes(format: &DocumentFormat, bytes: &[u8], rel_path: &str) -> ParseOutcome {
    match format {
        DocumentFormat::Txt => parse_txt(bytes, rel_path),
        DocumentFormat::Markdown => parse_markdown(bytes, rel_path),
        DocumentFormat::Json => parse_json(bytes, rel_path),
        DocumentFormat::Csv => parse_csv(bytes, rel_path),
        DocumentFormat::Xml => parse_xml(bytes, rel_path),
        DocumentFormat::Pdf => parse_pdf(bytes, rel_path),
        DocumentFormat::Docx => parse_docx(bytes, rel_path),
        DocumentFormat::Xlsx => parse_xlsx(bytes, rel_path),
        DocumentFormat::Unsupported { extension } => ParseOutcome::UnsupportedFormat {
            extension: extension.clone(),
            reason: format!(
                "Extension '.{}' is not a supported document format; refusing to guess",
                extension
            ),
        },
    }
}

fn decode_lossy_bounded(bytes: &[u8], rel_path: &str) -> (String, Vec<DocumentParseWarning>) {
    let mut warnings = Vec::new();
    // Reject NUL-heavy binary masquerading as text.
    let nul_count = bytes.iter().filter(|&&b| b == 0).count();
    if !bytes.is_empty() && nul_count * 4 > bytes.len() {
        warnings.push(warn(
            "BINARY_CONTENT",
            "Content looks binary (high NUL ratio); decoded lossily with placeholders",
            Some(rel_path.to_string()),
        ));
    }
    (String::from_utf8_lossy(bytes).to_string(), warnings)
}

// ---------------- TXT ----------------

fn parse_txt(bytes: &[u8], rel_path: &str) -> ParseOutcome {
    let (text, mut warnings) = decode_lossy_bounded(bytes, rel_path);
    let mut blocks = Vec::new();
    let mut full = String::new();
    let mut para: Vec<String> = Vec::new();
    let mut line_no: u32 = 0;
    let mut para_start: u32 = 1;
    for line in text.lines() {
        line_no += 1;
        if line.trim().is_empty() {
            if !para.is_empty() {
                let t = para.join("\n");
                push_block(
                    &mut blocks,
                    &mut full,
                    DocumentBlockKind::Paragraph,
                    t,
                    None,
                    None,
                    Some(para_start),
                    Some(line_no),
                    format!("{}:L{}-L{}", rel_path, para_start, line_no),
                );
                para.clear();
            }
            para_start = line_no + 1;
        } else {
            if para.is_empty() {
                para_start = line_no;
            }
            para.push(line.to_string());
        }
    }
    if !para.is_empty() {
        let t = para.join("\n");
        push_block(
            &mut blocks,
            &mut full,
            DocumentBlockKind::Paragraph,
            t,
            None,
            None,
            Some(para_start),
            Some(line_no),
            format!("{}:L{}-L{}", rel_path, para_start, line_no),
        );
    }
    if blocks.is_empty() && !text.trim().is_empty() {
        push_block(
            &mut blocks,
            &mut full,
            DocumentBlockKind::Paragraph,
            text.trim().to_string(),
            None,
            None,
            Some(1),
            Some(line_no.max(1)),
            format!("{}:L1", rel_path),
        );
    }
    let _ = &mut warnings;
    ParseOutcome::Parsed(ParsedDocument {
        title: None,
        sections: Vec::new(),
        pages: Vec::new(),
        blocks,
        full_text: full,
        warnings,
    })
}

// ---------------- Markdown ----------------

fn parse_markdown(bytes: &[u8], rel_path: &str) -> ParseOutcome {
    let (text, warnings) = decode_lossy_bounded(bytes, rel_path);
    let mut blocks = Vec::new();
    let mut full = String::new();
    let mut sections: Vec<DocumentSection> = Vec::new();
    let mut current_section: Option<String> = None;
    let mut in_code = false;
    let mut code_buf: Vec<String> = Vec::new();
    let mut code_start: u32 = 0;
    let mut para_buf: Vec<String> = Vec::new();
    let mut para_start: u32 = 1;
    let mut line_no: u32 = 0;

    let flush_para = |blocks: &mut Vec<DocumentBlock>,
                      full: &mut String,
                      buf: &mut Vec<String>,
                      start: u32,
                      end: u32,
                      sec: &Option<String>| {
        if buf.is_empty() {
            return;
        }
        let t = buf.join("\n");
        push_block(
            blocks,
            full,
            DocumentBlockKind::Paragraph,
            t,
            None,
            sec.clone(),
            Some(start),
            Some(end),
            format!("{}:L{}-L{}", rel_path, start, end),
        );
        buf.clear();
    };

    for line in text.lines() {
        line_no += 1;
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_code {
                in_code = false;
                let t = code_buf.join("\n");
                push_block(
                    &mut blocks,
                    &mut full,
                    DocumentBlockKind::Code,
                    t,
                    None,
                    current_section.clone(),
                    Some(code_start),
                    Some(line_no),
                    format!("{}:L{}-L{}", rel_path, code_start, line_no),
                );
                code_buf.clear();
            } else {
                flush_para(
                    &mut blocks,
                    &mut full,
                    &mut para_buf,
                    para_start,
                    line_no.saturating_sub(1),
                    &current_section,
                );
                in_code = true;
                code_start = line_no;
            }
            continue;
        }
        if in_code {
            code_buf.push(line.to_string());
            continue;
        }
        if trimmed.starts_with('#') {
            flush_para(
                &mut blocks,
                &mut full,
                &mut para_buf,
                para_start,
                line_no.saturating_sub(1),
                &current_section,
            );
            let level = trimmed.chars().take_while(|&c| c == '#').count().min(6) as u32;
            let title = trimmed.trim_start_matches('#').trim().to_string();
            let sec_id = format!("sec-{}", sections.len() + 1);
            // close previous section line_end
            if let Some(last) = sections.last_mut() {
                if last.line_end.is_none() {
                    last.line_end = Some(line_no.saturating_sub(1));
                }
            }
            sections.push(DocumentSection {
                id: sec_id.clone(),
                title: title.clone(),
                level,
                line_start: Some(line_no),
                line_end: None,
                block_ids: Vec::new(),
            });
            current_section = Some(sec_id.clone());
            push_block(
                &mut blocks,
                &mut full,
                DocumentBlockKind::Heading,
                title,
                None,
                Some(sec_id),
                Some(line_no),
                Some(line_no),
                format!("{}:L{}", rel_path, line_no),
            );
            para_start = line_no + 1;
            continue;
        }
        if trimmed.is_empty() {
            flush_para(
                &mut blocks,
                &mut full,
                &mut para_buf,
                para_start,
                line_no,
                &current_section,
            );
            para_start = line_no + 1;
            continue;
        }
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            flush_para(
                &mut blocks,
                &mut full,
                &mut para_buf,
                para_start,
                line_no.saturating_sub(1),
                &current_section,
            );
            push_block(
                &mut blocks,
                &mut full,
                DocumentBlockKind::ListItem,
                trimmed[2..].to_string(),
                None,
                current_section.clone(),
                Some(line_no),
                Some(line_no),
                format!("{}:L{}", rel_path, line_no),
            );
            para_start = line_no + 1;
            continue;
        }
        if para_buf.is_empty() {
            para_start = line_no;
        }
        para_buf.push(line.to_string());
    }
    flush_para(
        &mut blocks,
        &mut full,
        &mut para_buf,
        para_start,
        line_no,
        &current_section,
    );
    if in_code && !code_buf.is_empty() {
        push_block(
            &mut blocks,
            &mut full,
            DocumentBlockKind::Code,
            code_buf.join("\n"),
            None,
            current_section.clone(),
            Some(code_start),
            Some(line_no),
            format!("{}:L{}-L{}", rel_path, code_start, line_no),
        );
    }
    if let Some(last) = sections.last_mut() {
        if last.line_end.is_none() {
            last.line_end = Some(line_no);
        }
    }
    // link block ids to sections
    for s in sections.iter_mut() {
        s.block_ids = blocks
            .iter()
            .filter(|b| b.section_id.as_ref() == Some(&s.id))
            .map(|b| b.id.clone())
            .collect();
    }
    let title = sections.first().map(|s| s.title.clone());
    ParseOutcome::Parsed(ParsedDocument {
        title,
        sections,
        pages: Vec::new(),
        blocks,
        full_text: full,
        warnings,
    })
}

// ---------------- JSON ----------------

fn parse_json(bytes: &[u8], rel_path: &str) -> ParseOutcome {
    let text = String::from_utf8_lossy(bytes).to_string();
    let v: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            return ParseOutcome::Failed {
                reason: format!("Invalid JSON: {}", e),
                warnings: vec![warn(
                    "JSON_PARSE_ERROR",
                    format!("Invalid JSON: {}", e),
                    Some(rel_path.to_string()),
                )],
            };
        }
    };
    let mut blocks = Vec::new();
    let mut full = String::new();
    let warnings = Vec::new();
    match &v {
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                let t = match val {
                    serde_json::Value::String(s) => s.clone(),
                    other => serde_json::to_string_pretty(other).unwrap_or_default(),
                };
                push_block(
                    &mut blocks,
                    &mut full,
                    DocumentBlockKind::JsonValue,
                    format!("{}: {}", k, truncate_text(&t, 8000)),
                    None,
                    None,
                    None,
                    None,
                    format!("{}#{}", rel_path, k),
                );
                if blocks.len() >= MAX_BLOCKS {
                    break;
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let t = match val {
                    serde_json::Value::String(s) => s.clone(),
                    other => serde_json::to_string_pretty(other).unwrap_or_default(),
                };
                push_block(
                    &mut blocks,
                    &mut full,
                    DocumentBlockKind::JsonValue,
                    truncate_text(&t, 8000),
                    None,
                    None,
                    None,
                    None,
                    format!("{}#[{}]", rel_path, i),
                );
                if blocks.len() >= MAX_BLOCKS {
                    break;
                }
            }
        }
        _ => {
            push_block(
                &mut blocks,
                &mut full,
                DocumentBlockKind::JsonValue,
                truncate_text(&text, 20_000),
                None,
                None,
                None,
                None,
                rel_path.to_string(),
            );
        }
    }
    ParseOutcome::Parsed(ParsedDocument {
        title: None,
        sections: Vec::new(),
        pages: Vec::new(),
        blocks,
        full_text: full,
        warnings,
    })
}

// ---------------- CSV ----------------

fn split_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            if in_q && chars.peek() == Some(&'"') {
                cur.push('"');
                chars.next();
            } else {
                in_q = !in_q;
            }
        } else if c == ',' && !in_q {
            out.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    out.push(cur.trim().to_string());
    out
}

fn parse_csv(bytes: &[u8], rel_path: &str) -> ParseOutcome {
    let (text, mut warnings) = decode_lossy_bounded(bytes, rel_path);
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return ParseOutcome::Parsed(ParsedDocument {
            title: None,
            sections: Vec::new(),
            pages: Vec::new(),
            blocks: Vec::new(),
            full_text: String::new(),
            warnings,
        });
    }
    let headers = split_csv_line(lines[0]);
    let mut blocks = Vec::new();
    let mut full = String::new();
    push_block(
        &mut blocks,
        &mut full,
        DocumentBlockKind::CsvHeader,
        headers.join(" | "),
        None,
        None,
        Some(1),
        Some(1),
        format!("{}:L1", rel_path),
    );
    // column-count consistency warning
    for (idx, line) in lines.iter().skip(1).enumerate() {
        let lineno = (idx + 2) as u32;
        if line.trim().is_empty() {
            continue;
        }
        let cols = split_csv_line(line);
        if cols.len() != headers.len() {
            warnings.push(warn(
                "CSV_RAGGED_ROW",
                format!(
                    "Row {} has {} columns, expected {}",
                    lineno,
                    cols.len(),
                    headers.len()
                ),
                Some(format!("{}:L{}", rel_path, lineno)),
            ));
        }
        let row_text = headers
            .iter()
            .zip(cols.iter().chain(std::iter::repeat(&String::new())))
            .take(headers.len().max(cols.len()))
            .map(|(h, c)| format!("{}={}", h, c))
            .collect::<Vec<_>>()
            .join("; ");
        push_block(
            &mut blocks,
            &mut full,
            DocumentBlockKind::CsvRow,
            row_text,
            None,
            None,
            Some(lineno),
            Some(lineno),
            format!("{}:L{}", rel_path, lineno),
        );
        if blocks.len() >= MAX_BLOCKS {
            warnings.push(warn(
                "TRUNCATED",
                format!("CSV truncated at {} blocks", MAX_BLOCKS),
                Some(rel_path.to_string()),
            ));
            break;
        }
    }
    ParseOutcome::Parsed(ParsedDocument {
        title: Some(format!("CSV: {} ({} cols)", rel_path, headers.len())),
        sections: Vec::new(),
        pages: Vec::new(),
        blocks,
        full_text: full,
        warnings,
    })
}

// ---------------- XML (safe, no DTD/entities) ----------------

fn parse_xml(bytes: &[u8], rel_path: &str) -> ParseOutcome {
    let (text, mut warnings) = decode_lossy_bounded(bytes, rel_path);
    let upper = text.to_uppercase();
    for dangerous in ["<!DOCTYPE", "<!ENTITY"] {
        if upper.contains(dangerous) {
            return ParseOutcome::Failed {
                reason: "XML with DOCTYPE/ENTITY declarations is rejected (XXE protection)"
                    .to_string(),
                warnings: vec![warn(
                    "XML_XXE_BLOCKED",
                    "DOCTYPE/ENTITY declarations rejected",
                    Some(rel_path.to_string()),
                )],
            };
        }
    }
    // Simple element text extraction: collect text between tags.
    let mut blocks = Vec::new();
    let mut full = String::new();
    let mut buf = String::new();
    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut elem_stack: Vec<String> = Vec::new();
    for c in text.chars() {
        if c == '<' {
            let t = buf.trim().to_string();
            if !t.is_empty() {
                let path = if elem_stack.is_empty() {
                    rel_path.to_string()
                } else {
                    format!("{}#{}", rel_path, elem_stack.join("/"))
                };
                push_block(
                    &mut blocks,
                    &mut full,
                    DocumentBlockKind::XmlElement,
                    t,
                    None,
                    None,
                    None,
                    None,
                    path,
                );
            }
            buf.clear();
            in_tag = true;
            tag_buf.clear();
        } else if c == '>' && in_tag {
            in_tag = false;
            let tag = tag_buf.trim().to_string();
            if tag.starts_with('/') {
                elem_stack.pop();
            } else if tag.ends_with('/') {
                // self-closing
            } else if !tag.starts_with('?') && !tag.starts_with('!') {
                let name = tag.split_whitespace().next().unwrap_or("").to_string();
                if !name.is_empty() && elem_stack.len() < 32 {
                    elem_stack.push(name);
                }
            }
            tag_buf.clear();
            if blocks.len() >= MAX_BLOCKS {
                warnings.push(warn(
                    "TRUNCATED",
                    "XML truncated",
                    Some(rel_path.to_string()),
                ));
                break;
            }
        } else if in_tag {
            tag_buf.push(c);
        } else {
            buf.push(c);
        }
    }
    let tail = buf.trim().to_string();
    if !tail.is_empty() {
        push_block(
            &mut blocks,
            &mut full,
            DocumentBlockKind::XmlElement,
            tail,
            None,
            None,
            None,
            None,
            rel_path.to_string(),
        );
    }
    ParseOutcome::Parsed(ParsedDocument {
        title: None,
        sections: Vec::new(),
        pages: Vec::new(),
        blocks,
        full_text: full,
        warnings,
    })
}

// ---------------- PDF (heuristic, dependency-free) ----------------

fn extract_pdf_strings(data: &[u8]) -> (Vec<String>, Vec<String>) {
    // Returns (literal_texts, warnings)
    let mut texts = Vec::new();
    let mut warnings = Vec::new();
    // Fast path: find parenthesized strings with escape handling.
    let mut i = 0usize;
    let mut found_any = false;
    while i < data.len() {
        if data[i] == b'(' {
            found_any = true;
            let mut s = String::new();
            let mut depth = 1usize;
            i += 1;
            let mut guard = 0usize;
            while i < data.len() && depth > 0 && guard < 100_000 {
                guard += 1;
                let b = data[i];
                if b == b'\\' && i + 1 < data.len() {
                    let n = data[i + 1];
                    match n {
                        b'n' => s.push('\n'),
                        b'r' => s.push('\r'),
                        b't' => s.push('\t'),
                        b'(' | b')' | b'\\' => s.push(n as char),
                        b'\r' | b'\n' => {}
                        d if d.is_ascii_digit() => {
                            // octal escape: up to 3 digits
                            let mut val: u32 = 0;
                            let mut k = 0;
                            while k < 3
                                && i + 1 + k < data.len()
                                && data[i + 1 + k].is_ascii_digit()
                            {
                                val = val * 8 + (data[i + 1 + k] - b'0') as u32;
                                k += 1;
                            }
                            if let Some(ch) = char::from_u32(val) {
                                s.push(ch);
                            }
                            i += k; // extra advance below handles +1
                        }
                        _ => s.push(n as char),
                    }
                    i += 2;
                } else if b == b'(' {
                    depth += 1;
                    s.push('(');
                    i += 1;
                } else if b == b')' {
                    depth -= 1;
                    if depth > 0 {
                        s.push(')');
                    }
                    i += 1;
                } else {
                    s.push(b as char);
                    i += 1;
                }
            }
            let t = s.trim().to_string();
            if t.len() > 1 && t.chars().any(|c| c.is_alphanumeric()) {
                texts.push(t);
            }
        } else {
            i += 1;
        }
        if texts.len() > 20_000 {
            warnings.push("PDF string cap reached; truncated".to_string());
            break;
        }
    }
    if !found_any {
        warnings.push(
            "No parenthesized PDF text strings found; file may be scanned/image-only or encoded"
                .to_string(),
        );
    }
    (texts, warnings)
}

fn parse_pdf(bytes: &[u8], rel_path: &str) -> ParseOutcome {
    let mut warnings: Vec<DocumentParseWarning> = Vec::new();
    if bytes.len() < 5 || &bytes[0..5] != b"%PDF-" {
        return ParseOutcome::Failed {
            reason: "Not a PDF file (missing %PDF- magic)".to_string(),
            warnings: vec![warn(
                "PDF_MAGIC",
                "Missing %PDF- header",
                Some(rel_path.to_string()),
            )],
        };
    }
    // Count pages heuristically.
    let hay = String::from_utf8_lossy(bytes);
    let mut page_count = hay.matches("/Type /Page").count() + hay.matches("/Type/Page").count();
    // subtract /Pages matches
    let pages_decl = hay.matches("/Type /Pages").count() + hay.matches("/Type/Pages").count();
    page_count = page_count.saturating_sub(pages_decl);
    if page_count == 0 {
        // fallback: count /Page occurrences bounded
        page_count = hay.matches("/Page").count().min(5000);
        if page_count == 0 {
            page_count = 1;
        }
        warnings.push(warn(
            "PDF_PAGE_HEURISTIC",
            format!("Page count heuristic used: {} page(s)", page_count),
            Some(rel_path.to_string()),
        ));
    }
    page_count = page_count.clamp(1, 5000);

    let (mut texts, str_warnings) = extract_pdf_strings(bytes);
    for w in str_warnings {
        warnings.push(warn("PDF_EXTRACT", w, Some(rel_path.to_string())));
    }

    // Try FlateDecode streams: find `stream ... endstream`, inflate, extract strings.
    let mut extra_texts: Vec<String> = Vec::new();
    let mut search_from = 0usize;
    let mut streams_inflated = 0usize;
    let mut streams_failed = 0usize;
    while search_from + 6 < bytes.len() && streams_inflated + streams_failed < 200 {
        let rel = &bytes[search_from..];
        let s_pos = match find_subslice(rel, b"stream") {
            Some(p) => p,
            None => break,
        };
        let abs_stream = search_from + s_pos;
        let e_pos = match find_subslice(&bytes[abs_stream..], b"endstream") {
            Some(p) => abs_stream + p,
            None => break,
        };
        // stream data starts after "stream" + optional \r\n
        let mut data_start = abs_stream + 6;
        if data_start < bytes.len() && bytes[data_start] == b'\r' {
            data_start += 1;
        }
        if data_start < bytes.len() && bytes[data_start] == b'\n' {
            data_start += 1;
        }
        if e_pos > data_start && e_pos - data_start < 30_000_000 {
            // Check if FlateDecode mentioned in the ~2KB before stream.
            let dict_start = abs_stream.saturating_sub(2048);
            let dict = &bytes[dict_start..abs_stream];
            if find_subslice(dict, b"FlateDecode").is_some()
                || find_subslice(dict, b"Flate").is_some()
            {
                let raw = &bytes[data_start..e_pos];
                // strip trailing CRLF
                let raw = if raw.ends_with(b"\r\n") {
                    &raw[..raw.len() - 2]
                } else if raw.ends_with(b"\n") || raw.ends_with(b"\r") {
                    &raw[..raw.len() - 1]
                } else {
                    raw
                };
                // Try zlib-wrapped (skip 2-byte header + 4-byte adler) then raw.
                let mut inflated: Option<Vec<u8>> = None;
                if raw.len() > 6 {
                    // zlib header 0x78 ... common; try skipping 2 bytes
                    if let Ok(v) = crate::documents::inflate::inflate_raw(
                        &raw[2..raw.len().saturating_sub(4)],
                        10_000_000,
                    ) {
                        if !v.is_empty() {
                            inflated = Some(v);
                        }
                    }
                }
                if inflated.is_none() {
                    if let Ok(v) = crate::documents::inflate::inflate_raw(raw, 10_000_000) {
                        inflated = Some(v);
                    }
                }
                match inflated {
                    Some(v) => {
                        streams_inflated += 1;
                        let (t, _) = extract_pdf_strings(&v);
                        extra_texts.extend(t);
                    }
                    None => streams_failed += 1,
                }
            }
        }
        search_from = e_pos + 9;
    }
    if streams_failed > 0 {
        warnings.push(warn(
            "PDF_STREAM",
            format!(
                "{} Flate stream(s) could not be decompressed; page text may be incomplete",
                streams_failed
            ),
            Some(rel_path.to_string()),
        ));
    }
    texts.extend(extra_texts);

    let full_joined = texts.join("\n");
    if full_joined.trim().is_empty() {
        warnings.push(warn(
            "PDF_NO_TEXT",
            "No extractable text found; PDF may be scanned images (OCR not available offline)",
            Some(rel_path.to_string()),
        ));
    }

    // Distribute text across pages round-robin by lines for provenance.
    // page_count is clamped to >= 1 above, so div_ceil cannot divide by zero.
    let lines: Vec<String> = full_joined.lines().map(|l| l.to_string()).collect();
    let per_page = lines.len().div_ceil(page_count).max(1);
    let mut pages: Vec<DocumentPage> = Vec::new();
    let mut blocks: Vec<DocumentBlock> = Vec::new();
    let mut full = String::new();
    for p in 1..=(page_count as u32) {
        let start = ((p as usize - 1) * per_page).min(lines.len());
        let end = (start + per_page).min(lines.len());
        let page_text = if start < end {
            lines[start..end].join("\n")
        } else {
            String::new()
        };
        pages.push(DocumentPage {
            number: p,
            text: truncate_text(&page_text, 50_000),
            warnings: Vec::new(),
        });
        if !page_text.trim().is_empty() {
            push_block(
                &mut blocks,
                &mut full,
                DocumentBlockKind::PdfPage,
                page_text,
                Some(p),
                None,
                None,
                None,
                format!("{}#page={}", rel_path, p),
            );
        }
    }
    ParseOutcome::Parsed(ParsedDocument {
        title: Some(format!("PDF: {}", rel_path)),
        sections: Vec::new(),
        pages,
        blocks,
        full_text: full,
        warnings,
    })
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

// ---------------- DOCX ----------------

fn xml_text_between(data: &str, open: &str, close: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(s) = data[from..].find(open) {
        let a = from + s + open.len();
        if let Some(e) = data[a..].find(close) {
            out.push(data[a..a + e].to_string());
            from = a + e + close.len();
        } else {
            break;
        }
        if out.len() > 20_000 {
            break;
        }
    }
    out
}

#[allow(dead_code)]
fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(c);
        }
    }
    xml_unescape(&out)
}

fn xml_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn parse_docx(bytes: &[u8], rel_path: &str) -> ParseOutcome {
    let mut warnings = Vec::new();
    if !zipmod::is_zip_magic(bytes) {
        return ParseOutcome::Failed {
            reason: "DOCX is not a ZIP/OOXML container (bad PK magic)".to_string(),
            warnings: vec![warn(
                "DOCX_MAGIC",
                "Missing ZIP magic",
                Some(rel_path.to_string()),
            )],
        };
    }
    let doc_xml = match zipmod::extract_file(bytes, "word/document.xml", MAX_ZIP_OUTPUT) {
        Ok(v) => v,
        Err(e) => {
            return ParseOutcome::Failed {
                reason: format!("DOCX missing word/document.xml: {}", e.0),
                warnings: vec![warn(
                    "DOCX_STRUCTURE",
                    format!("Missing document.xml: {}", e.0),
                    Some(rel_path.to_string()),
                )],
            };
        }
    };
    let doc_str = String::from_utf8_lossy(&doc_xml).to_string();
    if doc_str.contains("<!DOCTYPE") || doc_str.contains("<!ENTITY") {
        return ParseOutcome::Failed {
            reason: "DOCX inner XML with DOCTYPE/ENTITY rejected (XXE protection)".to_string(),
            warnings: vec![warn(
                "XML_XXE_BLOCKED",
                "DOCTYPE/ENTITY rejected",
                Some(rel_path.to_string()),
            )],
        };
    }
    let mut blocks = Vec::new();
    let mut full = String::new();
    // Tables first (to preserve structure), then paragraphs outside tables.
    // Simplest robust approach: iterate <w:p> in order; detect ancestor <w:tbl> by
    // checking surrounding context is complex — instead extract tables separately
    // and paragraphs sequentially. Provenance notes table vs paragraph.
    let paras = xml_text_between(&doc_str, "<w:p", "</w:p>");
    let mut para_idx = 0usize;
    for p in paras {
        para_idx += 1;
        // heading style?
        let kind = if p.contains("w:pStyle") {
            let style_pos = p.find("w:pStyle").unwrap_or(0);
            let snippet = &p[style_pos..style_pos.saturating_add(200).min(p.len())];
            let lower = snippet.to_lowercase();
            if lower.contains("heading") || lower.contains("title") {
                DocumentBlockKind::Heading
            } else {
                DocumentBlockKind::Paragraph
            }
        } else {
            DocumentBlockKind::Paragraph
        };
        // extract <w:t> runs
        let runs = xml_text_between(&p, "<w:t", "</w:t>");
        let mut text_parts: Vec<String> = Vec::new();
        for r in runs {
            // strip attributes prefix up to '>'
            let content = match r.find('>') {
                Some(i) => &r[i + 1..],
                None => r.as_str(),
            };
            text_parts.push(xml_unescape(content));
        }
        let text = text_parts.join("").trim().to_string();
        if text.is_empty() {
            continue;
        }
        push_block(
            &mut blocks,
            &mut full,
            kind,
            text,
            None,
            None,
            None,
            None,
            format!("{}#para{}", rel_path, para_idx),
        );
        if blocks.len() >= MAX_BLOCKS {
            warnings.push(warn(
                "TRUNCATED",
                "DOCX truncated",
                Some(rel_path.to_string()),
            ));
            break;
        }
    }
    // Tables: extract cell texts for structure preservation.
    let tbls = xml_text_between(&doc_str, "<w:tbl", "</w:tbl>");
    if !tbls.is_empty() {
        warnings.push(warn(
            "DOCX_TABLES",
            format!(
                "{} table(s) flattened into paragraph blocks with cell context",
                tbls.len()
            ),
            Some(rel_path.to_string()),
        ));
        for (ti, t) in tbls.iter().enumerate() {
            let rows = xml_text_between(t, "<w:tr", "</w:tr>");
            for (ri, r) in rows.iter().enumerate() {
                let cells = xml_text_between(r, "<w:tc", "</w:tc>");
                let mut cell_texts: Vec<String> = Vec::new();
                for c in cells {
                    let runs = xml_text_between(&c, "<w:t", "</w:t>");
                    let mut parts = Vec::new();
                    for run in runs {
                        let content = match run.find('>') {
                            Some(i) => &run[i + 1..],
                            None => run.as_str(),
                        };
                        parts.push(xml_unescape(content));
                    }
                    cell_texts.push(parts.join(""));
                }
                let row_text = cell_texts
                    .iter()
                    .enumerate()
                    .map(|(ci, v)| format!("C{}={}", ci + 1, v))
                    .collect::<Vec<_>>()
                    .join(" | ");
                if !row_text.trim().is_empty() {
                    push_block(
                        &mut blocks,
                        &mut full,
                        DocumentBlockKind::TableRow,
                        format!("Table{} Row{}: {}", ti + 1, ri + 1, row_text),
                        None,
                        None,
                        None,
                        None,
                        format!("{}#table{}/row{}", rel_path, ti + 1, ri + 1),
                    );
                }
            }
        }
    }
    if blocks.is_empty() {
        warnings.push(warn(
            "DOCX_EMPTY",
            "No paragraphs extracted",
            Some(rel_path.to_string()),
        ));
    }
    // Title: first heading or first paragraph.
    let title = blocks.first().map(|b| truncate_text(&b.text, 120));
    ParseOutcome::Parsed(ParsedDocument {
        title,
        sections: Vec::new(),
        pages: Vec::new(),
        blocks,
        full_text: full,
        warnings,
    })
}

// ---------------- XLSX ----------------

fn parse_xlsx(bytes: &[u8], rel_path: &str) -> ParseOutcome {
    let mut warnings = Vec::new();
    if !zipmod::is_zip_magic(bytes) {
        return ParseOutcome::Failed {
            reason: "XLSX is not a ZIP/OOXML container (bad PK magic)".to_string(),
            warnings: vec![warn(
                "XLSX_MAGIC",
                "Missing ZIP magic",
                Some(rel_path.to_string()),
            )],
        };
    }
    // shared strings
    let shared: Vec<String> =
        match zipmod::extract_file(bytes, "xl/sharedStrings.xml", MAX_ZIP_OUTPUT) {
            Ok(v) => {
                let s = String::from_utf8_lossy(&v).to_string();
                if s.contains("<!DOCTYPE") || s.contains("<!ENTITY") {
                    return ParseOutcome::Failed {
                        reason: "XLSX sharedStrings with DOCTYPE/ENTITY rejected".to_string(),
                        warnings: vec![warn(
                            "XML_XXE_BLOCKED",
                            "rejected",
                            Some(rel_path.to_string()),
                        )],
                    };
                }
                xml_text_between(&s, "<t>", "</t>")
                    .iter()
                    .map(|x| xml_unescape(x))
                    .collect()
            }
            Err(_) => {
                warnings.push(warn(
                    "XLSX_SHARED_STRINGS",
                    "No sharedStrings.xml; inline strings only",
                    Some(rel_path.to_string()),
                ));
                Vec::new()
            }
        };
    // workbook sheet names
    let mut sheet_names: Vec<(String, String)> = Vec::new(); // (sheetId/rId, name)
    if let Ok(v) = zipmod::extract_file(bytes, "xl/workbook.xml", MAX_ZIP_OUTPUT) {
        let s = String::from_utf8_lossy(&v).to_string();
        // <sheet name="..." sheetId="1" .../>
        let mut from = 0usize;
        while let Some(p) = s[from..].find("<sheet") {
            let a = from + p;
            let end = match s[a..].find('>') {
                Some(e) => a + e,
                None => break,
            };
            let tag = &s[a..end];
            let name = attr_value(tag, "name")
                .unwrap_or_else(|| format!("Sheet{}", sheet_names.len() + 1));
            let sid =
                attr_value(tag, "sheetId").unwrap_or_else(|| format!("{}", sheet_names.len() + 1));
            sheet_names.push((sid, name));
            from = end + 1;
        }
    }
    // worksheets
    let sheets = match zipmod::extract_prefix(bytes, "xl/worksheets/", 64, MAX_ZIP_OUTPUT) {
        Ok(v) => v,
        Err(e) => {
            return ParseOutcome::Failed {
                reason: format!("XLSX worksheets unreadable: {}", e.0),
                warnings: vec![warn("XLSX_STRUCTURE", e.0, Some(rel_path.to_string()))],
            };
        }
    };
    if sheets.is_empty() {
        warnings.push(warn(
            "XLSX_EMPTY",
            "No worksheets found",
            Some(rel_path.to_string()),
        ));
    }
    let mut blocks = Vec::new();
    let mut full = String::new();
    let mut unsupported_features = false;
    for (idx, (name, data)) in sheets.iter().enumerate() {
        let sheet_label = sheet_names
            .get(idx)
            .map(|(_, n)| n.clone())
            .unwrap_or_else(|| name.clone());
        let s = String::from_utf8_lossy(data).to_string();
        if s.contains("<!DOCTYPE") || s.contains("<!ENTITY") {
            warnings.push(warn(
                "XML_XXE_BLOCKED",
                format!("Sheet {} rejected", sheet_label),
                Some(rel_path.to_string()),
            ));
            continue;
        }
        if s.contains("<mergeCell")
            || s.contains("<hyperlinks")
            || s.contains("<drawings")
            || s.contains("<chart")
        {
            unsupported_features = true;
        }
        // rows: <row r="1" ...> ... <c r="A1" t="s"><v>3</v><f>SUM(...)</f></c> ... </row>
        let rows = xml_text_between(&s, "<row", "</row>");
        if rows.is_empty() {
            // fallback: any <c ...> cells
            warnings.push(warn(
                "XLSX_ROWS",
                format!("Sheet '{}' has no <row> elements", sheet_label),
                Some(format!("{}#{}", rel_path, sheet_label)),
            ));
            continue;
        }
        for row_xml in rows {
            // row number
            let row_num = attr_value(
                &row_xml[..row_xml.find('>').unwrap_or(0).min(row_xml.len())],
                "r",
            )
            .unwrap_or_default();
            let cells = xml_text_between(&row_xml, "<c", "</c>");
            // self-closing <c .../> cells (empty) are missed; note as warning-free skip.
            let mut parts: Vec<String> = Vec::new();
            for cell in cells {
                let head_end = cell.find('>').unwrap_or(0);
                let head = &cell[..head_end];
                let cref = attr_value(head, "r").unwrap_or_default();
                let ctype = attr_value(head, "t").unwrap_or_default();
                let inner = &cell[head_end.min(cell.len())..];
                // formula?
                let formula = if let Some(fs) = inner.find("<f") {
                    let rest = &inner[fs..];
                    if let Some(fe) = rest.find("</f>") {
                        let fbody = &rest[..fe];
                        let ftext = match fbody.find('>') {
                            Some(i) => fbody[i + 1..].to_string(),
                            None => String::new(),
                        };
                        Some(xml_unescape(&ftext))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let value_raw = if let Some(vs) = inner.find("<v>") {
                    let a = vs + 3;
                    let e = inner[a..]
                        .find("</v>")
                        .map(|e| a + e)
                        .unwrap_or(inner.len());
                    inner[a..e].to_string()
                } else {
                    String::new()
                };
                let value = if ctype == "s" {
                    value_raw
                        .parse::<usize>()
                        .ok()
                        .and_then(|i| shared.get(i).cloned())
                        .unwrap_or(value_raw)
                } else if ctype == "str" || ctype == "inlineStr" {
                    // <is><t>..</t></is>
                    let tvals = xml_text_between(inner, "<t>", "</t>");
                    if tvals.is_empty() {
                        value_raw
                    } else {
                        tvals.join("")
                    }
                } else {
                    value_raw
                };
                let col = col_of_ref(&cref);
                if value.trim().is_empty() && formula.as_deref().unwrap_or("").trim().is_empty() {
                    continue;
                }
                if let Some(f) = formula {
                    if !f.trim().is_empty() {
                        parts.push(format!(
                            "{}({})=[{}] formula:{}",
                            cref_or_col(&cref, col.clone(), &value),
                            col_label(col.clone()),
                            value,
                            f
                        ));
                    } else {
                        parts.push(format!(
                            "{}({})={}",
                            cref_or_col(&cref, col.clone(), &value),
                            col_label(col.clone()),
                            value
                        ));
                    }
                } else {
                    parts.push(format!(
                        "{}({})={}",
                        cref_or_col(&cref, col.clone(), &value),
                        col_label(col),
                        value
                    ));
                }
            }
            if parts.is_empty() {
                continue;
            }
            let row_text = format!(
                "Sheet '{}' Row{}: {}",
                sheet_label,
                row_num,
                parts.join("; ")
            );
            push_block(
                &mut blocks,
                &mut full,
                DocumentBlockKind::TableRow,
                row_text,
                None,
                None,
                None,
                None,
                format!("{}#{}:row{}", rel_path, sheet_label, row_num),
            );
            if blocks.len() >= MAX_BLOCKS {
                warnings.push(warn(
                    "TRUNCATED",
                    "XLSX truncated",
                    Some(rel_path.to_string()),
                ));
                break;
            }
        }
    }
    if unsupported_features {
        warnings.push(warn(
            "XLSX_FEATURES",
            "Unsupported workbook features present (merged cells, drawings, charts, hyperlinks); values extracted, layout may differ",
            Some(rel_path.to_string()),
        ));
    }
    ParseOutcome::Parsed(ParsedDocument {
        title: Some(format!("Workbook: {}", rel_path)),
        sections: Vec::new(),
        pages: Vec::new(),
        blocks,
        full_text: full,
        warnings,
    })
}

fn attr_value(tag: &str, attr: &str) -> Option<String> {
    // find attr="..." or attr='...'
    let pat1 = format!("{}=\"", attr);
    if let Some(p) = tag.find(&pat1) {
        let a = p + pat1.len();
        if let Some(e) = tag[a..].find('"') {
            return Some(tag[a..a + e].to_string());
        }
    }
    let pat2 = format!("{}='", attr);
    if let Some(p) = tag.find(&pat2) {
        let a = p + pat2.len();
        if let Some(e) = tag[a..].find('\'') {
            return Some(tag[a..a + e].to_string());
        }
    }
    None
}

fn col_of_ref(cref: &str) -> String {
    let mut col = String::new();
    for c in cref.chars() {
        if c.is_ascii_alphabetic() {
            col.push(c);
        } else {
            break;
        }
    }
    col
}

fn col_label(col: String) -> String {
    if col.is_empty() {
        "?".to_string()
    } else {
        col
    }
}

fn cref_or_col(cref: &str, _col: String, _v: &str) -> String {
    if cref.is_empty() {
        "cell".to_string()
    } else {
        cref.to_string()
    }
}
