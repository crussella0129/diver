# Test Critique — Sprint 11

## Concerns

### C-001: the populated `diver dive` binary run is not E2E-tested
- **Where:** `e2e-tests.md` "Coverage note"; `INT-0012` AC4
- **Quote:** "A full binary run would require a seeded real DB"
- **Failure mode:** e2e-cop-out
- **Why it matters:** the E2E smokes cover the CLI surface and the empty path, but
  no binary run exercises `dive` over a corpus with extracted assertions and
  prints a real neighborhood.
- **Suggested response:** defer-with-rationale — the exact seed → `compute_relations`
  → `build_dive` neighborhood the handler renders is covered deterministically by
  `test_dive_pipeline` (a node with its claim and a `SharedCategory` related
  paper), and the handler is thin glue over that public API. A binary run needs a
  seeded real DB (network ingest + LLM extract, or a fixture DB), which would make
  the E2E stateful/costly for no added logic coverage. Surface + empty path
  (binary) + library neighborhood is sufficient; a fixture-DB binary test is
  possible future hardening.

### C-002: relation edges are exact category/author matches — no semantic relatedness
- **Where:** `unit-tests.md` T-1101; `graph.rs` `compute_relations`
- **Quote:** "one `SharedCategory` edge per shared category code and one
  `SharedAuthor` edge per shared author"
- **Failure mode:** weak-assertion (shallow graph)
- **Why it matters:** two papers about the same topic in different categories, or
  by different authors, won't be linked — the graph may feel sparse.
- **Suggested response:** defer-with-rationale — intentional for v1 per INT-0012's
  non-goals: exact category/author edges are deterministic and reproducible, which
  is the right foundation. `RelationKind` is an extensible enum, so semantic /
  co-assertion / citation edges are additive in a later sprint without changing
  the `dive` pipeline. The tests pin the intended exact-match behavior.

## Confidence
proceed-with-caveats
