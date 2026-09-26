# Sprint 19 Research Report

## Intents Reviewed
- [INT-0020](../../../intents/INT-0020-first-class-concepts.md) — selected and **revised**; relevance: the sprint's target — give concepts identity instead of resolving them by substring; current state: `proposed`. Research changed its Intent (what "many-to-one" means, how phrases form, how concepts stay fresh), one acceptance criterion (AC2 gains suggestions for unresolved terms), and added AC7 (freshness). All revisions are recorded in the chapter with rationale.
- [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) — selected (context); relevance: its stoplist becomes an input to concept formation, and research found it must be split into two categories to allow phrases; current state: `realized` (not reopened — INT-0020 subsumes the change).
- [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) — selected (context); relevance: IDF weighting and the `--temperature` dial must keep their semantics when co-assertion re-keys onto concepts (INT-0020 AC5); current state: `realized`.
- [INT-0024](../../../intents/INT-0024-incremental-materialization.md) — selected (context); relevance: concepts are materialized by full deterministic rebuild, which is explicitly *not* the incremental materialization INT-0024 defers; current state: `proposed`, unchanged.
- [INT-0021](../../../intents/INT-0021-typed-epistemic-relations.md) — selected (context); relevance: the reason concept identity comes first — relations presuppose knowing two claims are about the same thing; current state: `proposed`, unchanged.

## 1. Sprint Goal

Replace substring concept matching with deterministic, persisted concept identity
(INT-0020). Today `diver dive <term>` resolves a concept as `claim LIKE '%term%'`, and
co-assertion edges key on raw tokens; on the real corpus that is both imprecise and
incomplete. This sprint forms concepts from the stored claims — single words folded
across plural inflection, plus recurring two-word phrases — persists them with every
observed surface form and a link back to each claim that formed them, resolves `dive`
queries through them, and re-keys co-assertion onto them without disturbing the IDF
weighting or the temperature dial. Out of scope: stemming beyond plural inflection,
three-word phrases, acronym expansion, embeddings, and any model involvement in
concept formation.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| `diver-core/src/store.rs` (`papers_asserting`) | high | `claim LIKE '%' \|\| ?1 \|\| '%'` — the substring resolution this sprint replaces. |
| `diver-core/src/store.rs` (`init_schema`, `backfill_fts`) | high | Schema has no concept tables. `backfill_fts` is working precedent for lazily populating a derived table in an existing database. |
| `diver-core/src/store.rs` (`save_assertions`, `all_claims`) | high | The only assertion write path (idempotent delete-then-insert) and the whole-corpus claim read that formation consumes. `AUTOINCREMENT` ids are never reused, which makes `(COUNT, MAX(id))` a sound change fingerprint. |
| `diver-core/src/graph.rs` (`significant_terms`, `STOPWORDS`) | high | The tokenizer (split on non-alphanumeric, lowercase, ≥3 chars, ≥1 letter, not stopped). Reusable for unigram concepts; unusable as-is for phrases. |
| `diver-core/src/graph.rs` (`compute_coassertion_relations`) | high | IDF weighting (`w = ln(N/df)/ln(N/2)`) and temperature gate over per-paper term sets. Re-keying swaps the term source only. Iterates `HashMap`s but sorts output, so it is already order-stable. |
| `diver-core/src/graph.rs` (`RelationKind::CoAssertion { term, weight }`) | high | Matched by `display.rs`, `tests/coassertion.rs`, `tests/real_corpus.rs`, and graph unit tests — the churn surface if the variant's shape changes. |
| `diver-core/src/graph.rs` (`build_dive`) | medium | Consumes `(arxiv_id, claim)` pairs; unaffected if concept resolution produces the same pair shape. |
| `diver-core/src/stopwords.txt` | high | 525 words, uncommented, two alphabetical blocks: 303 common-English words, then 230 research-filler words beginning at `abstract`. The filler block includes `model`, `models`, `mechanism`, `learning`, `method`, `data`, and the web tokens `https`/`github`/`com`. |
| `diver-cli/src/main.rs` (`Commands::Dive`) | high | Calls `papers_asserting`, then `compute_relations` + `compute_coassertion_relations(all_claims)`, then `build_dive`. The resolution call site. |
| `diver-core/src/display.rs` (`display_dive`, `relation_reason`) | medium | Prints "No papers assert about …" on empty results and `co-asserts {term} (w=…)` per edge. Needs a resolved-concept header and an unresolved-with-suggestions path. |
| `diver-core/tests/coassertion.rs` | medium | Temperature fixture using placeholder term `networks` (renamed from `models` in Sprint 17). Plural folding changes that term's concept key to `network`. |
| `diver-core/tests/real_corpus.rs` | medium | 7-paper fixture; asserts a co-assertion edge exists on a technical term. Must keep passing (INT-0020 AC5). |
| `docs/sprints/s19/sprint-research/probe_concepts.py` | high | Read-only probe of the real 13-paper corpus (opened `mode=ro`). Replicates the tokenizer and measures the numbers below. |

