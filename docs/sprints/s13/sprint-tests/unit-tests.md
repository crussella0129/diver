# Sprint 13 Unit Tests

- **Tested head:** `74474dd70531b36456465ee96f894a124b66acc2`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets`
- **Result:** `diver_core` lib unittests — **112 passed; 0 failed** (108 prior + 4
  new co-assertion weighting tests; `test_relation_reason_coassertion` renamed to
  `_weight`). `diver-cli` bin unittests — **1 passed; 0 failed** (new). Clippy: 0 warnings.

## New / updated tests (INT-0014)

### T-1301 — IDF weighting + temperature threshold (diver-core/src/graph.rs, display.rs)
- **Intent:** [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) — AC1/AC2/AC3/AC5
- `test_coassertion_weighted_threshold`: 4-paper corpus (`rare` df 2, `mid` df 3,
  `common` df 4). At `t=0.5` only `rare` (w=1.0) links A–B; `mid`/`common` drop. At
  `t=1.0` the `mid` edge carries weight `ln(4/3)/ln(4/2)` (asserted to 1e-9). **pass** (AC1)
- `test_coassertion_temperature_endpoints`: `t=1.0` → A–B shares all three terms;
  `t=0.0` → only `df==2` (`rare`) survives, on exactly the one pair. **pass** (AC2)
- `test_coassertion_temperature_monotonic`: edge set at `t=0.0 ⊆ t=0.5 ⊆ t=1.0`,
  and strictly grows here. **pass** (AC2)
- `test_coassertion_small_corpus_guard`: N==2 at `t=0.0` → the shared term kept
  with finite weight `1.0` (no NaN from `ln(N/2)==0`). **pass** (AC3)
- `test_relation_reason_coassertion_weight`: `relation_reason(CoAssertion { term:
  "attention", weight: 0.82 })` contains "co-asserts", "attention", and "w=0.82". **pass** (AC5)
- Migrated (signature/enum): `test_coassertion_shared_term`, `_no_self_edges`,
  `_dedups_repeated_term`, `_disjoint_none`, `_sorted_deterministic` — pass a
  temperature and match `CoAssertion { term, weight }`. **pass**

### T-1302 — `--temperature` parser (diver-cli/src/main.rs)
- **Intent:** [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) — AC4
- `test_parse_temperature`: accepts `0.0`/`0.5`/`1.0`; rejects `-0.1`, `1.1`,
  `NaN`, `inf`, and `warm` (each `Err`). **pass** (AC4 negative paths)

## Raw result
```
cargo clippy --workspace --all-targets  →  0 warnings
Running unittests src\lib.rs (diver_core)  →  112 passed; 0 failed
Running unittests src\main.rs (diver-cli)  →    1 passed; 0 failed
```
