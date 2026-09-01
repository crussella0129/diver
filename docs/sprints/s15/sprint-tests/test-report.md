# Sprint 15 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC1: Anthropic tool-use request + parse | T-1501 / `test_extract_anthropic_tool_use` | pass | Test evidence links this report |
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC2: OpenAI structured request + parse | T-1501 / `test_extract_openai_structured` (= Animus_Ferric path) | pass | Test evidence links this report |
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC3: error paths + grounding | T-1501 / `test_extract_*_error`, `test_parse_claims_requires_structured`, `_grounded`, `_drops_hallucinated` | pass (OpenAI structured-parse-error direct test deferred, critique C-001) | Test evidence links this report |
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC4: config resolution + back-compat | T-1502 / `test_resolve_provider_*` (4) | pass (`from_env` file/env wiring via the pure helper, critique C-002) | Test evidence links this report |
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC5: heuristics gone + docs + no regression | T-1501/T-1503 + full suite | pass (128/128; fence tests removed; README/INT-0009 updated) | Test evidence links this report |

## Summary
- Unit tests: 118 passed / 0 failed (`diver_core` lib) + 1 passed (`diver-cli` bin)
- Integration/transport: 4 new per-shape wiremock `#[tokio::test]`s (lib binary) + 9 existing
  integration-binary tests (2 migrated to structured envelopes), all pass
- E2E tests: offline — the per-shape wiremock tests are the HTTP-boundary E2E; `--deterministic`
  intact; clippy clean (no dead code from removed heuristics)
- Clippy: 0 warnings
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `19ee28db6feff91e27f6f522f138078053011b53`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` →
  `test result: ok` for every binary (`diver_core` lib 118, `diver-cli` bin 1, `coassertion` 2,
  `dive_graph` 1, `dive_pipeline` 1, `extract_pipeline` 1, `ingest_pipeline` 2,
  `llm_extract_pipeline` 1, `persist_pipeline` 1) = 128 total. `cargo build` clean;
  `cargo clippy --workspace --all-targets` → 0. Records: [unit](unit-tests.md),
  [integration](integration-tests.md), [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- OpenAI-shape structured-parse error branch not directly unit-tested (symmetric to the tested
  Anthropic parser; happy + transport-error paths covered) — critique C-001, backlog.
- `from_env`'s file/`DIVER_PROVIDER` wiring covered via the pure `resolve_provider` helper, not
  end-to-end (needs disk + unsafe env mutation) — critique C-002.
- `response_format: json_schema` portability across every OpenAI-compatible target; function-
  calling `tools` is the recorded fallback contract — critique C-003.

## Coverage Observations
- Every acceptance criterion has a named, executed test asserting the SHALL response, including
  negative paths (non-2xx per shape; missing/malformed structured payload; missing provider key).
- Deterministic: fresh `MockServer` per transport test on a random port; config resolution via a
  pure helper with env/file injected — no global-env mutation; no real network or key.
- The OpenAI-shape transport test is, by construction, the offline proof for OpenAI, Grok, and
  Animus_Ferric (identical OpenAI-compatible contract; only `base_url` differs in production).
