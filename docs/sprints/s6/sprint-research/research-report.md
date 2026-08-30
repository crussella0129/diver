# Sprint 6 Research Report

## Intents Reviewed
- [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) — created; relevance: primary; current state: proposed
- [INT-0007](../../../intents/INT-0007-workspace-restructure.md) — created; relevance: primary; current state: proposed

## 1. Sprint Goal

Sprint 6 does the "prepare the substrate before the semantic layer" bookkeeping
that Sprint 5 left open. First, reconcile the eight hardening fixes that landed
on `main` outside the loop (commit `dd69859`, merged via PR #4): record them in
the Book as a durable intent and add regression tests for the three invariants
that shipped untested (foreign-key enforcement, stale-FTS-on-older-version
reingest, and taxonomy-cache determinism), then bring `dev` back in line with
`main`. Second, split the single `diver` crate into a `diver-core` library and a
`diver-cli` binary under a Cargo workspace — preserving all CLI behavior and the
`diver` binary name — so the epistemic engine has a reusable boundary to grow the
Observation/Assertion layer into. No new product behavior, schema change, or
semantic-layer type is in scope.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| src/lib.rs | high | Declares the 8 library modules; becomes `diver-core`'s lib root. |
| src/main.rs | high | The clap CLI; uses `diver::...` imports → becomes `diver-cli`, imports `diver_core::`. Binary must stay named `diver`. |
| src/id.rs | high | `include_str!("../taxonomy/arxiv_categories.json")` (line 7) — path is repo-root-relative; must be repointed when `taxonomy/` moves under `diver-core/`. Owns `ArxivCategory` taxonomy cache (`OnceLock`, fix #4). |
| src/store.rs | high | Owns the FTS/versioning invariants from fixes #1,#2,#5,#7,#8. `PRAGMA foreign_keys=ON` at line 49; `ON CONFLICT DO UPDATE` at line 154; `INSERT OR IGNORE INTO papers` at line 134. |
| src/fact.rs | medium | Touched by fix #3 (`ArxivCategory::unknown()`); has `test_source_fact_unknown_category_skipped`. |
| Cargo.toml | high | Currently `[package] name="diver"` + empty `[workspace]`. Becomes a virtual workspace manifest with members `diver-core`, `diver-cli`. |
| taxonomy/arxiv_categories.json | high | Embedded via `include_str!`; moves under `diver-core/`. |
| tests/ingest_pipeline.rs | medium | Integration test; `use diver::...` → `use diver_core::...`. 2 tests. |
| tests/dive_pipeline.rs | medium | Integration test; `use diver::...` → `use diver_core::...`. 1 test (`test_find_pipeline`). |
| src/display.rs, src/parse.rs, src/query.rs, src/client.rs, src/model.rs | low | Move wholesale into `diver-core`; no import-path changes internal to the crate. |
| docs/intents/INT-0005-harden-factual-substrate.md | medium | Prior substrate intent; the review fixes harden the same code paths it realized. |

Baseline (post fast-forward to `origin/main`, commit `df1f230`): `cargo test`
is green — **62 library unit tests + 3 integration tests = 65 passing**, clean
compile.

Review-fix provenance (`dd69859`) touched `src/fact.rs`, `src/id.rs`,
`src/store.rs` and added exactly three tests: `test_taxonomy_rejects_meta_key`
(fix #6), `test_unknown_preserves_code` (fix #3), and
`test_store_metadata_correction_applied` (fix #2). The remaining fixes rely on
pre-existing or absent coverage. The eight fixes:

1. FTS refresh reads the latest stored version, not the incoming fact — prevents
   a stale search index when re-ingesting an older version. *(existing:
   `test_upsert_updates_fts`; gap: no explicit older-version reingest case.)*
2. `paper_versions` upsert `INSERT OR IGNORE` → `ON CONFLICT DO UPDATE` so
   same-version metadata corrections apply. *(covered: `test_store_metadata_correction_applied`.)*
3. `parse_category_lenient` preserves the original code via
   `ArxivCategory::unknown()` instead of substituting `cs.OH`. *(covered:
   `test_unknown_preserves_code`.)*
4. `ArxivCategory::parse` caches parsed taxonomy JSON in a `OnceLock`. *(gap: no
   determinism/repeat-call test.)*
5. `PRAGMA foreign_keys=ON` enforces the `paper_versions` FK constraint. *(gap:
   no test proves enforcement.)*
6. `_meta` key rejected by `parse()` as metadata, not a category. *(covered:
   `test_taxonomy_rejects_meta_key`.)*
7. `list()` uses `row_to_fact` directly, removing a triple-nested `Result`.
   *(behavioral: `test_store_list`.)*
8. `search()` reads all display columns from `paper_versions` (source of truth)
   rather than mixing FTS + `paper_versions`. *(behavioral: `test_search_*`.)*

Regression-test gaps to close: **#5 (FK enforcement)**, **#1 (stale-FTS on older
reingest)**, **#4 (taxonomy-cache determinism)**.

## 3. External Sources
- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) — virtual manifest + `members`, shared `Cargo.lock`/`target`, path deps.
- [Cargo `[[bin]]` target](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries) — set `name = "diver"` so the `diver-cli` package still emits a `diver` binary.
- [`include_str!`](https://doc.rust-lang.org/std/macro.include_str.html) — path is relative to the current source file, so it moves with `id.rs` under `diver-core/`.
- [SQLite `PRAGMA foreign_keys`](https://www.sqlite.org/foreignkeys.html) — FK enforcement is per-connection and off by default; the regression test must assert enforcement on the store's own connection.

## 4. Risks, Unknowns, Dependencies

- **Risk:** `include_str!` path breakage. Moving `taxonomy/` under `diver-core/`
  without repointing `id.rs:7` fails the build. Mitigation: move file and macro
  path together; `cargo build` catches it immediately.
- **Risk:** losing the `diver` binary name. If `diver-cli`'s package name alone
  drives the binary, the command becomes `diver-cli`. Mitigation: an explicit
  `[[bin]] name = "diver"` target in `diver-cli`.
- **Risk:** integration tests fail to compile after the rename (`diver::` →
  `diver_core::`). Mitigation: tests move with `diver-core`; update the two
  `use` lines; `cargo test` verifies.
- **Risk:** the FK-enforcement test is a no-op if it exercises the normal
  `save()` path (which always inserts the parent `papers` row first). Mitigation:
  the test must insert directly into `paper_versions` with an absent `paper_id`
  and assert the error.
- **Unknown:** whether `Cargo.lock` / dependency resolution shifts under the
  workspace. Expected identical (same deps, same versions); verify the lock diff
  is limited to workspace-member entries.
- **Dependency:** reconciling `dev` with `main` is already done locally (`dev`
  fast-forwarded to `df1f230`). Pushing `origin/dev` to match is remote-affecting
  and gated on explicit request / the remote profile — out of scope for the
  build itself.

## 5. Recommended Approach

Primary: do INT-0006 (provenance) and INT-0007 (workspace) as one sprint,
INT-0006 first so the substrate is recorded and its invariants are locked before
the code physically moves.

- INT-0006: author the intent (done in Research), add three regression tests
  (FK enforcement, older-version-reingest FTS, taxonomy-cache determinism)
  against the current single-crate layout, confirm green, then attach evidence.
- INT-0007: introduce a virtual workspace. Move `src/{lib,client,display,fact,
  id,model,parse,query,store}.rs` + `taxonomy/` into `diver-core/`; move
  `main.rs` + `tests/` into `diver-cli/` (or keep `tests/` with the crate whose
  API they exercise — `diver-core`). Add `[[bin]] name = "diver"` to
  `diver-cli`. Repoint `include_str!`. Update `use diver::` → `use diver_core::`.
  Verify `cargo build`, `cargo test` (65 green), and manual `diver --help` +
  subcommand parity.

Sequencing note: adding the INT-0006 regression tests *before* the move means
they travel into `diver-core` with the rest of the store/id tests and
immediately re-run in the new layout, doubling as split-verification.

Alternative considered: do INT-0007 first, then add tests in the new layout —
rejected because it entangles "did the split break something" with "did the fix
regress," making a failure harder to localize.

Rationale: recording provenance and locking invariants before moving files keeps
each acceptance criterion independently verifiable, and the small current surface
makes the workspace split low-risk today and expensive to defer.

## Artifacts
- No standalone snippet/trace artifacts saved; all evidence is inline above and
  in the linked source files and commit `dd69859`.
