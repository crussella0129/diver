Finalized - DO NOT EDIT

# Sprint 3 Test Plan

## Intents Reviewed
- [INT-0003 Local knowledge search](../../../intents/INT-0003-local-knowledge-search.md)

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| INT-0003 | AC-1: dive returns ranked results | T-015 / WHEN search with matches THEN ranked Vec | `test_search_ranked_results` |
| INT-0003 | AC-2: results display ID, title, snippet | T-016 / WHEN display_dive_results THEN all fields | `test_display_dive_results` |
| INT-0003 | AC-3: --max-results limits output | T-015 / WHEN search limited THEN Vec.len() <= N, T-017 / WHEN --max-results THEN limited | `test_search_max_results` |
| INT-0003 | AC-4: no results message | T-015 / WHEN no matches THEN empty Vec, T-016 / WHEN empty THEN "No matching papers found." | `test_search_no_results`, `test_display_dive_results_empty` |
| INT-0003 | AC-5: FTS index on ingest | T-014 / WHEN save THEN FTS updated | `test_save_populates_fts`, `test_upsert_updates_fts` |
| INT-0003 | AC-6: unit tests pass | all EARS clauses | all unit tests below |
| INT-0003 | AC-7: clippy+fmt clean | all tasks | `cargo clippy` + `cargo fmt --check` |

## Unit Tests

### T-014 unit tests
- **Intent:** INT-0003
- `test_save_populates_fts`: save a SourceFact → search by title keyword → returns matching result
- `test_upsert_updates_fts`: save same arxiv_id with new title → search old title returns nothing, search new title returns result
- `test_fts_indexes_multiple_fields`: save fact → search by author name → returns result; search by category → returns result
- `test_init_schema_backfills_existing_facts`: insert rows directly into source_facts (bypassing save), call init_schema again → FTS search finds those rows

### T-015 unit tests
- **Intent:** INT-0003
- `test_search_ranked_results`: save 3 papers, one with "attention" in title only, one in summary only, one in both → search "attention" → results ordered by relevance
- `test_search_no_results`: search "xyznonexistent" → empty Vec
- `test_search_max_results`: save 5 papers matching "test" → search with max_results=2 → Vec.len() == 2
- `test_search_phrase`: save papers → search `"attention mechanism"` (phrase) → only exact-phrase matches

### T-016 unit tests
- **Intent:** INT-0003
- `test_display_dive_results`: 2 results → output contains both titles, categories, arxiv IDs
- `test_display_dive_results_empty`: empty vec → "No matching papers found."

## Integration Tests

### Dive pipeline integration
- **Intents:** INT-0003
- `test_dive_pipeline`: save 3 papers via store → search → verify ranked results match expected papers (no network)

## End-to-End Tests
- **Status:** not-yet-possible
- Unlocked by: same constraint — manual E2E during build: `diver ingest` several papers, then `diver dive "query"`.

## Verification Checklist

1. `cargo fmt --check` — clean
2. `cargo clippy` — zero warnings
3. `cargo test` — all tests pass
4. Manual: `diver ingest 2301.00001` then `diver ingest 1706.03762`
5. Manual: `diver dive "attention"` — shows ranked results
6. Manual: `diver dive "attention" --max-results 1` — shows 1 result
7. Manual: `diver dive "xyznonexistent"` — prints "No matching papers found."
