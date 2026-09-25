# Sprint 19 Integration Tests

- **Tested head:** `d08eaa61cbccb32a468e72b06b3588874a638637`
- **Runner:** `cargo test --workspace`
- **Result:** every binary green — **163 passed, 0 failed** workspace-wide.

## `diver-core/tests/concepts.rs` (new) — [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- `test_concepts_pipeline` — **pass**. On a real on-disk store via `Store::open_at`:
  `networks` and `network` claims → `papers_asserting("networks")` returns both papers;
  one concept-keyed co-assertion edge links them and `build_dive` lists each as the
  other's related paper; after dropping and reopening the file the concept still
  resolves, and a third paper saved afterwards is counted with no rebuild call. (That
  reads after a reopen use *persisted* concepts rather than rebuilding is asserted by the
  converse check in `test_concepts_freshness`.) (AC1, AC2, AC5, AC7; also
  composes INT-0019's `open_at`).
- `test_real_corpus_concepts` — **pass**. The checked-in 7-paper fixture, offline:
  `machine translation` is a phrase in 2+ papers; `network` carries both inflections;
  `papers_asserting("networks")` includes *"We propose a new simple network
  architecture…"*, which substring matching missed; phrase papers ⊆ term papers on real
  claims; formation forward == reversed after index remapping (AC2, AC3, AC4, AC5
  precondition).

## Pre-existing integration binaries (regression guard)
All unmodified, all green, now exercising concept resolution and concept-keyed edges:
`coassertion.rs` (2), `dive_graph.rs` (1), `dive_pipeline.rs` (1), `extract_pipeline.rs` (1),
`ingest_pipeline.rs` (2), `llm_extract_pipeline.rs` (1), `persist_pipeline.rs` (1),
`real_corpus.rs` (1). CLI: `db_override.rs` (2).

## Raw result
```
diver_core lib 146 · diver-cli bin 1 · db_override 2 · dive_concepts 2 · coassertion 2
concepts 2 · dive_graph 1 · dive_pipeline 1 · extract_pipeline 1 · ingest_pipeline 2
llm_extract_pipeline 1 · persist_pipeline 1 · real_corpus 1 · doc-tests 0
= 163 passed; 0 failed
```
