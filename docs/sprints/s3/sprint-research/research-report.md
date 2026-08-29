# Sprint 3 Research Report

## Intents Reviewed
- [INT-0003 Local knowledge search](../../intents/INT-0003-local-knowledge-search.md) — created

## Sprint Goal

Add local knowledge search: `diver dive <query>` searches stored papers using
SQLite FTS5 for full-text search across titles, abstracts, authors, and
categories, ranked by BM25 relevance.

## Existing Code Survey

### Current architecture (`src/`)

| Module       | Purpose                           | Sprint 3 impact |
|--------------|-----------------------------------|-----------------|
| `store.rs`   | SQLite CRUD for SourceFact        | Add FTS5 virtual table, search method, FTS index triggers |
| `display.rs` | Terminal output (results, facts)  | Add search result display with relevance snippets |
| `main.rs`    | CLI with search/ingest/inspect/list | Add `dive` subcommand |
| `fact.rs`    | SourceFact domain type            | No changes |
| `client.rs`  | ArxivClient (search, fetch_by_id) | No changes |
| `model.rs`   | Paper struct                      | No changes |
| `parse.rs`   | Atom XML parser                   | No changes |
| `query.rs`   | ArXiv query builder               | No changes |
| `lib.rs`     | Module declarations               | No changes |

### Dependencies (`Cargo.toml`)

No new dependencies needed. The `bundled` feature of `rusqlite` compiles
SQLite with `-DSQLITE_ENABLE_FTS5`, so FTS5 is already available.

### Database schema (current)

```sql
CREATE TABLE source_facts (
    arxiv_id         TEXT PRIMARY KEY,
    title            TEXT NOT NULL,
    authors          TEXT NOT NULL,  -- JSON array
    summary          TEXT NOT NULL,
    primary_category TEXT NOT NULL,
    published        TEXT NOT NULL,
    updated          TEXT NOT NULL,
    pdf_url          TEXT NOT NULL,
    source_url       TEXT NOT NULL,
    arxiv_version    TEXT NOT NULL,
    ingested_at      TEXT NOT NULL
);
```

## External Sources

### SQLite FTS5

FTS5 (Full-Text Search 5) is SQLite's current full-text search extension.
Key design points for diver:

**Content-sync FTS table:** Use `content=source_facts` to avoid data
duplication. The FTS virtual table reads content from the main table and
maintains its own inverted index.

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS source_facts_fts
USING fts5(
    title, authors, summary, primary_category,
    content=source_facts,
    content_rowid=rowid
);
```

**Index maintenance:** With external content, FTS5 does not automatically
track changes to the content table. Two approaches:
1. **Triggers** on `source_facts` (INSERT, UPDATE, DELETE) that update the FTS
   index. More complex but fully automatic.
2. **Manual rebuild** via `INSERT INTO source_facts_fts(source_facts_fts) VALUES('rebuild')`.
   Simpler but requires explicit call after every save.

Decision: **Manual rebuild after save.** For a CLI tool ingesting one paper at
a time, rebuilding the FTS index after each save is negligible overhead (sub-
millisecond for hundreds of papers). This avoids trigger complexity and
content-sync rowid issues.

Actually, even simpler: use a **standalone FTS table** (no `content=` clause)
that stores its own copy of the indexed columns. This adds ~100 bytes per paper
but eliminates the content-sync complexity entirely. At our scale (hundreds of
papers), the storage overhead is trivial.

**Final decision: standalone FTS table.** Insert into both `source_facts` and
`source_facts_fts` during save. Delete from FTS before re-inserting on upsert.

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS source_facts_fts
USING fts5(arxiv_id, title, authors, summary, primary_category);
```

**Search query:**
```sql
SELECT arxiv_id, title, authors, summary, primary_category,
       rank
FROM source_facts_fts
WHERE source_facts_fts MATCH ?
ORDER BY rank
LIMIT ?;
```

FTS5's `rank` column contains the BM25 score (negative, lower = more
relevant). The MATCH operator supports:
- Simple terms: `attention`
- Phrases: `"attention mechanism"`
- Boolean: `attention AND transformer`
- Column filters: `title:attention`

**Snippet extraction:** FTS5 provides `snippet()` for extracting context
around matches, but it requires the auxiliary function to be registered. For
simplicity, we'll just display the full summary truncated, since our abstracts
are already short (~200 chars in display).

### rusqlite FTS5 support

Confirmed: `rusqlite` with `features = ["bundled"]` includes FTS5. The
`bundled` feature compiles SQLite from source with `-DSQLITE_ENABLE_FTS5`.
No additional feature flags or dependencies needed.

FTS5 virtual tables are created and queried using standard SQL via
`conn.execute()` and `conn.prepare()` / `stmt.query_map()`.

## Risks, Unknowns, and Dependencies

| Risk | Severity | Mitigation |
|------|----------|------------|
| FTS5 not enabled in bundled SQLite | Low | Verified: `-DSQLITE_ENABLE_FTS5` is set in build.rs. Will confirm with test. |
| FTS index out of sync after crash mid-save | Low | Both inserts (source_facts + FTS) happen in same transaction |
| Authors stored as JSON in source_facts but FTS needs plain text | Low | Join authors into comma-separated string for FTS indexing |
| Existing ingested papers won't be in FTS index | Medium | Add `rebuild_fts_index()` method that populates FTS from source_facts. Call during `init_schema()` for existing databases. |

## Recommended Approach

1. Extend `init_schema()` in `store.rs` to create the `source_facts_fts` virtual table.
2. Add `rebuild_fts_index()` to populate FTS from existing `source_facts` rows.
3. Modify `save()` to also insert into `source_facts_fts` within a transaction.
4. Add `Store::search(query, max_results)` method using FTS5 MATCH + BM25 rank.
5. Add `display_dive_results()` formatter showing ranked results with relevance.
6. Add `diver dive <query> --max-results N` subcommand.
7. Unit tests using in-memory SQLite with pre-populated data.

## Referenced Artifacts

- [INT-0003 Local knowledge search](../../intents/INT-0003-local-knowledge-search.md)
- [SQLite FTS5 Extension](https://www.sqlite.org/fts5.html)
- [rusqlite crate](https://crates.io/crates/rusqlite) — bundled feature includes FTS5
