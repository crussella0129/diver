# Test Critique — Sprint 12

## Concerns

### C-001: the populated `diver dive` binary run (with co-assertion edges) is not E2E-tested
- **Where:** `e2e-tests.md` "Coverage note"; `INT-0013` AC5
- **Quote:** "A full binary run over a seeded corpus is out of the automated suite"
- **Failure mode:** e2e-cop-out
- **Why it matters:** the E2E smoke only covers the no-data path; no binary run
  shows a real co-assertion neighborhood printed.
- **Suggested response:** defer-with-rationale — the exact `all_claims` →
  `compute_coassertion_relations` → `build_dive` flow the handler runs is covered
  deterministically by `test_coassertion_pipeline`, which even isolates the
  epistemic edge (distinct categories/authors, so only the shared claim term
  links the papers). The handler change is a two-line `relations.extend(...)`.
  Surface + no-data path (binary) + library pipeline is sufficient; a fixture-DB
  binary test is possible future hardening (consistent with INT-0012).

### C-002: no test asserts co-assertion noise stays bounded on common domain words
- **Where:** `unit-tests.md` T-1201; `graph.rs` `STOPWORDS`/`significant_terms`
- **Quote:** "domain words are not stopped in v1"
- **Failure mode:** weak-relation (untested noise ceiling)
- **Why it matters:** claims sharing only a ubiquitous word ("model", "results")
  would still produce an edge, and no test characterizes that behavior.
- **Suggested response:** defer-with-rationale — this is the intended v1 behavior
  (plan critique C-001): term-significance weighting is future work, and the
  display cap bounds per-node output. `test_coassertion_disjoint_none` and
  `test_significant_terms` pin the term set precisely, so a future stopword/weight
  change is a conscious, tested edit. Characterizing "acceptable noise" is a
  quality-tuning concern for the weighting sprint, not a correctness gate here.

## Confidence
proceed-with-caveats
