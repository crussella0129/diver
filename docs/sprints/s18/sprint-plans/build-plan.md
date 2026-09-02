Finalized - DO NOT EDIT

# Sprint 18 Build Plan

## Intents
- [INT-0019](../../../intents/INT-0019-configurable-store-path.md) — state: planned; acceptance criteria covered: 1 (unset default unchanged), 2 (override honored, platform data directory untouched), 3 (`open_at` creates parents and initializes schema), 4 (durable round-trip on a scratch path), 5 (README documents `DIVER_DB`).

Authored this sprint but **not** scheduled — each remains `proposed`, i.e. described
and deliberately not accepted into executable work. They carry no task here:
[INT-0020](../../../intents/INT-0020-first-class-concepts.md),
[INT-0021](../../../intents/INT-0021-typed-epistemic-relations.md),
[INT-0022](../../../intents/INT-0022-relation-evaluation-harness.md),
[INT-0023](../../../intents/INT-0023-full-text-evidence.md),
[INT-0024](../../../intents/INT-0024-incremental-materialization.md).

## Schema Tree
- Sprint Goal — realign the roadmap into durable intent, and clear the one maintenance item that gates it
  - Roadmap realignment (landed in the Research phase; no build task)
    - Six intent chapters, the epistemic charter in `docs/README.md`, and the preserved external review
  - Store path configurability
    - T-1801: pure path resolution + a path-taking `Store` constructor
    - T-1802: CLI-level verification that `DIVER_DB` is actually wired through `open()`
    - T-1803: document `DIVER_DB`

## Execution Sequence

### T-1801: Resolve the store path purely, and add a path-taking `Store` constructor
- **Intent:** [INT-0019](../../../intents/INT-0019-configurable-store-path.md)
- **Touches:** `diver-core/src/store.rs`, `diver-core/Cargo.toml`, `Cargo.lock`
- **Depends on:** (none)
- **Acceptance criterion:** INT-0019 criteria 1–4 — default resolution unchanged when
  `DIVER_DB` is unset; the override is used when set, leaving the platform data
  directory untouched; `open_at` creates missing parents and initializes the schema;
  a corpus persisted to a scratch path survives reopen.
- **Success criterion (EARS):**
  - **WHEN** `resolve_db_path(None, Some(d))` is called, **THEN** it **SHALL** return
    `d/diver/diver.db`, identical to the path `open()` computes today.
  - **WHEN** `resolve_db_path(None, None)` is called, **THEN** it **SHALL** return
    `.diver/diver.db`, identical to today's no-data-directory fallback.
  - **WHEN** `resolve_db_path(Some(p), _)` is called with a non-empty `p`, **THEN** it
    **SHALL** return exactly `p`, and **SHALL NOT** consult the data directory argument.
  - **WHEN** `resolve_db_path(Some(""), Some(d))` is called with a set-but-empty
    override, **THEN** it **SHALL** treat it as unset and return `d/diver/diver.db`.
  - **WHEN** `resolve_db_path` is called with any arguments, **THEN** it **SHALL NOT**
    create, read, or otherwise touch any directory on disk.
  - **WHEN** `resolve_db_path(None, dirs::data_dir())` is evaluated, **THEN** it
    **SHALL** equal the exact expression `open()` uses today —
    `dirs::data_dir().map(|d| d.join("diver")).unwrap_or_else(|| PathBuf::from(".diver")).join("diver.db")`
    — computed independently in the test.
  - **WHEN** `open_at(p)` is called and `p`'s parent directory does not exist, **THEN**
    it **SHALL** create the parent directories and initialize the schema.
  - **WHEN** a paper and its assertions are saved through `open_at(p)`, the store is
    dropped, and `open_at(p)` is called again, **THEN** it **SHALL** return the same
    papers and the same assertions.
- **Notes:** `resolve_db_path(override_value: Option<OsString>, data_dir: Option<PathBuf>)
  -> PathBuf` takes **both** the override value and the data directory as parameters.
  This is deliberate and closes three problems at once: it makes every branch —
  including the no-data-directory fallback — unit-testable **without** mutating process
  environment (sidestepping the `unsafe` `std::env::set_var` hazard recorded in backlog
  T-1410), and it removes all filesystem side effects from resolution. Today `open()`
  calls `create_dir_all(&data_dir)` *before* computing the path (`store.rs` l.37-40);
  that call must move into `open_at`, so the platform data directory is untouched when
  an override is set. `open()` becomes
  `open_at(resolve_db_path(std::env::var_os("DIVER_DB"), dirs::data_dir()))`.
  `open_in_memory()` is untouched. Reuse the existing `create_dir_all` +
  `Connection::open` + `init_schema` sequence rather than writing a second one.
  Add `tempfile` as a **dev**-dependency of `diver-core` for scratch paths — the
  workspace has no temp-directory helper and no `rand`, so hand-rolling unique names
  under `std::env::temp_dir()` would risk collisions under `cargo test`'s default
  parallelism and leak directories on panic.
  The last clause exists because the composition is the one line no unit test would
  otherwise reach, and its most natural refactor error is silent: carrying today's
  `.map(|d| d.join("diver"))` (l.33-35) into the call site as well as into
  `resolve_db_path` yields `<data>/diver/diver/diver.db`, relocating every existing
  user's corpus while the whole suite stays green.

