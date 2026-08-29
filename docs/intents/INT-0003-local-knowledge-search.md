# INT-0003 — Local knowledge search

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0003
- **State:** realized
- **Work evidence:** [Sprint 3 build plan](../sprints/s3/sprint-plans/build-plan.md)
- **Completion evidence:** [Sprint 3 completed tasks](../work/completed-tasks.md) (T-014 through T-017)
- **Code evidence:** [store.rs](../../src/store.rs), [display.rs](../../src/display.rs), [main.rs](../../src/main.rs)
- **Test evidence:** [Sprint 3 test report](../sprints/s3/sprint-tests/test-report.md)
- **Documentation evidence:** none

## Intent

Add full-text search across ingested papers with `diver dive <query>`. This is
the project's core value proposition — "find knowledge, not just papers" —
applied to the local knowledge base. Users search their curated collection of
ingested papers by concept, author, or topic, with results ranked by relevance.

The deliverables are:
- `diver dive <query>` — full-text search across stored SourceFacts, ranked by
  BM25 relevance, displaying matched papers with highlighted context.
- `diver dive <query> --max-results N` — limit results.
- An FTS5 virtual table (`source_facts_fts`) that indexes title, authors,
  summary, and primary_category from the `source_facts` table.
- FTS index automatically maintained on ingest (insert/update triggers).

Non-goals for this intent:
- Semantic or embedding-based search.
- Cross-paper concept extraction or knowledge graph queries.
- Full-text search across PDF content (only metadata and abstracts).
- Search result pagination.

## Acceptance criteria

1. `diver dive "attention mechanism"` returns stored papers whose title,
   abstract, authors, or category match the query, ranked by BM25 relevance.
2. Results display ArXiv ID, title, category, and a snippet of the matching
   abstract with the query terms visible.
3. `diver dive "attention" --max-results 3` limits output to 3 results.
4. `diver dive "xyznonexistent"` prints "No matching papers found." and exits 0.
5. The FTS index is automatically populated when `diver ingest` stores or
   updates a paper — no separate rebuild step required.
6. `cargo test` passes with unit tests for FTS search operations.
7. `cargo clippy` and `cargo fmt --check` pass cleanly.

## Rationale

Users ingest papers to build a personal knowledge base. Without local search,
the only way to find a previously ingested paper is `diver list` (scan all) or
`diver inspect` (know the exact ID). Full-text search closes the loop: ingest
from ArXiv, then dive into your collection by concept.

SQLite FTS5 is the right tool because it's already bundled via rusqlite, adds
no new dependencies, supports BM25 ranking natively, and handles the expected
scale (hundreds to low thousands of papers) with zero configuration.

## Alternatives

- **LIKE queries on source_facts.** Rejected: no ranking, poor performance on
  large collections, no tokenization or stemming.
- **External search engine (tantivy, meilisearch).** Rejected: massive
  dependency for a CLI tool searching hundreds of records. FTS5 is built into
  the SQLite we already ship.
- **Embedding-based semantic search.** Deferred: requires an embedding model
  and vector storage. A future intent for when the knowledge base grows beyond
  keyword-searchable scale.

## Consequences

- `source_facts_fts` virtual table becomes part of the schema. Future schema
  migrations must account for it.
- The `save()` method in Store gains FTS index maintenance responsibility.
- `diver dive` is the first subcommand that queries the local knowledge base
  rather than the ArXiv API, establishing the pattern for future knowledge
  operations.

## Transition history
- 2026-08-28: created as `proposed`.
- 2026-08-28: `proposed` → `active` (Sprint 3 build plan attached, skipping planned since single-sprint intent).
- 2026-08-29: `active` → `realized` (all 7 acceptance criteria verified by Sprint 3 test report).
