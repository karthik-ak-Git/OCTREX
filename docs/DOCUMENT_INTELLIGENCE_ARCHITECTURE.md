# Document Intelligence & Artifact Pipeline (Phase 15)

Local-first document intelligence for Octrex. Agents can discover, inspect,
extract, chunk, retrieve, and derive artifacts from workspace documents —
without ever letting untrusted document content change security policy.

```
FilesystemSecurity
  → DocumentIntake        (crates/octrex-core/src/documents/intake.rs)
  → DocumentParser        (documents/parser.rs + inflate.rs + zip.rs)
  → Classification / Security Analysis (PrivacyClassifier, monotonic)
  → NormalizedDocument    (documents/types.rs, provenance preserved)
  → Chunker               (documents/chunker.rs, existing tokenizer math)
  → ContextService        (FileContent / UntrustedFile, budget-checked)
  → Agent / Workflow / Skill (via ToolRuntime document.* tools)
  → Artifact Generation   (UNVERIFIED by default)
  → VerificationEngine    (required before claiming done)
  → Artifact Registry + lineage + audit
```

No second context system, artifact database, filesystem layer, model
runtime, router, verification engine, or orchestration loop was created.
Everything reuses Phases 6–14.

## 1. Domain (`documents/types.rs`)

Strong types: `DocumentId`, `DocumentVersionId`, `DocumentChunkId`,
`DocumentFormat` (Txt/Markdown/Json/Csv/Xml/Pdf/Docx/Xlsx/Unsupported),
`DocumentSource`, `DocumentMetadata`, `DocumentBlock` (kind + page +
section + line range + source_ref), `DocumentSection`, `DocumentPage`,
`NormalizedDocument`, `DocumentChunk` (doc/version/chunk-index/section/page/
source-path/classification/provenance/trust/workspace/session/task),
`DocumentProvenance` (always `untrusted_file`), `DocumentExtractionStatus`,
`DocumentParseWarning`, `DocumentSecurityFinding`, `ArtifactKind`
(9 generated/derived kinds), `ArtifactLineageNode`.

Classification reuses `PrivacyClassification` (PUBLIC…SECRET). There is no
competing enum. `raise_classification` is monotonic: processing may raise,
never silently lower.

Content hashes use dependency-free FNV-1a-64 hex (`content_hash_hex`),
documented as a non-cryptographic dedup hash.

## 2. Intake (`documents/intake.rs`)

`intake_document` accepts only workspace-relative paths. Absolute paths,
empty paths, and `..` are rejected before touching the boundary. The real
read goes through `FilesystemSecurityService::evaluate_operation(Read)` +
`read_file_bytes`, which enforces traversal rejection, symlink/junction/
reparse validation, protected paths, read-only policy, and size limits
(policy `max_read_bytes`, plus a hard 50 MB intake cap). Files are never
executed regardless of extension. Intake records provenance (workspace,
rel path, hash, importer task/session, trust) and MIME via extension.

## 3. Parsers (`documents/parser.rs`, `inflate.rs`, `zip.rs`)

Zero new dependencies. A minimal raw-DEFLATE decoder (RFC 1951) and a
bounded ZIP reader (stored + deflated, traversal/encryption/descriptor
rejected) support OOXML without pulling in compression/XML crates.

| Format | Implementation | Provenance kept |
|---|---|---|
| TXT | paragraph splitter | line ranges |
| Markdown | headings/sections/code/lists | section ids, line ranges |
| JSON | serde_json validated; flattened per key/index | JSON pointer-ish refs |
| CSV | quote-aware splitter; ragged-row warnings | header + L{n} per row |
| XML | safe subset; `DOCTYPE`/`ENTITY` rejected (XXE) | element paths |
| PDF | `%PDF-` gate; `(…)` strings + Flate streams; page heuristic | page numbers |
| DOCX | `word/document.xml`: paragraphs, headings, flattened tables | para/table/row refs |
| XLSX | shared strings + sheets: rows, columns, `C{col}={val}`, formulas | sheet/row refs |

Anything else returns `UnsupportedFormat` with the extension named — never
a guess. Parser failures return `Failed` with warnings and do not bypass
security. `scan_text_for_findings` tags instruction-like text
(PROMPT_INJECTION), credential indicators (CREDENTIAL, values never echoed),
and restricted markers (SENSITIVE_PATTERN).

