Finalized - DO NOT EDIT

# Sprint 12 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC1: CoAssertion variant + display | T-1201 / variant + relation_reason arm | `test_relation_reason_coassertion` |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC2: co-assertion edges | T-1201 / shared term → edge; no self; dedup; sorted | `test_coassertion_shared_term`, `_no_self_edges`, `_dedups_repeated_term`, `_disjoint_none`, `_sorted_deterministic` |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC3: significant_terms tokenization | T-1201 / alnum+lower+len>=3+stopwords | `test_significant_terms` |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC4: all_claims | T-1202 / returns all; empty | `test_all_claims`, `test_all_claims_empty` |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC5: dive includes co-assertion | T-1203 / pipeline edge | `test_coassertion_pipeline` |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC6: docs + no regression | T-1203 + all tasks | README updated; full `cargo test --workspace` |

## Unit Tests

### T-1201 unit tests (diver-core/src/graph.rs, display.rs)
- **Intent:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md)
- `test_significant_terms`: "Attention improves the RNN accuracy!" → `["attention",
  "improves", "rnn", "accuracy"]` ("the" is a stopword; punctuation/case ignored;
  a 2-char token like "AI" would be dropped by the length ≥ 3 rule — asserted).
- `test_coassertion_shared_term`: two papers whose claims both contain "attention"
  → one `CoAssertion("attention")` edge with correct from/to.
- `test_coassertion_no_self_edges`: one paper (and a duplicated id) → no edge.
- `test_coassertion_dedups_repeated_term`: a paper repeating a term across claims →
  at most one edge per shared term per pair.
- `test_coassertion_disjoint_none`: papers with no shared significant term → none.
- `test_coassertion_sorted_deterministic`: two shared terms → edges in sorted term
  order (stable across runs).
- `test_relation_reason_coassertion`: `relation_reason(&CoAssertion("attention"))`
  contains "co-asserts" and "attention".

### T-1202 unit tests (diver-core/src/store.rs)
- **Intent:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md)
- `test_all_claims`: `save_assertions` for two papers → `all_claims()` returns all
  their `(arxiv_id, claim)` rows.
- `test_all_claims_empty`: no stored assertions → empty vec.

## Integration Tests

### Co-assertion pipeline (diver-core/tests/coassertion.rs)
- **Intents:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md) (AC5, AC6)
- `test_coassertion_pipeline`: `open_in_memory` → save two papers (distinct
  categories/authors so the only link is epistemic) → `save_assertions` on each
  with a claim sharing the term "attention" → `compute_coassertion_relations(&store.all_claims())`
  yields a `CoAssertion("attention")` edge between them; `build_dive` for a seed
  lists that co-assertion relation. Imports via `diver_core::`.

## End-to-End Tests
- **Status:** possible (offline)
- `e2e_dive_help` (scripted smoke): `cargo run -p diver-cli -- dive --help` still
  exits 0 with `<CONCEPT>` (surface unchanged). The populated co-assertion path is
  covered at the library level by `test_coassertion_pipeline`; a binary run over a
  seeded corpus is out of the automated suite (needs a seeded DB — same rationale
  as INT-0012).
