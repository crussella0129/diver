# Plan Critique — Sprint 9

## Concerns

### C-001: the full binary round-trip (`extract` persists → `assertions` shows) is not E2E-tested
- **Where:** `test-plan.md` E2E; `build-plan.md` T-903
- **Quote:** "`diver assertions 9999.99999` prints a 'No stored assertions' message"
- **Failure mode:** e2e-cop-out
- **Why it matters:** the E2E smokes cover the CLI surface and the empty path, but
  no test runs `diver extract <seeded-id> --deterministic` and then `diver
  assertions <seeded-id>` through the binary to prove the persist-then-read loop.
- **Suggested response:** defer-with-rationale — a binary round-trip needs a
  seeded real database (network ingest or a fixture DB), which makes the E2E
  stateful. The persist→read loop is covered deterministically at the library
  level by `test_persist_pipeline` (`save_assertions` → `get_assertions`), and the
  `Extract` handler's added line is a single `store.save_assertions(...)?` call
  over the same API. Surface + empty path (binary) + library round-trip is
  sufficient; a fixture-DB binary test is possible future hardening.

### C-002: `get_assertions` returns assertions across all versions of a paper
- **Where:** `build-plan.md` T-902 Notes ("filtered on `arxiv_id`")
- **Quote:** "returns one `StoredAssertion` per stored assertion … newest first"
- **Failure mode:** intent-drift (potential display ambiguity)
- **Why it matters:** if a paper was extracted at v1 and later at v2, `get_assertions`
  returns both sets; a reader might not realize two versions are mixed.
- **Suggested response:** defer-with-rationale — each `StoredAssertion` carries its
  `version`, and `display_stored_assertions` shows it, so the result is lossless
  and unambiguous. Returning all versions is simpler and correct; scoping to the
  latest version is a display-filter refinement, not a storage concern, and is
  easy to add later without schema change.

### C-003: idempotent replace correctness depends on the FK cascade actually firing
- **Where:** `build-plan.md` T-901 Notes ("cascade drops support")
- **Quote:** "`DELETE FROM assertions WHERE paper_id=?1 AND version=?2` (cascade drops support)"
- **Failure mode:** correctness edge
- **Why it matters:** if `ON DELETE CASCADE` or `PRAGMA foreign_keys=ON` were not
  active, a re-save would leave orphaned `assertion_support` rows.
- **Suggested response:** fix-in-plan — `test_save_assertions_idempotent_replace`
  must assert the `assertion_support` **row count** after the replace (not just the
  `assertions` count), so an orphaned-support regression fails the test. `PRAGMA
  foreign_keys=ON` is already set at open (store.rs:49) and independently proven by
  the existing `test_fk_constraint_enforced`; the new `test_assertion_support_fk_enforced`
  reconfirms it for the new table.

## Confidence
proceed-with-caveats
