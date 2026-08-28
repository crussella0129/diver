# Sprint 1 — Research Report

## Sprint Goal

Bootstrap the **diver** Rust project: scaffold the crate, implement an ArXiv
API client, build a CLI search command, and validate the end-to-end path from
query to formatted terminal output.

## Intents Reviewed

- **[INT-0001](../../intents/INT-0001-arxiv-search-cli.md)** — *ArXiv search
  CLI foundation.* **Created** this sprint. Captures the foundational search
  CLI and its acceptance criteria.

## 1. Existing Code Survey

The repository is greenfield — no source code exists. Present artifacts:

| File | Role |
|---|---|
| `README.md` | One-line project description |
| `LICENSE` | Apache 2.0 |
| `docs/` | Sprint Loops Book (schema v2), empty intents and work ledgers |

There are no `Cargo.toml`, `src/`, or configuration files. Everything will be
created from scratch.

## 2. External Sources

1. **ArXiv API User's Manual**
   (https://info.arxiv.org/help/api/user-manual.html) — Documents the query
   interface, Atom XML response schema, field prefixes (`ti:`, `au:`, `abs:`,
   `cat:`), boolean operators, sort options, and pagination. Rate limit: 3
   req/s.

2. **`arxiv-rs` crate** (https://crates.io/crates/arxiv-rs) — Existing Rust
   wrapper at v0.2.0. Provides `ArxivQueryBuilder` and paper struct. Low
   maintenance activity; limited API surface. Decision: build our own thin
   client for full control.

3. **`arxiv-tools` crate** (https://lib.rs/crates/arxiv-tools) — Another Rust
   crate. Less mature than `arxiv-rs`.

4. **ArXiv Atom feed schema** — Responses use Atom 1.0 with `arxiv:` namespace
   extensions. Key entry fields: `<title>`, `<summary>`, `<author><name>`,
   `<arxiv:primary_category>`, `<published>`, `<updated>`, `<link>`,
   `<arxiv:doi>`, `<arxiv:journal_ref>`.

5. **ArXiv category taxonomy** (https://arxiv.org/category_taxonomy) — Full
   list of subject categories for filtering and display.

## 3. Risks, Unknowns, and Dependencies

| Risk | Severity | Mitigation |
|---|---|---|
| ArXiv API availability / rate limiting | Medium | Respect 3 req/s limit; add retry with backoff |
| `quick-xml` may struggle with Atom namespaces | Low | Well-tested crate; namespace handling documented |
| Windows path / encoding issues with terminal output | Low | Use `crossterm` or `colored` for portable ANSI |
| No offline test data for XML parsing | Medium | Save sample responses as test fixtures |

**Dependencies (Rust crates):**
- `reqwest` (HTTP client, async)
- `quick-xml` + `serde` (XML deserialization)
- `clap` (CLI argument parsing, derive API)
- `tokio` (async runtime)
- `colored` or `owo-colors` (terminal formatting)
- `anyhow` (error handling)

## 4. Recommended Approach

### Architecture

```
diver/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI entry, clap setup
│   ├── client.rs        # ArXiv API HTTP client
│   ├── query.rs         # Query builder (field prefixes, booleans, pagination)
│   ├── model.rs         # Paper, Author, Category structs
│   ├── parse.rs         # Atom XML → model deserialization
│   └── display.rs       # Terminal formatting
```

### Sprint scope

1. `cargo init` the project with the dependency set above.
2. Implement query builder supporting free-text, field-prefix, max-results,
   sort-by, and start-index.
3. Implement Atom XML parser that deserializes `<entry>` elements into a
   `Paper` struct.
4. Implement `diver search <query>` subcommand that ties client → parse →
   display.
5. Unit tests for query construction and XML parsing (using saved fixtures).
6. `cargo fmt` + `cargo clippy` clean.

### What is deferred

- Category browsing / filtering subcommand.
- Paper detail view (full abstract, references, citations).
- PDF download.
- Semantic search / LLM-powered features.
- Persistent config or cache.

## 5. Referenced Artifacts

- [INT-0001 intent chapter](../../intents/INT-0001-arxiv-search-cli.md)
