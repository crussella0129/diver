# Sprint 17 Integration Tests

- **Tested head:** `0aaa31cfeb486ccdee045905dc42f9fb7af24077`
- **Runner:** `cargo test --workspace`

## Regression guard (INT-0018) — `diver-core/tests/real_corpus.rs`
- **Intent:** [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) — AC3 (didn't empty the graph)
- `test_real_corpus_dive` (unchanged assertions) still passes after de-noising: the ≥1 weighted
  co-assertion edge it requires now lands on a surviving **technical** term (real terms like
  `attention`/`convolutional`/`encoder` survive; only generic filler was removed). This proves the
  stoplist tightened the graph without emptying it. **pass**
- `diver-core/tests/coassertion.rs` (2), `dive_graph.rs` (1), `dive_pipeline.rs` (1),
  `extract_pipeline.rs` (1), `ingest_pipeline.rs` (2), `llm_extract_pipeline.rs` (1),
  `persist_pipeline.rs` (1) — all green (coassertion's fixture updated `models`→`networks`).

## Raw result
```
Running tests\real_corpus.rs      →  test_real_corpus_dive ok
Running tests\coassertion.rs      →  2 passed
```
