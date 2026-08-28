Finalized - DO NOT EDIT

# Sprint 1 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md) | AC-1: binary builds | T-001 / WHEN cargo build THEN zero warnings | `test_cargo_build` (integration) |
| [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md) | AC-2: search returns formatted results | T-002 / Display, T-004 / parse, T-006 / display | `test_paper_display`, `test_parse_valid_feed`, `test_display_showing_count` |
| [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md) | AC-3: --max-results, --sort-by | T-003 / WHEN max_results set THEN URL contains param | `test_query_max_results`, `test_query_sort_by_submitted` |
| [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md) | AC-4: error exit code | T-005 / WHEN HTTP fails THEN Err | `test_client_non_200_status` |
| [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md) | AC-5: unit tests pass | T-003, T-004 / all EARS clauses | all unit tests below |
| [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md) | AC-6: clippy+fmt clean | T-001 / WHEN clippy run THEN zero warnings | `cargo clippy` + `cargo fmt --check` |

## Unit Tests

### T-002 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- `test_paper_display`: Paper with all fields → Display output includes title, comma-joined authors, ArXiv link
- Struct field accessibility is implicitly verified by T-004 parse tests.

### T-003 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- `test_query_default`: no options → URL has `max_results=10`, `sortBy=relevance`
- `test_query_max_results`: `max_results(5)` → URL contains `max_results=5`
- `test_query_sort_by_submitted`: sort_by submitted → `sortBy=submittedDate`
- `test_query_sort_by_updated`: sort_by updated → `sortBy=lastUpdatedDate`
- `test_query_multiword`: "transformer attention" → proper URL encoding

### T-004 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- `test_parse_valid_feed`: fixture XML with 3 entries → 3 Papers with correct fields
- `test_parse_entry_fields`: single entry → title, authors, summary, category, links all populated
- `test_parse_total_results`: totalResults element → correct count
- `test_parse_empty_feed`: feed with 0 entries → empty Vec, total = 0
- `test_parse_malformed_xml`: broken XML → descriptive Err
- Fixtures: `tests/fixtures/sample_feed.xml`, `tests/fixtures/empty_feed.xml`

### T-005 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- `test_client_non_200_status`: mock/stub returning 503 → Err naming status code

### T-006 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- `test_display_truncates_abstract`: abstract > 200 chars → truncated with "..."
- `test_display_empty_results`: empty vec → "No results found."
- `test_display_showing_count`: 3 papers, total=50 → output contains "Showing 3 of 50 results."

## Integration Tests

### ArXiv search pipeline integration
- **Intents:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- `test_query_to_parse_pipeline`: build query + parse saved fixture → correct Papers (no network)

## End-to-End Tests
- **Status:** not-yet-possible
- Unlocked by: live ArXiv API access in CI is unreliable due to rate limits. A future sprint can add a recorded-response (wiremock) E2E layer. Manual E2E verification will be performed during the build phase by running `diver search "attention"` against the live API.
