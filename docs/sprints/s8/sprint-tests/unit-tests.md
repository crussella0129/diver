# Sprint 8 Unit Tests

- **Tested head:** `4d73c28491ec40da62c0d61523f63fc2cc60d2cc`
- **Runner:** `cargo test --workspace` (no CI; local cargo is canonical)
- **Result:** `diver_core` lib unittests — **82 passed; 0 failed** (73 prior + 9
  new); `diver-cli` bin — 0 tests.

## New tests (INT-0009)

### T-801 — `parse_claims` + grounding (diver-core/src/extract.rs)
- **Intent:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) — AC2/AC3
- `test_parse_claims_grounded`: a Messages-API body with a claim whose quote is in
  the abstract → 1 candidate; claim text + supporting `Observation` provenance
  (`ArxivId 2301.00001`, `ArxivVersion(2)`) correct. **pass**
- `test_parse_claims_drops_hallucinated`: mixed body (one grounded, one fabricated
  quote) → only the grounded claim survives. **pass** (grounding gate)
- `test_parse_claims_tolerates_fences`: ```json … ``` wrapped output parses. **pass**
- `test_parse_claims_tolerates_prose`: JSON embedded in prose parses (outer-array
  slice fallback). **pass**
- `test_parse_claims_empty_array`: `[]` → no candidates. **pass**
- `test_parse_claims_malformed_errors`: non-JSON envelope, missing text block, and
  non-array text each → `Err`, no panic. **pass** (negative paths)
- `test_is_grounded_whitespace_and_case`: normalized case/whitespace substring;
  empty quote is never grounded. **pass**

### T-802 — `LlmExtractor` config (diver-core/src/extract.rs)
- **Intent:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) — AC4
- `test_build_missing_key_errors`: `build(None, _)` and `build(Some("   "), _)` →
  `Err` naming `ANTHROPIC_API_KEY`. **pass** (negative path)
- `test_build_model_default_and_override`: default `claude-opus-5`; `DIVER_MODEL`
  override honored; blank model falls back to default. **pass**

## Secret hygiene
`LlmExtractor` holds the API key but has a **manual `Debug` impl that redacts it**
(`api_key: "<redacted>"`), so the key never appears in debug output.

## Raw result
```
Running unittests src\lib.rs (diver_core)
running 82 tests — test result: ok. 82 passed; 0 failed
```
