# Sprint 7 Integration Tests

- **Tested head:** `df09a674f89676b92c3abdbc2f8384f27df8c5fe`
- **Runner:** `cargo test --workspace`
- **Result:** `diver-core/tests/extract_pipeline.rs` — 1 passed; 0 failed. Prior
  integration suites (`dive_pipeline` 1, `ingest_pipeline` 2) still pass.

## `test_extract_pipeline` (diver-core/tests/extract_pipeline.rs)
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) — AC5
- Composes the whole library pipeline through the public `diver_core::` API:
  `SourceFact` (3-sentence summary) → `extract_observations` (3 observations) →
  `candidate_assertions` (3 candidates) → `validate().ok()` collected into
  `Vec<Assertion<Supported>>` (3 supported).
- Asserts: 3 observations, 3 candidates, 3 supported; the first supported
  assertion's claim is the first sentence and its supporting observation's
  provenance is `ArxivId "2301.00001"` + `ArxivVersion(2)` — provenance survives
  end to end.
- **Executed:** pass.

## Raw result
```
Running tests\extract_pipeline.rs
running 1 test — test result: ok. 1 passed; 0 failed
```
