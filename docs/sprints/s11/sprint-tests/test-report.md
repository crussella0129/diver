# Sprint 11 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC1: compute_relations edges | T-1101 / `test_compute_relations_*` (4) | pass | Test evidence links this report |
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC2: papers_asserting seed query | T-1102 / `test_papers_asserting_matches`, `_empty` | pass | Test evidence links this report |
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC3: build_dive assembly | T-1101 / `test_build_dive_assembles_neighborhood`, `_groups_claims_by_paper` | pass | Test evidence links this report |
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC4: dive displays / empty message | T-1103 / `test_dive_pipeline` + e2e smokes | pass (populated path via library test, critique C-001) | Test evidence links this report |
| [INT-0012](../../../intents/INT-0012-graph-dive.md) | AC5: docs + no regression | T-1103 + full suite | pass (103/103; README updated) | Test evidence links this report |

## Summary
- Unit tests: 96 passed / 0 failed / 96 total (`diver_core`); 0 in `diver-cli`
- Integration tests: 7 passed / 0 failed / 7 total (incl. new `dive_graph`)
- E2E tests: 2 passed / 0 failed / 2 total (`dive --help`, `dive <none>`)
- Clippy: 0 warnings
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `de281c76affc76e696f03f55331b8ae3de0c8aeb`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` →
  `test result: ok` for every binary (`diver_core` lib 96, `dive_graph` 1,
  `dive_pipeline` 1, `extract_pipeline` 1, `ingest_pipeline` 2,
  `llm_extract_pipeline` 1, `persist_pipeline` 1, `diver-cli` bin 0). `cargo build`
  clean; `diver --help` lists `dive`; `diver dive zzznonexistentconcept` prints the
  actionable message and exits 0. `cargo clippy --workspace --all-targets` → 0
  warnings. Records: [unit](unit-tests.md), [integration](integration-tests.md),
  [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- The populated `dive` binary round-trip is not in the automated suite (needs a
  seeded DB); covered at the library level by `test_dive_pipeline` (critique C-001).
- Relation edges are exact category/author matches; semantic / co-assertion /
  citation edges are future work on the extensible `RelationKind` (critique C-002).
- Relation computation is O(n²) over stored papers — fine at local scale; an
  indexed/persisted graph is a future optimization (per INT-0012).

## Coverage Observations
- Every acceptance criterion has a named, executed test asserting the SHALL
  response, including negative paths (disjoint → no edges, no self-edges, unknown
  concept → empty, no-data dive → actionable message exit 0).
- Tests are deterministic: in-memory SQLite + fixed fixtures; E2E offline.
- `dive` traverses the persisted, validated assertion layer — the graph reflects
  the extracted knowledge by design.
