# Sprint 4 Plan Critique

**Verdict:** proceed

## Assessment

The plan is minimal and well-scoped. Two tasks compose existing infrastructure
(`ArxivClient::search`, `Store::save`, `SourceFact::from_paper`) into a new
`collect` subcommand with no new dependencies.

## Risks reviewed

- **C-001 (re-ingestion semantics):** `Store::save()` uses `INSERT OR REPLACE`,
  so re-collecting the same paper is safe. `Store::exists()` distinguishes
  new vs. updated for progress reporting. No action needed.
- **C-002 (source_url provenance):** Using `QueryBuilder::build()` as source_url
  for batch-collected papers is acceptable — it identifies the search query that
  produced the result. Consistent with `fetch_by_id` returning the query URL.

## Conclusion

No caveats. Proceed to build.
