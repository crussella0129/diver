# Sprint 1 — Test Report

## Summary

All 14 automated unit tests pass. Manual E2E verification confirms the CLI
works end-to-end against the live ArXiv API. Clippy reports 0 warnings,
`cargo fmt --check` is clean, and `cargo build --release` succeeds with 0
warnings.

## Tested head

`6d68d19ce5e2242fb58787e71a66fe1c5cf2b0a5`

## Intent acceptance coverage

### INT-0001 — ArXiv search CLI foundation

| AC | Status | Evidence |
|----|--------|----------|
| AC-1: binary builds without warnings | PASS | `cargo build --release` — 0 warnings |
| AC-2: search returns formatted results | PASS | `test_parse_valid_feed`, `test_parse_entry_fields`, `test_paper_display`, `test_display_showing_count`, manual E2E |
| AC-3: --max-results and --sort-by | PASS | `test_query_max_results`, `test_query_sort_by_submitted`, `test_query_sort_by_updated`, manual E2E |
| AC-4: non-zero exit on error | PASS | Manual E2E (missing query arg → clap error, exit 1) |
| AC-5: cargo test passes | PASS | 14/14 pass |
| AC-6: clippy and fmt clean | PASS | 0 clippy warnings, fmt clean |

## Test suite results

| Suite | Tests | Passed | Failed |
|-------|-------|--------|--------|
| Unit (lib) | 14 | 14 | 0 |
| Unit (bin) | 0 | 0 | 0 |
| Doc-tests | 0 | 0 | 0 |
| Integration | N/A (covered by unit fixtures) | — | — |
| E2E (manual) | 3 scenarios | 3 | 0 |

## Caveats

- T-005 non-200 status path is tested by inspection only; automated mock
  coverage deferred to a future sprint adding wiremock.
- E2E is manual; automated recorded-response E2E deferred.
