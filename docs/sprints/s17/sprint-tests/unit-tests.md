# Sprint 17 Unit Tests

- **Tested head:** `0aaa31cfeb486ccdee045905dc42f9fb7af24077`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets`
- **Result:** `diver_core` lib — **121 passed; 0 failed** (+1 net new); `diver-cli` bin — 1. Clippy: 0.

## New / updated (INT-0018), `diver-core/src/graph.rs`
- **Intent:** [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) — AC1/AC2
- `test_significant_terms_stoplist` (new): `significant_terms("The model shows existing results
  between multiple https github.com repos")` drops **all** of `the`/`model`/`shows`/`existing`/
  `results`/`between`/`multiple`/`https`/`github`/`com`; `significant_terms("attention
  convolutional diffusion transformer translation bleu")` keeps **all six** domain terms. **pass** (AC1)
- `test_significant_terms` (updated): now reflects the stoplist — `improves` dropped from
  `"Attention improves the RNN accuracy!"` → `[attention, rnn, accuracy]`; `trained`/`data`/`with`
  dropped from the number-token case → `[epochs, gpt3]`. **pass**
- AC2 (O(1) membership) is structural: `static STOPWORDS: LazyLock<HashSet<&'static str>>`
  built once from `include_str!("stopwords.txt")`, replacing the old linear `&[&str].contains`.

## Migrated fixture
- `diver-core/tests/coassertion.rs::test_coassertion_temperature_pipeline`: its ubiquitous
  placeholder term `models` is now stopped, so it was renamed to `networks` (a non-stopword),
  preserving the df 2/3/4 structure and all assertions. **pass**

## Raw result
```
cargo clippy --workspace --all-targets  →  0 warnings
Running unittests src\lib.rs (diver_core)  →  121 passed; 0 failed
```
