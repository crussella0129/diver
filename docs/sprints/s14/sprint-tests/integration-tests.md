# Sprint 14 Integration Tests

- **Tested head:** `4f956d798cbc2ca230219b61cc5fc6885455f0cf`
- **Runner:** `cargo test --workspace`

## Transport integration (INT-0015)

The two `wiremock`-backed `#[tokio::test]`s (`test_extract_http_happy_path`,
`test_extract_http_error_status`) are the integration coverage this sprint adds: a real
`reqwest` client performs the actual HTTP round-trip against a local `MockServer`,
exercising the request construction (endpoint `{base_url}/v1/messages`, `x-api-key` /
`anthropic-version` headers, model/system/abstract body), the 2xx → `parse_claims` →
grounded-candidate path, and the non-2xx → error path. They live in the extractor module
(to reach the private `build` with the mock's dynamic URI) and run in the `diver_core`
lib unittest binary. **2 passed.**

## Existing integration binaries (unchanged, all green)
- `coassertion` 2, `dive_graph` 1, `dive_pipeline` 1, `extract_pipeline` 1,
  `ingest_pipeline` 2, `llm_extract_pipeline` 1, `persist_pipeline` 1. **9 passed.**
- `llm_extract_pipeline` still covers `parse_claims`/grounding on a canned body; the new
  transport tests complement it at the HTTP layer that was previously untested.

## Isolation / determinism
- Each transport test starts its own `MockServer` on a random port and injects
  `server.uri()` as the extractor's `base_url` — no shared state, no real network, no API
  key (a throwaway `sk-test`). Fixed request/response bodies; deterministic assertions.

## Raw result
```
Running unittests src\lib.rs (diver_core)  →  test_extract_http_happy_path ok, test_extract_http_error_status ok
```
