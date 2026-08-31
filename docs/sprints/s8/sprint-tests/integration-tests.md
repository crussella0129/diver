# Sprint 8 Integration Tests

- **Tested head:** `4d73c28491ec40da62c0d61523f63fc2cc60d2cc`
- **Runner:** `cargo test --workspace`
- **Result:** `diver-core/tests/llm_extract_pipeline.rs` — 1 passed; 0 failed.
  Prior integration suites (`dive_pipeline` 1, `extract_pipeline` 1,
  `ingest_pipeline` 2) still pass.

## `test_llm_extract_pipeline` (diver-core/tests/llm_extract_pipeline.rs)
- **Intents:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) (AC1 via parse→validate, AC6)
- Feeds a fixture **Messages-API JSON body** (the exact response shape the live
  call returns) whose model output asserts two claims — one grounded in the
  abstract, one hallucinated — through the full library pipeline:
  `parse_claims` → `.filter_map(validate().ok())` → `Vec<Assertion<Supported>>`.
- Asserts: the hallucinated claim is dropped (1 candidate), it validates to 1
  supported assertion, the claim text is correct, and provenance
  (`ArxivId 2301.00001`, `ArxivVersion(2)`) survives end to end.
- This is the automated evidence for AC1 short of the live HTTP call (see e2e /
  critique C-001): it exercises everything from the API response body onward.
- **Executed:** pass.

## Raw result
```
Running tests\llm_extract_pipeline.rs
running 1 test — test result: ok. 1 passed; 0 failed
```
