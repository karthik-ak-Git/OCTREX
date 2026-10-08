//! Phase 15: Document Intelligence & Artifact Pipeline — canonical domain types.
//!
//! Documents are UNTRUSTED DATA. They can never change policy, permissions,
//! privacy classification decisions, routing mode, capabilities, or any other
//! security decision. All filesystem access goes through
//! `FilesystemSecurityService`. Classification is monotonic (may only rise).

use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::privacy::PrivacyClassification;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl DocumentId {
    pub fn new() -> Self {
        Self(format!("doc-{}", uuid::Uuid::new_v4().simple()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DocumentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DocumentId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentVersionId(pub String);

impl DocumentVersionId {
    pub fn new() -> Self {
        Self(format!("docver-{}", uuid::Uuid::new_v4().simple()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for DocumentVersionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DocumentVersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DocumentVersionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DocumentVersionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentChunkId(pub String);

impl DocumentChunkId {
    pub fn new() -> Self {
        Self(format!("dchunk-{}", uuid::Uuid::new_v4().simple()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for DocumentChunkId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DocumentChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Supported document formats. Anything else is explicitly unsupported —
/// never guessed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentFormat {
    Txt,
    Markdown,
    Json,
    Csv,
    Xml,
    Pdf,
    Docx,
    Xlsx,
    Unsupported { extension: String },
}

impl DocumentFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().trim_start_matches('.') {
            "txt" | "text" | "log" => DocumentFormat::Txt,
            "md" | "markdown" | "mdown" => DocumentFormat::Markdown,
            "json" | "jsonl" => DocumentFormat::Json,
            "csv" => DocumentFormat::Csv,
            "xml" | "svg" => DocumentFormat::Xml,
            "pdf" => DocumentFormat::Pdf,
            "docx" => DocumentFormat::Docx,
            "xlsx" => DocumentFormat::Xlsx,
            other => DocumentFormat::Unsupported {
                extension: other.to_string(),
            },
        }
    }

    pub fn is_supported(&self) -> bool {
        !matches!(self, DocumentFormat::Unsupported { .. })
    }

    pub fn as_str(&self) -> String {
        match self {
            DocumentFormat::Txt => "TXT".to_string(),
            DocumentFormat::Markdown => "MARKDOWN".to_string(),
            DocumentFormat::Json => "JSON".to_string(),
            DocumentFormat::Csv => "CSV".to_string(),
            DocumentFormat::Xml => "XML".to_string(),
            DocumentFormat::Pdf => "PDF".to_string(),
            DocumentFormat::Docx => "DOCX".to_string(),
            DocumentFormat::Xlsx => "XLSX".to_string(),
            DocumentFormat::Unsupported { extension } => format!("UNSUPPORTED:{}", extension),
        }
    }

    pub fn mime(&self) -> &'static str {
        match self {
            DocumentFormat::Txt => "text/plain",
            DocumentFormat::Markdown => "text/markdown",
            DocumentFormat::Json => "application/json",
            DocumentFormat::Csv => "text/csv",
            DocumentFormat::Xml => "application/xml",
            DocumentFormat::Pdf => "application/pdf",
            DocumentFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            DocumentFormat::Xlsx => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            DocumentFormat::Unsupported { .. } => "application/octet-stream",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSource {
    pub workspace_id: WorkspaceId,
    pub rel_path: String,
    pub resolved_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProvenance {
    pub document_id: String,
    pub version_id: String,
    pub workspace_id: String,
    pub source_path: String,
    pub content_hash: String,
    pub imported_at: u64,
    pub importer_task: Option<String>,
    pub importer_session: Option<String>,
    /// Documents are always untrusted file content.
    pub trust: String,
}

impl DocumentProvenance {
    pub fn describe(&self) -> String {
        format!(
            "document:{} version:{} workspace:{} path:{} hash:{}",
            self.document_id,
            self.version_id,
            self.workspace_id,
            self.source_path,
            self.content_hash
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentExtractionStatus {
    Pending,
    Parsing,
    Completed,
    UnsupportedFormat,
    Failed,
}

impl fmt::Display for DocumentExtractionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentExtractionStatus::Pending => write!(f, "PENDING"),
            DocumentExtractionStatus::Parsing => write!(f, "PARSING"),
            DocumentExtractionStatus::Completed => write!(f, "COMPLETED"),
            DocumentExtractionStatus::UnsupportedFormat => write!(f, "UNSUPPORTED_FORMAT"),
            DocumentExtractionStatus::Failed => write!(f, "FAILED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentParseWarning {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSecurityFinding {
    pub category: String,
    pub severity: String,
    pub summary: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: DocumentId,
    pub version_id: DocumentVersionId,
    pub workspace_id: WorkspaceId,
    pub rel_path: String,
    pub file_name: String,
    pub format: DocumentFormat,
    pub mime: String,
    pub size_bytes: u64,
    pub content_hash: String,
    pub classification: PrivacyClassification,
    pub extraction_status: DocumentExtractionStatus,
    pub title: Option<String>,
    pub warnings: Vec<DocumentParseWarning>,
    pub findings: Vec<DocumentSecurityFinding>,
    pub provenance: DocumentProvenance,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentBlockKind {
    Paragraph,
    Heading,
    Table,
    TableRow,
    TableCell,
    Code,
    ListItem,
    JsonValue,
    CsvRow,
    CsvHeader,
    XmlElement,
    PdfPage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentBlock {
    pub id: String,
    pub kind: DocumentBlockKind,
    pub text: String,
    pub page: Option<u32>,
    pub section_id: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub source_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    pub id: String,
    pub title: String,
    pub level: u32,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub block_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPage {
    pub number: u32,
    pub text: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedDocument {
    pub document_id: DocumentId,
    pub version_id: DocumentVersionId,
    pub title: Option<String>,
    pub source: DocumentSource,
    pub format: DocumentFormat,
    pub sections: Vec<DocumentSection>,
    pub pages: Vec<DocumentPage>,
    pub blocks: Vec<DocumentBlock>,
    pub full_text: String,
    pub metadata: DocumentMetadata,
    pub classification: PrivacyClassification,
    pub provenance: DocumentProvenance,
    pub warnings: Vec<DocumentParseWarning>,
    pub findings: Vec<DocumentSecurityFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: DocumentChunkId,
    pub document_id: DocumentId,
    pub version_id: DocumentVersionId,
    pub chunk_index: u32,
    pub text: String,
    pub char_count: usize,
    pub token_estimate: usize,
    pub token_kind: String,
    pub classification: PrivacyClassification,
    pub provenance: DocumentProvenance,
    pub workspace_id: WorkspaceId,
    pub session_id: Option<SessionId>,
    pub task_id: Option<TaskId>,
    pub section: Option<String>,
    pub page: Option<u32>,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRetrievalResult {
    pub chunk_id: String,
    pub document_id: String,
    pub version_id: String,
    pub relevance: f64,
    pub text: String,
    pub source: String,
    pub page: Option<u32>,
    pub section: Option<String>,
    pub classification: PrivacyClassification,
    pub provenance: DocumentProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactKind {
    GeneratedDocument,
    GeneratedText,
    GeneratedJson,
    GeneratedCsv,
    GeneratedMarkdown,
    GeneratedCode,
    GeneratedReport,
    TransformedDocument,
    ExtractedDataset,
}

impl ArtifactKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactKind::GeneratedDocument => "GENERATED_DOCUMENT",
            ArtifactKind::GeneratedText => "GENERATED_TEXT",
            ArtifactKind::GeneratedJson => "GENERATED_JSON",
            ArtifactKind::GeneratedCsv => "GENERATED_CSV",
            ArtifactKind::GeneratedMarkdown => "GENERATED_MARKDOWN",
            ArtifactKind::GeneratedCode => "GENERATED_CODE",
            ArtifactKind::GeneratedReport => "GENERATED_REPORT",
            ArtifactKind::TransformedDocument => "TRANSFORMED_DOCUMENT",
            ArtifactKind::ExtractedDataset => "EXTRACTED_DATASET",
        }
    }

    /// Parse a user/supplied label into a kind. Named `parse_label` (not
    /// `from_str`) to avoid confusion with `std::str::FromStr::from_str`.
    pub fn parse_label(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GENERATED_DOCUMENT" | "DOCUMENT" => Some(ArtifactKind::GeneratedDocument),
            "GENERATED_TEXT" | "TEXT" => Some(ArtifactKind::GeneratedText),
            "GENERATED_JSON" | "JSON" => Some(ArtifactKind::GeneratedJson),
            "GENERATED_CSV" | "CSV" => Some(ArtifactKind::GeneratedCsv),
            "GENERATED_MARKDOWN" | "MARKDOWN" | "MD" => Some(ArtifactKind::GeneratedMarkdown),
            "GENERATED_CODE" | "CODE" => Some(ArtifactKind::GeneratedCode),
            "GENERATED_REPORT" | "REPORT" => Some(ArtifactKind::GeneratedReport),
            "TRANSFORMED_DOCUMENT" | "TRANSFORMED" => Some(ArtifactKind::TransformedDocument),
            "EXTRACTED_DATASET" | "DATASET" => Some(ArtifactKind::ExtractedDataset),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactLineageNode {
    pub artifact_id: String,
    pub parent_artifact_id: Option<String>,
    pub source_document_id: Option<String>,
    pub source_document_version: Option<String>,
    pub producing_workflow: Option<String>,
    pub producing_skill: Option<String>,
    pub producing_model: Option<String>,
    pub producing_provider: Option<String>,
    pub classification: PrivacyClassification,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    pub text: String,
    pub truncated: bool,
    pub total_chars: usize,
}

/// Raise classification monotonically: never silently lower.
pub fn raise_classification(
    current: PrivacyClassification,
    candidate: PrivacyClassification,
) -> PrivacyClassification {
    if candidate > current {
        candidate
    } else {
        current
    }
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// FNV-1a 64 hex content hash (dependency-free, deterministic).
/// Documented as non-cryptographic dedup hash; verification checksums
/// for artifacts reuse the same function for consistency.
pub fn content_hash_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut h = FNV_OFFSET;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    format!("fnv1a64-{:016x}-len{}", h, bytes.len())
}

pub fn file_name_of(rel_path: &str) -> String {
    rel_path
        .replace('\\', "/")
        .rsplit('/')
        .next()
        .unwrap_or(rel_path)
        .to_string()
}

pub fn extension_of(rel_path: &str) -> String {
    let name = file_name_of(rel_path);
    match name.rfind('.') {
        Some(i) if i + 1 < name.len() => name[i + 1..].to_lowercase(),
        _ => String::new(),
    }
}
