# Sprint 2 Research Report

## Intents Reviewed
- [INT-0002 Paper ingestion pipeline](../../intents/INT-0002-paper-ingestion.md)

## Sprint Goal

Add paper ingestion pipeline: `diver ingest <arxiv-id>` fetches full metadata,
introduces domain types (`SourceFact`), and local storage with SQLite.

## Existing Code Survey

### Current architecture (`src/`)

| Module       | Purpose                        | Lines | Sprint 2 impact |
|--------------|--------------------------------|-------|-----------------|
| `model.rs`   | `Paper` struct, Display impl   | 48    | Paper becomes the "raw API response" type; SourceFact wraps it with provenance |
| `client.rs`  | `ArxivClient::search(query)`   | 44    | Extend with `fetch_by_id(arxiv_id)` for single-paper retrieval |
| `parse.rs`   | Atom XML → `FeedResult`        | ~180  | Reuse as-is — single-paper response uses the same Atom format |
| `query.rs`   | `QueryBuilder` for search URLs | ~100  | Not modified; ingest uses `id_list` parameter directly |
| `display.rs` | Terminal output formatting     | ~80   | Extend with `display_fact()` and `display_list()` |
| `main.rs`    | Clap CLI with `search` subcommand | 71 | Add `ingest`, `inspect`, `list` subcommands |
| `lib.rs`     | Module declarations            | 5     | Add `store` and `fact` modules |

### Dependencies (`Cargo.toml`)

Current: anyhow, clap, owo-colors, quick-xml, reqwest, serde, tokio.

New additions needed:
- **`rusqlite`** — SQLite storage (with `bundled` feature for zero system deps)
- **`chrono`** — Timestamp handling for `ingested_at` field
- **`dirs`** — Cross-platform `~/.diver/` path resolution

## External Sources

### ArXiv API: Single paper fetch

The ArXiv API supports fetching individual papers via the `id_list` parameter:

```
GET https://export.arxiv.org/api/query?id_list=2301.00001&max_results=1
```

Key findings:
- Response format is identical to search results (Atom XML with `<entry>` elements).
- If the ID doesn't exist, the feed returns an entry with `<title>Error</title>`.
- Version suffixes (e.g., `2301.00001v2`) are optional; omitting returns latest.
- The existing `parse::parse_feed()` function handles the response without modification.
- Same rate limit applies: 3 requests/second.

### SQLite storage: rusqlite vs sqlx

| Factor | rusqlite | sqlx |
|--------|----------|------|
| Async | No (sync + `spawn_blocking`) | Yes (native async) |
| Compile-time queries | No | Yes |
| Bundle size | Smaller | Larger (proc macros) |
| API simplicity | Direct, minimal | More ceremony |
| Migration tooling | Manual | Built-in |
| Fit for CLI tool | Excellent | Over-engineered |

**Decision: `rusqlite` with `bundled` feature.** Rationale: this is a single-user
CLI tool making fast local writes. Async DB access adds complexity with no benefit.
The `bundled` feature statically links SQLite, eliminating system dependency.

### Cross-platform data directory

The `dirs` crate provides `dirs::data_dir()`:
- Linux: `~/.local/share/diver/`
- macOS: `~/Library/Application Support/diver/`
- Windows: `C:\Users\<user>\AppData\Roaming\diver\`

Convention: database at `{data_dir}/diver.db`. Create directory on first use.

## Domain Type Design

### SourceFact

`SourceFact` is the first domain type in the epistemic engine. It wraps a raw
`Paper` with provenance metadata, establishing the pattern for all subsequent
domain types (Observation, Assertion).

```rust
pub struct SourceFact {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub primary_category: String,
    pub published: String,
    pub updated: String,
    pub pdf_url: String,
    // Provenance fields
    pub source_url: String,      // ArXiv API URL used to fetch
    pub arxiv_version: String,   // e.g., "v1", "v2"
    pub ingested_at: String,     // ISO 8601 timestamp
}
```

Design decisions:
- **Flat struct, not `Paper` + provenance wrapper.** Avoids lifetime tangles and
  makes SQLite serialization straightforward. The type distinction from `Paper`
  is nominal (different struct), not structural (wrapper).
- **`authors` stored as JSON array in SQLite.** Simple, queryable with JSON1
  extension if needed later.
- **`arxiv_version` extracted from the entry ID URL** (e.g.,
  `http://arxiv.org/abs/2301.00001v2` → `"v2"`).

### Storage Schema

```sql
CREATE TABLE IF NOT EXISTS source_facts (
    arxiv_id        TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    authors         TEXT NOT NULL,  -- JSON array
    summary         TEXT NOT NULL,
    primary_category TEXT NOT NULL,
    published       TEXT NOT NULL,
    updated         TEXT NOT NULL,
    pdf_url         TEXT NOT NULL,
    source_url      TEXT NOT NULL,
    arxiv_version   TEXT NOT NULL,
    ingested_at     TEXT NOT NULL   -- ISO 8601
);
```

Upsert via `INSERT OR REPLACE` ensures re-ingestion updates rather than
duplicates.

## Risks, Unknowns, and Dependencies

| Risk | Severity | Mitigation |
|------|----------|------------|
| ArXiv returns error entry for invalid ID | Low | Check for `<title>Error</title>` in parsed result, return descriptive error |
| `rusqlite` `bundled` increases compile time | Low | Accept trade-off; avoids system SQLite dependency issues |
| Schema migrations in future sprints | Medium | Deferred — v1 schema is additive only; migration tooling is a future intent |
| `dirs::data_dir()` returns `None` on exotic platforms | Low | Fall back to `./diver.db` in current directory |

## Recommended Approach

1. Add `rusqlite` (bundled), `chrono`, `dirs` to Cargo.toml.
2. Create `src/fact.rs` with `SourceFact` type and `Paper` → `SourceFact` conversion.
3. Create `src/store.rs` with `Store` struct wrapping rusqlite `Connection`:
   `open()`, `save_fact()`, `get_fact()`, `list_facts()`.
4. Extend `ArxivClient` with `fetch_by_id(id)` method using `id_list` parameter.
5. Add `ingest`, `inspect`, `list` subcommands to `main.rs`.
6. Extend `display.rs` with fact-specific formatters.
7. Add unit tests for store operations (in-memory SQLite) and SourceFact construction.

## Referenced Artifacts

- [INT-0002 Paper ingestion pipeline](../../intents/INT-0002-paper-ingestion.md)
- [Architecture vision](memory: project-architecture-vision) — Rust epistemic engine,
  typestate assertions, domain type hierarchy
- [ArXiv API documentation](https://info.arxiv.org/help/api/user-manual.html)
- [rusqlite crate](https://crates.io/crates/rusqlite)
- [dirs crate](https://crates.io/crates/dirs)
