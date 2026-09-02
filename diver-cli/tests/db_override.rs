//! End-to-end coverage of the `DIVER_DB` override through the real binary.
//!
//! `Store::open()` is reachable only from the CLI, and every other test in the
//! workspace uses `Store::open_in_memory()` or `Store::open_at()`. So the one line
//! that composes the override — the `var_os("DIVER_DB")` read — has no coverage
//! anywhere else: an inverted `Option`, a `var`/`var_os` slip, or a misspelled
//! variable name would ship with a green suite. These tests close that gap.
//!
//! The variable is set with `Command::env`, which is safe and needs no
//! `std::env::set_var` (unsafe in edition 2024). `list` is the cheapest subcommand
//! that opens the store, and it makes no network call.

use std::path::PathBuf;
use std::process::Command;

/// Run `diver list` with `DIVER_DB` pointed at `db_path`.
fn run_list_with_db(db_path: &PathBuf) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_diver"))
        .arg("list")
        .env("DIVER_DB", db_path)
        .output()
        .expect("the diver binary runs")
}

#[test]
fn test_cli_diver_db_override_creates_db_at_path() {
    let scratch = tempfile::tempdir().expect("scratch dir");
    // A parent that does not exist yet, so this also proves `open_at` creates it.
    let db_path = scratch.path().join("nested").join("scratch.db");

    let output = run_list_with_db(&db_path);

    assert!(
        output.status.success(),
        "diver list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // This assertion is the real regression signal for the whole override: if
    // `open()` ignored DIVER_DB, the binary would have opened the platform
    // default instead and this path would never appear.
    assert!(
        db_path.exists(),
        "DIVER_DB was ignored — no database at {}",
        db_path.display()
    );
}

#[test]
fn test_cli_diver_db_override_leaves_default_db_unmodified() {
    // A best-effort clean-machine guard, NOT the regression signal — see
    // `test_cli_diver_db_override_creates_db_at_path` for that. On a machine that
    // has already run `diver ingest` this cannot discriminate: SQLite deletes the
    // -wal/-shm sidecars when the last connection closes cleanly, and against an
    // already-initialized corpus `PRAGMA journal_mode=WAL` plus
    // `CREATE TABLE IF NOT EXISTS` write nothing, so mtime need not change either.
    // It still catches a default database created by mistake on a clean machine.
    let Some(data_dir) = dirs::data_dir() else {
        return; // No platform data directory: nothing to guard.
    };
    let default_db = data_dir.join("diver").join("diver.db");
    let existed_before = default_db.exists();

    let scratch = tempfile::tempdir().expect("scratch dir");
    let db_path = scratch.path().join("scratch.db");
    let output = run_list_with_db(&db_path);
    assert!(output.status.success());

    if !existed_before {
        assert!(
            !default_db.exists(),
            "DIVER_DB was set, but the platform default database was created at {}",
            default_db.display()
        );
    }
}
