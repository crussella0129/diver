# Plan Critique — Sprint 3

## Concerns

### C-001: Missing test for FTS backfill on existing database
- **Where:** `build-plan.md` T-014 EARS clause 4 / `test-plan.md` T-014 unit tests
- **Quote:** "WHEN init_schema() is called on an existing database without FTS table, THEN it SHALL create the FTS table and populate it from existing source_facts rows."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The migration path for pre-existing databases is a Medium-severity risk identified in research. Without a test, a regression in the backfill logic could silently leave existing papers unsearchable after upgrade.
- **Suggested response:** fix-in-plan — add `test_init_schema_backfills_existing_facts` to T-014 unit tests: insert rows directly into source_facts (bypassing save), then call init_schema, then verify FTS search finds those rows.

## Confidence
proceed-with-caveats
