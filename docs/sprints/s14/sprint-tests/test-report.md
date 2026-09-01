# Sprint 14 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC1: injectable base_url + from_env override | T-1401 / `test_build_base_url_default_and_override` | pass (from_env env-read residual, critique C-001) | Test evidence links this report |
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC2: 2xx happy path + request shape | T-1402 / `test_extract_http_happy_path` | pass | Test evidence links this report |
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC3: non-2xx error path | T-1402 / `test_extract_http_error_status` | pass | Test evidence links this report |
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC4: dev-deps only | T-1402 / `cargo tree -e no-dev` + `cargo build` | pass | Test evidence links this report |
| [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) | AC5: no regression + docs | T-1401/T-1403 + full suite | pass (126/126; README + INT-0009 updated) | Test evidence links this report |

## Summary
- Unit tests: 116 passed / 0 failed (`diver_core` lib, +3 new) + 1 passed (`diver-cli` bin)
- Integration/transport tests: 2 new wiremock `#[tokio::test]`s (in the lib binary) + 9
  existing integration-binary tests, all pass
- E2E tests: offline — transport tests are the HTTP-boundary E2E; dev-deps confirmed
  runtime-absent; `--deterministic` path intact
- Clippy: 0 warnings
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `4f956d798cbc2ca230219b61cc5fc6885455f0cf`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` →
  `test result: ok` for every binary (`diver_core` lib 116, `diver-cli` bin 1,
  `coassertion` 2, `dive_graph` 1, `dive_pipeline` 1, `extract_pipeline` 1,
  `ingest_pipeline` 2, `llm_extract_pipeline` 1, `persist_pipeline` 1) = 126 total.
  `cargo build` (non-test) clean; `cargo tree -p diver-core -e no-dev` contains no
  `wiremock`. `cargo clippy --workspace --all-targets` → 0 warnings. Records:
  [unit](unit-tests.md), [integration](integration-tests.md), [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- `from_env`'s `ANTHROPIC_BASE_URL` read is covered only indirectly (via `build`);
  directly testing it needs unsafe/racy env mutation — critique C-001, deferred.
- No transport test for a 2xx response with a malformed body; the two halves (200→parse
  wiring, and parse_claims malformed handling) are each tested separately — critique C-002,
  deferred.
- Migrating the request/response contract to Anthropic **tool-use / structured outputs**
  remains a future intent (the transport harness built here is its prerequisite).

## Coverage Observations
- Every acceptance criterion has a named, executed test asserting the SHALL response,
  including the negative HTTP path (non-2xx → error with status + body) and the dev-only
  dependency guarantee.
- Tests are deterministic: each transport test owns a fresh `MockServer` on a random port,
  fixed request/response bodies, a throwaway key, no real network.
- The transport tests exercise the real `reqwest` round-trip — the exact request build and
  response/error handling that was previously verified only manually.