## 3. External Sources

- [`rust-stemmers` on crates.io](https://crates.io/crates/rust-stemmers) — **verified** to exist this sprint (`cargo search`: v1.2.0, "a rust implementation of some popular snowball stemming algorithms"). The candidate if aggressive stemming were chosen; rejected below.
- [Snowball stemmer project](https://snowballstem.org/) — Porter-family stemmers are designed to conflate for retrieval and are known to over-stem unrelated words onto one stem (the classic example: *university* / *universe*). **Characterization from memory; not measured on this corpus** — no stemmer was run, because doing so would have meant downloading one.
- Schwartz & Hearst (2003), "A simple algorithm for identifying abbreviation definitions in biomedical text" — deterministic extraction of `long form (SF)` pairs from the text itself; the natural future route to linking `NMT` with *neural machine translation* without a model. **Cited from memory, not re-verified.** Out of scope this sprint.

## 4. Risks, Unknowns, Dependencies

Measured on the real corpus (13 papers, 89 claims; full output in [`probe-output.txt`](probe-output.txt)):

- **Finding — substring resolution is badly imprecise.** Claims matched by `LIKE` vs. claims actually containing the word: `model` 46 vs 18 (28 via `models`/`modeling`); `net` 12 vs 1 (`network`, `imagenet`, `magnetic`); `art` 13 vs 6 (`apart`, `counterpart`, `partial`); `image` 24 vs 15 (`imagenet`, `images`); `gan` 2 vs **0** (both hits are `organized`).
- **Finding — substring resolution also misses, asymmetrically.** 40 singular/plural pairs co-occur. `LIKE '%networks%'` cannot find the papers that say only `network`, so `dive networks` sees 4 papers where the concept spans 6; likewise `images` (4 of 8) and `languages` (2 of 5).
- **Finding — the stoplist cannot build phrases as-is.** Adjacent word pairs with neither word stopped yield 14 phrases shared by ≥2 papers and **lose** `diffusion models` (6 papers), `attention mechanism(s)`, `bleu scores`, `transformer model`, because `model`/`mechanism`/`scores` are filler. Admitting filler anywhere recovers them but lets in verb fragments (`model establishes`, `demonstrated superior`, `trained text`) and URL debris (`page https`).
- **Finding — the recommended rule works.** Common words and web tokens break phrases; a filler word may be a phrase's *head* but never its modifier; plural inflection folds on every token. Result: 25 shared phrase concepts, with the fragments and URL debris gone and the key phrases kept. Plural folding merges exactly the right variants in both layers — 18 unigram folds (`image/images`, `network/networks`, `family/families`, …) and 8 phrase folds (`diffusion model/models`, `attention mechanism/mechanisms`, `translation task/tasks`, …) — with **no false merge observed**.
- **Risk — residual phrase noise.** 2 of 25 shared phrases are paper boilerplate (`project page`, `brief introduction`). Tunable stoplist data rather than a rule defect; tuning against a 13-paper corpus would overfit, so this is recorded, not chased.
- **Risk — bigram fragments.** `neural machine` and `convolutional neural` are halves of three-word terms. Trigrams would absorb them; deferred as a known consequence.
- **Risk — recall change.** Concept resolution drops substring's accidental derivational matches: `dive attention` no longer reaches `attentional`. Measured: 1 claim in the corpus. Accepted, and recorded in the chapter, because the same mechanism is what removes `organized` from `gan`.
- **Risk — stoplisted queries stop resolving.** `model` is filler, so it forms no concept; `dive model` would drop from 46 (mostly wrong) hits to none. A bare "no concept" is a poor answer to the corpus's most common word, so INT-0020 AC2 now requires naming the concepts that contain the query as a word (`diffusion model`, `transformer model`, `translation model`).
- **Risk — stale concepts.** Concepts depend on the whole corpus (a phrase is a concept only when ≥2 papers share it), so any assertion write can change them, and so can a change to the formation rules. A rebuild that some write path forgets to trigger leaves `dive` silently wrong. Addressed by a fingerprint (claim count, max claim id, formation version) checked on read — new INT-0020 AC7.
- **Risk — non-deterministic output.** Rust's `HashMap` iteration order is randomized per instance, so any formation path that emits in map order would violate AC3 without failing a single-run test. Formation must use ordered collections, and the determinism test must also feed claims in a different order.
- **Unknown — whether plural-only folding is enough.** It found zero false merges here, but 13 papers is a small sample. Traceability (AC6) keeps every merge inspectable if one turns up.
- **Dependency:** none blocking. INT-0019's `DIVER_DB` makes a scratch corpus possible for manual verification.
- **Finding — Sprint 18's evidence was wrong about CI.** Its test report says "CI not configured". In fact `.github/workflows/rust.yml` (`cargo build` + `cargo test` on `ubuntu-latest`, triggered by PRs to and pushes on `main`) was in the `dev` tree throughout Sprint 18 and passed on PR #18 — 142 tests, including both `DIVER_DB` tests. The error is corrected with a dated erratum on the Sprint 18 test report rather than a silent rewrite. Two consequences for this sprint: CI does **not** run on `dev`, so the Test Phase can record only local results and the authoritative CI conclusion arrives with the checkpoint PR; and CI runs neither `clippy` nor `fmt`, so those stay local gates.

## 5. Recommended Approach

Primary: realize INT-0020 in full this sprint — one coherent change rather than two
half-states. Deterministic formation in a new `concept` module: unigram concepts from
significant terms with plural folding; phrase concepts from adjacent pairs under the
common/filler split (filler may head, never modify), kept only when ≥2 papers share
them. The stoplist splits into two files so the category is explicit rather than
inferred from word order. A concept's id is its folded key, stable across rebuilds and
corpus growth; its label is its most frequent surface form, ties broken
lexicographically. Persist concepts, surface forms, and claim links; rebuild in full when
a fingerprint of the stored claims plus a formation-version constant changes. `dive`
resolves the query by folding it the same way, reports an unresolved term with the
concepts containing it, and co-assertion keys on concept ids with IDF and temperature
untouched.

Alternative considered: Snowball stemming via `rust-stemmers`. Rejected — it would
merge `attention`/`attentional`, but it merges by suffix-stripping without regard to
meaning, and a wrong merge is invisible in aggregate output. Plural folding covers every
variant the probe found with no false merge, and anything subtler belongs to a later
propose-then-confirm layer.

Alternative considered: two sprints (formation and dive first, co-assertion re-key
second). Rejected — between them `dive` would find seed papers by concept but link them
by raw token, printing `co-asserts networks` and `co-asserts network` as unrelated
edges. The re-key is small once concepts exist.

Alternative considered: part-of-speech tagging to identify noun phrases. Rejected — it
needs a model or a sizeable dependency, and the frequency-plus-category rule recovered
the important phrases on real data.

Rationale: the probe shows the substring mechanism is wrong in both directions on this
corpus, and that a small, deterministic, inspectable rule fixes both without inventing
any structure. That is exactly INT-0020's premise, now with numbers behind it.

## Artifacts
- [`probe_concepts.py`](probe_concepts.py) — read-only corpus probe (opens the database with `mode=ro`); replicates the tokenizer and evaluates each phrase and folding rule.
- [`probe-output.txt`](probe-output.txt) — its full output on the real corpus, the source of every number in §4.
