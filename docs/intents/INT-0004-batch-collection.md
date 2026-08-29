# INT-0004 — Batch collection

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0004
- **State:** realized
- **Work evidence:** [Sprint 4 build plan](../sprints/s4/sprint-plans/build-plan.md)
- **Completion evidence:** [Sprint 4 completed tasks](../work/completed-tasks.md) (T-018 through T-019)
- **Code evidence:** [src/display.rs](../../src/display.rs), [src/main.rs](../../src/main.rs)
- **Test evidence:** [Sprint 4 test report](../sprints/s4/sprint-tests/test-report.md)
- **Documentation evidence:** none

## Intent

Add `diver collect <query>` to search ArXiv and ingest all matching papers in
one step. Currently a user must run `diver search`, visually scan results,
then `diver ingest <id>` for each paper individually. This is the primary
friction point for building a knowledge base.

The deliverables are:
- `diver collect <query>` — search ArXiv for papers matching the query, then
  ingest each result, reporting progress and skipping already-ingested papers.
- `diver collect <query> --max-results N` — limit how many papers to collect.
- `diver collect <query> --sort-by <relevance|submitted|updated>` — control
  result ordering (same options as `diver search`).
- Progress output showing each paper as it is ingested, with skip/new/update
  status per paper and a final summary count.

Non-goals for this intent:
- Pagination beyond a single ArXiv API response (max 2000 per call).
- Background or async collection.
- Filtering by category, date range, or author before ingestion.
- Deduplication across multiple `collect` invocations beyond the existing
  `INSERT OR REPLACE` behavior in `Store::save`.

## Acceptance criteria

1. `diver collect "attention mechanisms" --max-results 5` fetches up to 5
   papers from ArXiv and ingests each one, printing per-paper status.
2. Already-ingested papers are detected and reported as "updated" rather than
   "ingested".
3. `diver collect "attention" --sort-by submitted` sorts results by submission
   date before ingesting.
4. The final output line reports a summary: "Collected N new, M updated, K
   skipped." (skipped = 0 for now, reserved for future filters).
5. `diver collect "xyznonexistent"` prints "No papers found." and exits 0.
6. `cargo test` passes with unit tests for collection logic.
7. `cargo clippy` and `cargo fmt --check` pass cleanly.

## Rationale

The search → ingest → dive workflow is diver's core loop. Without batch
collection, building a meaningful knowledge base requires tedious per-paper
manual ingestion. `collect` reduces a multi-minute manual workflow to a
single command, making the tool practical for real research sessions.

The implementation reuses the existing `ArxivClient::search()` and
`Store::save()` infrastructure. The only new code is the orchestration loop
and progress display.

## Alternatives

- **Shell script wrapper.** Rejected: not cross-platform, loses progress
  reporting and skip detection.
- **Interactive selection from search results.** Deferred: useful but more
  complex (TUI selection). Batch-all-results is the simpler first step.
- **`diver ingest --query`.** Rejected: overloading `ingest` with query
  semantics confuses the single-paper and batch-collection use cases.

## Consequences

- `collect` becomes the third ArXiv-network subcommand (alongside `search`
  and `ingest`). Future rate-limiting or retry logic applies to all three.
- The `fact::SourceFact::from_paper` path is now exercised in a loop,
  validating its idempotency under `INSERT OR REPLACE`.
- Progress display establishes a pattern for future batch operations.

## Transition history
- 2026-08-29: created as `proposed`.
- 2026-08-29: `proposed` → `active` (Sprint 4 plan finalized).
- 2026-08-29: `active` → `realized` (Sprint 4 complete, all AC verified).
