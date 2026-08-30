# Sprint 6 Integration Tests

- **Tested head:** `6b4dbef6fcc81bbb187a45d6a5e533a3e26845c3`
- **Runner:** `cargo test --workspace`
- **Result:** `diver-core/tests/dive_pipeline.rs` — 1 passed; 0 failed.
  `diver-core/tests/ingest_pipeline.rs` — 2 passed; 0 failed.

These pre-existing library-pipeline integration tests were **moved into
`diver-core/tests/`** and repointed from `use diver::` to `use diver_core::`
during T-604. Their passing under the new layout is the primary evidence for
INT-0007 AC6.

## `test_find_pipeline` (dive_pipeline.rs)
- **Intent:** [INT-0007](../../../intents/INT-0007-workspace-restructure.md) — AC6
- Saves three facts, then asserts `search("attention")` returns the expected
  ids, `max_results` is honored, and an unknown term returns empty. Compiles
  under `use diver_core::`.
- **Executed:** pass.

## `test_ingest_pipeline` (ingest_pipeline.rs)
- **Intent:** [INT-0007](../../../intents/INT-0007-workspace-restructure.md) — AC6
- Reads `tests/fixtures/sample_feed.xml` (relative path resolved with CWD =
  `diver-core/` package root), parses → `extract_paper` → `SourceFact::from_paper`
  → `store.save` → `store.get` round-trip. Proves the fixtures moved with the
  tests and the runtime path resolves post-split.
- **Executed:** pass.

## `test_ingest_pipeline_multi_category` (ingest_pipeline.rs)
- **Intent:** [INT-0007](../../../intents/INT-0007-workspace-restructure.md) — AC6
- Parsed paper preserves both `cs.CL` and `cs.AI` categories.
- **Executed:** pass.

## Note on `diver-core`'s compile-time fixture coupling
The `diver_core` **unit** build also depends on `tests/fixtures/`:
`src/parse.rs` (`#[cfg(test)]`) uses
`include_str!("../tests/fixtures/{sample_feed,empty_feed}.xml")`, and
`src/client.rs`'s unit test reads `tests/fixtures/sample_feed.xml` at runtime.
All 65 unit tests plus these 3 integration tests compiling and passing confirms
the entire fixtures directory landed at `diver-core/tests/fixtures/`.

## Raw result
```
Running tests\dive_pipeline.rs
running 1 test — test result: ok. 1 passed; 0 failed
Running tests\ingest_pipeline.rs
running 2 tests — test result: ok. 2 passed; 0 failed
```
