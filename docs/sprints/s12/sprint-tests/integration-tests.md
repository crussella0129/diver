# Sprint 12 Integration Tests

- **Tested head:** `473197e821fdbd1a6b5106c5a810338c5caa2030`
- **Runner:** `cargo test --workspace`
- **Result:** `diver-core/tests/coassertion.rs` — 1 passed; 0 failed. Prior
  integration suites (`dive_graph` 1, `dive_pipeline` 1, `extract_pipeline` 1,
  `ingest_pipeline` 2, `llm_extract_pipeline` 1, `persist_pipeline` 1) still pass —
  8 integration tests total.

## `test_coassertion_pipeline` (diver-core/tests/coassertion.rs)
- **Intents:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md) (AC5, AC6)
- Two papers with **distinct categories and authors** (so the only possible link
  is epistemic), each with a persisted assertion whose claim contains "attention".
  Through the public `diver_core::` API:
  `compute_coassertion_relations(&store.all_claims())` yields exactly one
  `CoAssertion("attention")` edge between them, and `build_dive` for the
  "attention" seed lists Paper B under Paper A's related papers via that edge.
- This is the pipeline the `diver dive` handler now runs (structural +
  co-assertion edges unioned), proving the epistemic link surfaces where
  category/author would not.
- **Executed:** pass.

## Raw result
```
Running tests\coassertion.rs
running 1 test — test result: ok. 1 passed; 0 failed
```
