Finalized - DO NOT EDIT

# Sprint 13 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC1: IDF weight + keep rule | T-1301 / WHEN `w >= 1.0 - temperature` THEN emit `CoAssertion { term, weight }` | `test_coassertion_weighted_threshold` |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC2: temperature endpoints | T-1301 / WHEN t==1.0 THEN all; WHEN t==0.0 THEN only df==2 | `test_coassertion_temperature_endpoints` |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC2: monotonic in temperature | T-1301 / WHEN kept at t THEN kept at t' >= t | `test_coassertion_temperature_monotonic` |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC3: N≤2 guard | T-1301 / WHEN N<=2 THEN keep all, weight 1.0, no NaN | `test_coassertion_small_corpus_guard` |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC5: weight shown | T-1301 / WHEN relation_reason(CoAssertion) THEN term + 2-dp weight | `test_relation_reason_coassertion_weight` |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC4: CLI flag + validation | T-1302 / WHEN --temperature t (in/out of range, NaN) | `test_parse_temperature` |
| [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) | AC4/AC6: pipeline + docs | T-1303 / WHEN rare+common shared terms THEN low keeps rare only, high keeps both | `test_coassertion_temperature_pipeline`; README updated; full suite |

## Unit Tests

### T-1301 unit tests (`diver-core/src/graph.rs`, `diver-core/src/display.rs`)
- **Intent:** [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md)
- `test_coassertion_weighted_threshold`: corpus where a rare term (low df) and a common term (high df) are both shared by a pair; at a mid temperature the rare-term edge is kept and the common-term edge is dropped. Asserts the emitted `weight` matches `ln(N/df)/ln(N/2)`.
- `test_coassertion_temperature_endpoints`: t==1.0 emits an edge for every shared term; t==0.0 emits an edge only for df==2 terms.
- `test_coassertion_temperature_monotonic`: the set of emitted `(from,to,term)` at a lower temperature is a subset of that at a higher temperature across t ∈ {0.0, 0.5, 1.0}.
- `test_coassertion_small_corpus_guard`: N==2, shared term → one edge with weight 1.0 at t==0.0 (no NaN/∞).
- `test_relation_reason_coassertion_weight`: `relation_reason(CoAssertion { term: "attention", weight: 0.82 })` contains `"attention"` and `"w=0.82"`.
- Updated existing tests (signature migration): `test_coassertion_shared_term`, `test_coassertion_no_self_edges`, `test_coassertion_dedups_repeated_term`, `test_coassertion_disjoint_none`, `test_coassertion_sorted_deterministic`, `test_significant_terms` (unchanged) — pass a temperature (1.0 to preserve prior assertions) and match `CoAssertion { term, .. }`.
- Stubs: none (pure functions over in-memory data).

### T-1302 unit tests (`diver-cli/src/main.rs`)
- **Intent:** [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md)
- `test_parse_temperature`: accepts `"0.0"`, `"0.5"`, `"1.0"`; rejects `"-0.1"`, `"1.1"`, `"NaN"`, and a non-numeric string, each returning `Err`.

## Integration Tests

### Co-assertion temperature pipeline (`diver-core/tests/coassertion.rs`)
- **Intents:** [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md)
- `test_coassertion_temperature_pipeline`: T-1301 + T-1302 composed. Seed ≥3 papers with distinct categories/authors; construct claims so one shared term is rare (df==2) and another is corpus-ubiquitous (df==N). Assert that at a low temperature `compute_coassertion_relations` + `build_dive` surface only the rare-term edge, and at a high temperature both edges appear.
- The existing `test_coassertion_pipeline` is updated to the new signature (temperature 1.0) and asserts `CoAssertion { term, .. }`.

## End-to-End Tests
- **Status:** possible (offline)
- `diver dive --help` lists `--temperature`; the no-data path is unchanged (`display_dive` with empty nodes). Manual verification: `diver dive <concept> --temperature 0.0` / `1.0` run; `--temperature 1.5` errors with non-zero status.
- The populated co-assertion path with weighting is covered at the library level by `test_coassertion_temperature_pipeline`; a seeded-DB binary run is deferred (consistent with INT-0012/INT-0013 — unlocked by a future fixture-DB E2E harness).
