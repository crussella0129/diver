# Sprint 2 Test Report

## Summary

All automated tests pass. 29 tests total: 28 unit tests + 1 integration test.
`cargo clippy` and `cargo fmt --check` clean.

**Tested head:** `e27e1c7e2dc01c6ff89103dcfb4d91a7cd6c4bd7`

## Intent Verification

### INT-0002 — Paper ingestion pipeline

| Acceptance criterion | Verified by | Status |
|----------------------|-------------|--------|
| AC-1: ingest stores to SQLite | `test_store_save_and_get`, `test_extract_paper_valid`, `test_ingest_pipeline` | pass |
| AC-2: re-ingest updates | `test_store_upsert` | pass |
| AC-3: inspect prints all metadata | `test_display_fact_all_fields` | pass |
| AC-4: list prints table | `test_display_fact_list`, `test_display_fact_list_empty` | pass |
| AC-5: SourceFact distinct with provenance | `test_source_fact_from_paper`, `test_source_fact_version_extraction`, `test_source_fact_default_version` | pass |
| AC-6: unit tests pass | all 28 unit tests | pass |
| AC-7: clippy+fmt clean | `cargo clippy` + `cargo fmt --check` | pass |

## Coverage Summary

- **Unit tests:** 14 new (T-009: 3, T-010: 5, T-011: 3, T-012: 3) + 14 pre-existing = 28 total
- **Integration tests:** 1 new (`test_ingest_pipeline`) = 1 total
- **E2E tests:** not-yet-possible (ArXiv API unreliable for CI)

## Regression Check

All 14 pre-existing tests from Sprint 1 continue to pass. No regressions detected.

## Critique Resolution

- C-001 (CLI integration): deferred — data path proven by integration test; CLI parsing by clap upstream; manual verification documented.
- C-002 (in-memory SQLite): rejected — in-memory SQLite is the same engine, not a mock.

## Verdict

**Pass** — all acceptance criteria verified with automated tests. CLI wiring
deferred to E2E coverage in a future sprint.
