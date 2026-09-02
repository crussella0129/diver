Finalized - DO NOT EDIT

# Sprint 17 Build Plan

## Intents
- [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) — state: planned; acceptance criteria covered: AC1, AC2, AC3, AC4

## Schema Tree
- Sprint Goal: de-noise co-assertion terms with a common-word stoplist
  - Stoplist (diver-core)
    - T-1701: `stopwords.txt` + `LazyLock<HashSet>` + `significant_terms` + tests
  - Docs
    - T-1702: README note

## Execution Sequence

### T-1701: common-word stoplist + significant_terms + tests
- **Intent:** [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md)
- **Touches:** `diver-core/src/graph.rs`, `diver-core/src/stopwords.txt` (new)
- **Depends on:** (none)
- **Acceptance criterion:** AC1 (noise dropped / signal kept), AC2 (O(1) membership), AC3 (real-corpus flips technical)
- **Success criterion (EARS):**
  - **WHEN** `significant_terms` sees a common English word, a generic research-filler word (`model`/`results`/`existing`/`method`/`propose`/…), a near-function word (`however`/`between`/`due`/`even`), or a web token (`https`/`http`/`www`/`github`), **THEN** it **SHALL** exclude it.
  - **WHEN** `significant_terms` sees a domain term (`attention`/`convolutional`/`diffusion`/`transformer`/`translation`/`bleu`), **THEN** it **SHALL** keep it.
  - **WHEN** the stoplist is consulted, **THEN** membership **SHALL** be O(1) (a `HashSet` built once, not a linear scan).
- **Notes:** Add `stopwords.txt` (curated: ~300–600 most-common English words + the probe's generic research filler + web/URL tokens; **no** domain terms). Add `static STOPWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| include_str!("stopwords.txt").split_whitespace().collect());` and switch `significant_terms` to `!STOPWORDS.contains(t.as_str())`. New `test_significant_terms_stoplist`. **Adjust existing co-assertion test fixtures** whose placeholder words are now stopped: `tfidf_corpus`'s `"rare mid common"` → non-stopword tokens keeping df 2/3/4; audit `test_coassertion_*` (graph.rs), `coassertion.rs`, `dive_graph.rs`, `real_corpus.rs` for stopped words (`models`/`accuracy`/`cost`/`improves`/`attention`(keep)) and update each to a surviving term with the same assertion. Re-run the research `noise_probe.py` to confirm the flip.

### T-1702: docs
- **Intent:** [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md)
- **Touches:** `README.md`
- **Depends on:** T-1701
- **Acceptance criterion:** AC4 (docs)
- **Success criterion (EARS):**
  - **WHEN** the README describes co-assertion, **THEN** it **SHALL** note that links are by distinctive shared terms (a common-word stoplist filters generic vocabulary).
- **Notes:** One or two lines in the `dive` / concept-exploration section; no behavior claim beyond the stoplist. IDF/temperature wording unchanged.
