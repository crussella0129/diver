# Sprint 11 Unit Tests

- **Tested head:** `de281c76affc76e696f03f55331b8ae3de0c8aeb`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets`
- **Result:** `diver_core` lib unittests — **96 passed; 0 failed** (88 prior + 8
  new); `diver-cli` bin — 0 tests. Clippy: 0 warnings.

## New tests (INT-0012)

### T-1101 — `graph.rs` (diver-core/src/graph.rs)
- **Intent:** [INT-0012](../../../intents/INT-0012-graph-dive.md) — AC1/AC3
- `test_compute_relations_shared_category`: two papers sharing `cs.CL` → one
  `SharedCategory("cs.CL")` edge with correct `from`/`to`. **pass**
- `test_compute_relations_shared_author`: shared author → one `SharedAuthor` edge. **pass**
- `test_compute_relations_no_edges_when_disjoint`: disjoint categories+authors →
  no edges. **pass**
- `test_compute_relations_no_self_edges`: single paper → none; duplicated id →
  no self-edge. **pass**
- `test_build_dive_assembles_neighborhood`: one asserting paper → a `DiveNode` with
  title, claim, and a `SharedCategory` relation to the other paper. **pass**
- `test_build_dive_groups_claims_by_paper`: two claims for one paper group into a
  single node with two claims. **pass**

### T-1102 — `Store::papers_asserting` (diver-core/src/store.rs)
- **Intent:** [INT-0012](../../../intents/INT-0012-graph-dive.md) — AC2
- `test_papers_asserting_matches`: `papers_asserting("ATTENTION")` returns the
  `(id, claim)` for the "Attention improves accuracy." claim (case-insensitive);
  a non-matching concept returns nothing. **pass**
- `test_papers_asserting_empty`: unknown concept / no stored assertions → empty. **pass**

## Raw result
```
cargo clippy --workspace --all-targets  →  0 warnings
Running unittests src\lib.rs (diver_core)  →  96 passed; 0 failed
```
