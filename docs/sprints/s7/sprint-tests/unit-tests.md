# Sprint 7 Unit Tests

- **Tested head:** `df09a674f89676b92c3abdbc2f8384f27df8c5fe`
- **Runner:** `cargo test --workspace` (no CI configured; local cargo is canonical)
- **Result:** `diver_core` lib unittests — **73 passed; 0 failed** (65 prior + 8
  new this sprint); `diver-cli` bin — 0 tests.

## New tests (INT-0008)

### T-701 — `Observation` + extractor (diver-core/src/observation.rs, id.rs)
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) — AC1
- **EARS:** N sentences → N observations w/ provenance; `ArxivVersion::parse("v2")`
  → `ArxivVersion(2)`, malformed → `ArxivVersion(1)`.
- `test_extract_observations_splits_sentences`: 3-sentence summary → 3
  observations with expected texts. **pass**
- `test_extract_observations_drops_trivial_fragments`: "Fig. 2." (short) is not
  emitted; only the substantial sentence survives. **pass**
- `test_observation_provenance`: each observation carries `ArxivId "2301.00001"`
  and parsed `ArxivVersion(3)`. **pass**
- `test_extract_observations_empty_summary`: empty summary → no observations. **pass**
- `test_arxiv_version_parse`: `v2→2`, `v10→10`, `garbage→1`, `v0→1`, `""→1`. **pass**

### T-702 — `Assertion` typestate (diver-core/src/assertion.rs)
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) — AC2/AC3/AC4
- **EARS:** non-empty support → `Ok(Supported)`; empty support → `Err(Candidate)`.
- `test_validate_supported`: candidate with one observation validates to
  `Supported`; claim + support preserved. **pass**
- `test_validate_rejects_unsupported`: candidate with empty support → `Err`, the
  returned candidate unchanged. **pass** (negative path)
- `test_candidate_assertions_from_observations`: N observations → N candidates,
  claim = observation text, `support().len() == 1`. **pass**

## AC2 / AC4 (compile-time gate) — structural evidence
`Assertion<Supported>` has **no** public constructor: `assertion.rs` gives it
only shared `impl<State>` accessors, while `new` and `validate` live in
`impl Assertion<Candidate>`; the struct fields are private. `validate` consumes a
`Candidate` and re-tags it, so it is the sole path to `Supported`. The workspace
builds with the `diver-cli` handler obtaining `Supported` only via
`candidate.validate().ok()` — the external happy path compiles, and no direct
construction exists to fail at runtime. (No `trybuild` compile-fail test added —
see plan critique C-001.)

## Raw result
```
Running unittests src\lib.rs (diver_core)
running 73 tests — test result: ok. 73 passed; 0 failed
Running unittests src\main.rs (diver-cli)
running 0 tests — test result: ok. 0 passed; 0 failed
```
