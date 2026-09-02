# Sprint 18 Research Report

## Intents Reviewed
- [INT-0019](../../../intents/INT-0019-configurable-store-path.md) — created; relevance: the hardcoded `Store::open()` path blocks reproducible corpora and is therefore on the critical path for every evaluation intent below; current state: `proposed`.
- [INT-0020](../../../intents/INT-0020-first-class-concepts.md) — created; relevance: names the next major abstraction (concept identity), which this sprint's survey found to be the true bottleneck under the review's headline recommendation; current state: `proposed`.
- [INT-0021](../../../intents/INT-0021-typed-epistemic-relations.md) — created; relevance: the review's central directional ask (`supports`/`contradicts`/`refines` edges); current state: `proposed`.
- [INT-0022](../../../intents/INT-0022-relation-evaluation-harness.md) — created; relevance: makes INT-0021 judgeable rather than merely plausible; current state: `proposed`.
- [INT-0023](../../../intents/INT-0023-full-text-evidence.md) — created; relevance: records the abstracts-only fidelity ceiling that bounds INT-0021's achievable quality; current state: `proposed`.
- [INT-0024](../../../intents/INT-0024-incremental-materialization.md) — created; relevance: records the O(n^2) query-time graph as a *deliberate* deferral with explicit trigger conditions; current state: `proposed`.
- [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) — selected (context only); relevance: the co-assertion + temperature machinery INT-0020 would sit beneath; current state: `realized`.
- [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) — selected (context only); relevance: the stoplist is a lexical patch on a problem INT-0020 addresses structurally; current state: `realized`.

## 1. Sprint Goal

Convert an external architectural review into durable, non-redundant project
authority, and clear the one maintenance item that blocks the roadmap it
implies. Concretely: author six intent chapters stating where Diver goes after
Sprint 17 (and, where Diver's direction departs from the review, say so in the
chapter rather than diverging silently); add a cross-cutting epistemic charter
to the Project Book root so future intents are constrained by stated principles
rather than by recollection; and land `DIVER_DB` (backlog T-1611), because
`Store::open()`'s hardcoded path makes the reproducible fixture corpora that
INT-0022 depends on impossible. This sprint deliberately writes no graph or
extractor logic — the roadmap is the deliverable, and the review's own strongest
advice is to stop laying foundations before deciding what is being built on them.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| `diver-core/src/store.rs` (l.32) | high | `Store::open()` hardcodes `dirs::data_dir()/diver/diver.db`; only `open_in_memory()` escapes it. The T-1611 / INT-0019 target. |
| `diver-core/src/store.rs` (l.55-100) | high | Schema is `papers` -> `paper_versions` -> `assertions` -> `assertion_support`. No concept, relation, or confidence table exists anywhere. |
| `diver-core/src/store.rs` (l.535) | high | `papers_asserting()` resolves "the concept" as `claim LIKE '%?1%'`. Concept identity is currently a substring accident — the most load-bearing finding for INT-0020. |
| `diver-core/src/graph.rs` (l.16) | high | `RelationKind` is `SharedCategory` / `SharedAuthor` / `CoAssertion{term,weight}`. All three are paper-to-paper. No claim-to-claim edge type exists; INT-0021 introduces the first. |
| `diver-core/src/graph.rs` (l.49, l.203) | high | `compute_relations()` and `compute_coassertion_relations()` are query-time and pairwise over the whole corpus — the O(n^2) that INT-0024 records as an accepted deferral. |
| `diver-core/src/graph.rs` (l.170) | medium | `significant_terms()` plus the `stopwords.txt` `HashSet`: a lexical filter standing in for concept identity. INT-0020 subsumes it rather than extending it. |
| `diver-core/src/assertion.rs` | high | The typestate gate. `Assertion<Supported>` has no public constructor; `validate()` is the sole path. INT-0021 must reuse this pattern for relations, not invent a parallel one. |
| `diver-core/src/assertion.rs` (`is_supported`) | medium | Still the v1 rule: non-empty support. Deliberately weak; INT-0021 does not change it. |
| `diver-core/src/observation.rs` | high | `extract_observations()` splits `SourceFact::summary` — **the abstract**. Confirms the corpus is abstracts-only; the evidence base for INT-0023's fidelity-ceiling argument. |
| `diver-core/src/extract.rs` (l.315) | high | `claims_schema()` emits `{claim, quote}` — flat text plus a verbatim span. The shape INT-0021 would extend, and why the SPO question is live. |
| `diver-core/src/extract.rs` (l.38) | medium | `ProviderShape` already abstracts Anthropic vs OpenAI-compatible, so relation *proposal* needs no new transport work — it reuses this seam. |
| `diver-cli/src/main.rs` | medium | Surface: `search`/`ingest`/`inspect`/`extract`/`assertions`/`dive`/`list`/`collect`/`find`. `dive --temperature` is the only "weird" verb; Find is boring, as the review recommends. |
| `diver-core/tests/real_corpus.rs` | high | Parses a checked-in 7-entry Atom fixture into an `open_in_memory()` store: the only end-to-end evidence of dive quality, and the natural seed for INT-0022's gold set — but it rebuilds the corpus every run and cannot hold a durable, versioned one until INT-0019 lands. |
| `docs/work/tasks.md` | high | T-1611 (`DIVER_DB`) is already backlogged; this sprint promotes it because it gates INT-0022. T-1310 / T-1410 / T-1510 / T-1710 stay in backlog. |
| `docs/history/2026-09-02-external-review-gpt-5-6.md` | high | The reviewed document, preserved non-authoritatively so six intents do not depend on a file outside the repo. |

