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
- **Surface forms map many-to-one — for inflection only.** `network` and
  `networks`, or `diffusion model` and `diffusion models`, are one concept, with
  every observed surface form retained and inspectable. A phrase is its own
  concept, *linked to* its constituent words rather than merged into them:
  `attention mechanism` is not `attention`, and treating it as such would be a
  broader/narrower judgement disguised as identity. Derivational variants
  (`attention` / `attentional`) stay distinct until a later layer can propose such
  merges for deterministic confirmation. *(Revised in Sprint 19 research; see
  Transition history.)*
- **Phrases form from adjacent words by a category rule.** The stoplist splits into
  common words, which break a phrase, and research filler, which may be a phrase's
  *head* but never its modifier — so `language model` and `attention mechanism`
  form while `model establishes` does not. A phrase becomes a concept only when at
  least two papers share it, since recurrence is the deterministic evidence that a
  word pair is a unit and not a coincidence. Two-word phrases only for now. This
  subsumes the deferred bigram work (T-1710) rather than doing it twice.
- **Concepts are materialized and never stale.** Formation runs over the whole
  stored corpus and persists its result. Because any claim write — and any change
  to the formation rules — can change the concept set, a fingerprint of the stored
  claims plus a formation-version constant is checked on read, and a mismatch
  triggers a full, deterministic rebuild. This is a full rebuild on change, not the
  incremental maintenance [[incremental-materialization]] (INT-0024) defers.
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
2. `diver dive <term>` resolves the term to a concept, and a paper using a
   different surface form of that concept appears in the neighborhood — something
   the current substring match cannot do (on the real corpus, `dive networks`
   misses the papers that only say `network`). When the term does not resolve, it
   says so and names the concepts that contain the term as a word, so a stoplisted
   query like `model` points at `diffusion model` rather than returning nothing.
3. Concept formation is deterministic: the same corpus produces the same concept
   set, byte-for-byte, across runs.
4. Multi-word concepts are representable, and `machine translation` is a distinct
   concept from `machine` and `translation`.
5. Co-assertion edges name a concept; existing temperature semantics and tests
   still hold.
6. Every concept is traceable to the claims and papers whose surface forms formed
   it.
7. Concepts are never stale: after any change to the stored claims, or to the
   formation rules' version, the next read reflects the new concept set without a
   manual rebuild step. *(Added in Sprint 19 research.)*

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

Sprint 19 measured the problem on the real 13-paper corpus
([probe](../sprints/s19/sprint-research/probe-output.txt)). Substring resolution is
wrong in both directions. It over-matches: `model` hits 46 claims though only 18
contain the word, `net` hits 12 though only 1 does, and `gan` hits two claims, both
via `organized`. It also under-matches: `networks` cannot find the papers that say
only `network`. The same probe showed that plural folding plus the category rule
for phrases recovers `diffusion model`, `attention mechanism`, and `bleu score` —
which the unmodified stoplist would have destroyed — with no false merge observed
across 26 folds.

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
- **Snowball / Porter stemming** (the `rust-stemmers` crate exists) — rejected. It
  would merge `attention`/`attentional`, but it strips suffixes without regard to
  meaning, and Porter-family stemmers are known to put unrelated words on one stem.
  A wrong merge is invisible in aggregate output, which is the failure this chapter
  most needs to avoid. Plural folding covered every variant the probe found. *(Not
  measured on this corpus; no stemmer was run.)*
- **Merging a phrase into its head word** (`attention mechanism` → `attention`) —
  rejected. It is a broader/narrower judgement, not identity. Linking the two keeps
  the relation visible without asserting they are the same thing.
- **Part-of-speech tagging to find noun phrases** — rejected for now: it needs a
  model or a sizeable dependency, and the frequency-plus-category rule recovered
  the important phrases on real data.
- **Three-word phrases** — deferred. They would absorb bigram fragments such as
  `neural machine` (from *neural machine translation*), at the cost of more
  combinatorial noise.

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
- **Recall changes as well as precision.** Resolution by concept drops substring's
  accidental derivational matches — `dive attention` no longer reaches
  `attentional` (1 claim on the real corpus). Accepted: it is the same mechanism
  that stops `gan` matching `organized`.
- **Stoplisted words stop resolving on their own.** `model` is filler, so it forms
  no concept; criterion 2's suggestions are what keep that from being a dead end.
- `stopwords.txt` splits into two files, making the phrase-breaking category
  explicit instead of inferred from word order.
- Two-word phrases leave fragments (`neural machine`, `convolutional neural`), and a
  little boilerplate survives (`project page`); both are recorded, not chased —
  tuning against 13 papers would overfit.

## Transition history
- 2026-09-02: created as `proposed` during Sprint 18 roadmap realignment, in response to the external review; ordering deliberately inverted relative to that review's recommendation, with reasoning recorded above.
- 2026-09-21: revised during Sprint 19 research, still `proposed` (no state change). A read-only probe of the real corpus showed that (a) "many-to-one" must mean inflection only — merging `attention mechanism` into `attention` would disguise a broader/narrower judgement as identity; (b) the unmodified stoplist destroys key phrases, so it must split into phrase-breaking and head-eligible categories; (c) materialized concepts need a freshness guarantee. Intent, criterion 2 (suggestions for unresolved terms), new criterion 7 (freshness), Rationale, Alternatives, and Consequences updated accordingly.
