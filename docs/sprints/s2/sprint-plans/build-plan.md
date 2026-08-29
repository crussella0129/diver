Finalized - DO NOT EDIT

# Sprint 2 Build Plan

## Intents
- [INT-0002](../../../intents/INT-0002-paper-ingestion.md) — state: planned; acceptance criteria covered: AC-1 through AC-7 (all)

## Schema Tree
- Sprint Goal: Paper ingestion pipeline with SourceFact and SQLite storage
  - Dependencies
    - T-008: Add rusqlite, chrono, dirs dependencies
  - Domain layer
    - T-009: Create SourceFact type and Paper conversion
  - Storage layer
    - T-010: SQLite storage (Store struct with CRUD operations)
  - Client extension
    - T-011: ArxivClient::fetch_by_id for single-paper retrieval
  - Presentation
    - T-012: Display formatters for SourceFact inspection and listing
  - Integration
    - T-013: Wire ingest/inspect/list CLI subcommands

## Execution Sequence

### T-008: Add rusqlite, chrono, dirs to Cargo.toml and declare new modules
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- **Touches:** `Cargo.toml`, `src/lib.rs`
- **Depends on:** (none)
- **Acceptance criterion:** AC-7 (clippy/fmt clean)
- **Success criterion (EARS):**
  - **WHEN** `cargo check` is run after adding `rusqlite` (bundled), `chrono`, and `dirs`, **THEN** the compiler **SHALL** succeed with zero errors.
- **Notes:** `rusqlite = { version = "0.32", features = ["bundled"] }`, `chrono = "0.4"`, `dirs = "6"`. Declare `pub mod fact;` and `pub mod store;` in lib.rs.

### T-009: Define SourceFact type and Paper→SourceFact conversion
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- **Touches:** `src/fact.rs`
- **Depends on:** T-008
- **Acceptance criterion:** AC-5 (SourceFact distinct from Paper with provenance fields)
- **Success criterion (EARS):**
  - **WHEN** `SourceFact::from_paper(paper, source_url)` is called with a valid `Paper`, **THEN** the result **SHALL** contain all Paper fields plus `ingested_at` (ISO 8601 current time), `source_url`, and `arxiv_version` extracted from the paper's ArXiv ID URL.
  - **WHEN** the Paper's `arxiv_id` contains a version suffix (e.g., `2301.00001v2`), **THEN** `arxiv_version` **SHALL** be `"v2"` and `arxiv_id` **SHALL** be the bare ID `"2301.00001"`.
  - **WHEN** the Paper's `arxiv_id` has no version suffix, **THEN** `arxiv_version` **SHALL** default to `"v1"`.

### T-010: SQLite storage layer with Store struct
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- **Touches:** `src/store.rs`
- **Depends on:** T-009
- **Acceptance criterion:** AC-1 (stores to SQLite), AC-2 (re-ingest updates), AC-6 (unit tests for storage)
- **Success criterion (EARS):**
  - **WHEN** `Store::open()` is called and the data directory does not exist, **THEN** the store **SHALL** create the directory and `diver.db` with the `source_facts` table.
  - **WHEN** `store.save(fact)` is called with a new SourceFact, **THEN** the fact **SHALL** be inserted into the `source_facts` table.
  - **WHEN** `store.save(fact)` is called with an existing `arxiv_id`, **THEN** the row **SHALL** be replaced (upsert), not duplicated.
  - **WHEN** `store.get(arxiv_id)` is called with a stored ID, **THEN** it **SHALL** return `Some(SourceFact)` with all fields reconstructed including `authors` deserialized from JSON.
  - **WHEN** `store.get(arxiv_id)` is called with an unknown ID, **THEN** it **SHALL** return `None`.
  - **WHEN** `store.list()` is called, **THEN** it **SHALL** return all stored SourceFacts ordered by `ingested_at` descending.

### T-011: Extend ArxivClient with fetch_by_id
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- **Touches:** `src/client.rs`
- **Depends on:** (none — uses existing parse module)
- **Acceptance criterion:** AC-1 (fetches paper by ID from ArXiv)
- **Success criterion (EARS):**
  - **WHEN** `client.fetch_by_id("2301.00001")` is called, **THEN** it **SHALL** send a GET to `https://export.arxiv.org/api/query?id_list=2301.00001&max_results=1` and return the parsed `Paper`.
  - **WHEN** the ArXiv API returns an error entry (title contains "Error"), **THEN** `fetch_by_id` **SHALL** return an `Err` with a message indicating the paper was not found.
  - **WHEN** the HTTP request fails, **THEN** `fetch_by_id` **SHALL** return an `Err` with network context.
- **Notes:** Error detection is implemented as a standalone `extract_paper(feed: FeedResult) → Result<Paper>` function (pub for testing) that `fetch_by_id` calls after parsing. This function checks for empty results, error entries, and extracts the single paper.

### T-012: Display formatters for SourceFact and list view
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- **Touches:** `src/display.rs`
- **Depends on:** T-009
- **Acceptance criterion:** AC-3 (inspect prints all metadata including provenance), AC-4 (list prints table)
- **Success criterion (EARS):**
  - **WHEN** `display_fact(fact)` is called, **THEN** it **SHALL** print title, authors, abstract, category, ArXiv URL, source URL, ArXiv version, and ingestion timestamp.
  - **WHEN** `display_fact_list(facts)` is called with a non-empty list, **THEN** it **SHALL** print a table with columns: ArXiv ID, title (truncated to 50 chars), category, and ingestion date.
  - **WHEN** `display_fact_list(facts)` is called with an empty list, **THEN** it **SHALL** print "No ingested papers."

### T-013: Wire ingest/inspect/list subcommands into CLI
- **Intent:** [INT-0002](../../../intents/INT-0002-paper-ingestion.md)
- **Touches:** `src/main.rs`
- **Depends on:** T-010, T-011, T-012
- **Acceptance criterion:** AC-1 (ingest works), AC-2 (re-ingest updates), AC-3 (inspect works), AC-4 (list works)
- **Success criterion (EARS):**
  - **WHEN** `diver ingest 2301.00001` is run, **THEN** the CLI **SHALL** fetch the paper, convert to SourceFact, store it, print a confirmation message, and exit 0.
  - **WHEN** `diver ingest 2301.00001` is run for an already-ingested paper, **THEN** the CLI **SHALL** update the record and print an "updated" confirmation.
  - **WHEN** `diver inspect 2301.00001` is run for a stored paper, **THEN** the CLI **SHALL** print all metadata via `display_fact`.
  - **WHEN** `diver inspect` is run for an unknown ID, **THEN** the CLI **SHALL** print "Paper not found" to stderr and exit non-zero.
  - **WHEN** `diver list` is run, **THEN** the CLI **SHALL** print the table via `display_fact_list`.
