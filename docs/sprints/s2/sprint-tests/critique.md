# Test Critique — Sprint 2

## Concerns

### C-001: T-013 CLI integration lacks automated tests
- **Where:** `test-plan.md` T-013, `unit-tests.md`
- **Quote:** "WHEN diver ingest ... THEN the CLI SHALL fetch the paper ... and exit 0"
- **Failure mode:** intent-coverage
- **Why it matters:** T-013 has 5 EARS clauses for CLI behavior (ingest, re-ingest update message, inspect, inspect unknown, list) but no automated tests — all are manual E2E only. The ingest pipeline integration test covers the data path but not the CLI wiring (clap parsing, exit codes, stdout messages).
- **Suggested response:** defer-with-rationale — CLI integration tests require either a process-spawning harness or captured stdout. The `test_ingest_pipeline` integration test proves the data path end-to-end. CLI argument parsing is handled by clap's derive macros which are well-tested upstream. The manual verification during build is documented. A future sprint can add a CLI test harness.

### C-002: store tests use in-memory SQLite, not file-based
- **Where:** `unit-tests.md` T-010
- **Quote:** "Uses: in-memory SQLite (`:memory:`) for all store tests"
- **Failure mode:** stub-leak
- **Why it matters:** In-memory SQLite behavior is identical to file-based for the operations tested (INSERT, SELECT, CREATE TABLE). The only untested path is `Store::open()` which creates the data directory and file — this is verified manually.
- **Suggested response:** reject — in-memory SQLite is not a mock; it's the same SQLite engine with the same SQL dialect. The `open()` path involves OS filesystem calls that are better tested via integration/E2E than unit tests.

## Confidence
proceed-with-caveats
