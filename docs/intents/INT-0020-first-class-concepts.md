# INT-0020 — Concepts as first-class entities

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0020
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Diver has no concept. `diver dive attention` resolves "attention" by
`claim LIKE '%attention%'` (`Store::papers_asserting`), and co-assertion edges
are keyed on raw lowercased tokens that survive a stoplist
([[coassertion-stoplist]], INT-0018). A concept is currently a substring — an
accident of spelling, not an entity.

Give concepts identity, derived deterministically from the corpus:

- **A `Concept` entity with a stable id**, persisted, that claims attach to.
  Papers stay provenance containers; claims stay the epistemic unit; concepts
  become the axis you navigate along.
- **Deterministic, corpus-derived formation.** Concepts come from surface forms
  observed in claims plus corpus statistics (document frequency, distinctiveness)
  and arXiv's own vocabulary — categories, and the terminology the taxonomy
  already implies. No model is asked to invent an ontology.
- **Surface forms map many-to-one.** `attention`, `attentional`, and
  `attention mechanism` should be able to resolve to one concept, with every
  surface form retained and inspectable. Multi-word forms are included here,
  which subsumes the deferred bigram work (T-1710) rather than doing it twice.
- **Co-assertion re-keys onto concepts.** `RelationKind::CoAssertion` carries a
  concept, not a token. IDF weighting and the `--temperature` dial
  ([[weighted-coassertion-temperature]], INT-0014) keep their semantics and
  operate over concepts instead of strings.

Non-goals:
- No cross-lingual or cross-field semantic equivalence yet — that is the hard
  research problem, and this intent is the substrate it would need, not the
  solution to it.
- No LLM-authored concept hierarchy. A model may later *propose* that two
  surface forms are the same concept; it does not get to write that into the
  canonical set unaided.
- No change to extraction, grounding, or the typestate gate.

## Acceptance criteria

1. A persisted `Concept` has a stable id, a canonical label, and one or more
   observed surface forms; claims link to concepts rather than being matched by
   substring at query time.
2. `diver dive <term>` resolves the term to a concept (or reports that it does
   not resolve), and a paper using a different surface form of that concept
   appears in the neighborhood — something the current substring match cannot do.
3. Concept formation is deterministic: the same corpus produces the same concept
   set, byte-for-byte, across runs.
4. Multi-word concepts are representable, and `machine translation` is a distinct
   concept from `machine` and `translation`.
5. Co-assertion edges name a concept; existing temperature semantics and tests
   still hold.
6. Every concept is traceable to the claims and papers whose surface forms formed
   it.

## Rationale

This is the abstraction everything downstream needs, and it is the point where
this project's direction departs from the
[external review](../history/2026-09-02-external-review-gpt-5-6.md). The review recommends typed epistemic relations
between claims as the next move. That is the right destination, but relations
presuppose concept identity: to say Paper A's claim `refines` Paper B's claim,
the system must first know the two claims are *about the same thing*. Today it
cannot, because "the same thing" means "shares a substring." Building
[[typed-epistemic-relations]] (INT-0021) first would mean asking a model to
supply, per candidate pair, the identity judgement the substrate is missing —
which is exactly the "let the LLM secretly become the database" failure the
review itself warns against.

Concept identity is also the most *deterministically tractable* of the remaining
problems. Corpus statistics, morphological variants, and arXiv's taxonomy get a
long way without a model in the loop, which preserves the property that makes
Diver interesting: its skeleton was not hallucinated.

## Alternatives

- **Typed relations first** (the review's ordering) — recorded above; rejected as
  a starting point, though the two intents may partially overlap in practice.
- **Embedding-based concept clustering** — deferred. It is the obvious way to
  catch `attention` ≈ `alignment-based weighting`, but it makes concept identity
  a similarity threshold with no inspectable justification, and it is not
  reproducible across model versions. A candidate *later* layer that proposes
  merges for deterministic confirmation, not the substrate.
- **An imported external ontology** (MeSH, Wikidata, arXiv-adjacent taxonomies) —
  deferred. Real appeal (someone else maintains identity) but poor fit for
  fast-moving CS/ML vocabulary, and it imports an ontology's blind spots
  wholesale.
- **Keep lexical matching and widen the stoplist** — rejected: [[coassertion-stoplist]]
  already showed the ceiling of that approach. It removes noise; it cannot create
  identity.

## Consequences

- New persisted entity and link tables; `papers_asserting` stops being a `LIKE`
  scan; `graph.rs` re-keys co-assertion onto concepts.
- `significant_terms` and `stopwords.txt` become inputs to concept formation
  rather than the mechanism itself; INT-0018's stoplist is subsumed, not wasted.
- T-1710 (phrase/bigram co-assertion) is absorbed here.
- Unlocks the questions the review is actually after — where a concept first
  appears, how it migrates across arXiv categories, which communities discuss it
  without citing each other — because all of them are queries over a concept
  axis. Those become tractable follow-ons once this and INT-0021 exist.
- Concept formation is a new place to be wrong, and wrong merges are invisible
  in aggregate output. Traceability (criterion 6) is the non-negotiable guard.

## Transition history
- 2026-09-02: created as `proposed` during Sprint 18 roadmap realignment, in response to the external review; ordering deliberately inverted relative to that review's recommendation, with reasoning recorded above.
