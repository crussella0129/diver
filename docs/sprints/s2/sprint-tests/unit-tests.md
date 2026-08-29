# Sprint 2 Unit Test Results

**Tested head:** `e27e1c7e2dc01c6ff89103dcfb4d91a7cd6c4bd7`
**Runner:** `cargo test` (Rust 2024 edition)
**Result:** 28 passed, 0 failed

## T-009 — SourceFact type (src/fact.rs)

| Test | EARS clause | Result |
|------|-------------|--------|
| `test_source_fact_from_paper` | WHEN from_paper THEN contains all Paper fields plus provenance | pass |
| `test_source_fact_version_extraction` | WHEN arxiv_id has version suffix THEN extract version, strip bare ID | pass |
| `test_source_fact_default_version` | WHEN no version suffix THEN default to v1 | pass |

## T-010 — SQLite storage (src/store.rs)

| Test | EARS clause | Result |
|------|-------------|--------|
| `test_store_save_and_get` | WHEN save new fact THEN inserted; WHEN get stored ID THEN Some(fact) | pass |
| `test_store_upsert` | WHEN save existing arxiv_id THEN replaced, not duplicated | pass |
| `test_store_get_unknown` | WHEN get unknown ID THEN None | pass |
| `test_store_list` | WHEN list THEN all facts ordered by ingested_at desc | pass |
| `test_store_list_empty` | WHEN list on fresh store THEN empty Vec | pass |

## T-011 — fetch_by_id / extract_paper (src/client.rs)

| Test | EARS clause | Result |
|------|-------------|--------|
| `test_extract_paper_valid` | WHEN valid feed THEN extract correct Paper | pass |
| `test_extract_paper_error_entry` | WHEN error entry THEN Err "not found" | pass |
| `test_extract_paper_empty_feed` | WHEN empty feed THEN Err "not found" | pass |

## T-012 — Display formatters (src/display.rs)

| Test | EARS clause | Result |
|------|-------------|--------|
| `test_display_fact_all_fields` | WHEN display_fact THEN output contains all metadata fields | pass |
| `test_display_fact_list` | WHEN display_fact_list non-empty THEN table with IDs, titles, categories | pass |
| `test_display_fact_list_empty` | WHEN display_fact_list empty THEN "No ingested papers." | pass |

## Pre-existing tests (Sprint 1, still passing)

| Test | Module | Result |
|------|--------|--------|
| `test_paper_display` | model | pass |
| `test_query_default` | query | pass |
| `test_query_max_results` | query | pass |
| `test_query_sort_by_submitted` | query | pass |
| `test_query_sort_by_updated` | query | pass |
| `test_query_multiword` | query | pass |
| `test_parse_valid_feed` | parse | pass |
| `test_parse_entry_fields` | parse | pass |
| `test_parse_total_results` | parse | pass |
| `test_parse_empty_feed` | parse | pass |
| `test_parse_malformed_xml` | parse | pass |
| `test_display_truncates_abstract` | display | pass |
| `test_display_empty_results` | display | pass |
| `test_display_showing_count` | display | pass |
