# Sprint 9 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC1: schema tables created idempotently | T-901 / `test_assertion_schema_created` | pass | Test evidence links this report |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC2: save persists claim + support (validated-only) | T-901 / `test_save_assertions_persists` | pass | Test evidence links this report |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC3: idempotent replace per (paper,version) | T-901 / `test_save_assertions_idempotent_replace` | pass | Test evidence links this report |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC4: get returns stored data; empty for unknown | T-902 / `test_get_assertions_round_trip`, `test_get_assertions_unknown_empty` | pass | Test evidence links this report |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC5: extract persists; `diver assertions` displays | T-903 / `test_persist_pipeline` + e2e smokes | pass (binary round-trip via library test, critique C-001) | Test evidence links this report |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC6: FKs enforced; existing tests pass | T-901 / `test_assertion_support_fk_enforced` + full suite | pass (94/94) | Test evidence links this report |

## Summary
- Unit tests: 88 passed / 0 failed / 88 total (`diver_core`); 0 in `diver-cli`
- Integration tests: 6 passed / 0 failed / 6 total (incl. new `persist_pipeline`)
- E2E tests: 2 passed / 0 failed / 2 total (`assertions --help`, unknown-id)
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `cb613a38d985818e08ca2ed22b8781fa28411424`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` →
  `test result: ok` for every binary (`diver_core` lib 88, `dive_pipeline` 1,
  `extract_pipeline` 1, `ingest_pipeline` 2, `llm_extract_pipeline` 1,
  `persist_pipeline` 1, `diver-cli` bin 0). `cargo build` clean; `diver --help`
  lists `assertions`; `diver assertions 9999.99999` prints "No stored assertions"
  and exits 0. `cargo clippy --workspace --all-targets` — no new warnings in the
  new code. Records: [unit](unit-tests.md), [integration](integration-tests.md),
  [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- The on-disk binary persist→read round-trip is not in the automated suite (needs
  a seeded DB); covered by `test_persist_pipeline` at the library level (critique
  C-001). A fixture-DB binary test is possible future hardening.
- `get_assertions` returns assertions across all versions of a paper (each carries
  its version); a latest-version display filter is a future refinement (plan
  critique C-002).
- 7 pre-existing clippy warnings in `diver-core` remain (tracked since Sprint 6).

## Coverage Observations
- Every acceptance criterion has a named, executed test asserting the SHALL
  response, including negative paths (FK violation, unknown-id empty, idempotent
  replace with cascade).
- Tests are deterministic (in-memory SQLite, fixtures); E2E smokes are offline.
- Storage gate: `save_assertions(&[Assertion<Supported>])` extends the compile-time
  typestate guarantee to persistence — the DB holds only validated knowledge.
