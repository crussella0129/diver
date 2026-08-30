# Sprint 6 Unit Tests

- **Tested head:** `6b4dbef6fcc81bbb187a45d6a5e533a3e26845c3`
- **Runner:** `cargo test --workspace` (no CI configured in this repo; local
  cargo is the canonical suite runner)
- **Result:** `diver_core` lib unittests — **65 passed; 0 failed** (62 carried
  over from Sprint 5 + 3 new this sprint). `diver-cli` bin unittests — 0 tests.

## New tests this sprint (INT-0006 regression coverage)

### T-601 — `test_fk_constraint_enforced` (diver-core/src/store.rs)
- **Intent:** [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) — AC2
- **EARS:** WHEN a row is inserted into `paper_versions` with a `paper_id` absent
  from `papers`, THEN the store connection SHALL return a foreign-key constraint
  error.
- **Arrangement → assertion:** direct `INSERT INTO paper_versions` with
  `paper_id = 99999` (no parent) → `result.is_err()` **and** error string
  contains `FOREIGN KEY`. Negative-path test; proves `PRAGMA foreign_keys=ON`
  is live, not merely declared.
- **Executed:** pass.

### T-602 — `test_reingest_older_version_keeps_latest_in_fts` (diver-core/src/store.rs)
- **Intent:** [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) — AC3
- **EARS (2 clauses):**
  - WHEN v2 (ingested_at Tb) then older v1 (ingested_at Ta<Tb) is saved, THEN
    `search()` for a v2-unique term SHALL return that paper.
  - WHEN that sequence occurs, THEN `search()` for a v1-unique term SHALL return
    no results.
- **Arrangement → assertion:** save v2 (`latestquantumfoo`, Tb=02:00), then v1
  (`olderclassicbar`, Ta=01:00) → `search("latestquantumfoo").len() == 1` **and**
  `search("olderclassicbar").is_empty()`. Both clauses asserted.
- **Executed:** pass.

### T-603 — `test_taxonomy_parse_repeated_consistent` (diver-core/src/id.rs)
- **Intent:** [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) — AC4
- **EARS:** WHEN `ArxivCategory::parse` is invoked repeatedly and interleaved
  across multiple valid and invalid codes, THEN every call SHALL return the
  correct result for its own code.
- **Arrangement → assertion:** 3 iterations over `cs.CV`/`math.NA`/`stat.ML`
  (name assertions) + `invalid.XX` (`is_err()`). Guards the `OnceLock` taxonomy
  cache. Includes a negative path (`invalid.XX`).
- **Executed:** pass.

## INT-0007 unit coverage (restructure)
- The full 65-test `diver_core` lib suite compiles and passes **under the new
  crate layout**, proving the modules and the embedded taxonomy moved correctly.
  In particular `test_taxonomy_valid_code` passing proves INT-0007 AC3
  (`include_str!("../taxonomy/arxiv_categories.json")` resolves from
  `diver-core/src/id.rs`).

## Raw result
```
Running unittests src\lib.rs (diver_core)
running 65 tests
test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
Running unittests src\main.rs (diver-cli)
running 0 tests
test result: ok. 0 passed; 0 failed
```
