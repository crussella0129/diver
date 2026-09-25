# INT-0020 — Concepts as first-class entities

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0020
- **State:** active
- **Work evidence:** [Sprint 19 build plan](../sprints/s19/sprint-plans/build-plan.md) (T-1901 – T-1906)
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
  observed in claims plus corpus statistics (document frequency, distinctiveness).
  No model is asked to invent an ontology. arXiv's own taxonomy vocabulary as a
  further input is deferred — see Alternatives. *(Narrowed after Sprint 19's plan
  critique: the first realization uses claim text only.)*
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
  claims, a formation-version constant and a digest of the stoplists is checked on
  read, and a mismatch
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
problems. Corpus statistics and morphological variants — and, later, arXiv's
taxonomy — get a long way without a model in the loop, which preserves the property that makes
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
- **Seeding concepts from arXiv's taxonomy vocabulary** — deferred. Category names
  are coarse (*Computation and Language*) and would add concepts that no stored claim
  uses; they fit better as a later cross-reference layer mapping claim concepts onto
  categories than as a formation input.
- **Classifying a token by its raw form** (the stoplist as written) — rejected in plan
  review. The stoplist often lists one inflection but not the other (`states` unlisted,
  `state` listed), so raw classification would build a concept from half its forms and
  break the phrase-subsumption invariant below.

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
- **Phrases stay inside a clause.** Punctuation (`. , ; : ! ? ( ) [ ] { } "`), numbers,
  and short tokens break a phrase; hyphens and slashes do not, so `encoder-decoder` and
  `pre-trained` form.
- **Tokens are classified by their folded form.** The stoplists are folded too, and a
  token's category (common, filler, content) is the category of its folded key, common
  taking precedence. Every inflection of a word therefore gets the same treatment. Two
  directions change on the current lists: the plural of a listed singular is now stopped
  with it (for example `states`, `inputs` and `details` become filler, `parts` and `ones`
  common), and six singulars whose plural was listed become filler — `application`,
  `condition`, `consist`, `contain`, `contribution`, `effect`. Filler words can still
  head a phrase (`hidden state`, `boundary condition`).
- **Folding is a suffix heuristic, tuned to prefer a missed merge over a wrong one.**
  Rules apply in order: `ies` → `y`; `sses` → drop `es`; words ending in `ics` are left
  alone; otherwise a final `s` is dropped unless the word ends in `ss`, `us` or `is`. The
  `ics` exclusion exists because stripping `s` would merge derivational pairs the Intent
  keeps apart (`semantics`/`semantic`, `logistics`/`logistic`); it costs genuine plurals
  such as `metrics`/`metric` their merge. Known limitations, recorded rather than chased:
  - *Missed merges:* singulars ending in `s` (`bias` → `bia`, `biases` → `biase`);
    `-ches`/`-shes`/`-xes` plurals (`batches`, `patches`); acronym plurals ending in `us`
    (`gpus` stays apart from `gpu` — both appear in the checked-in fixture); and verb
    forms whose stems are listed differently (`establish` is filler, `establishes`
    content).
  - *Wrong merges still possible:* `news` folds to the common word `new` and so never
    resolves; a plural noun whose singular is listed for its adjective or verb sense is
    stopped with it (`priors` → common `prior`, `keys` → filler `key`).
  Traceability (criterion 6) keeps every merge inspectable. A lexicon-backed
  lemmatizer would fix most of these, at the cost of a dependency and a vocabulary that
  lags the field; deferred.
- **A phrase subsumes its words in co-assertion.** When two papers share a phrase, the
  edges for its individual content words are dropped for that pair, so one overlap is
  one edge rather than three. This keeps edges monotonic in temperature *because*
  tokens are classified by folded form: every paper containing a phrase then contains a
  token of the same category folding to each of its content words, so the phrase's paper
  count never exceeds theirs and its weight is never below theirs. (A filler head forms
  no word concept, so there is nothing to subsume.) Under that invariant, suppressing
  the word edge unconditionally and suppressing it only when the phrase edge clears the
  gate are the same rule. *(Corrected after Sprint 19's plan critique, which showed the
  earlier justification failed under raw-form classification.)*
- **This refines INT-0014's `t = 1.0` endpoint.** [[weighted-coassertion-temperature]]
  says the fully permissive temperature keeps every shared term. It now keeps every
  shared concept except the words a pair's shared phrase subsumes. The IDF formula,
  the gate, the small-corpus guard and `N` (distinct papers with at least one claim) are
  unchanged.
- **Stoplist edits force a rebuild.** The freshness fingerprint includes a digest of both
  stoplists' word lists, so editing either list invalidates persisted concepts even if
  nobody bumps the formation version.
