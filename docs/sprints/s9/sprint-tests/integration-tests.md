# Sprint 9 Integration Tests

- **Tested head:** `cb613a38d985818e08ca2ed22b8781fa28411424`
- **Runner:** `cargo test --workspace`
- **Result:** `diver-core/tests/persist_pipeline.rs` — 1 passed; 0 failed. Prior
  integration suites (`dive_pipeline` 1, `extract_pipeline` 1, `ingest_pipeline` 2,
  `llm_extract_pipeline` 1) still pass — 6 integration tests total.

## `test_persist_pipeline` (diver-core/tests/persist_pipeline.rs)
- **Intents:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) (AC5, AC6)
- The full persist loop through the public `diver_core::` API: `Store::open_in_memory`
  → `store.save(&fact)` (ingest a paper) → `parse_claims`(fixture Messages body
  grounded in the abstract) → `validate` → `store.save_assertions` →
  `store.get_assertions`.
- Asserts the read-back returns 1 `StoredAssertion` with the right claim
  (`"Attention improves accuracy."`), version (`v2`), and support quote — the same
  extract→validate→persist→read path the `diver extract` / `diver assertions`
  commands run.
- **Executed:** pass.

## Raw result
```
Running tests\persist_pipeline.rs
running 1 test — test result: ok. 1 passed; 0 failed
```
