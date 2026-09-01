Finalized - DO NOT EDIT

# Sprint 11 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC1: compute_relations edges | T-1101 / shared cat/author → edge; disjoint → none; no self-edges | `test_compute_relations_shared_category`, `_shared_author`, `_no_edges_when_disjoint`, `_no_self_edges` |
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC2: papers_asserting seed query | T-1102 / claim substring case-insensitive; empty | `test_papers_asserting_matches`, `test_papers_asserting_empty` |
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC3: build_dive assembly | T-1101 / node w/ claims + related | `test_build_dive_assembles_neighborhood` |
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC4: dive displays / empty message | T-1103 / neighborhood; none → message exit 0 | `test_dive_pipeline` + e2e `dive --help`, `dive <none>` |
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC5: docs + no regression | T-1103 + all tasks | README updated; full `cargo test --workspace` |

## Unit Tests

### T-1101 unit tests (diver-core/src/graph.rs)
- **Intent:** [INT-0012](../../../intents/INT-0012-graph-dive.md)
- `test_compute_relations_shared_category`: two facts with a common category code
  → one `SharedCategory(code)` edge (`from`/`to` correct).
- `test_compute_relations_shared_author`: two facts with a common author → one
  `SharedAuthor(name)` edge.
- `test_compute_relations_no_edges_when_disjoint`: facts with disjoint
  categories+authors → no edges.
- `test_compute_relations_no_self_edges`: a single fact (and a duplicate id) →
  no self-edge; only `i < j` pairs considered.
- `test_build_dive_assembles_neighborhood`: two category-sharing facts, one
  asserting a matching claim → one `DiveNode` for the asserting paper with its
  title, its claim, and a `SharedCategory` relation to the other paper.

### T-1102 unit tests (diver-core/src/store.rs)
- **Intent:** [INT-0012](../../../intents/INT-0012-graph-dive.md)
- `test_papers_asserting_matches`: `save_assertions` claims incl. "Attention
  improves accuracy." → `papers_asserting("attention")` returns that (id, claim)
  (case-insensitive); a non-matching concept excludes it.
- `test_papers_asserting_empty`: unknown concept / no stored assertions → empty vec.

## Integration Tests

### Dive graph pipeline (diver-core/tests/dive_graph.rs)
- **Intents:** [INT-0012](../../../intents/INT-0012-graph-dive.md) (AC4, AC5)
- `test_dive_pipeline`: `open_in_memory` → save two papers that share a category →
  `save_assertions` on paper A with a claim mentioning the concept →
  `store.papers_asserting(concept)` + `compute_relations(&store.list())` +
  `build_dive` → a single `DiveNode` for paper A carrying its claim and a
  `SharedCategory` relation whose other endpoint is paper B. Imports via
  `diver_core::`.

## End-to-End Tests
- **Status:** possible (offline)
- `e2e_dive_help` (scripted smoke): `cargo run -p diver-cli -- dive --help` exits 0
  and shows `<CONCEPT>`; `diver --help` lists `dive`.
- `e2e_dive_no_data` (scripted smoke): `cargo run -p diver-cli -- dive
  zzznonexistentconcept` prints the actionable "No papers assert about…" message
  and exits 0 (empty is not an error). Read-only DB access.
