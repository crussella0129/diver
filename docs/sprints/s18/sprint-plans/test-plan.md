Finalized - DO NOT EDIT

# Sprint 18 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 1 — with `DIVER_DB` unset, the resolved path is unchanged from current behaviour | T-1801 / WHEN `resolve_db_path(None, Some(d))` is called, THEN it SHALL return `d/diver/diver.db` | `test_resolve_db_path_default` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 1 — unchanged behaviour, no-data-directory fallback | T-1801 / WHEN `resolve_db_path(None, None)` is called, THEN it SHALL return `.diver/diver.db` | `test_resolve_db_path_no_data_dir` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 1 — a set-but-empty override must not silently create a throwaway corpus | T-1801 / WHEN `resolve_db_path(Some(""), Some(d))` is called, THEN it SHALL treat it as unset | `test_resolve_db_path_empty_override` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 2 — the override is honored | T-1801 / WHEN `resolve_db_path(Some(p), _)` is called with non-empty `p`, THEN it SHALL return exactly `p` | `test_resolve_db_path_override` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 2 — the platform data directory is untouched | T-1801 / WHEN `resolve_db_path` is called with any arguments, THEN it SHALL NOT create, read, or touch any directory | `test_resolve_db_path_is_side_effect_free` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 1 — the *composition* in `open()` resolves to today's path, not just the helper | T-1801 / WHEN `resolve_db_path(None, dirs::data_dir())` is evaluated, THEN it SHALL equal the exact expression `open()` uses today | `test_default_db_path_matches_legacy` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 2 — the override is honored *by the shipped binary* | T-1802 / WHEN the binary runs with `DIVER_DB` set to a path whose parent does not exist, THEN it SHALL exit successfully and create a database at exactly that path | `test_cli_diver_db_override_creates_db_at_path` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 2 — the platform default database is not created (best-effort clean-machine guard; see limits below) | T-1802 / WHEN the binary runs with `DIVER_DB` set and no platform default database exists, THEN it SHALL NOT be created | `test_cli_diver_db_override_leaves_default_db_unmodified` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 3 — `open_at(path)` creates missing parents and initializes the schema | T-1801 / WHEN `open_at(p)` is called and `p`'s parent does not exist, THEN it SHALL create the parent directories and initialize the schema | `test_open_at_creates_parent_dirs` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 4 — a corpus persisted to a scratch path can be reopened and read back | T-1801 / WHEN a paper and its assertions are saved through `open_at(p)`, dropped, and reopened, THEN it SHALL return the same papers and assertions | `test_open_at_round_trip_persists` |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | 5 — the README documents `DIVER_DB` | T-1803 / WHEN a reader consults the README, THEN it SHALL state that `DIVER_DB` overrides the default corpus path, that an unset or empty value selects the default, and SHALL warn that a stray value silently redirects the corpus | Documentation review at Test Phase (no automated test; asserted by reading the rendered section) |

## Unit Tests

### T-1801 unit tests
- **Intent:** [INT-0019](../../../intents/INT-0019-configurable-store-path.md)
- Added to the existing `mod tests` in `diver-core/src/store.rs`, matching the
  surrounding convention. No environment mutation anywhere in this sprint.
- `test_resolve_db_path_default`: `resolve_db_path(None, Some(PathBuf::from("/data")))`
  → `/data/diver/diver.db`. Pins the pre-change default expression.
- `test_resolve_db_path_no_data_dir`: `resolve_db_path(None, None)` → `.diver/diver.db`.
  Covers the fallback branch that was previously unreachable from a test because
  `dirs::data_dir()` cannot be stubbed without env mutation.
- `test_default_db_path_matches_legacy`: assert `resolve_db_path(None, dirs::data_dir())`
  equals `dirs::data_dir().map(|d| d.join("diver")).unwrap_or_else(|| PathBuf::from(".diver")).join("diver.db")`,
  with the right-hand side written out inline in the test. Hermetic — no env mutation,
  no I/O. This is the only test that pins the *composition* rather than the helper:
  `Store::open()` is called nowhere outside `diver-cli/src/main.rs`, and every existing
  test in the workspace uses `open_in_memory()`, so nothing else in the suite can catch
  a double-join like `<data>/diver/diver/diver.db` — which would silently relocate every
  existing user's corpus while the suite stays green.
- `test_resolve_db_path_override`: `resolve_db_path(Some(OsString::from(p)), Some(d))` →
  exactly `PathBuf::from(p)`, with `d` ignored, including when `p`'s parent does not exist.
- `test_resolve_db_path_empty_override`: `resolve_db_path(Some(OsString::new()), Some(d))`
  → `d/diver/diver.db`. Guards the silent-throwaway-corpus hazard: `std::env::var_os`
  returns `Some("")` for a set-but-empty variable, and SQLite treats an empty filename as
  a private temporary database that vanishes on close.
- `test_resolve_db_path_is_side_effect_free`: call `resolve_db_path` with a data-directory
  argument pointing at a nonexistent path under a scratch dir, then assert that path still
  does not exist. Proves resolution creates nothing — the structural guarantee behind
  "platform data directory untouched".
