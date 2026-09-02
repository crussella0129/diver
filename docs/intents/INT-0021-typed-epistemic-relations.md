# INT-0021 — Typed epistemic relations between claims

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0021
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Every edge Diver computes today is paper-to-paper: shared category, shared
author, shared claim term ([[graph-dive]], INT-0012; [[coassertion-relations]],
INT-0013). None of them says anything about *what the papers assert*. Two papers
that flatly contradict each other and two that merely share vocabulary produce
the same edge.

Introduce the first claim-to-claim edge, carrying an epistemic type:

- **A relation vocabulary**, deliberately small at first. Start with the types the
  evidence can actually distinguish — `supports`, `contradicts`, `refines`,
  `uses` — and add `generalizes`, `replicates`, `derives-from` only when
  [[relation-evaluation-harness]] (INT-0022) shows they can be told apart.
  A vocabulary is a liability before it is measurable.
- **Machine proposes, deterministic gate disposes.** A model proposes candidate
  relations; each candidate must carry evidence spans from both claims' source
  text; the spans must ground (occur verbatim in their sources) exactly as claim
  quotes do today. Reuse the typestate pattern from `assertion.rs` — a
  `Relation<Candidate>` promotable only through a validating constructor, so an
  ungrounded relation cannot be persisted by a programmer who forgot to check.
- **Confidence and provenance are mandatory fields, not optional decoration.**
  Every persisted relation records what proposed it (model, prompt version,
  extraction version), what evidence grounded it, and how confident the proposal
  was. There is no path to a bare, unattributed edge.
- **Disagreement is represented, never resolved.** If five claims support X and
  three contradict it, all eight edges persist. Nothing in this layer adjudicates,
  scores a winner, or collapses a dispute into a verdict.

Non-goals:
- No natural-language summary of what the literature "says". That is a view over
  this object, and building the view before the object is the failure mode this
  intent exists to avoid.
- No change to the claim extraction schema *unless* the shape question below is
  settled first.
- No automatic relation inference (transitive closure, "A supports B supports C
  therefore A supports C"). Epistemic relations are not transitive.

## Acceptance criteria

1. A persisted relation names two claims, a type from the vocabulary, an evidence
   span per side, a confidence, and its extraction provenance.
2. A relation whose evidence spans do not ground in their source text cannot be
   persisted — enforced by the type system, and proven by a test that tries.
3. `diver dive` can show, for a concept, the claims asserted about it and the
   typed relations among them, with every edge traceable to its evidence.
4. Contradictory relations coexist: a fixture where claims both support and
   contradict the same claim persists all edges and displays the disagreement.
5. Relation quality is reported by INT-0022's harness, not asserted in prose.
   This intent is not `realized` on a working pipeline alone — it needs measured
   agreement against a gold set.

## Rationale

This is the review's central ask ([external review](../history/2026-09-02-external-review-gpt-5-6.md)) and the point where
Diver stops being an arXiv graph browser. The distinction it buys is real: a
citation graph says two papers are connected; a claim relation says *how the
second one bears on the first*, which is the thing a researcher actually wants
and the thing no amount of co-occurrence weighting can produce.

The architecture is constrained by the [epistemic charter](../README.md#epistemic-principles): the model is
an instrument, not the authority. That is not caution for its own sake — a wrong
`contradicts` edge is worse than a missing one, because the structure implies the
system computed it. Grounding, mandatory confidence, and preserved disagreement
are what keep a structured representation from lending unearned authority to a
guess.

## Alternatives

- **Subject-predicate-object shredding of claims** — the review's proposal, and a
  genuine option, recorded here rather than dismissed. The argument against
  committing to it: scientific claims are n-ary and condition-laden. *"Loss follows
  a power law in parameters, for compute-optimal training, on autoregressive
  objectives"* has no faithful SPO form; the qualifiers either vanish or land in a
  bag that no query touches, and the triple-store literature has decades of
  reification pain to show for it. The counter-proposal is to keep the claim as
  text — the human-verifiable unit — and normalize *around* it: concept links
  (INT-0020) plus typed inter-claim edges (this intent). That likely yields the
  same query power without shredding. **Not settled.** Both shapes stay live until
  INT-0022 can measure them, because this is an empirical question wearing a
  taste-shaped costume.
- **Deriving relations from citation context** (the approach attributed to Scite
  in the [external review](../history/2026-09-02-external-review-gpt-5-6.md) —
  *that attribution is unverified; confirm before relying on it*) — plausibly
  strong prior art, and cheaper. Deferred: arXiv metadata does not give reliable
  citation context, and it answers a different question — how a *paper* cites
  another, not how a *claim* bears on another. Worth revisiting if a
  citation-context source arrives.
- **Letting the model write edges directly** — rejected. It is the whole failure
  mode. It would produce an impressive demo and a graph nobody should trust.
- **A large relation vocabulary up front** — rejected: unmeasurable distinctions
  are indistinguishable from noise. Grow the vocabulary on evidence.

## Consequences

- New persisted relation tables with provenance and confidence columns; new
  typestate types mirroring `assertion.rs`; a second extractor mode reusing the
  `ProviderShape` transport seam from [[structured-claim-extraction]] (INT-0016).
- Relation proposal is an O(pairs) LLM cost, which will force candidate-pair
  selection — concept co-membership (INT-0020) is the obvious filter, another
  reason for that ordering.
- Depends on INT-0020 for reliable candidate pairing and on INT-0022 to be
  judgeable at all.
- Bounded above by [[full-text-evidence]] (INT-0023): abstracts state conclusions,
  not the conditions that separate `refines` from `contradicts`. Expect a real
  quality ceiling here and report it honestly rather than tuning until the demo
  looks good.
- Once typed relations and concepts both exist, the temporal questions the review
  is chasing — where a claim first appeared, which later claims strengthened it,
  where exceptions were found — become queries rather than new subsystems.

## Transition history
- 2026-09-02: created as `proposed` during Sprint 18 roadmap realignment; relation-shape question (SPO vs. claim-text-plus-attachments) deliberately left open pending INT-0022.
