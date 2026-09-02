Finalized - DO NOT EDIT

# Sprint 17 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) | AC1: noise dropped / signal kept | T-1701 / WHEN filler/web token THEN drop; WHEN domain term THEN keep | `test_significant_terms_stoplist` |
| [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) | AC2: O(1) membership | T-1701 / WHEN stoplist consulted THEN HashSet | `HashSet`/`LazyLock` in code (review) |
| [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) | AC3: real-corpus flips to technical | T-1701 | `noise_probe.py` re-run (technical-dominated) + `test_real_corpus_dive` finds a weighted co-assertion edge on a technical term |
| [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) | AC4: docs + no regression | T-1701/T-1702 | `cargo test --workspace`; README updated |

## Unit Tests (graph.rs)
- **Intent:** [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md)
- `test_significant_terms_stoplist`:
  - `significant_terms("The model shows existing results between multiple https github.com repos")`
    contains **none** of `model`, `shows`, `existing`, `results`, `between`, `multiple`, `https`,
    `github`, `com` (all stopped);
  - `significant_terms("attention convolutional diffusion transformer translation bleu")` equals
    `[attention, convolutional, diffusion, transformer, translation, bleu]` (all kept).
- `test_significant_terms` (existing) updated if any asserted word is now stopped; numeric/`gpt3`
  cases unchanged.
- **Migrated fixtures:** `tfidf_corpus`'s `rare/mid/common` → non-stopword tokens keeping the
  df 2/3/4 structure; any `test_coassertion_*` / `coassertion.rs` / `dive_graph.rs` /
  `real_corpus.rs` claim using a now-stopped placeholder → a surviving token, same assertions.
  (These are compile/behavior-forced migrations, done in T-1701.)

## Integration / End-to-End
- `test_real_corpus_dive` (`diver-core/tests/real_corpus.rs`): unchanged assertions, but now the
  ≥1 weighted co-assertion edge it requires must exist on the **de-noised** term set (real
  technical terms like `attention`/`convolutional` survive) — a regression guard that de-noising
  did not empty the graph.

## Validation (research probe, offline)
- Re-run `noise_probe.py` over the persisted real 13-paper corpus after the change: the list of
  terms shared by ≥2 papers should be dominated by technical terms (`transformer`, `attention`,
  `diffusion`, `convolutional`, `translation`, `bleu`, `neural`, `decoder`) with the generic
  filler (`model`, `results`, `existing`, `however`, `https`, `github`) gone. Recorded in the test
  report as before/after evidence.
- Manual (offline, no arXiv): `diver dive attention` shows co-assertion terms free of
  `eight`/`existing`/`literature`/`https`.
