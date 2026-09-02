# INT-0023 — Full-text evidence beyond abstracts

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0023
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Every claim Diver holds was extracted from an abstract. `extract_observations`
splits `SourceFact::summary`; the LLM extractor is handed the same field;
grounding checks that a quote occurs in that same abstract. The corpus is one
paragraph per paper.

Extend the evidence base to arXiv full text, so claims can be grounded in the
sections that actually qualify them:

- **Ingest full text** for papers already in the corpus, from arXiv's freely
  available sources — preferring LaTeX source where available and falling back to
  PDF extraction.
- **Preserve document structure.** A quote's section (abstract, method, results,
  limitations, related work) is evidence about the quote. A claim grounded in
  Results carries different weight from one grounded in Related Work, and
  flattening the document to a token soup throws that away.
- **Grounding stays exact.** Evidence spans must still occur verbatim in their
  source. A larger haystack does not license a looser needle — if anything it
  demands a stricter one, since a long document makes accidental substring
  matches likelier.
- **Provenance extends, it does not change.** Claims already carry paper and
  version; they gain a location within the document.

Non-goals:
- No non-arXiv sources. Publisher agreements and paywalls are a different problem
  with a legal dimension; arXiv's openness is the whole reason this is tractable.
- No change to the epistemic model. Claims, concepts, and relations mean the same
  things — they just get better evidence.
- No full-text search surface. `find` stays boring ([charter](../README.md#epistemic-principles)); this is
  about extraction quality, not a new retrieval feature.

## Acceptance criteria

1. A stored paper can carry full text with section structure, versioned
   alongside the paper version it came from.
2. Claim extraction can draw from full text, and grounding validates spans
   against it with the same exactness applied to abstracts today.
3. A claim records where in the document its evidence came from.
4. PDF and LaTeX extraction failures are recorded as failures, not silently
   producing an empty or mangled document — a paper with unusable full text must
   be distinguishable from one with none.
5. Measured against [[relation-evaluation-harness]] (INT-0022): full-text
   extraction is shown to change relation quality, in a stated direction, on the
   gold set. If it does not help, that is a finding worth having and worth
   recording here.

## Rationale

This records a **ceiling**, and the recording matters as much as the eventual
work. [[typed-epistemic-relations]] (INT-0021) proposes to distinguish
`supports` from `refines` from `contradicts`. Abstracts cannot reliably carry
that distinction: they state conclusions, and systematically omit the
conditions, effect sizes, populations, and scope limits that separate *"X
increases Y"* from *"X increases Y only when Z"*. Those live in Methods, Results,
and Limitations. A relation layer built only on abstracts will assign confident
types to claims whose distinguishing content it never saw.

The [external review](../history/2026-09-02-external-review-gpt-5-6.md) treats full text as a later
competitive problem, framed around paywalls and publisher relationships. For
arXiv that framing does not apply — the full text is free, and the obstacle is
parsing effort, not access rights. That makes this materially cheaper than the
review implies, and it should be weighed accordingly.

It is nonetheless staged *after* INT-0021 rather than before, for a specific
reason: the epistemic model and its validation gate are the durable parts, and
the evidence corpus is swappable beneath them. Build the model on abstracts,
measure its ceiling honestly, then raise the ceiling. Building the parser first
would be exactly the "another five sprints of ingestion plumbing" the review
rightly warns against.

## Alternatives

- **Stay abstracts-only permanently** — rejected, but it is a coherent position:
  abstracts are clean, uniform, and cheap, and a modest tool built well on them
  beats an ambitious one built badly. Rejected because it caps INT-0021 at a
  quality the project is explicitly aiming past.
- **PDF-only extraction** — simpler, universally available, and lossy: math,
  tables, and multi-column layouts degrade badly, and those are where results
  live. Prefer LaTeX source where arXiv provides it, fall back to PDF.
- **Full text before typed relations** — rejected on sequencing above.
- **Third-party parsed full text** (GROBID, S2ORC, or similar) — genuinely worth
  investigating when scheduled; it could remove most of the parsing cost. Deferred
  as an open question, and it would introduce an external dependency in the
  evidence path, which deserves its own scrutiny.

## Consequences

- Storage grows by orders of magnitude — full text is not abstracts, and the
  SQLite substrate's limits should be re-examined when this is scheduled.
- New parsing dependencies and a new class of failure (malformed PDFs, exotic
  LaTeX) that must be surfaced rather than swallowed.
- Extraction cost per paper rises substantially; chunking and section selection
  become live questions.
- Raises INT-0021's quality ceiling — which is the point, and which is only
  demonstrable because INT-0022 exists to measure it.
- Until this lands, any quality claim about typed relations must be stated as
  *abstract-level*. That qualification is a requirement, not a caveat.

## Transition history
- 2026-09-02: created as `proposed` during Sprint 18 roadmap realignment, to record the abstracts-only fidelity ceiling explicitly rather than discovering it as a disappointment mid-INT-0021.
