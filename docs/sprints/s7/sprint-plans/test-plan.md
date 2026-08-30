Finalized - DO NOT EDIT

# Sprint 7 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC1: Observation + extractor with provenance | T-701 / WHEN summary has N sentences THEN N observations w/ id+version | `test_extract_observations_splits_sentences`, `test_observation_provenance`, `test_arxiv_version_parse` |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC2: typestate exists, no public `Supported` ctor | T-702 / (structural) | design review: private fields + no `Assertion::<Supported>::new`; crate builds with `validate` as only path |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC3: validate gates on support rule | T-702 / supported→Ok, empty→Err | `test_validate_supported`, `test_validate_rejects_unsupported` |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC4: `Supported` unconstructable outside validation | T-702 / (structural) | private fields + no public `Supported` ctor (documented); runtime gate proven by AC3 tests |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC5: `diver extract` pipeline + error path | T-703 / stored→display supported; unknown→error | `test_extract_pipeline` + e2e `extract --help` / unknown-id smokes |
| [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) | AC6: existing tests still pass | all tasks | full `cargo test --workspace` green |

## Unit Tests

### T-701 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md)
- `test_extract_observations_splits_sentences`: a `SourceFact` whose `summary` is
  "First finding here. Second finding follows. Third one too." → 3 `Observation`s
  with the expected texts.
- `test_observation_provenance`: each returned `Observation` carries the fact's
  `ArxivId` and parsed `ArxivVersion`.
- `test_arxiv_version_parse`: `ArxivVersion::parse("v2") == ArxivVersion(2)`;
  `parse("v10") == ArxivVersion(10)`; `parse("garbage") == ArxivVersion(1)`.
- Stubs: none.

### T-702 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md)
- `test_validate_supported`: `Assertion::<Candidate>::new(claim, vec![obs])`
  `.validate()` → `Ok`; the `Supported` value's `claim()`/`support()` match.
- `test_validate_rejects_unsupported`: `new(claim, vec![]).validate()` → `Err`,
  and the returned `Err` carries a `Candidate` whose `claim()` is unchanged.
- `test_candidate_assertions_from_observations`: N observations → N candidates,
  each `claim()` equal to its observation text and `support().len() == 1`.
- Stubs: none.

## Integration Tests

### Extract pipeline (`diver-core/tests/extract_pipeline.rs`)
- **Intents:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) (AC5)
- `test_extract_pipeline`: construct a `SourceFact` with a multi-sentence
  `summary`, then `extract_observations` → `candidate_assertions` →
  `validate().ok()` collected into `Vec<Assertion<Supported>>`; assert it is
  non-empty, the count matches the sentence count, and provenance (`ArxivId`,
  `ArxivVersion`) survives from fact to supported assertion. Imports via
  `diver_core::`.

## End-to-End Tests
- **Status:** possible
- `e2e_extract_help` (scripted smoke): `cargo run -p diver-cli -- extract --help`
  exits 0 and shows `<ARXIV_ID>`; `diver --help` lists `extract`. Side-effect-free
  (`--help` only). Proves the subcommand is wired.
- `e2e_extract_unknown_errors` (scripted smoke): `cargo run -p diver-cli --
  extract 9999.99999` exits non-zero and prints a "not found" message — proves the
  AC5 error path. Read-only DB access (schema init only), like `inspect`.
- Rationale for scripting over `assert_cmd`: no test-harness dependency added
  (consistent with prior sprints). Recorded in `e2e-tests.md` during the Test
  Phase.
