Finalized - DO NOT EDIT

# Sprint 1 Build Plan

## Intents
- [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md) — state: planned; acceptance criteria covered: AC-1 through AC-6 (all)

## Schema Tree
- Sprint Goal: ArXiv search CLI foundation
  - Project scaffold
    - T-001: Initialize Rust project with dependencies
  - Data layer
    - T-002: Define Paper/Author data model
    - T-003: Build ArXiv query constructor
    - T-004: Parse Atom XML responses into model types
  - Client layer
    - T-005: HTTP client for ArXiv API
  - Presentation layer
    - T-006: Terminal display formatter
  - Integration
    - T-007: CLI entry point with clap, wiring all layers

## Execution Sequence

### T-001: Initialize Rust project with Cargo.toml and module structure
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- **Touches:** `Cargo.toml`, `src/main.rs`, `src/lib.rs`
- **Depends on:** (none)
- **Acceptance criterion:** AC-1 (binary builds), AC-6 (clippy/fmt clean)
- **Success criterion (EARS):**
  - **WHEN** `cargo build --release` is run, **THEN** the compiler **SHALL** produce a `diver` binary with zero warnings.
  - **WHEN** `cargo clippy` is run, **THEN** it **SHALL** report zero warnings.
- **Notes:** Dependencies: reqwest (rustls-tls), quick-xml + serde, clap (derive), tokio (rt-multi-thread, macros), anyhow, owo-colors. Module files: `client.rs`, `query.rs`, `model.rs`, `parse.rs`, `display.rs`.

### T-002: Define Paper and Author data model
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- **Touches:** `src/model.rs`
- **Depends on:** T-001
- **Acceptance criterion:** AC-2 (results contain title, authors, abstract, category, link)
- **Success criterion (EARS):**
  - **WHEN** a `Paper` struct is constructed with all fields, **THEN** `title`, `authors`, `summary`, `primary_category`, `published`, `updated`, `arxiv_id`, and `pdf_url` **SHALL** all be accessible.
  - **WHEN** a `Paper` is formatted with `Display`, **THEN** the output **SHALL** include title, authors joined by commas, and the ArXiv link.

### T-003: Build ArXiv query constructor
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- **Touches:** `src/query.rs`
- **Depends on:** T-001
- **Acceptance criterion:** AC-2 (search query works), AC-3 (max-results, sort-by)
- **Success criterion (EARS):**
  - **WHEN** `QueryBuilder::new("transformer attention").max_results(5).sort_by(SortBy::Relevance).build()` is called, **THEN** the builder **SHALL** return a URL string containing `search_query=all:transformer+AND+all:attention&max_results=5&sortBy=relevance`.
  - **WHEN** `sort_by` is set to `SubmittedDate`, **THEN** the query URL **SHALL** contain `sortBy=submittedDate`.
  - **WHEN** `max_results` is not set, **THEN** the query URL **SHALL** default to `max_results=10`.

### T-004: Parse Atom XML responses into Paper structs
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- **Touches:** `src/parse.rs`, `tests/fixtures/`
- **Depends on:** T-002
- **Acceptance criterion:** AC-2 (correct field extraction), AC-5 (unit tests for parsing)
- **Success criterion (EARS):**
  - **WHEN** valid ArXiv Atom XML with 3 entries is parsed, **THEN** the parser **SHALL** return a `Vec<Paper>` of length 3 with all fields populated.
  - **WHEN** the XML contains `<opensearch:totalResults>`, **THEN** the parser **SHALL** extract the total result count.
  - **WHEN** malformed XML is passed, **THEN** the parser **SHALL** return an `Err` with a descriptive message.

### T-005: HTTP client for ArXiv API
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- **Touches:** `src/client.rs`
- **Depends on:** T-003, T-004
- **Acceptance criterion:** AC-2 (returns results), AC-4 (error on network failure)
- **Success criterion (EARS):**
  - **WHEN** `ArxivClient::search(query)` is called with a valid query, **THEN** the client **SHALL** send a GET request to `export.arxiv.org/api/query` and return parsed `Vec<Paper>`.
  - **WHEN** the HTTP request fails (timeout, DNS error), **THEN** the client **SHALL** return an `Err` with context including the original error.
  - **WHEN** the API returns a non-200 status, **THEN** the client **SHALL** return an `Err` naming the status code.

### T-006: Terminal display formatter
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- **Touches:** `src/display.rs`
- **Depends on:** T-002
- **Acceptance criterion:** AC-2 (readable output with all fields)
- **Success criterion (EARS):**
  - **WHEN** `display_results(papers, total)` is called with a non-empty list, **THEN** each paper **SHALL** be printed with a numbered header, title (bold), authors, truncated abstract (first 200 chars + "..."), primary category, and ArXiv URL.
  - **WHEN** `display_results(papers, total)` is called with an empty list, **THEN** the output **SHALL** print "No results found."
  - **WHEN** total results exceeds displayed count, **THEN** the output **SHALL** show "Showing N of M results."

### T-007: CLI entry point wiring search subcommand
- **Intent:** [INT-0001](../../../intents/INT-0001-arxiv-search-cli.md)
- **Touches:** `src/main.rs`
- **Depends on:** T-005, T-006
- **Acceptance criterion:** AC-2 (end-to-end search), AC-3 (flags), AC-4 (error exit code)
- **Success criterion (EARS):**
  - **WHEN** `diver search "quantum computing" --max-results 5 --sort-by submitted` is run, **THEN** the binary **SHALL** print up to 5 papers sorted by submission date and exit 0.
  - **WHEN** `diver search` is run without a query argument, **THEN** clap **SHALL** print a usage error and exit non-zero.
  - **WHEN** the network is unreachable, **THEN** the binary **SHALL** print a human-readable error to stderr and exit non-zero.
