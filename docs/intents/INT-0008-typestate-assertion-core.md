# INT-0008 — Typestate assertion core

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0008
- **State:** realized
- **Work evidence:** [Sprint 7 build plan](../sprints/s7/sprint-plans/build-plan.md) (T-701, T-702, T-703)
- **Completion evidence:** [T-701/T-702/T-703 completion](../work/completed-tasks.md#t-701--sprint-7)
- **Code evidence:** [diver-core/src/observation.rs](../../diver-core/src/observation.rs), [diver-core/src/assertion.rs](../../diver-core/src/assertion.rs), [diver-core/src/display.rs](../../diver-core/src/display.rs), [diver-cli/src/main.rs](../../diver-cli/src/main.rs)
- **Test evidence:** [Sprint 7 test report](../sprints/s7/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md) (`diver extract` command)

## Intent

Establish the epistemic type architecture that the hardened substrate
([[reconcile-review-hardening]] and INT-0005) was built to support. In
`diver-core`, introduce:

- **`Observation`** — a deterministically-extracted unit of what a source paper
  said, carrying provenance: the paper's `ArxivId`, its `ArxivVersion`, and the
  observed text (a sentence from the abstract).
- **`Assertion<Candidate>` and `Assertion<Supported>`** — a single assertion type
  parameterized by a typestate marker, where an `Assertion<Supported>` can be
  produced **only** by validating an `Assertion<Candidate>`. Downstream consumers
  that require `Assertion<Supported>` therefore cannot be handed an unvalidated
  assertion.
- **A deterministic extractor** `extract_observations(&SourceFact) -> Vec<Observation>`
  over stored abstracts (no LLM, no network).
- **The `Candidate → Supported` validation transition** with a deterministic
  support rule.
- **A minimal `diver extract <arxiv_id>` subcommand** that demonstrates the
  pipeline over an already-stored paper.

Non-goals:
- No LLM or network extraction — the extractor is deterministic and rule-based.
- No graph layer: no `ComputedRelation`, no `diver dive` traversal.
- No persistence of observations or assertions to SQLite — compute in memory over
  already-stored `SourceFact`s. An assertion storage schema is a later sprint.
- No `diver-domain` crate yet — the types live in `diver-core` (grow into the
  crate split later, per the architecture vision).
- No TypeScript / UI.

## Acceptance criteria

1. `diver-core` exposes `Observation` with provenance (`ArxivId`, `ArxivVersion`,
   text) and a deterministic `extract_observations(&SourceFact)` that yields one
   `Observation` per non-trivial sentence of the abstract, each linked to the
   source paper's id and version.
2. `Assertion<Candidate>` and `Assertion<Supported>` exist as one typestate-
   parameterized type; `Assertion<Supported>` has **no** public constructor other
   than the validation transition.
3. `Assertion<Candidate>::validate(...)` yields a `Supported` assertion when the
   deterministic support rule is met, and reports failure (returns the candidate
   or an error) when it is not — proven by tests of both paths.
4. Code outside the assertion module cannot construct an `Assertion<Supported>`
   except through validation (enforced by private fields + no public `Supported`
   constructor; the crate builds with `validate` as the only path).
5. `diver extract <arxiv_id>` loads a stored paper, extracts observations, builds
   candidate assertions, validates them, and displays the supported ones; an
   unknown id errors the way `inspect` does.
6. All previously passing tests still pass; new unit tests cover extraction and
   provenance, the successful validation transition, and the failing one.

## Rationale

The architecture vision names the typestate assertion system as the project's
"interesting IP": making epistemic integrity a **compile-time** property so
unvalidated content cannot flow into the downstream graph and synthesis layers.
INT-0005 hardened the `SourceFact` substrate specifically so this layer could be
built on provenance-safe ground. Building the types now — deterministically,
before any LLM extractor — settles the architecture cheaply; a later LLM-backed
extractor simply produces `Observation`s and `Assertion<Candidate>`s that flow
through the same validated gate without changing it.

## Alternatives

- **Two distinct structs `CandidateAssertion` / `SupportedAssertion`** — rejected:
  the vision specifies `Assertion<Candidate> → Assertion<Supported>`; the generic
  typestate keeps one type with shared behavior while making the states distinct
  and the transition explicit.
- **Start with the LLM extractor** — rejected for this sprint: it couples the type
  architecture to an external API, network, and cost surface before the types are
  settled. Deterministic-first lets the type gate stabilize.
- **Persist assertions now** — rejected: the storage schema for observations and
  assertions is a separate concern; compute over already-stored `SourceFact`s
  first.

## Consequences

- New `diver-core` modules (`observation`, `assertion`); `diver-cli` gains an
  `extract` subcommand and `diver-core`'s display layer gains an assertion
  renderer.
- The deterministic support rule is intentionally simple in v1; later sprints
  refine it and add LLM-sourced observations **without** changing the typestate
  gate.
- `Assertion<Supported>` becomes the currency required by future graph builders,
  so an unvalidated assertion is un-representable at those boundaries.

## Transition history
- 2026-08-30: created as `proposed`.
- 2026-08-30: `proposed` → `planned`; linked to Sprint 7 build plan (T-701
  Observation + extractor, T-702 Assertion typestate, T-703 `diver extract`).
- 2026-08-30: `planned` → `active` (Sprint 7 build started; T-701 first).
- 2026-08-30: `active` → `realized` (Sprint 7: `Observation` + deterministic
  extractor, `Assertion<Candidate>`/`Assertion<Supported>` with validation-only
  transition to `Supported`, and `diver extract`; 77 tests pass. AC2/AC4
  compile-time gate evidenced structurally — see test report / critique C-001).
