Finalized - DO NOT EDIT

# Sprint 6 Build Plan

## Intents
- [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) — state: planned; acceptance criteria covered: AC1 (provenance record), AC2 (FK enforcement, T-601), AC3 (stale-FTS reingest, T-602), AC4 (taxonomy cache, T-603), AC5 (dev↔main reconciled, git step), AC6 (existing tests pass)
- [INT-0007](../../../intents/INT-0007-workspace-restructure.md) — state: planned; acceptance criteria covered: AC1/AC2/AC3/AC6 (T-604), AC2/AC5 (T-605), AC4 (full build+test)

## Schema Tree
- Sprint Goal: reconcile out-of-loop review hardening + restructure into a Cargo workspace
  - Regression tests (INT-0006)
    - T-601: FK-enforcement test (fix #5)
    - T-602: stale-FTS-on-older-reingest test (fix #1)
    - T-603: taxonomy-cache determinism test (fix #4)
  - Workspace restructure (INT-0007)
    - T-604: virtual workspace + `diver-core` library
    - T-605: `diver-cli` binary + import repoint

## Execution Sequence

Sequencing rationale: add the three INT-0006 regression tests first, against the
current single-crate layout, so they travel into `diver-core` during T-604 and
re-run there as split-verification. This keeps "did the split break something"
separable from "did a fix regress."

### T-601: FK-enforcement regression test (fix #5)
- **Intent:** [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md)
- **Touches:** src/store.rs (tests module)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0006 AC2 — a regression test asserts the
  `paper_versions → papers` foreign-key constraint is enforced.
- **Success criterion (EARS):**
  - **WHEN** a row is inserted into `paper_versions` with a `paper_id` absent from
    `papers`, **THEN** the store connection **SHALL** return a foreign-key
    constraint error.
- **Notes:** `test_fk_constraint_enforced` — the tests module already reaches the
  private `store.conn` (e.g. `test_save_populates_fts`); issue a direct INSERT
  into `paper_versions` with `paper_id = 99999`, assert `is_err()` and the error
  string contains `FOREIGN KEY`. Fails if `PRAGMA foreign_keys=ON` (store.rs:49)
  regresses.

### T-602: Stale-FTS-on-older-reingest regression test (fix #1)
- **Intent:** [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md)
- **Touches:** src/store.rs (tests module)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0006 AC3 — re-ingesting an older version does not
  overwrite the FTS index with stale data.
- **Success criterion (EARS):**
  - **WHEN** a paper is saved at v2 (`ingested_at` = Tb) and then re-saved at v1
    (`ingested_at` = Ta, Ta < Tb), **THEN** `search()` for a term unique to the v2
    text **SHALL** return that paper.
  - **WHEN** that same sequence occurs, **THEN** `search()` for a term unique to
    the v1 text **SHALL** return no results.
- **Notes:** `test_reingest_older_version_keeps_latest_in_fts` — save v2
  (`ingested_at` "2026-08-28T02:00:00Z", summary contains `latestquantumfoo`),
  then save v1 (`ingested_at` "2026-08-28T01:00:00Z", summary contains
  `olderclassicbar`). Assert `search("latestquantumfoo")` len 1 and
  `search("olderclassicbar")` empty. Exercises the latest-by-`ingested_at` FTS
  refresh (store.rs:184-195); distinct from `test_upsert_updates_fts`
  (newer-replaces only).

### T-603: Taxonomy-cache determinism regression test (fix #4)
- **Intent:** [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md)
- **Touches:** src/id.rs (tests module)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0006 AC4 — repeated `ArxivCategory::parse` calls
  return equal, correct results (OnceLock memoization).
- **Success criterion (EARS):**
  - **WHEN** `ArxivCategory::parse` is invoked repeatedly and interleaved across
    multiple valid and invalid codes, **THEN** every call **SHALL** return the
    correct result for its own code.
- **Notes:** `test_taxonomy_parse_repeated_consistent` — loop 3×: `cs.CV` →
  "Computer Vision and Pattern Recognition", `math.NA` → "Numerical Analysis",
  `stat.ML` → "Machine Learning", `invalid.XX` → `Err`. Guards the `OnceLock`
  taxonomy cache (id.rs:92-98) against corrupted or cross-contaminated entries.

### T-604: Introduce Cargo workspace + `diver-core` library
- **Intent:** [INT-0007](../../../intents/INT-0007-workspace-restructure.md)
- **Touches:** Cargo.toml (→ virtual workspace manifest); diver-core/Cargo.toml
  (new); move src/{lib,client,display,fact,id,model,parse,query,store}.rs →
  diver-core/src/; move taxonomy/ → diver-core/taxonomy/; move tests/ (incl.
  tests/fixtures/) → diver-core/tests/ with `use diver::` → `use diver_core::`.
- **Depends on:** T-601, T-602, T-603
- **Acceptance criterion:** INT-0007 AC1 (workspace with members `diver-core`,
  `diver-cli`), AC2 (`diver-core` is a library crate), AC3 (embedded taxonomy
  `include_str!` resolves), AC6 (integration tests reference `diver_core::` and
  pass).
- **Success criterion (EARS):**
  - **WHEN** `cargo build -p diver-core` runs at the workspace root, **THEN**
    `diver-core` **SHALL** compile as a library with
    `include_str!("../taxonomy/arxiv_categories.json")` resolving from
    `diver-core/src/id.rs`.
  - **WHEN** the moved integration tests compile, **THEN** they **SHALL** import
    the library as `diver_core::` and pass.
- **Notes:** `include_str!` path is unchanged — `src/` and `taxonomy/` move
  together. **Fixtures coupling (verified during plan review):** `tests/fixtures/`
  is a *compile-time* dependency of `diver-core`'s **test** build, not only the
  integration tests — `src/parse.rs:174-175` (inside `#[cfg(test)]`) does
  `include_str!("../tests/fixtures/{sample_feed,empty_feed}.xml")`, and
  `src/client.rs`'s unit test + the `ingest_pipeline` integration test do a
  runtime `read_to_string("tests/fixtures/sample_feed.xml")`. All resolve iff the
  **entire** `tests/fixtures/` dir (sample/empty/error feeds) lands at
  `diver-core/tests/fixtures/`: the `../tests/fixtures/` include path resolves
  from `diver-core/src/parse.rs`, and the runtime path resolves because cargo runs
  tests with CWD = the package root (`diver-core/`). This is the decisive reason
  the tests move to `diver-core`, never `diver-cli`. A plain `cargo build -p
  diver-core` does **not** need the fixtures (the `include_str!` is `cfg(test)`).
  `diver-core` deps = all current deps except `clap` and `tokio` (verified:
  `client.rs` tests are plain `#[test]`, so no `tokio` dev-dependency is needed).
  Root `Cargo.toml` → `[workspace]` with
  `members = ["diver-core","diver-cli"]`, `resolver = "2"`; shared `Cargo.lock` /
  `target/` remain at the root.
  - **Build-phase check (per critique C-003):** after the move, inspect the
    `Cargo.lock` diff and confirm it only drops the old `diver` package entry and
    adds `diver-core` + `diver-cli` — no third-party dependency version or feature
    churn. Treat any wider diff as a signal to reconcile before committing.

### T-605: Add `diver-cli` binary + repoint imports
- **Intent:** [INT-0007](../../../intents/INT-0007-workspace-restructure.md)
- **Touches:** diver-cli/Cargo.toml (new, `[[bin]] name = "diver"`); move
  src/main.rs → diver-cli/src/main.rs with `use diver::` → `use diver_core::`.
- **Depends on:** T-604
- **Acceptance criterion:** INT-0007 AC2 (`diver-cli` is a binary crate producing
  a binary named `diver`), AC5 (every subcommand behaves as before the split).
- **Success criterion (EARS):**
  - **WHEN** `cargo build` runs at the workspace root, **THEN** `diver-cli`
    **SHALL** produce a binary named `diver`.
  - **WHEN** `diver --help` is run, **THEN** it **SHALL** list subcommands
    `search`, `ingest`, `inspect`, `list`, `collect`, `find` exactly as before
    the split.
- **Notes:** `diver-cli` deps = `anyhow, clap, tokio` + `diver-core = { path =
  "../diver-core" }`. Binary name preserved via an explicit `[[bin]]` target.

### Provenance & branch steps (INT-0006 AC1, AC5 — Book/git, not code tasks)
- **AC1:** INT-0006 enumerates the eight fixes; at Loop-phase realization attach
  Code evidence linking commit `dd69859` and PR #4. Verified by `check-book.sh`
  and doc review.
- **AC5:** `dev` already contains every commit on `main` (fast-forwarded to
  `df1f230`). Verified by `git merge-base --is-ancestor origin/main dev`. Pushing
  `origin/dev` to match is remote-affecting and out of scope for the build —
  optional follow-up gated on explicit request / the remote profile.
