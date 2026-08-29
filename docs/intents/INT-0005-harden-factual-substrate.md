# INT-0005 — Harden the factual substrate

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0005
- **State:** realized
- **Work evidence:** [Sprint 5 build plan](../sprints/s5/sprint-plans/build-plan.md)
- **Completion evidence:** [Sprint 5 completed tasks](../work/completed-tasks.md) (T-501 through T-508)
- **Code evidence:** [src/id.rs](../../src/id.rs), [src/store.rs](../../src/store.rs), [src/fact.rs](../../src/fact.rs), [src/parse.rs](../../src/parse.rs), [src/display.rs](../../src/display.rs), [src/main.rs](../../src/main.rs)
- **Test evidence:** [Sprint 5 test report](../sprints/s5/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent

Make Diver's representation of what arXiv actually told it rigorous and provenance-safe before any semantic layer is built on top.

Concretely:

1. **Rename `Dive` → `Find`** and reserve `diver dive` for eventual graph traversal.
2. **Introduce identifier newtypes** — `ArxivId`, `ArxivVersion`, `ArxivCategory` — so the compiler prevents category codes and paper IDs from mixing.
3. **Preserve all arXiv categories** (primary + all secondary) from the Atom feed; the parser currently silently drops every category after the first.
4. **Bundle an arXiv taxonomy snapshot** (`taxonomy/arxiv_categories.json`) and validate `ArxivCategory` values against it at parse time.
5. **Stop destroying paper versions** — split the single `source_facts` table (keyed on bare `arxiv_id`) into `papers` + `paper_versions` (keyed on `(arxiv_id, version)`) so ingesting v3 does not erase the stored v2 record.

Non-goals for this sprint:
- No `Observation`, `Concept`, or `Assertion` types yet.
- No LLM integration.
- No change to the network layer or query builder.
- No TypeScript / UI work.

## Acceptance criteria

1. `diver find <query>` performs local FTS search (previously `diver dive`); `diver dive` is absent from the CLI (reserved for a future sprint).
2. `diver search`, `diver ingest`, `diver collect`, `diver inspect`, and `diver list` continue to work as before.
3. `diver inspect <arxiv_id>` displays primary category name (from taxonomy), all secondary categories, and all stored versions.
4. Ingesting the same paper a second time with a different version (e.g., v1 then v2) stores both versions; the older record is not deleted.
5. Passing a bare `String` where `ArxivId` is expected fails to compile.
6. Passing a bare `String` where `ArxivCategory` is expected fails to compile.
7. The taxonomy JSON snapshot is embedded in the binary (or loaded from a known path) and `ArxivCategory::parse("invalid.XX")` returns `Err`.
8. All existing tests pass; new tests cover multi-version storage, taxonomy validation, and category preservation.

## Rationale

GPT 5.6 review identified the current `Dive` command as a misleading product name (it is local FTS, not graph traversal), naked `String` identifiers as a future source of type confusion, single-category storage as an epistemic lossy step, and `INSERT OR REPLACE` keyed on bare `arxiv_id` as a provenance hazard that will eventually make evidence unreproducible. These are all substrate concerns that must be correct before Observations and Assertions are built on top.

## Alternatives

- **Keep `String` identifiers longer**: rejected because the type confusion becomes harder to fix with every new layer added above it.
- **Use Postgres instead of SQLite**: rejected. The tool is local-first; SQLite + FTS5 is the correct choice at this scale.
- **Add taxonomy validation later**: rejected. If the taxonomy is embedded now, category codes are authoritative from the beginning; retrofitting it later would require migrating stored data.

## Consequences

- The `store.rs` schema migration will require either a fresh database or a SQLite migration shim. Existing local databases will be incompatible until migrated (acceptable for pre-1.0 local tooling).
- `SourceFact` will gain a `categories: Vec<ArxivCategory>` field, which cascades into `display.rs` and all tests that construct `SourceFact` directly.
- The `Paper` model in `model.rs` will gain `categories: Vec<String>` to pass all category tags through the parser.

## Transition history

- 2026-08-29: created as `proposed`.
- 2026-08-29: moved to `planned`; linked to Sprint 5 build plan.
- 2026-08-29: `planned` → `active` (Sprint 5 build started).
- 2026-08-29: `active` → `realized` (Sprint 5 complete, all 8 acceptance criteria verified with 62 passing tests).