- **`CoAssertion.term` keeps its name** and now carries the concept's label. Renaming it
  would mean editing the tests that prove criterion 5 held; the rename is deferred.
- **`Store::papers_asserting` keeps its signature** but resolves through concepts. Its
  former LIKE-escaping test tested behaviour that no longer exists and is replaced by
  one asserting that wildcard characters match nothing.
- **Proving-ground findings (Sprint 19, real corpora of 480 and 2,068 papers across
  16 fields).** Driving the real binary changed four things, each recorded here because
  it changes behaviour the plan fixed:
  - *Adverbs are filler by rule* (folded key of 6+ characters ending in `ly`, except a
    short noun allowlist such as `anomaly`, `family`), and the filler list grew by 236
    generic modifiers, verbs and nouns observed creating false co-assertion edges
    (`wide range` linked 17 unrelated papers at the default temperature). Formation
    version 2. Domain words that merely span fields (`entropy`, `graph`, `network`)
    are deliberately kept: they are the cross-field bridges this intent exists to find.
  - *Suggestions rank by coverage.* When no concept contains every query word, the
    fallback ranks by how many query words a concept contains before paper count, so
    `quantum error correction` offers `error correction` and `quantum error` before
    `error`. This refines criterion 2's ordering, which ranked by paper count alone.
  - *Misspellings get a "did you mean".* When nothing contains the query, concepts within
    edit distance 1 (2 for 7+ characters) are offered: `atention` → `attention`.
  - *Related papers are grouped and ranked,* concepts first, then authors; papers sharing
    only an arXiv category are counted, never listed. At 480 papers every node had 700+
    related papers and the first ten were always category links.
- **Known limitation confirmed at scale: word concepts are polysemous.** `code` spans
  genetic code, error-correcting codes and source code across 150 papers. Phrases
  disambiguate cleanly (`surface code`: 5 of 5 quantum error-correction papers), and a
  word's `narrower:` list reads as its sense inventory. Sense separation is future work.
- Two synthetic test fixtures that were bags of content words (`"rare mid common"`,
  `"zebra apple mango"`) now legitimately form shared phrases, and gain comma separators;
  their assertions are unchanged.

## Transition history
- 2026-09-02: created as `proposed` during Sprint 18 roadmap realignment, in response to the external review; ordering deliberately inverted relative to that review's recommendation, with reasoning recorded above.
- 2026-09-21: revised during Sprint 19 research, still `proposed` (no state change). A read-only probe of the real corpus showed that (a) "many-to-one" must mean inflection only — merging `attention mechanism` into `attention` would disguise a broader/narrower judgement as identity; (b) the unmodified stoplist destroys key phrases, so it must split into phrase-breaking and head-eligible categories; (c) materialized concepts need a freshness guarantee. Intent, criterion 2 (suggestions for unresolved terms), new criterion 7 (freshness), Rationale, Alternatives, and Consequences updated accordingly.
- 2026-09-21: `proposed` → `planned`; linked to the Sprint 19 build plan (T-1901 stoplist split and vocabulary module, T-1902 formation, T-1903 persistence and freshness, T-1904 co-assertion re-key, T-1905 `dive`, T-1906 README). Plan-time design decisions recorded under Consequences: phrases stay inside a clause, a phrase subsumes its words in co-assertion (monotonicity preserved), `CoAssertion.term` and `papers_asserting` keep their names, two bag-of-words fixtures gain separators.
- 2026-09-21: revised after Sprint 19 plan critique round 1, still `planned` (no state change). Tokens are classified by folded form, which restores the phrase-subsumption invariant the critique showed failing under raw-form classification; folding order fixed as `ies`, `sses`, `s` with the `bias`/`biases` limitation recorded; the refinement of INT-0014's `t = 1.0` endpoint recorded; the freshness fingerprint gains a stoplist digest; arXiv taxonomy vocabulary narrowed out of the Intent and recorded as a deferred alternative.
- 2026-09-21: revised after Sprint 19 plan critique round 2, still `planned` (no state change). The folding consequence overclaimed ("misses a merge rather than making a wrong one"); corrected with an `ics` exclusion — preferring a missed merge over a derivational one, per this chapter's own principle — and an explicit list of the remaining missed-merge and wrong-merge classes.
- 2026-09-24: `planned` → `active` (Sprint 19 build started; T-1901 first).
- 2026-09-24: revised during the Sprint 19 proving-ground drive, still `active`. Adverb rule and generic-vocabulary additions (formation version 2), suggestion coverage ranking, a "did you mean" fallback, and grouped related papers recorded under Consequences; polysemy of word concepts recorded as a confirmed limitation.
