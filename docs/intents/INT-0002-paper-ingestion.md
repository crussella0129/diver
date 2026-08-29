# INT-0002 — Paper ingestion pipeline

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0002
- **State:** realized
- **Work evidence:** [Sprint 2 build plan](../sprints/s2/sprint-plans/build-plan.md)
- **Completion evidence:** [Sprint 2 completed tasks](../work/completed-tasks.md) (T-008 through T-013)
- **Code evidence:** [fact.rs](../../src/fact.rs), [store.rs](../../src/store.rs), [client.rs](../../src/client.rs), [display.rs](../../src/display.rs), [main.rs](../../src/main.rs)
- **Test evidence:** [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md)
- **Documentation evidence:** [Sprint 2 research report](../sprints/s2/sprint-research/research-report.md)

## Intent

Add a paper ingestion pipeline that fetches individual ArXiv papers by ID,
wraps them in a provenance-tracked `SourceFact` domain type, and persists them
to local SQLite storage. This establishes the foundation for the epistemic
engine: stored facts with traceable origin that downstream extraction and
assertion systems can consume.

The deliverables are:
- `diver ingest <arxiv-id>` — fetch a paper by ID from ArXiv and store it.
- `diver inspect <arxiv-id>` — display a stored paper's full metadata.
- `diver list` — show all ingested papers.
- A `SourceFact` type distinct from the raw `Paper` search result, carrying
  ingestion timestamp, source URL, and ArXiv version.

Non-goals for this intent:
- PDF retrieval or full-text parsing.
- Batch ingestion or background processing.
- Assertion extraction or knowledge graph construction.
- Migration tooling for schema changes.

## Acceptance criteria

1. `diver ingest 2301.00001` fetches the paper from ArXiv and stores it in a
   local SQLite database at `~/.diver/diver.db`.
2. Re-ingesting the same ID updates the record rather than duplicating it.
3. `diver inspect 2301.00001` prints all stored metadata including ingestion
   timestamp and source provenance.
4. `diver list` prints a table of all ingested papers (ID, title, category,
   ingestion date).
5. `SourceFact` is a distinct type from `Paper`, carrying `ingested_at`,
   `source_url`, and `arxiv_version` fields.
6. `cargo test` passes with unit tests for storage operations and SourceFact
   construction.
7. `cargo clippy` and `cargo fmt --check` pass cleanly.

## Rationale

The architectural vision calls for a Rust epistemic engine where domain types
enforce provenance boundaries at compile time. `SourceFact` is the first such
type — it records what ArXiv told us about a paper, when, and at what version.
Future intents will layer extraction (observations, assertions) on top of these
stored facts, and the type system will prevent accidentally treating raw API
responses as validated knowledge.

SQLite via `rusqlite` is chosen over a full async database because the CLI tool
performs local, fast, single-threaded DB operations. No web server or concurrent
writes exist yet.

## Alternatives

- **Use sqlx with async SQLite.** Rejected: adds compile-time query checking
  complexity and async overhead for operations that complete in microseconds
  locally. Can migrate later if a server component arrives.
- **Store as JSON files.** Rejected: no query capability, no schema
  enforcement, poor for the `list` command.
- **Skip SourceFact, just store Paper.** Rejected: conflates API response with
  stored knowledge. The type distinction is architecturally load-bearing for
  the epistemic engine.

## Consequences

- `rusqlite` becomes a core dependency. The storage schema is the project's
  first durable state; schema migrations will be needed in future sprints.
- `~/.diver/diver.db` becomes the default database location, establishing a
  user-level data directory convention.
- The `Paper` → `SourceFact` conversion is the first domain boundary; future
  types (Observation, Assertion) will follow the same pattern.

## Transition history
- 2026-08-28: created as `proposed`.
- 2026-08-28: `proposed` → `planned` (Sprint 2 build plan attached).
- 2026-08-28: `planned` → `active` (Sprint 2 build phase started).
- 2026-08-28: `active` → `realized` (all acceptance criteria verified, test report attached).
