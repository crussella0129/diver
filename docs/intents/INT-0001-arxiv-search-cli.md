# INT-0001 — ArXiv search CLI foundation

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0001
- **State:** active
- **Work evidence:** [T-001–T-007 build plan](../sprints/s1/sprint-plans/build-plan.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Establish a working Rust CLI tool that searches ArXiv and displays results in a
human-readable terminal format. This is the foundational layer on which
knowledge-extraction features (semantic search, summarisation, concept mapping)
will be built.

The deliverable is a single binary, `diver`, that accepts a search query and
prints matching papers with title, authors, abstract snippet, categories, and
ArXiv link. It connects directly to the ArXiv Atom API over HTTPS.

Non-goals for this intent:
- Full-text PDF retrieval or parsing.
- Semantic/embedding-based search.
- Persistent storage or caching.
- GUI or web interface.

## Acceptance criteria

1. `cargo build --release` produces a `diver` binary without warnings.
2. `diver search "transformer attention"` returns ArXiv results and prints them
   with title, authors, abstract (truncated), primary category, and link.
3. Supports `--max-results N` (default 10) and `--sort-by` (relevance,
   submitted, updated).
4. Exits with a non-zero code and a human-readable message on network failure
   or invalid query.
5. `cargo test` passes with at least unit tests for query construction and XML
   response parsing.
6. `cargo clippy` and `cargo fmt --check` pass cleanly.

## Rationale

A greenfield project needs a runnable skeleton before higher-level features.
ArXiv's public Atom API is free, unauthenticated, and returns structured XML —
a reliable foundation that doesn't require API keys or account setup. Building
a thin custom client (instead of using the 0.2.0 `arxiv-rs` crate) gives full
control over query construction, error handling, and XML field mapping as the
project grows toward knowledge-extraction features.

## Alternatives

- **Use `arxiv-rs` crate.** Rejected: version 0.2.0, limited API surface, low
  maintenance activity. We'd outgrow it quickly and need to fork.
- **Start with Python (`arxiv` PyPI package).** Rejected: user preference is
  Rust-first; the project intends to be a compiled CLI tool.
- **Start with semantic search / LLM integration.** Rejected: premature without
  a working data pipeline. Knowledge extraction is a follow-on intent.

## Consequences

- The project adopts `reqwest` + `quick-xml` + `clap` + `tokio` as core
  dependencies. Changing HTTP or XML libraries later would touch the whole
  client module.
- ArXiv's 3-request-per-second rate limit must be respected from the start.
- The CLI surface (`diver search ...`) becomes the stable entry point; future
  subcommands extend it.

## Transition history
- 2026-08-28: created as `proposed`.
- 2026-08-28: `proposed` → `planned`. Sprint 1 build plan covers all acceptance criteria.
- 2026-08-28: `planned` → `active`. Sprint 1 build phase started.
