# Sprint 17 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) | AC1: noise dropped / signal kept | T-1701 / `test_significant_terms_stoplist`, `test_significant_terms` | pass | Test evidence links this report |
| [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) | AC2: O(1) membership | T-1701 / `LazyLock<HashSet>` (structural) | pass (code review, critique C-002) | Test evidence links this report |
| [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) | AC3: real-corpus flips to technical | T-1701 / probe 189→92 + `test_real_corpus_dive` | pass (probe = one-time metric, critique C-001) | Test evidence links this report |
| [INT-0018](../../../intents/INT-0018-coassertion-stoplist.md) | AC4: docs + no regression | T-1701/T-1702 + full suite | pass (132/132; README updated) | Test evidence links this report |

## Summary
- Unit tests: 121 passed (`diver_core` lib, +1 net: `test_significant_terms_stoplist` added,
  `test_significant_terms` updated) + 1 (`diver-cli` bin)
- Integration: `real_corpus` regression guard still green (edge on a technical term);
  `coassertion` fixture updated (`models`→`networks`); all other binaries green
- E2E / validation: probe over the real 13-paper corpus (shared terms **189 → 92**,
  technical-dominated) + manual `dive` showing de-noised edges (`encoder`/`decoder`/`bleu` vs
  the old `eight`/`existing`/`https`)
- Clippy: 0 warnings
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `0aaa31cfeb486ccdee045905dc42f9fb7af24077`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` → `test result: ok`
  for every binary = **132 total**. `cargo build` clean; `cargo clippy --workspace --all-targets`
  → 0. Records: [unit](unit-tests.md), [integration](integration-tests.md), [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- The 189→92 flip is a research-script metric, not a committed invariant (the committed guard is
  `test_real_corpus_dive`) — critique C-001.
- AC2 O(1) is structural (HashSet), not runtime-tested; also retires the prior linear-scan
  finding — critique C-002.
- A few generic stragglers survive (`multi`, `low` — tokenizer fragments); phrase/bigram
  co-assertion is the cleaner follow-on for hyphenated fragments — critique C-003.

## Coverage Observations
- AC1 is pinned directly on named noise and named domain terms; AC3 is validated both by the
  measured real-corpus flip and the `test_real_corpus_dive` regression guard (de-noising did not
  empty the graph).
- Deterministic throughout; the manual dive and probe are reproducible over the persisted corpus.
