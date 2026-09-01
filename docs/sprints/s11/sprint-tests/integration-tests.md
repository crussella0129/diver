# Sprint 11 Integration Tests

- **Tested head:** `de281c76affc76e696f03f55331b8ae3de0c8aeb`
- **Runner:** `cargo test --workspace`
- **Result:** `diver-core/tests/dive_graph.rs` — 1 passed; 0 failed. Prior
  integration suites (`dive_pipeline` 1, `extract_pipeline` 1, `ingest_pipeline` 2,
  `llm_extract_pipeline` 1, `persist_pipeline` 1) still pass — 7 integration tests.

## `test_dive_pipeline` (diver-core/tests/dive_graph.rs)
- **Intents:** [INT-0012](../../../intents/INT-0012-graph-dive.md) (AC4, AC5)
- The full dive loop through the public `diver_core::` API: save two papers sharing
  the `cs.CL` category → persist a supported assertion for Paper A whose claim
  mentions the concept → `store.papers_asserting("attention")` +
  `compute_relations(&store.list())` + `build_dive` → a single `DiveNode` for
  Paper A carrying its claim and a `SharedCategory("cs.CL")` relation whose other
  endpoint is Paper B.
- Exercises exactly the pipeline the `diver dive` handler runs (minus the on-disk
  store), from persisted assertions through the neighborhood assembly.
- **Executed:** pass.

## Raw result
```
Running tests\dive_graph.rs
running 1 test — test result: ok. 1 passed; 0 failed
```
