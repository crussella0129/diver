Finalized - DO NOT EDIT

# Sprint 14 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC1: injectable base_url + from_env override | T-1401 / WHEN build Some/None THEN base/default; WHEN from_env THEN ANTHROPIC_BASE_URL | `test_build_base_url_default_and_override` |
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC2: 2xx happy path + request shape | T-1402 / WHEN mock 2xx THEN candidates + headers/body | `test_extract_http_happy_path` |
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC3: non-2xx error path | T-1402 / WHEN mock non-2xx THEN Err status+body | `test_extract_http_error_status` |
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC4: dev-deps only | T-1402 / [dev-dependencies] wiremock + tokio | `cargo build` non-test unchanged; `cargo tree -e no-dev` lacks wiremock/tokio |
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC5: no regression + docs | T-1401/T-1403 + full suite | full `cargo test --workspace`; README + INT-0009 updated |

## Unit Tests

### T-1401 unit tests (`diver-core/src/extract.rs`)
- **Intent:** [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md)
- `test_build_base_url_default_and_override`: `build(Some(key), None, Some("http://localhost:9"))` → `base_url == "http://localhost:9"`; `build(Some(key), None, None)` → `DEFAULT_BASE_URL`; a blank base URL falls back to the default.
- Updated (signature migration): `test_build_missing_key_errors`, `test_build_model_default_and_override` → 3-arg `build` (pass `None` base URL, preserving prior assertions).
- Stubs: none (pure construction).

## Integration Tests

### Extractor HTTP boundary (`diver-core/src/extract.rs` test module, `#[tokio::test]` + wiremock)
- **Intents:** [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md)
- `test_extract_http_happy_path`: start a `MockServer`; mount a `POST /v1/messages` mock that matches the `x-api-key` and `anthropic-version` headers and responds 200 with a canned Messages envelope (one grounded + one hallucinated claim). Build the extractor with `base_url = server.uri()`; assert `extract` returns exactly the grounded candidate, and inspect the recorded request body for the model, the system prompt, and the abstract text.
- `test_extract_http_error_status`: mount a mock returning 500 with body "upstream boom"; assert `extract(&fact)` returns `Err` whose message contains "500" and "upstream boom".
- These are real `reqwest` round-trips against a local mock — the offline end-to-end of the HTTP boundary that was previously untested.

## End-to-End Tests
- **Status:** possible (offline)
- The two wiremock transport tests ARE the offline end-to-end of the extractor's HTTP
  boundary (real client → local mock server, full request build + response/error handling).
- A live run against the real Anthropic API remains a manual check (requires a key) — now a
  much smaller residual (only the real-endpoint reachability/credentials, not the request or
  parsing logic).
