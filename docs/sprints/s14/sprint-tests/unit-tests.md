# Sprint 14 Unit Tests

- **Tested head:** `4f956d798cbc2ca230219b61cc5fc6885455f0cf`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets`
- **Result:** `diver_core` lib unittests — **116 passed; 0 failed** (113 prior + 3 new
  in the extractor module); `diver-cli` bin — 1 passed. Clippy: 0 warnings.

## New tests (INT-0015)

### T-1401 — injectable base URL (diver-core/src/extract.rs)
- **Intent:** [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) — AC1
- `test_build_base_url_default_and_override`: `build(key, None, None)` → `base_url ==
  https://api.anthropic.com`; a blank base URL falls back to the default; an explicit
  base URL (`http://127.0.0.1:9`) is stored verbatim. **pass** (AC1)
- Migrated (3-arg `build`): `test_build_missing_key_errors`, `test_build_model_default_and_override`
  — pass `None` base URL, prior assertions preserved. **pass**

### T-1402 — transport tests (diver-core/src/extract.rs, `#[tokio::test]` + wiremock)
- **Intent:** [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) — AC2, AC3
- `test_extract_http_happy_path`: mock `POST /v1/messages` (matching `x-api-key` +
  `anthropic-version`) → 200 with an envelope carrying one grounded + one hallucinated
  claim. `extract` returns exactly the grounded candidate; the recorded request body
  contains the configured model (`claude-sonnet-test`), the system prompt, and the
  abstract. **pass** (AC2)
- `test_extract_http_error_status`: mock → 500 body "upstream boom"; `extract` returns
  `Err` containing "500" and "upstream boom". **pass** (AC3)

(These run in the `diver_core` lib unittest binary because they call the private
`build` with the mock's dynamic URI — see the plan's in-module rationale.)

## Raw result
```
cargo clippy --workspace --all-targets  →  0 warnings
Running unittests src\lib.rs (diver_core)  →  116 passed; 0 failed
Running unittests src\main.rs (diver-cli)  →    1 passed; 0 failed
```
