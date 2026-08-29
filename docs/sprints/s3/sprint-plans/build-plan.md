Finalized - DO NOT EDIT

# Sprint 3 Build Plan

## Intents
- [INT-0003](../../../intents/INT-0003-local-knowledge-search.md) — state: active; acceptance criteria covered: AC-1 through AC-7 (all)

## Schema Tree
- Sprint Goal: Local knowledge search with FTS5
  - Storage extension
    - T-014: Add FTS5 virtual table and index maintenance to Store
  - Search layer
    - T-015: Add Store::search() with FTS5 MATCH and BM25 ranking
  - Presentation
    - T-016: Display formatter for dive search results
  - Integration
    - T-017: Wire `dive` subcommand into CLI

## Execution Sequence

### T-014: Add FTS5 virtual table and index maintenance to Store
- **Intent:** [INT-0003](../../../intents/INT-0003-local-knowledge-search.md)
- **Touches:** `src/store.rs`
- **Depends on:** (none)
- **Acceptance criterion:** AC-5 (FTS index populated on ingest)
- **Success criterion (EARS):**
  - **WHEN** `init_schema()` is called on a fresh database, **THEN** it **SHALL** create both `source_facts` and `source_facts_fts` (FTS5 virtual table indexing arxiv_id, title, authors, summary, primary_category).
  - **WHEN** `save(fact)` is called, **THEN** it **SHALL** insert/replace into both `source_facts` and `source_facts_fts` within a single transaction.
  - **WHEN** `save(fact)` is called for an existing arxiv_id, **THEN** the FTS index entry **SHALL** be deleted and re-inserted (not duplicated).
  - **WHEN** `init_schema()` is called on an existing database without FTS table, **THEN** it **SHALL** create the FTS table and populate it from existing `source_facts` rows.
- **Notes:** Standalone FTS table (no `content=` clause). Authors joined as comma-separated string for indexing. Transaction for atomicity. Backfill path verified by `test_init_schema_backfills_existing_facts`.

### T-015: Add Store::search() with FTS5 MATCH and BM25 ranking
- **Intent:** [INT-0003](../../../intents/INT-0003-local-knowledge-search.md)
- **Touches:** `src/store.rs`
- **Depends on:** T-014
- **Acceptance criterion:** AC-1 (search returns ranked results), AC-4 (no results)
- **Success criterion (EARS):**
  - **WHEN** `store.search("attention", 10)` is called with matching papers, **THEN** it **SHALL** return a `Vec<SearchResult>` ordered by BM25 rank (most relevant first), limited to the requested count.
  - **WHEN** `store.search("xyznonexistent", 10)` is called with no matches, **THEN** it **SHALL** return an empty `Vec`.
  - **WHEN** `store.search("\"attention mechanism\"", 10)` is called with a phrase query, **THEN** FTS5 **SHALL** match papers containing that exact phrase.
- **Notes:** `SearchResult` struct with arxiv_id, title, authors (Vec<String>), summary, primary_category, rank (f64). Query passed directly to FTS5 MATCH. Join to source_facts for full author JSON.

### T-016: Display formatter for dive search results
- **Intent:** [INT-0003](../../../intents/INT-0003-local-knowledge-search.md)
- **Touches:** `src/display.rs`
- **Depends on:** T-015
- **Acceptance criterion:** AC-2 (results display ID, title, category, abstract snippet)
- **Success criterion (EARS):**
  - **WHEN** `display_dive_results(results)` is called with a non-empty list, **THEN** each result **SHALL** be printed with a numbered header, title (bold), authors, truncated abstract (200 chars), category, and ArXiv URL.
  - **WHEN** `display_dive_results(results)` is called with an empty list, **THEN** it **SHALL** print "No matching papers found."

### T-017: Wire `dive` subcommand into CLI
- **Intent:** [INT-0003](../../../intents/INT-0003-local-knowledge-search.md)
- **Touches:** `src/main.rs`
- **Depends on:** T-015, T-016
- **Acceptance criterion:** AC-1 (dive returns ranked results), AC-3 (--max-results flag), AC-4 (no results message)
- **Success criterion (EARS):**
  - **WHEN** `diver dive "attention"` is run, **THEN** the CLI **SHALL** display matching papers ranked by relevance and exit 0.
  - **WHEN** `diver dive "attention" --max-results 3` is run, **THEN** the output **SHALL** contain at most 3 results.
  - **WHEN** `diver dive "xyznonexistent"` is run, **THEN** the CLI **SHALL** print "No matching papers found." and exit 0.
