# Sprint 4 Research Report

## Intents Reviewed
- [INT-0004 Batch collection](../../intents/INT-0004-batch-collection.md) — created

## Sprint Goal

Add batch collection: `diver collect <query>` searches ArXiv and ingests all
matching papers in one step, with per-paper progress reporting and a final
summary.

## Existing Code Survey

### Current architecture (`src/`)

| Module       | Purpose                                  | Sprint 4 impact |
|--------------|------------------------------------------|-----------------|
| `client.rs`  | ArxivClient with search() and fetch_by_id() | Reuse search() for batch fetch; no changes needed |
| `store.rs`   | Store with save(), exists(), search()    | Reuse save() and exists() for batch ingest; no changes needed |
| `fact.rs`    | SourceFact::from_paper()                 | Reuse for each paper in the batch; no changes needed |
| `display.rs` | Terminal output formatters               | Add collect progress display |
| `main.rs`    | CLI with search/ingest/inspect/list/dive | Add `collect` subcommand |
| `model.rs`   | Paper struct                             | No changes |
| `parse.rs`   | Atom XML parser                          | No changes |
| `query.rs`   | QueryBuilder                             | No changes |
| `lib.rs`     | Module declarations                      | No changes |

### Key reuse points

**`ArxivClient::search()`** already returns `FeedResult { papers: Vec<Paper>, total_results: u32 }`. The `collect` command calls this once and iterates over `papers`.

**`SourceFact::from_paper(paper, source_url)`** converts each Paper to a SourceFact. The `source_url` for batch-collected papers should reference the search query URL rather than an `id_list` URL, since the papers came from a search.

**`Store::save(fact)`** uses `INSERT OR REPLACE`, so re-collecting the same paper is safe — it updates rather than duplicates.

**`Store::exists(arxiv_id)`** lets us distinguish "new" vs "updated" papers for progress reporting.

### Dependencies

No new dependencies needed. All infrastructure exists.

## External Sources

No external research required. This sprint composes existing internal
components into a new workflow.

## Risks, Unknowns, and Dependencies

| Risk | Severity | Mitigation |
|------|----------|------------|
| ArXiv API rate limiting on rapid sequential requests | Low | `collect` issues one search call (not per-paper fetches), then saves locally. Only one HTTP request per collect. |
| Large result sets overwhelming terminal output | Low | Default max_results=10, same as search. User controls via --max-results. |
| Paper version IDs from search differ from fetch_by_id | Low | `SourceFact::from_paper` handles version parsing (tested in Sprint 2). Search results include the same ArXiv ID format. |
| Source URL semantics differ for search vs fetch_by_id | Low | Use the search query URL as source_url for all papers in a batch. Acceptable since it identifies provenance. |

## Recommended Approach

1. Add `Collect` variant to `Commands` enum with `query`, `max_results`, and
   `sort_by` arguments (same as `Search`).
2. Implement collection logic in `main.rs`: call `ArxivClient::search()`, iterate
   over results, check `Store::exists()` for each, call `SourceFact::from_paper()`
   and `Store::save()`, print per-paper status.
3. Add `display_collect_progress()` for per-paper status lines and
   `display_collect_summary()` for the final count.
4. Unit tests for display functions. Integration test not needed beyond
   the existing ingest_pipeline test since collect reuses the same save path.

## Referenced Artifacts

- [INT-0004 Batch collection](../../intents/INT-0004-batch-collection.md)
- [ArxivClient::search()](../../src/client.rs) — existing search method
- [Store::save()](../../src/store.rs) — existing save with FTS maintenance
- [SourceFact::from_paper()](../../src/fact.rs) — existing conversion