## 3. External Sources

- [Preserved external review](../../../history/2026-09-02-external-review-gpt-5-6.md) — the primary input: an assessment of Dive's architecture and its recommended direction.
- [Scite](https://scite.ai/features) — cited by the review as a production implementation of support/contrast citation classification; prior art for INT-0021's relation vocabulary. **Not independently verified this sprint.**
- [Semantic Scholar API](https://www.semanticscholar.org/product/api) — cited as prior art for citation-intent classification at scale; relevant to INT-0021 and to any post-arXiv corpus expansion. **Not independently verified this sprint.**
- [OpenAlex](https://help.openalex.org/data/works/) — cited as evidence that very large scholarly graphs are tractable; relevant to INT-0024's scale trigger. **Not independently verified this sprint.**
- [Elicit](https://elicit.com/solutions/systematic-review) — cited as prior art for evidence-carrying claim extraction; nearest neighbour to Diver's existing extract layer. **Not independently verified this sprint.**

## 4. Risks, Unknowns, Dependencies

- **Dependency:** INT-0022 (evaluation) cannot run reproducibly until INT-0019 (`DIVER_DB`) lands. That is the only reason a small maintenance item sits inside a roadmap sprint.
- **Dependency:** INT-0021 (typed relations) is substantially harder to do well before INT-0020 (concept identity) — relations between claims presuppose knowing which claims are *about* the same thing. The review's ordering implies relations first; this survey argues the reverse, and both chapters record the disagreement.
- **Risk:** the corpus is abstracts-only. Abstracts state conclusions, not conditions, effect sizes, or scope qualifiers — exactly the content that separates `refines` from `contradicts`. Building INT-0021 on abstracts risks confident-looking edges at a fidelity the evidence cannot support. Mitigated by INT-0023 recording the ceiling explicitly and INT-0022 measuring against it rather than assuming it away.
- **Risk:** structured representation lends unearned authority. A wrong `contradicts` edge is worse than a missing one, because the graph's shape implies it was computed. The charter's "represent disagreement, do not resolve it" and INT-0021's mandatory confidence field exist for this.
- **Unknown:** whether subject/predicate/object shredding (the review's proposal) or claim-text-plus-typed-attachments (this sprint's counter-proposal) is the right assertion shape. Not resolvable from the current corpus; INT-0021 records both and commits to neither until INT-0022 can measure them.
- **Unknown:** the corpus size at which query-time O(n^2) actually hurts. INT-0024 states a measurable trigger rather than a guess, so the deferral can end on evidence instead of anxiety.
- **Risk (process):** six chapters authored at once can drift into a wish list. Mitigated by all five forward-looking chapters being `proposed` — described, explicitly *not* accepted into executable work — so scheduling stays a per-sprint decision.

## 5. Recommended Approach

Primary: treat intent chapters as the roadmap. Author INT-0019 through
INT-0024, add an "Epistemic principles" charter to `docs/README.md` for the
cross-cutting constraints that are not themselves units of work, and realize
only INT-0019 in Sprint 18. Leave INT-0020 through INT-0024 in `proposed`,
which is precisely the state meaning "described but not accepted into
executable work."

Alternative considered: a single `docs/ROADMAP.md`. Rejected — it would become
a second decision store competing with the intent chapters, it carries no state
machine, no evidence fields, and no transition history, and it would rot the
moment an intent moved without it. The Book already has the durable form; the
review's content should be poured into that form rather than beside it.

Alternative considered: adopting the review wholesale. Rejected on two points,
both recorded in the chapters rather than only here. (1) Ordering — the review
puts typed relations next; the code survey shows concept identity is the real
precondition, since `papers_asserting()` resolves a concept by substring.
(2) Shape — the review proposes shredding assertions into
subject/predicate/object; scientific claims are n-ary and condition-laden, and
SPO either discards the qualifiers or demotes them to a bag that does no query
work. INT-0021 records SPO as a genuine alternative and defers the choice to
measurement rather than taste.

Rationale: the review's most valuable advice is that Diver has earned the right
to stop building foundations and start attacking epistemic resolution. Acting on
that means deciding *what* is being built before building it, and recording the
decision where it stays legible ten sprints from now. The single code change in
this sprint is included only because it blocks the evaluation work that makes
everything after it falsifiable.

## Artifacts
- [`docs/history/2026-09-02-external-review-gpt-5-6.md`](../../../history/2026-09-02-external-review-gpt-5-6.md) — the preserved external review this sprint responds to.
