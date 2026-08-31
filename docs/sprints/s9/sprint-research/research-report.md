# Sprint 9 Research Report

## Intents Reviewed
- [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) — created; relevance: primary; current state: proposed
- [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) — selected; relevance: produces the `Assertion<Supported>`s this sprint persists; current state: realized

## 1. Sprint Goal

Make extracted knowledge durable. Extend the SQLite store with an `assertions`
table (claim + provenance to paper+version) and an `assertion_support` table
(supporting observation quotes), add `Store::save_assertions` /
`Store::get_assertions`, wire `diver extract` to persist what it computes
(idempotently per paper+version), and add `diver assertions <id>` to read them
back. `save_assertions` takes `&[Assertion<Supported>]`, so the typestate gate
extends to storage — only validated knowledge is persistable.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/store.rs | high | `init_schema` (store.rs:45) creates `papers`, `paper_versions`, `paper_versions_fts` with `PRAGMA foreign_keys=ON` (store.rs:49); add `assertions` + `assertion_support` here. `save()` (store.rs:121) shows the transaction + paper_id-lookup pattern to mirror. `Connection` is the private `conn` field (tests access it). |
| diver-core/src/assertion.rs | high | `Assertion<Supported>` accessors `claim() -> &str`, `support() -> &[Observation]`; `save_assertions` reads these. Taking `&[Assertion<Supported>]` is the compile-time storage gate. |
| diver-core/src/observation.rs | high | `Observation::text()` (the quote), `arxiv_id()`, `version()` — support quotes come from `obs.text()`. |
| diver-core/src/display.rs | medium | Add `display_stored_assertions(arxiv_id, &[StoredAssertion])`, mirroring `display_extract` (owo-colors). |
| diver-cli/src/main.rs | high | `Extract` handler persists after computing `supported`; new `Assertions { arxiv_id }` command mirrors `Inspect` (load-less: `get_assertions` returns empty for unknown). |
| diver-core/src/fact.rs | low | `SourceFact.arxiv_id` + `arxiv_version` are the persistence keys passed by the extract handler. |

Baseline: workspace at `3fbd94f`. `cargo test --workspace` green (82 unit + 4
integration = 86).

### Schema (added to `init_schema`)

```sql
CREATE TABLE IF NOT EXISTS assertions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    paper_id   INTEGER NOT NULL REFERENCES papers(id),
    version    TEXT NOT NULL,
    claim      TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS assertion_support (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    assertion_id INTEGER NOT NULL REFERENCES assertions(id) ON DELETE CASCADE,
    quote        TEXT NOT NULL
);
```

`ON DELETE CASCADE` + the existing `PRAGMA foreign_keys=ON` means deleting an
assertion drops its support rows — used by the idempotent replace.

### Store API

```rust
pub struct StoredAssertion { pub claim: String, pub version: String, pub support: Vec<String> }

impl Store {
    // Idempotent per (paper, version): delete prior rows for this paper+version,
    // then insert. Accepts only validated assertions.
    pub fn save_assertions(&self, arxiv_id: &str, version: &str,
                           assertions: &[Assertion<Supported>]) -> Result<()>;
    // Stored assertions for a paper (any version), newest first; empty if none.
    pub fn get_assertions(&self, arxiv_id: &str) -> Result<Vec<StoredAssertion>>;
}
```

`save_assertions`: `INSERT OR IGNORE INTO papers` (get/create paper_id, mirroring
`save()`), `DELETE FROM assertions WHERE paper_id=? AND version=?` (cascade drops
support), then per assertion `INSERT INTO assertions ...` + `last_insert_rowid()`
+ `INSERT INTO assertion_support` per `support()` quote — all in one transaction.
`created_at` = `chrono::Utc::now().to_rfc3339()` (chrono already used in
`fact.rs`). `get_assertions`: join `papers`→`assertions`→`assertion_support`,
group support by assertion, order by `created_at DESC`.

## 3. External Sources
- [SQLite foreign keys / ON DELETE CASCADE](https://www.sqlite.org/foreignkeys.html) — cascade requires `PRAGMA foreign_keys=ON` (already set); parent delete removes children.
- [rusqlite](https://docs.rs/rusqlite) — `execute`, `query_row`, `last_insert_rowid()`, `prepare`/`query_map`, transactions via `execute_batch("BEGIN;"/"COMMIT;")` (the pattern `save()` already uses).

## 4. Risks, Unknowns, Dependencies

- **Risk:** idempotent replace must delete support too. Mitigation: `ON DELETE
  CASCADE` on `assertion_support` + `foreign_keys=ON` (already on) means deleting
  the `assertions` rows drops their support; a test asserts no duplicate/orphan
  rows after re-save.
- **Risk:** reconstructing `Assertion<Supported>` on load would need a constructor
  the typestate deliberately withholds. Mitigation: `get_assertions` returns plain
  `StoredAssertion` display data — no typestate reconstruction this sprint; the
  graph sprint handles rehydration.
- **Risk:** transaction consistency on partial failure. Mitigation: wrap
  save_assertions in `BEGIN`/`COMMIT` with rollback on error, exactly like
  `save()` (store.rs:126-235).
- **Unknown:** which version's assertions to show. Decision: `get_assertions`
  returns **all** stored assertions for the paper (each carrying its `version`),
  newest first — simple and lossless; the extract handler saves under
  `fact.arxiv_version` (the latest ingested version).
- **Dependency:** none new. `rusqlite`, `chrono` already in `diver-core`.

## 5. Recommended Approach

Primary: extend the store, then wire the CLI.

- `store.rs`: add the two tables to `init_schema`; add `StoredAssertion`,
  `save_assertions` (transactional, idempotent replace), `get_assertions`.
- `display.rs`: `display_stored_assertions`.
- `main.rs`: `Extract` handler calls `store.save_assertions(&fact.arxiv_id,
  &fact.arxiv_version, &supported)` after computing `supported`; new `Assertions
  { arxiv_id }` command → `get_assertions` → `display_stored_assertions`.
- README: document `diver assertions` and that `diver extract` now persists.

Tests: `save_assertions` round-trips via `get_assertions`; re-save replaces (no
duplicates); support rows cascade-deleted on replace; FK enforced
(assertion_support with orphan assertion_id fails); empty/unknown → empty vec; an
integration test extracting (fixture body → validate) then persisting then
reading back. E2E: `diver assertions --help`; `diver assertions <unknown>` prints
"no assertions" cleanly.

Alternative considered: reconstruct `Assertion<Supported>` on load via a
crate-internal constructor — deferred to the graph sprint; display data suffices
now and keeps the typestate constructor closed.

## Artifacts
- No standalone snippet files; schema and API are inline in §2.
