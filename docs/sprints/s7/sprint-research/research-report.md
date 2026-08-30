# Sprint 7 Research Report

## Intents Reviewed
- [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) — created; relevance: primary; current state: proposed

## 1. Sprint Goal

Build the first layer of Diver's epistemic engine on top of the hardened
`SourceFact` substrate: a typestate assertion core in `diver-core`. Introduce an
`Observation` type (a deterministically-extracted, provenance-carrying unit of
what a paper said), an `Assertion<Candidate>`/`Assertion<Supported>` typestate
whose `Supported` variant is reachable only through a validation transition, a
deterministic `extract_observations(&SourceFact)` over stored abstracts, and a
minimal `diver extract <arxiv_id>` subcommand demonstrating the pipeline. The
value is the type architecture — making "unvalidated content" un-representable at
downstream boundaries — not sophisticated extraction; the extractor and support
rule are deliberately simple and LLM-free this sprint.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/fact.rs | high | `SourceFact` is the extraction source: `summary` (abstract), `title`, `authors`, `categories`, `arxiv_id`/`arxiv_version` (Strings). `extract_observations` reads `summary` and provenance fields. |
| diver-core/src/id.rs | high | `ArxivId` (String newtype, `new`/`as_str`) and `ArxivVersion(u32)` (Display `vN`) — reuse for `Observation` provenance. No `ArxivVersion::parse("v2")` helper yet; add one or parse the leading `v` inline. |
| diver-core/src/lib.rs | high | Module list; add `pub mod observation;` and `pub mod assertion;`. |
| diver-core/src/store.rs | medium | `Store::get(arxiv_id) -> Option<SourceFact>` feeds the `extract` command; no schema change this sprint. |
| diver-core/src/display.rs | medium | Home for an assertion/observation renderer (`display_extract`), mirroring `display_fact`. Uses `owo-colors`; tests run without a tty so color is disabled. |
| diver-cli/src/main.rs | high | `Commands` enum + match; add an `Extract { arxiv_id }` variant mirroring `Inspect` (load via `Store::get`, `bail!` on missing). |
| diver-core/src/model.rs | low | `Paper` — upstream of `SourceFact`; not touched. |
| diver-core/src/parse.rs | low | Abstract text arrives via parse→SourceFact; sentence-splitting for extraction is new code in `observation.rs`. |

Baseline: workspace at `29bb5d5` (post-merge). `cargo test --workspace` green
(65 unit + 3 integration). `SourceFact.summary` holds the abstract text to
extract from.

### Typestate design (Rust)

```rust
// assertion.rs
use std::marker::PhantomData;

pub struct Candidate;   // zero-sized typestate markers
pub struct Supported;

pub struct Assertion<State> {
    claim: String,
    support: Vec<Observation>,   // private fields
    _state: PhantomData<State>,
}

impl Assertion<Candidate> {
    pub fn new(claim: impl Into<String>, support: Vec<Observation>) -> Self { /* ... */ }
    /// The ONLY constructor of Assertion<Supported>.
    pub fn validate(self) -> Result<Assertion<Supported>, Assertion<Candidate>> {
        if self.is_supported() { Ok(/* re-tag */) } else { Err(self) }
    }
    fn is_supported(&self) -> bool { !self.support.is_empty() } // deterministic v1 rule
}

impl<S> Assertion<S> { pub fn claim(&self) -> &str; pub fn support(&self) -> &[Observation]; }
// Assertion<Supported> exposes accessors only — no public constructor.
```

`Assertion<Supported>` is unconstructable outside the module because its fields
are private and no public `Supported` constructor exists; `validate` re-tags an
existing `Candidate`. A downstream `fn build_graph(a: Assertion<Supported>)`
cannot be called with a candidate — a compile error, not a runtime check.

## 3. External Sources
- [Typestate pattern in Rust](https://cliffle.com/blog/rust-typestate/) — zero-sized state markers + `PhantomData`; transitions consume `self` and return the new-state value.
- [`std::marker::PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) — carry a type parameter that is otherwise unused, with no runtime cost.
- [trybuild](https://docs.rs/trybuild) — optional compile-fail test harness to prove AC4 (constructing `Assertion<Supported>` directly fails to compile); a dev-dependency, weighed in §4.

## 4. Risks, Unknowns, Dependencies

- **Risk:** proving AC4 (the compile-time guarantee). A `trybuild` ui test is the
  standard proof but adds a dev-dependency and can be brittle across compiler
  versions (error text drift). Mitigation: make the guarantee structural (private
  fields + no public `Supported` constructor + `validate` as sole path) and prove
  the *runtime* gate with unit tests; treat a `trybuild` compile-fail test as an
  optional strengthening decided at plan time, not a hard requirement.
- **Risk:** naive sentence-splitting mis-splits abstracts (e.g., "et al.",
  "Fig. 2", "e.g."). Mitigation: v1 splits on `. ` / `? ` / `! ` followed by a
  capital and trims; observations are provenance units, not NLP claims, so minor
  mis-splits are acceptable and covered by a fixed-input test.
- **Risk:** `ArxivVersion` parse. `SourceFact.arxiv_version` is `"v2"` (String)
  but `ArxivVersion(u32)` wants `2`. Mitigation: add `ArxivVersion::parse("vN")`
  (or strip the leading `v`) in `id.rs`; a bad value falls back to `v1`, matching
  `fact.rs::parse_arxiv_id`.
- **Unknown:** what counts as a "candidate assertion" deterministically. v1: each
  extracted `Observation` seeds one `Assertion<Candidate>` whose claim is the
  observation text and whose support is that observation. The support rule
  (`is_supported`) is `!support.is_empty()`, so every extracted observation
  yields a supported assertion — trivial but real; refinement is a later sprint.
- **Dependency:** the `extract` command depends only on `Store::get` (existing);
  no schema or network change.

## 5. Recommended Approach

Primary: implement the type core bottom-up, then wire the CLI.

- `observation.rs`: `Observation { arxiv_id: ArxivId, version: ArxivVersion, text:
  String }` + `extract_observations(&SourceFact) -> Vec<Observation>` (sentence
  split of `summary`, drop trivial fragments, tag each with the fact's id/version).
- `assertion.rs`: `Candidate`/`Supported` markers, `Assertion<State>` with private
  fields, `Assertion<Candidate>::new` + `validate`, and shared accessors. Add
  `candidate_assertions(&[Observation]) -> Vec<Assertion<Candidate>>`.
- `id.rs`: `ArxivVersion::parse` helper.
- `display.rs`: `display_extract(&[Assertion<Supported>])`.
- `diver-cli`: `Extract { arxiv_id }` subcommand mirroring `Inspect`.

Tests: extraction + provenance; `validate` success (supported) and failure
(empty-support candidate stays `Candidate` / returns `Err`); an `extract`
integration or E2E `--help`/round-trip smoke.

Alternative considered: two separate structs instead of a generic typestate —
rejected per the intent (vision specifies `Assertion<Candidate> →
Assertion<Supported>`).

Rationale: bottom-up keeps each type independently testable and the CLI a thin
consumer; deterministic v1 settles the gate before any LLM extractor is added.

## Artifacts
- No standalone snippet files; the design sketch is inline in §2.