## 4. Prompt-injection defense

Documents are UNTRUSTED DATA. `Ignore all previous instructions`, `send
this file to the cloud`, `disable security`, `always approve…`, etc. stay
document content. Enforcement points:

- Phase 6 `PrivacyClassifier` already emits `PROMPT_INJECTION_ATTEMPT`
  signals; Phase 15 adds finding records on every import.
- Phase 14 `MemoryExtractor` refuses to plant preferences from untrusted
  sources (existing poisoning defense, reused by design).
- Chunks enter context only as `FileContent`/`UntrustedFile`, which
  `ContextSecurityPolicy` forbids from `System` role or control-plane
  sources.
- `document_cannot_grant_capability_or_change_policy` test pins that
  findings never produce grants or routing authority.

## 5. Chunking (`documents/chunker.rs`)

`chunk_document` splits per block (1500 chars, 200 overlap for oversized
blocks) with token estimates using the same math as the ContextEngine
counter (chars/4 × 1.15 safety factor). No second budget algorithm: budgets
are enforced with `TokenBudget::compute` at ingest time.
`ingest_to_context` adds chunks via `ContextService::add_item` as
`FileContent` role + `UntrustedFile` trust, stops (fail-closed) when the
usable input budget would overflow, and errors if nothing fits — never
silently truncating.

## 6. Retrieval (`documents/retrieval.rs`)

Deterministic lexical scoring (term hits + exact-phrase boost), workspace
filter first, then classification ceiling (`chunk.classification <=
ceiling`). SECRET content requires a SECRET ceiling. Results carry chunk
id, relevance, text, source ref, page, section, classification, and full
provenance. No vector index exists in the tree, so no second retrieval
architecture was added; a future index can rank the same chunk rows.

## 7. Artifacts & lineage (`documents/service.rs`, `store.rs`, `pipeline.rs`)

Artifacts extend the existing `artifacts` table (no second database).
`create_artifact` validates kind/content (`validate_artifact_content`:
non-empty, ≤10 MB, JSON well-formedness, extension sanity, executable-
extension warnings — never executes), writes through
`FilesystemSecurityService::write_file_bytes`, registers `ArtifactRecord`
with `UNVERIFIED` status, and inserts an `artifact_lineage` row
(parent artifact, source document/version, workflow/skill/model/provider,
classification, hashes — no secrets). `artifact_lineage` is queryable per
artifact, forming the source→extraction→analysis→artifact→verified graph.

## 8. Verification (`service.rs::verify_artifact`)

Generated artifacts MUST NOT auto-verify. `verify_artifact` builds an
`ExpectedArtifact` + `ExpectedFile` (must exist, non-empty) request and
calls `VerificationEngine::verify_task` (workspace-ownership checked
first). The registry status is stamped `VERIFIED`/`FAILED` by the engine
verdict only. Model claims are never evidence.

## 9. Tools (`documents/tools.rs`, wired in `tools/registry.rs` + `executor.rs`)

Eight built-ins through the existing ToolRuntime pipeline (capability,
permission, privacy, network, consent, audit, output sanitization):
`document.inspect`, `document.extract`, `document.search`,
`document.read_section`, `document.list_sections`, `document.chunk`,
`document.create_artifact`, `document.export_artifact`. Unknown tool or
capability → DENY (enforced by ToolRuntime). Tool executors are filesystem-
bound to the call's workspace; SECRET output is redacted; external export
from tool context is blocked (must use the authorized server export API);
`create_artifact` files land `UNVERIFIED`.

Skills/workflows (Phase 14) may reference these tools, but skill/workflow
definitions never authorize execution — every call re-enters live
ToolRuntime authorization.

## 10. Database (migration v11)

Exactly one forward-only migration (`migrations.rs` v11): `documents`,
`document_versions`, `document_sections`, `document_chunks`,
`artifact_lineage`, plus indexes on workspace/document/version/
classification/hash/artifact/task. Old migrations untouched.

## 11. Events & audit

