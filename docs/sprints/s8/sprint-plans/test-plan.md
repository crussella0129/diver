Finalized - DO NOT EDIT

# Sprint 8 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC1: LlmExtractor calls Messages API → candidates | T-802 / async HTTP + T-801 parse | `test_llm_extract_pipeline` (parse→validate) + documented manual live run |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC2: pure `parse_claims`, fixture-tested | T-801 / grounded body → candidates | `test_parse_claims_grounded`, `test_parse_claims_tolerates_fences`, `test_parse_claims_malformed_errors` |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC3: grounding drops hallucinations | T-801 / quote∉abstract → dropped | `test_parse_claims_drops_hallucinated` |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC4: model default + key from env | T-802 / missing key → err; DIVER_MODEL else opus-5 | `test_build_missing_key_errors`, `test_build_model_default_and_override` |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC5: CLI default LLM + `--deterministic` + error | T-803 / flag branches; unknown→err | e2e `extract --help` + `extract --deterministic` offline + unknown-id error |
| [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) | AC6: gate unchanged; existing tests pass | all tasks | full `cargo test --workspace` green |

## Unit Tests

### T-801 unit tests (diver-core/src/extract.rs)
- **Intent:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md)
- `test_parse_claims_grounded`: a fixture body with one claim whose quote is a
  substring of the fact's summary → 1 candidate; `claim()` and support
  `Observation.text()` + provenance (`ArxivId`, `ArxivVersion`) correct.
- `test_parse_claims_drops_hallucinated`: a claim whose quote is absent from the
  summary → dropped (0 candidates, or only the grounded ones from a mixed body).
- `test_parse_claims_tolerates_fences`: model text wrapped in ```json … ``` and/or
  leading prose → still parsed.
- `test_parse_claims_malformed_errors`: non-JSON / missing `content` → `Err`, no
  panic.

### T-802 unit tests (diver-core/src/extract.rs)
- **Intent:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md)
- `test_build_missing_key_errors`: `build(None, _)` and `build(Some(""), _)` →
  `Err` whose message contains `ANTHROPIC_API_KEY`.
- `test_build_model_default_and_override`: `build(Some("k"), None)` → model
  `claude-opus-5`; `build(Some("k"), Some("claude-haiku-4-5"))` → that model.
- Stubs: none (no global env mutation — logic tested through `build`).

## Integration Tests

### LLM extract pipeline (diver-core/tests/llm_extract_pipeline.rs)
- **Intents:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) (AC1 via parse→validate, AC6)
- `test_llm_extract_pipeline`: a fixture Messages-API JSON body with two claims —
  one grounded (quote ∈ abstract), one hallucinated (quote ∉ abstract) —
  constructed against a `SourceFact` → `parse_claims` → `.filter_map(validate().ok())`
  → `Vec<Assertion<Supported>>` of length 1; the surviving assertion's claim and
  provenance (`ArxivId`, `ArxivVersion`) are correct. Imports via `diver_core::`.

## End-to-End Tests
- **Status:** possible (offline paths in the suite; live LLM is manual)
- `e2e_extract_help` (scripted smoke): `cargo run -p diver-cli -- extract --help`
  exits 0 and shows `<ARXIV_ID>` and `--deterministic`.
- `e2e_extract_deterministic` (scripted smoke): with `ANTHROPIC_API_KEY` **unset**,
  `cargo run -p diver-cli -- extract 9999.99999 --deterministic` makes no network
  call and errors "not found" (exit 1) — proves the offline branch and error path.
- **Live LLM run (manual, not in suite):** `ANTHROPIC_API_KEY=… cargo run -p
  diver-cli -- extract <stored-id>` prints grounded supported assertions. Excluded
  from the automated suite: non-deterministic, networked, and costs money. Recorded
  as a manual verification note in `e2e-tests.md`.
