# Sprint 4 Test Plan
<!-- Finalized - DO NOT EDIT -->

**Status:** finalized

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| INT-0004 | AC-1: per-paper status | T-018 / display_collect_item, T-019 / collect loop | `test_display_collect_item_new`, `test_display_collect_item_update` |
| INT-0004 | AC-2: update detection | T-019 / WHEN exists THEN "Updated" | `test_display_collect_item_update` |
| INT-0004 | AC-3: sort-by flag | T-019 / WHEN --sort-by THEN query uses sort | verified via existing `QueryBuilder` tests |
| INT-0004 | AC-4: summary line | T-018 / display_collect_summary | `test_display_collect_summary` |
| INT-0004 | AC-5: no papers | T-018 / display_collect_empty | `test_display_collect_empty` |
| INT-0004 | AC-6: cargo test passes | all tasks | full test suite |
| INT-0004 | AC-7: clippy+fmt clean | all tasks | `cargo clippy` + `cargo fmt --check` |

## Unit Tests

### T-018 unit tests (src/display.rs)
- `test_display_collect_item_new`: is_update=false → output contains "Ingested:"
- `test_display_collect_item_update`: is_update=true → output contains "Updated:"
- `test_display_collect_summary`: new=3, updated=2 → output contains "Collected 3 new, 2 updated."
- `test_display_collect_empty`: output contains "No papers found."

## Integration Tests
Not needed beyond existing `ingest_pipeline` and `dive_pipeline` tests. The `collect` command reuses `save()` and `from_paper()` with no new data paths.

## End-to-End Tests
- **Status:** not-yet-possible (requires ArXiv network access)
- Manual: `diver collect "transformer" --max-results 3`
