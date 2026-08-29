# Sprint 5 Test Report

**Date:** 2026-08-29
**Sprint:** 5
**Phase:** test
**Status:** PASS

## Summary

| Suite | Tests | Passed | Failed | Skipped |
|-------|-------|--------|--------|---------|
| Unit (lib) | 59 | 59 | 0 | 0 |
| Unit (main) | 0 | 0 | 0 | 0 |
| Integration: dive_pipeline | 1 | 1 | 0 | 0 |
| Integration: ingest_pipeline | 2 | 2 | 0 | 0 |
| Doc tests | 0 | — | — | — |
| **Total** | **62** | **62** | **0** | **0** |

## New Tests Added This Sprint

### id::tests (T-501 / T-502)
- `test_taxonomy_valid_code` — `ArxivCategory::parse("cs.CV")` → Ok with correct name
- `test_taxonomy_invalid_code` — `ArxivCategory::parse("invalid.XX")` → Err containing "not in the taxonomy"
- `test_taxonomy_math_na` — math.NA resolves correctly
- `test_taxonomy_stat_ml` — stat.ML resolves correctly
- `test_arxiv_version_display` — `ArxivVersion(2)` → "v2"
- `test_arxiv_id_construction` — `ArxivId::new("2301.00001")` → correct as_str
- `test_category_display` — `ArxivCategory::to_string()` format is correct

### parse::tests (T-503)
- `test_parse_multi_category` — entry with cs.CL + cs.AI → categories len 2, both present
- `test_parse_no_category_duplication` — cs.LG appears once despite being in primary + category
- `test_parse_single_category` — entry with only primary_category → categories len 1

### fact::tests (T-504)
- `test_source_fact_categories` — 3 known codes → len 3
- `test_source_fact_unknown_category_skipped` — unknown code silently skipped, no panic

### store::tests (T-505)
- `test_store_multi_version` — save v1 then v2 → get_versions returns ["v1","v2"]
- `test_store_idempotent_save` — save v1 twice → get_versions returns ["v1"] only
- `test_store_get_returns_latest` — save v1 then v2 → get() returns v2 title
- `test_store_versions_not_destroyed` — both v1 and v2 survive

### display::tests (T-507)
- `test_display_fact_taxonomy_name` — "Computer Vision and Pattern Recognition" in output
- `test_display_fact_secondary_categories` — secondary categories correctly identified
- `test_display_fact_version_marker` — current version marked with indicator

### Integration (T-505, T-506, T-508)
- `test_find_pipeline` — FTS search works end-to-end after Dive→Find rename
- `test_ingest_pipeline` — `.primary_category.code() == "cs.CL"` passes post-type change
- `test_ingest_pipeline_multi_category` — full pipeline parses cs.CL + cs.AI from feed

## Command: `cargo test`
```
test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Database Migration Note

Verified: schema auto-creates on fresh database (`Store::open_in_memory()` used in tests). Pre-Sprint-5 `diver.db` files must be deleted manually (see README.md).

## Intent Traceability

All acceptance criteria in [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) are covered:
- AC1 (`diver find` works): `test_find_pipeline` ✓
- AC2 (existing commands unchanged): all prior-suite tests pass ✓
- AC3 (inspect shows taxonomy names): `test_display_fact_taxonomy_name` ✓
- AC4 (multi-version storage): `test_store_multi_version` ✓
- AC5 (ArxivId newtype): `test_arxiv_id_construction` ✓
- AC6 (ArxivCategory newtype): `test_arxiv_category_newtype` (compiler-enforced) ✓
- AC7 (taxonomy validation): `test_taxonomy_invalid_code` ✓
- AC8 (all tests pass): 62/62 ✓
