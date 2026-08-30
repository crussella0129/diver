Finalized - DO NOT EDIT

# Sprint 7 Build Plan

## Intents
- [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) — state: planned; acceptance criteria covered: AC1 (T-701), AC2/AC3/AC4 (T-702), AC5 (T-703), AC6 (full suite)

## Schema Tree
- Sprint Goal: typestate assertion core (Observation + Assertion<Candidate/Supported>)
  - Domain types (INT-0008)
    - T-701: `Observation` + deterministic extractor + `ArxivVersion::parse`
    - T-702: `Assertion<State>` typestate + `validate` transition
  - Demonstrator (INT-0008)
    - T-703: `diver extract` subcommand + display + pipeline integration test

## Execution Sequence

Bottom-up: build and test the domain types before the CLI consumes them, so each
type is independently verified and the CLI stays a thin consumer.

### T-701: `Observation` + deterministic extractor
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md)
- **Touches:** diver-core/src/observation.rs (new), diver-core/src/id.rs
  (`ArxivVersion::parse`), diver-core/src/lib.rs (`pub mod observation`)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0008 AC1 — `Observation` with provenance and a
  deterministic `extract_observations(&SourceFact)`.
- **Success criterion (EARS):**
  - **WHEN** `extract_observations` is called on a `SourceFact` whose `summary`
    holds N non-trivial sentences, **THEN** it **SHALL** return N `Observation`s,
    each carrying the fact's `ArxivId`, its `ArxivVersion`, and the sentence text.
  - **WHEN** `ArxivVersion::parse("v2")` is called, **THEN** it **SHALL** return
    `ArxivVersion(2)`, and a malformed value **SHALL** fall back to
    `ArxivVersion(1)`.
- **Notes:** `Observation { arxiv_id: ArxivId, version: ArxivVersion, text:
  String }`, `#[derive(Debug, Clone, PartialEq, Eq)]`. Split `summary` on `. `,
  `? `, `! ` boundaries; trim; drop fragments shorter than a small threshold
  (e.g. < 12 chars) so headers/initials do not become observations. Provenance
  from `SourceFact.arxiv_id` (→ `ArxivId::new`) and `arxiv_version` (→
  `ArxivVersion::parse`). Mirror `id.rs`'s existing newtype style.

### T-702: `Assertion` typestate + validation transition
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md)
- **Touches:** diver-core/src/assertion.rs (new), diver-core/src/lib.rs
  (`pub mod assertion`)
- **Depends on:** T-701
- **Acceptance criterion:** INT-0008 AC2 (typestate exists, no public `Supported`
  constructor), AC3 (validate gates on the support rule), AC4 (`Supported`
  unconstructable outside validation).
- **Success criterion (EARS):**
  - **WHEN** `Assertion<Candidate>::validate()` is called on a candidate with
    non-empty support, **THEN** it **SHALL** return `Ok(Assertion<Supported>)`.
  - **WHEN** `validate()` is called on a candidate with empty support, **THEN** it
    **SHALL** return `Err` carrying the original `Assertion<Candidate>`.
- **Notes:** zero-sized `Candidate`/`Supported` markers; `Assertion<State>` with
  private fields (`claim`, `support`, `PhantomData<State>`). `Assertion<Candidate>`
  gets `new` + `validate`; shared `impl<S>` accessors `claim()` / `support()`.
  `Assertion<Supported>` exposes **no** public constructor — `validate` is the
  only path (AC4, structural). Add
  `candidate_assertions(&[Observation]) -> Vec<Assertion<Candidate>>` (v1: one
  candidate per observation; claim = observation text; support = that observation).
  `is_supported` v1 rule: `!support.is_empty()`.

### T-703: `diver extract` subcommand + display + pipeline integration test
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md)
- **Touches:** diver-cli/src/main.rs (`Extract { arxiv_id }` variant + handler),
  diver-core/src/display.rs (`display_extract`),
  diver-core/tests/extract_pipeline.rs (new)
- **Depends on:** T-701, T-702
- **Acceptance criterion:** INT-0008 AC5 — `diver extract <arxiv_id>` runs the
  pipeline and displays supported assertions; unknown id errors.
- **Success criterion (EARS):**
  - **WHEN** `diver extract <arxiv_id>` runs for a stored paper, **THEN** it
    **SHALL** extract observations, build and validate candidate assertions, and
    display the supported assertions.
  - **WHEN** `diver extract <arxiv_id>` runs for an unknown id, **THEN** it
    **SHALL** exit with an error (like `inspect`'s `bail!`).
- **Notes:** handler mirrors `Commands::Inspect`: `Store::open` → `Store::get` →
  on `Some` run `extract_observations` → `candidate_assertions` → filter_map
  `validate().ok()` → `display::display_extract(&supported)`; on `None`
  `bail!("Paper not found: {arxiv_id}")`. `display_extract` mirrors `display_fact`
  (owo-colors, plain under non-tty). Integration test composes the full
  library pipeline to a non-empty `Vec<Assertion<Supported>>`, provenance intact.
