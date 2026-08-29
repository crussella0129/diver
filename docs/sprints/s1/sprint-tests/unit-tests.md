# Sprint 1 — Unit Test Results

- **Tested head:** `6d68d19ce5e2242fb58787e71a66fe1c5cf2b0a5`
- **Runner:** `cargo test`
- **Result:** 14 passed, 0 failed

## T-002 (model)

| Test | EARS clause | Result |
|------|------------|--------|
| `test_paper_display` | WHEN Paper formatted THEN output includes title, authors, link | PASS |

## T-003 (query)

| Test | EARS clause | Result |
|------|------------|--------|
| `test_query_default` | WHEN max_results not set THEN default 10, sortBy relevance | PASS |
| `test_query_max_results` | WHEN max_results(5) THEN URL contains max_results=5 | PASS |
| `test_query_sort_by_submitted` | WHEN sort_by SubmittedDate THEN sortBy=submittedDate | PASS |
| `test_query_sort_by_updated` | WHEN sort_by LastUpdatedDate THEN sortBy=lastUpdatedDate | PASS |
| `test_query_multiword` | WHEN multi-word query THEN proper URL encoding | PASS |

## T-004 (parse)

| Test | EARS clause | Result |
|------|------------|--------|
| `test_parse_valid_feed` | WHEN valid XML with 3 entries THEN Vec<Paper> len 3 | PASS |
| `test_parse_entry_fields` | WHEN single entry THEN all fields populated | PASS |
| `test_parse_total_results` | WHEN totalResults element THEN correct count | PASS |
| `test_parse_empty_feed` | WHEN 0 entries THEN empty Vec, total 0 | PASS |
| `test_parse_malformed_xml` | WHEN malformed XML THEN descriptive Err | PASS |

## T-006 (display)

| Test | EARS clause | Result |
|------|------------|--------|
| `test_display_truncates_abstract` | WHEN abstract > 200 chars THEN truncated with "..." | PASS |
| `test_display_empty_results` | WHEN empty list THEN "No results found." | PASS |
| `test_display_showing_count` | WHEN total > shown THEN "Showing N of M results." | PASS |

## Verification commands

| Check | Command | Result |
|-------|---------|--------|
| Clippy | `cargo clippy` | 0 warnings |
| Format | `cargo fmt --check` | Clean |
| Release build | `cargo build --release` | 0 warnings |