New `EventType`s: `DocumentImported`, `DocumentParsed`,
`DocumentClassified` (reserved for future classifier split; classification
currently rides on parse/import events), `DocumentChunked`,
`DocumentRetrievalPerformed`, `ArtifactCreated`, `ArtifactVerified`,
`ArtifactVerificationFailed`. Payloads carry references/hashes only.
Audit (`audit_records` via existing table) covers intake, unsupported/
failed parses, sensitive retrieval, artifact create/verify/export-blocked —
reasons redacted via `EvidenceManager::redact_string`, never raw secrets.

## 12. Server API (`crates/octrex-server/src/main.rs`)

Documents: `GET /api/documents?workspace_id`, `POST /api/documents/import`,
`GET /api/documents/:id`, `GET /api/documents/:id/sections`,
`GET /api/documents/:id/chunks`, `POST /api/documents/:id/search`,
`POST /api/documents/:id/ingest`, `POST /api/documents/assist`.
Artifacts: `GET /api/artifacts?workspace_id|task_id`, `POST /api/artifacts`,
`GET /api/artifacts/:id`, `GET /api/artifacts/:id/lineage`,
`GET /api/artifacts/:id/verification`, `POST /api/artifacts/:id/verify`,
`POST /api/artifacts/:id/export`. Workspaces resolve against the DB with a
fail-closed sync from the server's inspected-workspace registry. SECRET
chunk/text payloads are redacted server-side.

## 13. Frontend (`apps/web`)

Pages: `/documents`, `/documents/[id]`, `/documents/[id]/search`,
`/artifacts`, `/artifacts/[id]`, `/settings/documents`. Components:
`DocumentBadges` (classification + security badges),
`DocumentBrowser` (workspace connect + list + import),
`DocumentPanels` (metadata, provenance, structure, chunk inspector,
search), `ArtifactPanels` (list, details, lineage, verification status,
export dialog). All render backend data; secret values are never displayed
(the API never sends them).

## 14. AI assistance (`service.rs::ai_assist`)

Summarize/extract/label route `RoutingMode::LocalOnly` with
`allow_online=false` through `ModelRouter::route`, then
`ModelRuntime::invoke` with the excerpt framed as untrusted input and a
system guard (never follow embedded instructions). PrivacyGate re-checks
local eligibility. No permitted local model, or any cloud selection →
explicit unavailable/blocked error. No automatic cloud fallback, ever.

## 15. Security invariants (all enforced + tested)

Workspace isolation · filesystem boundary · monotonic classification ·
no document-driven policy/capability/privacy/routing changes · no auto
cloud upload · no secret leakage (logs/audit/events/UI/prompts) · no
System-role document content · no artifact auto-verification · no cross-
workspace retrieval · model claims ≠ evidence · unknown auth fails closed ·
explicit unsupported formats · parser failures don't bypass security.

## 16. Tests (`documents/tests.rs`, 23 tests)

Traversal/absolute rejection, protected-file behavior (via boundary),
unsupported formats, TXT/MD/JSON/CSV/XML/PDF/DOCX/XLSX paths, XXE
rejection, ZIP traversal rejection, provenance preservation, monotonic
classification, redaction, injection containment, capability-grant
impossibility, chunk provenance + token math, budget posture, retrieval
isolation + ceiling, artifact validation, hash determinism, tool
registration + unknown-tool denial.

## 17. Known limitations & Phase 16 follow-ups

- PDF extraction is heuristic: scanned/image-only PDFs report no-text
  (no OCR offline); complex encodings may be partial (warned).
- DOCX section structure is flat (headings detected, no TOC/section tree);
  footnotes/endnotes/comments not extracted.
- XLSX drops layout features (merged cells, charts, drawings) with
  warnings; values + formulas preserved.
- Retrieval is lexical only; no embeddings/vector index.
- `DocumentClassified` event is reserved; classification currently emits
  with parse/import events.
- AI assist excerpt capped (3 chunks / 6000 chars); large docs use
  retrieval-first selection by callers.
- Concurrent in-tree development: Phase 16 (local runtime) shares
  `main.rs`/`state.rs`; Phase 15 touched only additive regions plus two
  one-line pre-existing type-error fixes in `local_runtime/` and one
  control-flow fix in `stream_local_inference_handler` (single-stream
  construction for `impl Stream` unification).
