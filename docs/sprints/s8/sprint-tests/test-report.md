# Sprint 8 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC1: LlmExtractor calls Messages API → candidates | T-802 async HTTP + `test_llm_extract_pipeline` (parse→validate) | pass (post-response covered; socket manual) | Test evidence links this report; live run documented (critique C-001) |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC2: pure `parse_claims`, fixture-tested | T-801 / `test_parse_claims_grounded`, `_tolerates_fences`, `_tolerates_prose`, `_malformed_errors`, `_empty_array` | pass | Test evidence links this report |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC3: grounding drops hallucinations | T-801 / `test_parse_claims_drops_hallucinated`, `test_is_grounded_whitespace_and_case` | pass | Test evidence links this report |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC4: model default + key from env | T-802 / `test_build_missing_key_errors`, `test_build_model_default_and_override` | pass | Test evidence links this report |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC5: CLI default LLM + `--deterministic` + error | T-803 / `e2e_extract_help`, `e2e_extract_deterministic_no_key` | pass | Test evidence links this report |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC6: gate unchanged; existing tests pass | full suite | pass (86/86) | Test evidence links this report |

## Summary
- Unit tests: 82 passed / 0 failed / 82 total (`diver_core`); 0 in `diver-cli`
- Integration tests: 4 passed / 0 failed / 4 total (incl. new `llm_extract_pipeline`)
- E2E tests: 2 passed / 0 failed / 2 total (`extract --help`, `--deterministic` no-key)
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `4d73c28491ec40da62c0d61523f63fc2cc60d2cc`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` →
  `test result: ok` for every binary (`diver_core` lib 82, `dive_pipeline` 1,
  `extract_pipeline` 1, `llm_extract_pipeline` 1, `ingest_pipeline` 2, `diver-cli`
  bin 0). `cargo build` clean; `diver extract --help` shows `--deterministic`;
  `ANTHROPIC_API_KEY= diver extract 9999.99999 --deterministic` exits 1 "Paper not
  found" (no key needed). `cargo clippy --workspace --all-targets` — no new
  warnings in the new modules. Records: [unit](unit-tests.md),
  [integration](integration-tests.md), [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- The live HTTP call is not in the automated suite (networked, costs money,
  non-deterministic); covered by the parse→validate integration test + a
  documented manual run. A mocked-HTTP (`wiremock`) regression test is possible
  future hardening (critique C-001).
- Grounding is a conservative verbatim substring; semantic/fuzzy grounding is
  future work (critique C-002) — the current gate can only drop a real claim,
  never admit a hallucination.
- 7 pre-existing clippy warnings in `diver-core` remain (tracked since Sprint 6).

## Coverage Observations
- Every code-testable acceptance criterion has a named, executed test asserting
  the SHALL response, including negative paths (hallucination dropped, missing key,
  malformed model output, unknown id).
- The tested seam (`parse_claims`) is pure; non-determinism is confined to the
  socket, which is the only thing outside the suite.
- Secret hygiene: `LlmExtractor`'s `Debug` redacts the API key.
