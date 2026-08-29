Finalized - DO NOT EDIT

# Sprint 2 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0002](../../../intents/INT-0002-paper-ingestion.md) | AC-1: ingest stores to SQLite | T-010 / WHEN save THEN inserted, T-011 / WHEN fetch_by_id THEN parsed Paper, T-013 / WHEN diver ingest THEN stored | `test_store_save_and_get`, `test_fetch_by_id_fixture`, manual `diver ingest` |
| [INT-0002](../../../intents/INT-0002-paper-ingestion.md) | AC-2: re-ingest updates | T-010 / WHEN save existing THEN replaced, T-013 / WHEN ingest again THEN updated | `test_store_upsert` |
| [INT-0002](../../../intents/INT-0002-paper-ingestion.md) | AC-3: inspect prints all metadata | T-012 / WHEN display_fact THEN all fields, T-013 / WHEN diver inspect THEN display_fact | `test_display_fact_all_fields`, manual `diver inspect` |
| [INT-0002](../../../intents/INT-0002-paper-ingestion.md) | AC-4: list prints table | T-012 / WHEN display_fact_list THEN table, T-013 / WHEN diver list THEN table | `test_display_fact_list`, `test_display_fact_list_empty`, manual `diver list` |
| [INT-0002](../../../intents/INT-0002-paper-ingestion.md) | AC-5: SourceFact distinct with provenance | T-009 / WHEN from_paper THEN provenance fields | `test_source_fact_from_paper`, `test_source_fact_version_extraction` |
| [INT-0002](../../../intents/INT-0002-paper-ingestion.md) | AC-6: unit tests pass | T-009, T-010, T-012 / all EARS clauses | all unit tests below |
| [INT-0002](../../../intents/INT-0002-paper-ingestion.md) | AC-7: clippy+fmt clean | all tasks | `cargo clippy` + `cargo fmt --check` |

## Unit Tests

### T-009 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- `test_source_fact_from_paper`: Paper with all fields → SourceFact with matching fields plus `ingested_at`, `source_url`, `arxiv_version`
- `test_source_fact_version_extraction`: Paper with `arxiv_id` "2301.00001v2" → `arxiv_version` = "v2", bare `arxiv_id` = "2301.00001"
- `test_source_fact_default_version`: Paper with `arxiv_id` "2301.00001" (no suffix) → `arxiv_version` = "v1"

### T-010 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- `test_store_save_and_get`: save a SourceFact → get by ID returns matching fact with all fields
- `test_store_upsert`: save same arxiv_id twice with different title → get returns second title, single row
- `test_store_get_unknown`: get nonexistent ID → returns None
- `test_store_list`: save 3 facts → list returns 3 ordered by ingested_at desc
- `test_store_list_empty`: fresh store → list returns empty Vec
- Uses: in-memory SQLite (`:memory:`) for all store tests — `Store::open_in_memory()`

### T-011 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- `test_extract_paper_valid`: parse single-entry fixture XML → `extract_paper(feed)` returns correct Paper
- `test_extract_paper_error_entry`: parse fixture with `<title>Error</title>` → `extract_paper(feed)` returns descriptive Err
- `test_extract_paper_empty_feed`: parse empty feed → `extract_paper(feed)` returns Err "paper not found"

### T-012 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- `test_display_fact_all_fields`: SourceFact with all fields → output contains title, authors, category, source_url, arxiv_version, ingested_at
- `test_display_fact_list`: 2 facts → output contains both arxiv_ids, truncated titles, categories
- `test_display_fact_list_empty`: empty vec → "No ingested papers."

## Integration Tests

### Ingest pipeline integration
- **Intents:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- `test_ingest_pipeline`: parse fixture XML → convert to SourceFact → save to in-memory store → get by ID → verify round-trip fidelity (no network)

## End-to-End Tests
- **Status:** not-yet-possible
- Unlocked by: same constraint as Sprint 1 — live ArXiv API is unreliable for CI. Manual E2E performed during build: `diver ingest 2301.00001`, `diver inspect 2301.00001`, `diver list`.
