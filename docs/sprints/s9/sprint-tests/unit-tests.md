# Sprint 9 Unit Tests

- **Tested head:** `cb613a38d985818e08ca2ed22b8781fa28411424`
- **Runner:** `cargo test --workspace` (no CI; local cargo is canonical)
- **Result:** `diver_core` lib unittests — **88 passed; 0 failed** (82 prior + 6
  new); `diver-cli` bin — 0 tests.

## New tests (INT-0010, diver-core/src/store.rs)

### T-901 — schema + `save_assertions`
- **Intent:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) — AC1/AC2/AC3/AC6
- `test_assertion_schema_created`: after `open_in_memory`, `sqlite_master` contains
  both `assertions` and `assertion_support` tables. **pass** (AC1)
- `test_save_assertions_persists`: save two supported assertions (1 + 2 support
  quotes) → `assertions` count = 2, `assertion_support` count = 3, first claim
  correct. **pass** (AC2)
- `test_save_assertions_idempotent_replace`: save 2 then re-save 1 for the same
  `(paper, version)` → `assertions` = 1 **and** `assertion_support` = 1, proving no
  duplicates and that the `ON DELETE CASCADE` dropped the orphaned support. **pass**
  (AC3; critique C-003)
- `test_assertion_support_fk_enforced`: direct insert into `assertion_support` with
  an absent `assertion_id` → `Err` containing `FOREIGN KEY`. **pass** (AC6, negative)

### T-902 — `get_assertions`
- **Intent:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) — AC4
- `test_get_assertions_round_trip`: save 2 → `get_assertions` returns 2
  `StoredAssertion`s with matching claims, version `v2`, and the multi-quote
  assertion's 2 support quotes. **pass**
- `test_get_assertions_unknown_empty`: unknown id → empty; a stored paper with no
  assertions → empty. **pass** (negative)

## Storage gate
`Store::save_assertions(arxiv_id, version, &[Assertion<Supported>])` accepts only
validated assertions — the compile-time typestate gate extends to the storage
boundary (the database cannot hold unvalidated content).

## Raw result
```
Running unittests src\lib.rs (diver_core)
running 88 tests — test result: ok. 88 passed; 0 failed
```
