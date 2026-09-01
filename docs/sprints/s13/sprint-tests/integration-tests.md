# Sprint 13 Integration Tests

- **Tested head:** `74474dd70531b36456465ee96f894a124b66acc2`
- **Runner:** `cargo test --workspace`
- **Result:** `tests/coassertion.rs` — **2 passed; 0 failed** (1 prior migrated + 1
  new). All other integration binaries unchanged and green (`dive_graph` 1,
  `dive_pipeline` 1, `extract_pipeline` 1, `ingest_pipeline` 2,
  `llm_extract_pipeline` 1, `persist_pipeline` 1).

## New test (INT-0014)

### T-1303 — co-assertion temperature pipeline (diver-core/tests/coassertion.rs)
- **Intent:** [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) — AC4/AC6 (composed AC1–AC3, AC5)
- `test_coassertion_temperature_pipeline`: seeds 4 papers with distinct
  categories/authors; claims make `attention` rare (df 2, w=1.0) and `models`
  ubiquitous (df 4, w=0.0). Asserts:
  - at `--temperature 0.0` exactly one edge survives — `CoAssertion { term:
    "attention", weight: 1.0 }` on the A–B pair — and no `models` edge exists;
  - at `--temperature 1.0` strictly more edges appear, including `models` edges;
  - the distinctive `attention` edge still surfaces in Paper A's `build_dive`
    neighborhood. **pass**

### Migrated
- `test_coassertion_pipeline`: the Sprint 12 pipeline test, updated to the new
  signature/enum (temperature `1.0`, `CoAssertion { term, weight }`). **pass**

## Isolation / determinism
- All stores are `Store::open_in_memory()`; inputs are fixed; term emission is
  sorted; no network, clock, or randomness. The temperature test isolates the
  epistemic edge by giving papers distinct categories/authors and passing only the
  co-assertion relation set to `build_dive`.

## Raw result
```
Running tests\coassertion.rs  →  2 passed; 0 failed
```
