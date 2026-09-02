# Sprint 18 Unit Tests

- **Tested head:** `48c5fb873208ed3219910e1894ac937b69835a97`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --check`
- **Result:** `diver_core` lib — **129 passed; 0 failed** (+8 new); `diver-cli` bin — 1. Clippy: 0. fmt: clean.

## New (INT-0019), `diver-core/src/store.rs`

All eight are hermetic: no environment mutation anywhere in this sprint, no network,
no dependency on the developer's real corpus. This matters because the whole design of
`resolve_db_path` — taking both the override value *and* the data directory as
parameters — exists to make these testable without `std::env::set_var`, which is
`unsafe` in edition 2024 (the hazard recorded in backlog T-1410).

- **Intent:** [INT-0019](../../../intents/INT-0019-configurable-store-path.md) — AC1–AC4
- `test_resolve_db_path_default`: `resolve_db_path(None, Some("/data"))` →
  `/data/diver/diver.db`. **pass** (AC1; EARS clause 1)
- `test_resolve_db_path_no_data_dir`: `resolve_db_path(None, None)` → `.diver/diver.db`.
  Covers the fallback branch that was unreachable from any test before the data
  directory became a parameter. **pass** (AC1; EARS clause 2)
- `test_default_db_path_matches_legacy`: `current_db_path()` — the function `Store::open()`
  itself calls — equals the pre-change default expression, written out inline in the test.
  Skips when `DIVER_DB` is set in the test environment, since the default branch is then not
  the one `open()` would take.

  **Strengthened during the Test Phase** (a correction to T-1801, in response to test-critic C-001). As first written it asserted
  `resolve_db_path(None, dirs::data_dir())`, re-deriving the call-site argument rather than
  reading it — so it constrained only the helper, duplicating `test_resolve_db_path_default`,
  while three artifacts claimed it pinned the composition. The test critic (C-001) caught
  the overclaim. The call-site expression was extracted into `current_db_path()`, which both
  `open()` and the test now call.

  **Verified by fault injection rather than inspection:** temporarily changing the call site
  to `resolve_db_path(var_os(DB_PATH_ENV), dirs::data_dir().map(|d| d.join("diver")))` — the
  exact silent double-join the plan named as this sprint's one dangerous refactor error —
  makes the test fail with `left: "…\AppData\Roaming\diver\diver\diver.db"` vs
  `right: "…\AppData\Roaming\diver\diver.db"`. The fault was then reverted and the suite
  re-run green. (The pre-fix test would not have caught that injection — it never evaluated
  the call site — but that is reasoning about the old code, not an observed run.) **pass**
  (AC1; EARS clause 6)
- `test_resolve_db_path_override`: a non-empty override returns exactly the given path
  and the data-directory argument is not consulted. **pass** (AC2; EARS clause 3)
- `test_resolve_db_path_empty_override`: `Some("")` → the platform default, **not** the
  empty path. `std::env::var_os` yields `Some("")` for `DIVER_DB=`, and SQLite reads an
  empty filename as a private temporary database discarded on close, so honoring it
  literally would hand every command a silent throwaway corpus. **pass** (AC1; EARS clause 4)
- `test_resolve_db_path_is_side_effect_free`: resolution against a nonexistent
  data-directory path leaves that path absent. Proves resolution creates nothing — the
  structural half of "the platform data directory is untouched", and the reason
  `create_dir_all` moved out of path resolution into `open_at`. **pass** (AC2; EARS clause 5)
- `test_open_at_creates_parent_dirs`: `open_at` on a nested nonexistent path creates the
  file, and `list()` returns `Ok`, proving `init_schema` ran. **pass** (AC3; EARS clause 7)
- `test_open_at_round_trip_persists`: save a `SourceFact` + `Assertion<Supported>` through
  `open_at(p)`, drop the store, reopen → same paper and same claim. **pass** (AC4; EARS clause 8)

## Regression scope — what the pre-existing suite does and does not prove

All 121 pre-existing `diver_core` tests still pass unchanged. Stated precisely: every one
of them (and `diver-core/tests/real_corpus.rs`) goes through `open_in_memory()`, and
`Store::open()` has no caller outside the CLI. So the existing suite proves the **schema
and query behaviour** survived moving `open()`'s body behind `open_at` — it proves nothing
about `open()` itself. That gap is closed deliberately and only by
`test_default_db_path_matches_legacy` above and the two E2E tests in
[e2e-tests.md](e2e-tests.md). Without those three, `open()` would ship with zero coverage.

## Raw result
```
cargo clippy --workspace --all-targets -- -D warnings  →  0 warnings
cargo fmt --check                                      →  clean
Running unittests src\lib.rs (diver_core)              →  129 passed; 0 failed
Running unittests src\main.rs (diver-cli)              →    1 passed; 0 failed
```
