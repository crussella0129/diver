Finalized - DO NOT EDIT

# Sprint 5 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) | AC1: `diver find` performs FTS; `diver dive` absent | T-506 / WHEN user runs `diver find "q"` THEN CLI SHALL return FTS results | `test_find_command_works` |
| [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) | AC2: existing commands unchanged | T-506 / WHEN `diver search/ingest/collect/inspect/list` invoked THEN each SHALL behave as before | existing test suite passes |
| [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) | AC3: inspect shows taxonomy name + secondary categories + versions | T-507 / WHEN `display_fact(&fact, &versions)` called with cs.CV THEN output SHALL include "Computer Vision..." | `test_display_fact_taxonomy_name` |
| [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) | AC4: multi-version storage | T-505 / WHEN save v1 then v2 THEN get_versions SHALL return both | `test_store_multi_version` |
| [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) | AC5: ArxivId newtype prevents bare String | T-502 / compiler check | `test_arxiv_id_construction` |
| [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) | AC6: ArxivCategory newtype prevents bare String | T-502 / compiler check | `test_arxiv_category_newtype` |
| [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) | AC7: ArxivCategory::parse validates against taxonomy | T-502 / WHEN parse("invalid.XX") THEN SHALL return Err | `test_taxonomy_invalid_code` |
| [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) | AC8: all tests pass | T-501..T-508 / all EARS clauses | full `cargo test` suite |

## Unit Tests

### T-501 / T-502 — taxonomy and newtypes
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- `test_taxonomy_valid_code`: `ArxivCategory::parse("cs.CV")` → `Ok`; `.name()` → `"Computer Vision and Pattern Recognition"`
- `test_taxonomy_invalid_code`: `ArxivCategory::parse("invalid.XX")` → `Err` containing "not in taxonomy"
- `test_arxiv_version_display`: `ArxivVersion(2).to_string()` → `"v2"`
- `test_arxiv_id_construction`: `ArxivId::new("2301.00001")` → `Ok`; `.as_str()` → `"2301.00001"`

### T-503 — multi-category parsing
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- `test_parse_multi_category`: feed entry with primary `cs.CV` + secondary `math.NA` + `cs.AI` → `paper.categories.len() == 3`
- `test_parse_no_category_duplication`: primary `cs.CV` not duplicated when also listed as secondary
- `test_parse_single_category`: feed entry with only `primary_category` → `paper.categories.len() == 1`

### T-504 — typed SourceFact categories
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- `test_source_fact_categories`: `SourceFact::from_paper` with 3 known codes → `fact.categories.len() == 3`
- `test_source_fact_unknown_category_skipped`: paper with 1 unknown code + 1 known → `fact.categories.len() == 1` (no panic)

### T-505 — paper versions store
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- `test_store_multi_version`: save v1 then v2 → `get_versions("2301.00001")` returns `["v1", "v2"]`
- `test_store_idempotent_save`: save v1 twice → `get_versions` returns `["v1"]` only
- `test_store_get_returns_latest`: save v1 then v2 → `get("2301.00001")` returns v2 title
- `test_store_versions_not_destroyed`: save v1 then v2 → `get_versions` has both

### T-507 — display with taxonomy and versions
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- `test_display_fact_taxonomy_name`: output contains "Computer Vision and Pattern Recognition" for cs.CV
- `test_display_fact_secondary_categories`: output lists secondary categories under "Secondary:" heading
- `test_display_fact_version_marker`: output marks current version with `<-` indicator

## Integration Tests

### Full pipeline — ingest two versions, inspect
- **Intents:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- `test_pipeline_two_versions`: parse feed with 3 categories → SourceFact → Store save v1 → Store save v2 → `get_versions` returns both → `display_fact` output contains taxonomy name and both versions

## End-to-End Tests
- **Status:** not-yet-possible (requires live arXiv network call)
- Unlocked by: manual verification step in Test phase; full E2E will be added when a test-fixture HTTP server is introduced in a future sprint
