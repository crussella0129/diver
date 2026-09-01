# Sprint 12 Unit Tests

- **Tested head:** `473197e821fdbd1a6b5106c5a810338c5caa2030`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets`
- **Result:** `diver_core` lib unittests — **108 passed; 0 failed** (99 prior + 9
  new); `diver-cli` bin — 0 tests. Clippy: 0 warnings.

## New tests (INT-0013)

### T-1201 — co-assertion graph (diver-core/src/graph.rs, display.rs)
- **Intent:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md) — AC1/AC2/AC3
- `test_significant_terms`: "Attention improves the RNN accuracy!" →
  `[attention, improves, rnn, accuracy]` ("the" stopped, punctuation/case ignored,
  3-char acronym kept); "AI is ML" → empty (2-char tokens dropped). **pass** (AC3)
- `test_coassertion_shared_term`: two papers both asserting "attention" → one
  `CoAssertion("attention")` edge with correct from/to. **pass** (AC2)
- `test_coassertion_no_self_edges`: one paper, and two claims for the same paper →
  no edge (grouped into one node). **pass**
- `test_coassertion_dedups_repeated_term`: a term repeated across a paper's claims →
  one edge. **pass**
- `test_coassertion_disjoint_none`: papers with no shared significant term → none. **pass**
- `test_coassertion_sorted_deterministic`: three shared terms → edges in sorted
  order (apple, mango, zebra), stable output. **pass**
- `test_relation_reason_coassertion`: `relation_reason(CoAssertion("attention"))`
  contains "co-asserts" and "attention". **pass** (AC1)

### T-1202 — `Store::all_claims` (diver-core/src/store.rs)
- **Intent:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md) — AC4
- `test_all_claims`: two papers (one with two claims) → all 3 `(arxiv_id, claim)`
  rows returned. **pass**
- `test_all_claims_empty`: no stored assertions → empty vec. **pass**

## Raw result
```
cargo clippy --workspace --all-targets  →  0 warnings
Running unittests src\lib.rs (diver_core)  →  108 passed; 0 failed
```
