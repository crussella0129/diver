# Sprint 13 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC1: IDF weight + keep rule | T-1301 / `test_coassertion_weighted_threshold` | pass | Test evidence links this report |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC2: temperature endpoints + monotonic | T-1301 / `test_coassertion_temperature_endpoints`, `_monotonic` | pass | Test evidence links this report |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC3: N≤2 guard | T-1301 / `test_coassertion_small_corpus_guard` | pass | Test evidence links this report |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC4: CLI flag + validation | T-1302/T-1303 / `test_parse_temperature` + e2e (help, exit 2) | pass (structural-ungated by construction, critique C-001) | Test evidence links this report |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC5: weight shown | T-1301 / `test_relation_reason_coassertion_weight` | pass | Test evidence links this report |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC6: docs + no regression | T-1303 / `test_coassertion_temperature_pipeline` + full suite | pass (122/122; README updated) | Test evidence links this report |

## Summary
- Unit tests: 112 passed / 0 failed / 112 total (`diver_core` lib) + 1 passed (`diver-cli` bin)
- Integration tests: 9 passed / 0 failed (incl. `coassertion` 2 — 1 migrated + 1 new)
- E2E tests: offline manual — `--help` flag, out-of-range → exit 2, valid runs (pass)
- Clippy: 0 warnings
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `74474dd70531b36456465ee96f894a124b66acc2`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` →
  `test result: ok` for every binary (`diver_core` lib 112, `diver-cli` bin 1,
  `coassertion` 2, `dive_graph` 1, `dive_pipeline` 1, `extract_pipeline` 1,
  `ingest_pipeline` 2, `llm_extract_pipeline` 1, `persist_pipeline` 1) = 122 total.
  `cargo build` clean; `diver dive --temperature` runs (help + validation + no-data
  path). `cargo clippy --workspace --all-targets` → 0 warnings. Records:
  [unit](unit-tests.md), [integration](integration-tests.md), [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- "Structural edges are ungated by temperature" (AC4) is guaranteed by construction
  (`compute_relations` has no temperature parameter) but not directly asserted —
  critique C-001, deferred.
- The `--temperature` default (0.5) is evidenced by `--help` output, not a
  library/unit assertion — critique C-002, deferred (same populated-DB E2E gap as
  INT-0012/INT-0013).
- TF (within-paper term frequency) weighting, phrase detection, and observed-max
  normalization remain future refinements on the same `RelationKind`/weighting seam
  (INT-0014 non-goals / Alternatives).
- Relation computation remains O(P²) over the corpus per `dive` (INT-0012/0013 note).

## Coverage Observations
- Every acceptance criterion has a named, executed test asserting the SHALL response,
  including negative paths (out-of-range/NaN temperature rejected; ubiquitous-term
  edges dropped at low temperature; N≤2 guard against NaN).
- Tests are deterministic: in-memory stores, fixed inputs, sorted emission, exact or
  1e-9-epsilon float assertions, no network/clock/randomness.
- The integration test isolates the epistemic edge and proves the low-vs-high
  temperature edge-set difference end-to-end through the handler's exact flow.