- `test_open_at_creates_parent_dirs`: `open_at` on a nested, nonexistent path under a
  scratch directory → succeeds, the file exists, and `list()` returns `Ok`, proving
  `init_schema` ran.
- `test_open_at_round_trip_persists`: save a `SourceFact` and its `Assertion<Supported>`
  set through `open_at(p)`; drop the store; `open_at(p)` again → `get()` returns the same
  paper and `get_assertions()` the same claims. This is the test that actually proves
  durable fixture corpora are possible, which is the reason INT-0019 exists.
- Stubs: none. Scratch paths come from `tempfile::tempdir()` (dev-dependency added to
  `diver-core` in T-1801, and to `diver-cli` in T-1802 alongside `dirs = "6"` pinned to
  match `diver-core`); no environment mutation, no network, no dependency on the
  developer's corpus.

## Integration Tests

### Store persistence integration
- **Intents:** [INT-0019](../../../intents/INT-0019-configurable-store-path.md)
- `test_open_at_round_trip_persists` doubles as the integration check: it composes
  `open_at` + `save` + `save_assertions` + `get` + `get_assertions` across a real on-disk
  SQLite file and a store lifecycle boundary, which `open_in_memory` cannot exercise. No
  separate integration file is added — the surrounding store tests live in
  `diver-core/src/store.rs`, and splitting this one out would break that convention for
  no gain.
- Regression guard: the existing suite (132 tests) must still pass unchanged. Note
  precisely what that does and does not prove. Every existing test — including
  `diver-core/tests/real_corpus.rs` and all 30 `open_in_memory()` call sites in the
  `store.rs` module — goes through `open_in_memory()`, and `Store::open()` has no caller
  outside `diver-cli/src/main.rs`. So the existing suite proves the **schema and query
  behaviour** survived the refactor; it proves nothing about `open()` itself. That gap is
  covered deliberately and only by `test_default_db_path_matches_legacy` (the resolved
  default path) and the two T-1802 CLI tests (the env read). Without those three,
  `open()` would ship with zero coverage.

## End-to-End Tests
- **Status:** possible (for this sprint's deliverable)
- `test_cli_diver_db_override_creates_db_at_path` — in `diver-cli/tests/db_override.rs`.
  Run `env!("CARGO_BIN_EXE_diver")` with subcommand `list` and
  `.env("DIVER_DB", <scratch>/nested/scratch.db)`. Pass: exit status success **and**
  `<scratch>/nested/scratch.db` exists. Fail: non-zero exit, or the file is absent
  (meaning `open()` ignored the variable). `Command::env` is safe and requires no
  `std::env::set_var`.
- `test_cli_diver_db_override_leaves_default_db_unmodified` — same invocation, additionally
  asserting the platform default database is untouched. The test resolves
  `dirs::data_dir()`, and for `…/diver/diver.db` snapshots **before and after**: whether
  it exists, its modified time if it does, and whether a `diver.db-wal` sidecar is
  present. All three must be unchanged. Skips when `dirs::data_dir()` is `None`.
  **Stated limit — this is a best-effort clean-machine guard, not the regression signal.**
  On a machine that has already run `diver ingest`, it cannot discriminate: SQLite deletes
  the `-wal`/`-shm` sidecars when the last connection closes cleanly, and against an
  already-initialized corpus `PRAGMA journal_mode=WAL` plus `CREATE TABLE IF NOT EXISTS`
  write nothing, so mtime need not change either. All three observations can be identical
  whether or not the override was honored. The test is still worth having — on a clean
  machine or in CI it catches a created-by-mistake default database — but it must not be
  read as proof.
  **Where the real signal lives:** an ignored `DIVER_DB` means the scratch path is never
  created, which fails `test_cli_diver_db_override_creates_db_at_path`. That test, plus
  the hermetic `test_resolve_db_path_is_side_effect_free`, is the actual coverage of
  INT-0019 criterion 2. The snapshot below is a third, weaker check layered on top.
  **Why not redirect the data directory instead:** on Windows `dirs::data_dir()` resolves
  through the known-folder API (`SHGetKnownFolderPath`), not `%APPDATA%`, so an env
  redirect would silently fail to apply on this sprint's own platform and the test would
  pass for the wrong reason.
  **Flake risk, accepted:** the snapshot observes the developer's real corpus, so a
  concurrent `diver` invocation during the test run could fail it spuriously. Acceptable
  for a guard this cheap; if it ever flakes, delete it rather than weakening the two tests
  that carry the real signal.
- **Still not-yet-possible:** an *evaluation-quality* end-to-end run — ingest, extract,
  dive, and score against a fixed corpus — remains out of reach. Unlocked by
  [INT-0022](../../../intents/INT-0022-relation-evaluation-harness.md), which is chartered
  to design the fixture-corpus and gold-label format. Attempting it here would prejudge
  that intent's open questions. T-1801 exists precisely to make it possible.
