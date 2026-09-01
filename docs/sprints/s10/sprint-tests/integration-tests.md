# Sprint 10 Integration Tests

- **Tested head:** `084e652ca1332ada3746ecfbe614f555b976cc0c`
- **Runner:** `cargo test --workspace`

## No new integration tests (lint-only maintenance)
INT-0011 is lint hygiene with no behavior change, so no new integration tests are
added. The existing integration suites all pass unchanged at the tested head:
- `dive_pipeline` — 1 passed
- `extract_pipeline` — 1 passed
- `ingest_pipeline` — 2 passed
- `llm_extract_pipeline` — 1 passed
- `persist_pipeline` — 1 passed

Total: **6 integration tests, 0 failed** — confirming the clippy rewrites did not
alter behavior (AC3).
