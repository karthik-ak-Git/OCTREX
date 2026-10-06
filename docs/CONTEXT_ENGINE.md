# OCTREX CODE V4 — Context Engine & Repository Intelligence Specification

## 1. Overview

The Context Engine selects, prunes, compacts, and structures prompt payloads to ensure AI models receive relevant, high-density context without context bloat or token budget exhaustion.

---

## 2. Repository Intelligence Tier

```
[ Repository Source Files ]
           │
           ├──> ripgrep fast text indexing
           ├──> Tree-sitter AST symbol extractor
           ├──> Language Server Protocol (LSP) reference graph
           └──> Git history & diff parser
```

When an agent processes a task:
1. **Symbol Search:** Resolves target identifiers to explicit file definitions.
2. **Dependency Analysis:** Maps imports, exported interfaces, and caller hierarchy.
3. **Selective Snippet Retrieval:** Pulls only relevant code blocks into context rather than entire repositories.

---

## 3. Context Management & Budget Allocation

Each prompt payload is assembled according to strict token budget allocations:

| Segment | Budget % | Strategy |
| :--- | :--- | :--- |
| **System Rules & Agent Persona** | 10% | Static, high priority |
| **Project Memory & Conventions** | 10% | Persistent, dynamic key-value store |
| **Task & Acceptance Criteria** | 15% | Fixed per task run |
| **Selected Code Context** | 45% | Dynamically retrieved snippets and AST definitions |
| **Tool Outputs & Log Buffer** | 20% | Auto-pruned, truncated past line limits |

---

## 4. Conversation Compaction & Pruning Protocols

1. **Tool Output Truncation:** Large command/test logs retain only the first 20 and last 50 lines (where errors appear). Full logs remain accessible on disk.
2. **Old Conversation Compaction:** When context length exceeds 70% of model window limit, past chat turns are summarized into a concise state summary.
3. **Snippet Eviction:** Unmodified snippets referenced in older turns are evicted in favor of active edit regions.

---

## 5. Persistent Project Memory Store

OCTREX maintains project-specific facts in `.octrex/memory.json`:
- Discovered framework & runtime versions.
- Default build and test commands (`npm test`, `pytest`, etc.).
- Code style and naming conventions.
- Architectural decisions and recurring bug pattern mitigations.
