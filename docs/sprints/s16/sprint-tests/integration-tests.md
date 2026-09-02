# Sprint 16 Integration Tests

- **Tested head:** `bcf520f5204158e45125c89db0950df7d4d5acab`
- **Runner:** `cargo test --workspace`

## Real-corpus end-to-end (INT-0017) — `diver-core/tests/real_corpus.rs`
- **Intent:** [INT-0017](../../../intents/INT-0017-real-corpus-validation.md)
- `test_real_corpus_dive`: reads the committed `tests/fixtures/real_corpus_feed.xml` (7 genuine
  attention/NMT arXiv papers), then runs the whole pipeline **offline**:
  `parse::parse_feed` → `SourceFact::from_paper` + `Store::save` (in-memory) → deterministic
  extract (`candidate_assertions(&extract_observations(&fact))` → `validate` →
  `save_assertions`) → `compute_relations` + `compute_coassertion_relations`. Asserts on real
  content:
  - ≥ 2 papers; every paper produced ≥ 1 grounded assertion;
  - ≥ 1 structural edge (SharedCategory/SharedAuthor);
  - ≥ 1 weighted `CoAssertion` edge between two **distinct** papers at temperature 1.0, every
    weight finite in `[0.0, 1.0]`;
  - the temperature-0.5 co-assertion set ⊆ the 1.0 set (monotonic on real data);
  - `build_dive` over the "attention" seed yields a node whose `related` lists another paper.
  **pass.** (Assertions are on kinds/counts/invariants, not exact strings, so the test is robust.)

## Existing integration binaries (unchanged, all green)
- `coassertion` 2, `dive_graph` 1, `dive_pipeline` 1, `extract_pipeline` 1, `ingest_pipeline` 2,
  `llm_extract_pipeline` 1, `persist_pipeline` 1. **9 passed.**

## Fixture provenance
- `real_corpus_feed.xml`: the seven papers were ingested from the **live arXiv API** during the
  research probe (real titles, abstracts, authors, categories) and re-serialized offline into the
  Atom feed shape `parse_feed` consumes (arXiv was rate-limiting fresh raw captures at build
  time). The content is genuine; the test makes no network call.

## Raw result
```
Running tests\real_corpus.rs  →  test_real_corpus_dive ok
```
