# Project Book

This directory is the canonical Sprint Loops Book: project intent, executable
work, realization evidence, and sprint provenance live here together.

- [Intents](intents/README.md) — the semantic authority. Every durable decision
  about what Diver is becoming lives in an `INT-NNNN` chapter.
- [Tasks](work/tasks.md) / [Completed tasks](work/completed-tasks.md) — execution state.
- `sprints/sN/` — per-sprint provenance.
- `history/` — preserved source material. Non-authoritative, never a decision store.

## Epistemic principles

These are cross-cutting constraints, not units of work. They bind every intent
chapter; an intent that violates one should either be redesigned or should argue
explicitly, in its own Rationale, for changing the principle. They are recorded
here rather than in an intent because they are not something the project
*finishes* — they are what the project is required to keep being true.

**1. Claims are the epistemic unit; papers are provenance.**
A paper is the container an assertion came from, not the atom of knowledge.
Diver's value comes from what is asserted and how assertions relate — not from
organizing documents.

**2. Extraction is never truth.**
A model reporting that a passage asserts X produces a *candidate*. It does not
establish X. The gap between "a model said so" and "this is supported" is the
project's core discipline, and `Assertion<Supported>` exists to make the compiler
enforce it rather than trusting anyone to remember.

**3. The machine proposes; a deterministic gate disposes.**
Models may propose structure — candidate claims, candidate relations, candidate
concept merges. Nothing reaches canonical state without passing a deterministic
check. Whenever a problem looks solvable by "just ask the model what these are
related to," that is the signal to stop: the answer would destroy the property
that makes Diver worth building.

**4. Never discard provenance.**
Any edge, claim, or concept must be traceable down to the passage that justified
it: `concept → assertion → relation → evidence span → paper → version →
extraction method`. If a link in that chain cannot be reconstructed, the
representation is incomplete.

**5. Represent disagreement; do not resolve it.**
When sources conflict, Diver stores the conflict. It does not adjudicate, score a
winner, or collapse a live scientific dispute into a verdict. Characterizing
evidence is downstream machinery's job; deleting one side of it is nobody's.

**6. Uncertainty belongs in the data model.**
Extraction confidence, ambiguous equivalence, weak evidence, disputed relations,
unresolved identity — all must be representable. A schema with no place to record
doubt forces every value to look certain.

**7. Keep the deterministic skeleton.**
arXiv categories, dates, authorship, identifiers, and versions are valuable
precisely because Diver did not invent them. They are the coordinate space the
epistemic layer is plotted in, and they stay externally defined.

**8. Trustworthy small beats dubious large.**
A defensible graph over 10,000 papers teaches more than an unreliable one over
100 million. Scale is not a goal; it is a consequence of having something worth
scaling.

**9. Keep Find boring. Make Dive weird.**
`find` should be an excellent, predictable, unsurprising retrieval tool. `dive`
is where novel interaction belongs — temperature, concept paths, competing
branches, intellectual ancestry, unresolved questions.

**10. Dive is not a chatbot.**
Generated prose may eventually be offered as a *view* over the graph. It must
never become the product. The interesting object is the inspectable structure
underneath; a fluent paragraph that cannot be drilled into is the failure mode,
not the goal.

## North star

Two tests, applied periodically rather than continuously.

**The mental-model test.** Take a research question nobody on the project knows
well. Give one person arXiv and Google Scholar for an afternoon; give another
Diver. Ask each to describe the major claims in the area, the evidence for them,
where researchers disagree, how the ideas developed, and what looks unresolved.
The measure is not whether Diver surfaced more papers — it is whether the Diver
user ended up with a better model of the field.

**The discovery test.** Can Diver show a knowledgeable researcher a relationship
in their own field that they did not already know, with enough provenance that
they can verify it themselves? The first legitimate instance of that is the
milestone where Diver stops organizing knowledge and starts being an instrument
for finding structure within it.

> These principles were consolidated in Sprint 18 while responding to an
> [external architectural review](history/2026-09-02-external-review-gpt-5-6.md).
> That document is preserved history, not authority; where Diver's direction
> departs from it, the reasoning lives in the relevant intent chapter.
