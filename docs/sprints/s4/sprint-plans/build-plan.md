# Sprint 4 Build Plan
<!-- Finalized - DO NOT EDIT -->

**Status:** finalized
**Sprint goal:** Batch collection with `diver collect`
**Intents:** [INT-0004](../../intents/INT-0004-batch-collection.md)

## Schema Tree

- Sprint Goal: Batch collection with `diver collect`
  - Presentation
    - T-018: Collection progress and summary display functions
  - Integration
    - T-019: Wire `collect` subcommand into CLI

## Execution Sequence

### T-018: Collection progress and summary display functions
- **Intent:** [INT-0004](../../intents/INT-0004-batch-collection.md)
- **Touches:** `src/display.rs`
- **Depends on:** (none)
- **Acceptance criterion:** AC-1 (per-paper status), AC-4 (summary line), AC-5 (no papers message)
- **Success criterion (EARS):**
  - **WHEN** `display_collect_item(arxiv_id, title, is_update)` is called with `is_update=false`, **THEN** it **SHALL** print "  Ingested: {arxiv_id} — {title}".
  - **WHEN** `display_collect_item(arxiv_id, title, is_update)` is called with `is_update=true`, **THEN** it **SHALL** print "  Updated: {arxiv_id} — {title}".
  - **WHEN** `display_collect_summary(new_count, updated_count)` is called, **THEN** it **SHALL** print "Collected {new} new, {updated} updated.".
  - **WHEN** `display_collect_empty()` is called, **THEN** it **SHALL** print "No papers found.".

### T-019: Wire `collect` subcommand into CLI
- **Intent:** [INT-0004](../../intents/INT-0004-batch-collection.md)
- **Touches:** `src/main.rs`
- **Depends on:** T-018
- **Acceptance criterion:** AC-1 (batch ingest), AC-2 (update detection), AC-3 (sort-by), AC-4 (summary), AC-5 (no papers)
- **Success criterion (EARS):**
  - **WHEN** `diver collect "attention" --max-results 5` is run with matching ArXiv results, **THEN** the CLI **SHALL** search ArXiv, ingest each paper via `Store::save()`, print per-paper status, print the summary line, and exit 0.
  - **WHEN** `diver collect "attention" --sort-by submitted` is run, **THEN** the search query **SHALL** use `SortBy::SubmittedDate`.
  - **WHEN** a collected paper already exists in the store, **THEN** it **SHALL** be reported as "Updated" (not "Ingested").
  - **WHEN** `diver collect "xyznonexistent"` is run and ArXiv returns 0 results, **THEN** the CLI **SHALL** print "No papers found." and exit 0.