### T-1802: Verify the `DIVER_DB` wiring end-to-end through the real binary
- **Intent:** [INT-0019](../../../intents/INT-0019-configurable-store-path.md)
- **Touches:** `diver-cli/tests/db_override.rs` (new), `diver-cli/Cargo.toml`, `Cargo.lock`
- **Depends on:** T-1801
- **Acceptance criterion:** INT-0019 criterion 2 — with `DIVER_DB` set, the store reads
  and writes there and the platform data directory is untouched.
- **Success criterion (EARS):**
  - **WHEN** the `diver` binary is run with `DIVER_DB` set to a path whose parent does
    not exist, **THEN** the command **SHALL** exit successfully and **SHALL** create a
    database at exactly that path.
  - **WHEN** the `diver` binary is run with `DIVER_DB` set and no platform default
    database exists, **THEN** the platform default database **SHALL NOT** be created.
- **Notes:** this is the only verification that `open()` actually reads the environment.
  T-1801's unit tests prove `resolve_db_path` and `open_at` in isolation, but the single
  composing line — the `var_os("DIVER_DB")` call — has no coverage without this, so an
  inverted `Option`, a `var`/`var_os` slip, or a misspelled variable name would ship
  green. Set the variable with `std::process::Command::env`, which is **safe** and needs
  no `set_var`; locate the binary with `env!("CARGO_BIN_EXE_diver")`. `diver list` is the
  cheapest subcommand that opens the store and makes no network call.
  `diver-cli` currently has no `tests/` directory and no dev-dependencies. Add both:
  `tempfile` for the scratch path, and `dirs = "6"` — **pinned to the same major as
  `diver-core`'s `dirs` dependency**, because the test must resolve the same platform
  directory the binary does, and `dirs` is not re-exported from `diver-core`
  (`lib.rs` has no `pub use`) so it is not reachable transitively.
  The second clause is deliberately narrow. An earlier draft said the default database
  must be "left unmodified — neither created if absent, nor written to if present," but
  that is not decidable by any observation available to the test: SQLite removes the
  `-wal`/`-shm` sidecars when the last connection closes cleanly, and on an
  already-initialized corpus `PRAGMA journal_mode=WAL` plus `CREATE TABLE IF NOT EXISTS`
  write nothing, so the file's modified time need not change either. The clause therefore
  promises only what can be checked — that the default database is not *created* — and
  the real regression signal for a dropped override lives in the first clause: if
  `open()` ignored `DIVER_DB`, the scratch path would never appear and
  `test_cli_diver_db_override_creates_db_at_path` fails. The hermetic
  `test_resolve_db_path_is_side_effect_free` carries the structural half.

### T-1803: Document the `DIVER_DB` override
- **Intent:** [INT-0019](../../../intents/INT-0019-configurable-store-path.md)
- **Touches:** `README.md`
- **Depends on:** T-1801
- **Acceptance criterion:** INT-0019 criterion 5 — the README documents `DIVER_DB`.
- **Success criterion (EARS):**
  - **WHEN** a reader consults the README, **THEN** it **SHALL** state that `DIVER_DB`
    overrides the default corpus path, **SHALL** state that an unset or empty value
    selects the default, and **SHALL** warn that a stray value silently redirects the
    corpus.
- **Notes:** the clause deliberately does **not** pin a section. `## Database
  compatibility` is a pre-Sprint-5 schema-migration warning, whereas the env-var
  configuration style to match (`DIVER_PROVIDER`, `DIVER_PROVIDERS_CONFIG`) lives under
  `### Providers config`. Place the note where a reader looking for configuration will
  find it, and cross-reference from `## Database compatibility` if useful.

## Deferrals
- **INT-0019 Consequences names a second mitigation** — "a future `diver inspect`-style
  path echo would be a cheap further guard" — which is **not** scheduled this sprint.
  Deferred deliberately: T-1803's README warning is the agreed mitigation for a stray
  `DIVER_DB`, and a path-echo affordance is a CLI design question worth its own
  treatment. To be recorded as a backlog task in `docs/work/tasks.md` at Loop Phase so
  it is not lost the way T-1611 was.

## Loop Phase bookkeeping
Assigned explicitly so it is not assumed. No build task Touches `docs/work/tasks.md`;
all three items below are Loop-Phase edits:
- **Close T-1611.** It is the backlog item this sprint realizes and is still `- [ ]`.
- **Correct T-1611's intent tag.** It currently reads `[intent: INT-0017]`, which was
  never right — the work belongs to INT-0019, which did not exist when it was filed.
  Record the correction rather than silently rewriting it.
- **File the deferred path-echo item** from `## Deferrals` above as a new backlog task
  against INT-0019.
