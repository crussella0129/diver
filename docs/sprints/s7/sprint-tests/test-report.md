# Sprint 7 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC1: Observation + extractor with provenance | T-701 / `test_extract_observations_*`, `test_observation_provenance`, `test_arxiv_version_parse` | pass | Test evidence links this report |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC2: typestate exists, no public `Supported` ctor | T-702 / structural | pass (structural) | private fields + no public ctor; documented (critique C-001) |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC3: validate gates on support rule | T-702 / `test_validate_supported`, `test_validate_rejects_unsupported` | pass | Test evidence links this report |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC4: `Supported` unconstructable outside validation | T-702 / structural | pass (structural) | enforced by API shape; runtime gate proven by AC3 (critique C-001) |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC5: `diver extract` pipeline + error path | T-703 / `test_extract_pipeline` + e2e smokes | pass | Test evidence links this report |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC6: existing tests still pass | full suite | pass (77/77) | Test evidence links this report |

## Summary
- Unit tests: 73 passed / 0 failed / 73 total (`diver_core`); 0 in `diver-cli`
- Integration tests: 4 passed / 0 failed / 4 total (incl. new `extract_pipeline`)
- E2E tests: 2 passed / 0 failed / 2 total (`extract --help`, unknown-id error)
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `df09a674f89676b92c3abdbc2f8384f27df8c5fe`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local confirmations only. `cargo test
  --workspace` reported `test result: ok` for every binary (`diver_core` lib 73,
  `dive_pipeline` 1, `extract_pipeline` 1, `ingest_pipeline` 2, `diver-cli` bin 0,
  doc-tests 0). `cargo build` clean; `diver --help` lists `extract`; `diver
  extract 9999.99999` exits 1 with "Paper not found". `cargo clippy --workspace
  --all-targets` produced only the pre-existing warnings (no new ones in the new
  modules). Records: [unit](unit-tests.md), [integration](integration-tests.md),
  [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- AC2/AC4 (compile-time gate) lack an automated compile-fail test; a `trybuild`
  ui test is deferred (critique C-001). If the assertion API grows public surface,
  add one.
- The v1 support rule (`!support.is_empty()`) is intentionally trivial; refining
  it (and adding LLM-sourced observations) is future work that plugs into the same
  `validate` gate.
- 7 pre-existing clippy warnings in `diver-core` remain (tracked from Sprint 6;
  out of scope).

## Coverage Observations
- Every code-testable acceptance criterion has a named, executed test asserting
  the SHALL response, including negative paths (empty-support rejection,
  malformed version → v1, unknown-id CLI error, empty summary).
- Tests are deterministic: fixed in-memory inputs, no network/clock/randomness;
  E2E smokes are `--help` / unknown-id (read-only).
- AC2/AC4 are non-runtime criteria evidenced by API design + the crate compiling
  with `validate` as the only path to `Supported`.
