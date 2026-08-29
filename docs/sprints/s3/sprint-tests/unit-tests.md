# Sprint 3 Unit Test Results

- **Head SHA:** `f59b39e9a0c82bb303e20d1e7d4c404b7d52974f`
- **Runner:** `cargo test` (rustc 1.87, Windows 11)
- **Result:** 38 passed, 0 failed

## Sprint 3 tests (12 new)

### T-014 — FTS5 virtual table and index maintenance
| Test | Result | EARS clause |
|------|--------|-------------|
| `store::tests::test_save_populates_fts` | PASS | WHEN save THEN FTS populated |
| `store::tests::test_upsert_updates_fts` | PASS | WHEN save existing THEN FTS delete+reinsert |
| `store::tests::test_fts_indexes_multiple_fields` | PASS | WHEN save THEN FTS indexes all columns |
| `store::tests::test_init_schema_backfills_existing_facts` | PASS | WHEN init_schema on existing DB THEN backfill |

### T-015 — Store::search() with FTS5 MATCH and BM25
| Test | Result | EARS clause |
|------|--------|-------------|
| `store::tests::test_search_ranked_results` | PASS | WHEN matching papers THEN ranked Vec |
| `store::tests::test_search_no_results` | PASS | WHEN no matches THEN empty Vec |
| `store::tests::test_search_max_results` | PASS | WHEN limited THEN Vec.len() <= N |
| `store::tests::test_search_phrase` | PASS | WHEN phrase query THEN exact-phrase match |

### T-016 — Display formatter for dive results
| Test | Result | EARS clause |
|------|--------|-------------|
| `display::tests::test_display_dive_results` | PASS | WHEN non-empty THEN all fields shown |
| `display::tests::test_display_dive_results_empty` | PASS | WHEN empty THEN "No matching papers found." |

### Pre-existing tests (26 unchanged)
All 26 pre-existing unit tests continue to pass.
