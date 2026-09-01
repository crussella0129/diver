# Sprint 12 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC1: CoAssertion variant + display | T-1201 / `test_relation_reason_coassertion` | pass | Test evidence links this report |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC2: co-assertion edges | T-1201 / `test_coassertion_shared_term`, `_no_self_edges`, `_dedups_repeated_term`, `_disjoint_none`, `_sorted_deterministic` | pass | Test evidence links this report |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC3: significant_terms tokenization | T-1201 / `test_significant_terms` | pass | Test evidence links this report |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC4: all_claims | T-1202 / `test_all_claims`, `test_all_claims_empty` | pass | Test evidence links this report |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC5: dive includes co-assertion | T-1203 / `test_coassertion_pipeline` + e2e | pass (populated path via library test, critique C-001) | Test evidence links this report |
| [INT-0013](../../../intents/INT-0013-coassertion-relations.md) | AC6: docs + no regression | T-1203 + full suite | pass (116/116; README updated) | Test evidence links this report |

## Summary
- Unit tests: 108 passed / 0 failed / 108 total (`diver_core`); 0 in `diver-cli`
- Integration tests: 8 passed / 0 failed / 8 total (incl. new `coassertion`)
- E2E tests: 1 passed / 0 failed / 1 total (`dive` no-data path unchanged)
- Clippy: 0 warnings
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `473197e821fdbd1a6b5106c5a810338c5caa2030`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` →
  `test result: ok` for every binary (`diver_core` lib 108, `coassertion` 1,
  `dive_graph` 1, `dive_pipeline` 1, `extract_pipeline` 1, `ingest_pipeline` 2,
  `llm_extract_pipeline` 1, `persist_pipeline` 1, `diver-cli` bin 0). `cargo build`
  clean; `diver dive` still runs (no-data path unchanged). `cargo clippy
  --workspace --all-targets` → 0 warnings. Records: [unit](unit-tests.md),
  [integration](integration-tests.md), [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- The populated `dive` binary round-trip with co-assertion edges is covered at the
  library level, not through the binary (critique C-001).
- Co-assertion uses unweighted shared-term overlap — common domain words can
  over-link; term significance weighting (TF-IDF / document frequency), phrase
  detection, and a short-acronym allowlist are future refinements on the same
  `significant_terms`/`RelationKind` seam (critique C-002 + plan critique C-001/C-002).
- Relation computation remains O(P²) (structural + co-assertion); an inverted
  term→papers index is a future scale optimization (plan critique C-003).

## Coverage Observations
- Every acceptance criterion has a named, executed test asserting the SHALL
  response, including negative paths (disjoint → no edge, no self-edges, empty
  claims, 2-char/stopword tokens dropped).
- Tests are deterministic: fixed inputs, sorted term emission; E2E offline.
- The integration test isolates the epistemic edge (distinct categories/authors),
  proving co-assertion links papers that structural edges would not.
