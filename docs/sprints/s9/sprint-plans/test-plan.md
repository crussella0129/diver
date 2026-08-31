Finalized - DO NOT EDIT

# Sprint 9 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC1: schema tables created idempotently | T-901 / open → tables exist | `test_assertion_schema_created` |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC2: save persists claim + support (validated-only) | T-901 / save → rows | `test_save_assertions_persists` |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC3: idempotent replace per (paper,version) | T-901 / re-save → replace | `test_save_assertions_idempotent_replace` |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC4: get returns stored data; empty for unknown | T-902 / get → StoredAssertion / empty | `test_get_assertions_round_trip`, `test_get_assertions_unknown_empty` |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC5: extract persists; `diver assertions` displays | T-903 / persist + display; unknown clean | `test_persist_pipeline` + e2e `assertions --help` / unknown-id |
| [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) | AC6: FKs enforced; existing tests pass | T-901 + all tasks | `test_assertion_support_fk_enforced`; full `cargo test --workspace` |

## Unit Tests

### T-901 unit tests (store.rs)
- **Intent:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md)
- `test_assertion_schema_created`: after `open_in_memory`, `INSERT INTO assertions`
  / `assertion_support` (with a valid parent) succeed — the tables exist.
- `test_save_assertions_persists`: save two supported assertions for a paper →
  `SELECT COUNT(*)` on `assertions` = 2 and `assertion_support` = sum of support;
  claim + quote text match.
- `test_save_assertions_idempotent_replace`: save N, then save M (different) for
  the same `(paper, version)` → `SELECT COUNT(*)` on **both** `assertions` (= M)
  **and** `assertion_support` (= M's support total) confirm no duplicate assertion
  rows and **no orphaned support rows** (proving the `ON DELETE CASCADE` fired on
  the replace).
- `test_assertion_support_fk_enforced`: direct `INSERT INTO assertion_support` with
  an `assertion_id` that does not exist → `Err` (FK violation).

### T-902 unit tests (store.rs)
- **Intent:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md)
- `test_get_assertions_round_trip`: `save_assertions` two assertions → `get_assertions`
  returns 2 `StoredAssertion`s with matching claim, version, and support quotes.
- `test_get_assertions_unknown_empty`: `get_assertions("9999.99999")` → empty vec;
  a stored paper with no assertions → empty vec.

## Integration Tests

### Persist pipeline (diver-core/tests/persist_pipeline.rs)
- **Intents:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) (AC5, AC6)
- `test_persist_pipeline`: `open_in_memory` → `store.save(&fact)` (a paper) →
  `parse_claims`(fixture body grounded in the abstract) → `validate` →
  `store.save_assertions(&fact.arxiv_id, &fact.arxiv_version, &supported)` →
  `store.get_assertions(&fact.arxiv_id)` returns the persisted claim, the paper's
  version, and the supporting quote. Imports via `diver_core::`.

## End-to-End Tests
- **Status:** possible (offline)
- `e2e_assertions_help` (scripted smoke): `cargo run -p diver-cli -- assertions
  --help` exits 0 and shows `<ARXIV_ID>`; `diver --help` lists `assertions`.
- `e2e_assertions_unknown` (scripted smoke): `cargo run -p diver-cli -- assertions
  9999.99999` prints a "No stored assertions" message and exits 0 (empty is not an
  error). Read-only DB access (schema init only).
