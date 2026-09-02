# Sprint 17 Research Report

## Intents Reviewed
- [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) — created; relevance: primary; current state: proposed
- [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) — selected; relevance: the co-assertion weighting this de-noises; current state: realized
- [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) — selected; relevance: the real corpus that surfaced the noise; current state: realized

## 1. Sprint Goal

Cut the co-assertion noise that the Sprint 16 real dive exposed: `dive` links papers by
shared significant claim terms, but generic English/academic words (`eight`, `existing`,
`literature`, `model`, `results`) and even URL tokens (`https`, `github`) create spurious
edges. Replace the tiny function-word `STOPWORDS` with a substantial **common-word stoplist**
(general English + generic research filler + web tokens) so `significant_terms` keeps only
distinctive/technical terms, and IDF then weights *those*. Advances **INT-0018** (INT-0014
follow-on). Baseline: `c08aa82`, `cargo test --workspace` green (131), clippy 0.

## 2. Live probe (ran over the real 13-paper corpus)

A Python replica of `graph.rs::significant_terms` over the persisted claims (7 attention/NMT
+ 6 diffusion papers) ranked the **189 terms shared by ≥2 papers** (the ones that create
co-assertion edges) by document frequency:

- **Generic filler dominates** (high df, and low-df variants): `model`/`models` (10), `data`
  (8), `results`/`training` (7), `propose`/`state`/`time`/`between`/`even` (6), `experiments`/
  `however`/`how`/`limited`/`methods`/`paper`/`proposed`/`show`/`shown`/`trained` (5),
  `approach`/`called`/`different`/`existing`/`large`/`method`/`new`/`novel`/`performance`/
  `recent`/`single`/`work` (4), `complex`/`applications`/`benchmark` (3) …
- **Near-function words** the current 65-word stoplist misses: `even`, `how`, `however`, `due`,
  `between`, `limited`, `multi`.
- **URL junk**: `https` (4), `github` (4) — from project-page links in abstracts.
- **Genuine signal is a minority**: `transformer` (8), `image`/`neural`/`translation` (7),
  `attention`/`diffusion`/`machine` (6), `language`/`visual` (5), `convolutional`/`networks`/
  `text`/`english`/`bleu`/`scores`/`parameter`/`self` (4), `attentive`/`decoder`/`architecture` (3).

**Key finding — IDF cannot fix this.** IDF down-weights *corpus-common* words (`model` df 10 →
low weight), but the observed junk edges are *corpus-rare generic* words: `eight` and `literature`
have df 2, so IDF hands them `w = 1.0`. Only removing generic words **before** weighting fixes it.
So the right lever is the stoplist, not the weighting curve.

## 3. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/graph.rs | high | `STOPWORDS` (graph.rs:155) is ~65 function words; `significant_terms` (graph.rs:168) filters len≥3 + has-letter + `!STOPWORDS.contains`. Replace `STOPWORDS` with a large common-word set; switch membership from a linear `&[&str].contains` to an O(1) `HashSet` (also fixes the prior review's linear-scan note). `compute_coassertion_relations`/weighting unchanged. |
| diver-core/tests/coassertion.rs, dive_graph.rs, real_corpus.rs | med | Existing co-assertion tests use short made-up claims; check none rely on a now-stopped word. `test_real_corpus_dive` still asserts ≥1 co-assertion edge — must stay true after de-noising (real technical terms like `attention`/`transformer` survive). |

Baseline: workspace at `c08aa82`. green (131); clippy 0.

### Design
- Embed a curated stoplist (general-English high-frequency words + generic research filler seen
  in the probe + web/URL tokens) as `stopwords.txt` via `include_str!`, parsed once into a
  `LazyLock<HashSet<&'static str>>`. `significant_terms` checks it (O(1)).
- The list targets exactly the probe's noise (`model`, `data`, `results`, `method`, `approach`,
  `propose`, `novel`, `experiments`, `existing`, `new`, `large`, `even`, `however`, `between`,
  `https`, `github`, …) while never listing domain terms (`attention`, `transformer`,
  `diffusion`, `convolutional`, `translation`, `bleu`), which therefore survive.
- IDF weighting + temperature are unchanged; they now operate on a distinctive term set.

## 4. External Sources
- Common-English word-frequency lists (e.g. the widely-used ~1k most-frequent English words) —
  the basis for the general-frequency portion of the stoplist; combined with the probe's
  observed academic filler and web tokens.

## 5. Risks / Unknowns / Dependencies
- **Over-stopping technical terms.** A common-word list must not contain domain terms. Mitigation:
  curate from a *general* frequency list + the probe's filler, then **validate on the real corpus**
  (re-run the probe: the surviving shared-term list should be dominated by technical terms) and
  add tests asserting technical terms survive while named noise words are dropped.
- **Under-stopping.** Some filler may remain; acceptable — the goal is a large reduction, not
  perfection. Phrase (bigram) detection is a separate, larger enhancement (deferred).
- **A stopped word breaking an existing test.** Audit `coassertion.rs`/`dive_graph.rs` for
  made-up claim words that are now stopped (e.g. `apple`/`mango`/`zebra` are fine — not stopped;
  `attention` survives). Adjust fixtures if needed.
- **Performance.** A larger list makes the linear `.contains` slow; the `HashSet` switch keeps it
  O(1). No new dependency (`std::sync::LazyLock`, `HashSet`).

## 6. Recommended Approach

Replace the function-word `STOPWORDS` with a curated common-word stoplist (general English +
research filler + web tokens) in a `HashSet`, so `significant_terms` drops generic words before
IDF weighting. Validate on the real 13-paper corpus (the probe's shared-term list should flip from
filler-dominated to technical-dominated) and with unit tests pinning that named noise words
(`model`, `results`, `existing`, `eight`, `https`, `github`, `however`) are dropped while domain
terms (`attention`, `convolutional`, `diffusion`, `transformer`) survive. Bigram/phrase detection
is a documented future enhancement.

### Referenced artifacts
- [INT-0018 chapter](../../../intents/INT-0018-coassertion-stoplist.md)
- Build/test plans: `../sprint-plans/`
- Baseline evidence: `cargo test --workspace` 131/131, clippy 0 at `c08aa82`
