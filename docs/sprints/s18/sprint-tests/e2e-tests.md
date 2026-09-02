# Sprint 18 End-to-End Tests

- **Tested head:** `f651bb6d9353dda127bbfcfc223300ccf719225a`
- **Status:** **possible** for this sprint's deliverable (the `DIVER_DB` override);
  still **not-yet-possible** for evaluation-quality corpus scoring.
- **Location:** `diver-cli/tests/db_override.rs` (new)

## Why an E2E was required here

`Store::open()` is the only place the `DIVER_DB` environment variable is read, and it is
reachable only from the CLI binary. Every unit test proves `resolve_db_path` and
`open_at` in isolation; none of them touches the single composing line. An inverted
`Option`, a `var`/`var_os` slip, or a misspelled variable name would have shipped with a
fully green suite. These two tests are the only thing standing between that and a release.

Both set the variable with `std::process::Command::env`, which is **safe** — no
`std::env::set_var`, which is `unsafe` in edition 2024. The binary is located with
`env!("CARGO_BIN_EXE_diver")`. `list` is the cheapest subcommand that opens the store and
makes no network call.

## Results

- **Intent:** [INT-0019](../../../intents/INT-0019-configurable-store-path.md) — AC2

### `test_cli_diver_db_override_creates_db_at_path` — **pass**
Runs `diver list` with `DIVER_DB` set to `<scratch>/nested/scratch.db`, a path whose
parent does not exist. Asserts the process exits successfully **and** the database exists
at exactly that path.

This is the **real regression signal for the entire override**: had `open()` ignored
`DIVER_DB`, the binary would have opened the platform default instead and this path would
never have appeared. It also exercises `open_at`'s parent-directory creation through the
shipped binary rather than through a library call.

### `test_cli_diver_db_override_leaves_default_db_unmodified` — **pass (vacuous on this run)**
Same invocation, additionally asserting that the platform default database was not
*created* when it did not already exist. Skips when `dirs::data_dir()` is `None`.

**On the tested head this assertion did not execute.** `%APPDATA%\diver\diver.db` already
exists on this machine (it predates the run), so `existed_before` was true and the guarded
branch was skipped — the only thing asserted was that the process exited successfully,
which makes the test a strict subset of `test_cli_diver_db_override_creates_db_at_path`
here. Recorded explicitly because the test critic (C-103) was right that a bare "pass"
cannot be distinguished from "the guard ran and held." The guard carries signal only on a
clean machine or in CI; it is not part of this sprint's evidence for AC2.

### Deviation from the locked test plan — deliberate

The locked plan specified a richer observation: "snapshots **before and after**: whether it
exists, its modified time if it does, and whether a `diver.db-wal` sidecar is present. All
three must be unchanged." **The shipped test records only existence** and asserts only in
the `!existed_before` branch; modified time and the sidecar are not observed.

This reduction is intentional, and the test critic (C-003) was right that it needed saying.
The dropped observations could not have delivered what the plan hoped: as spelled out
below, neither mtime nor sidecar presence discriminates against an already-initialized
corpus, so they would have added apparent rigor without adding evidence. Dropping them also
retires the plan's own "Flake risk, accepted" note — an mtime snapshot could have been
raced by a concurrent `diver` invocation and failed spuriously, and existence cannot. The
narrower test is strictly more honest and strictly less flaky; it is simply not the
regression signal, which is why the section above says so twice.

**This is a best-effort clean-machine guard, not proof.** Stated plainly because an
earlier draft of the plan overclaimed here and the plan critic (round 3, C-301) was right
to reject it: on a machine that has already run `diver ingest`, no available observation
discriminates. SQLite deletes the `-wal`/`-shm` sidecars when the last connection closes
cleanly, and against an already-initialized corpus `PRAGMA journal_mode=WAL` plus
`CREATE TABLE IF NOT EXISTS` write nothing — so existence, modified time, and WAL presence
can all be identical whether or not the override was honored. The EARS clause was
consequently narrowed to what *is* decidable ("SHALL NOT be created" when absent).

Coverage of AC2 is therefore the **pair** of `test_cli_diver_db_override_creates_db_at_path`
(behavioural) and the hermetic `test_resolve_db_path_is_side_effect_free` (structural) —
not this guard alone.

**Why not redirect the data directory instead:** on Windows `dirs::data_dir()` resolves
through the known-folder API (`SHGetKnownFolderPath`), not `%APPDATA%`, so redirecting the
environment variable would silently fail to apply on this sprint's own platform and the
test would have passed for the wrong reason. This is why the plan rejected the round-1
critic's suggested mechanism and parameterized the data directory instead.

## Still not-yet-possible

An **evaluation-quality** end-to-end run — ingest, extract, dive, and score against a
fixed corpus with gold labels — remains out of reach.

- **Unlocked by:** [INT-0022](../../../intents/INT-0022-relation-evaluation-harness.md),
  which is chartered to design the fixture-corpus and gold-label format.
- **Rationale:** attempting it now would mean inventing that format ahead of the intent
  that owns the decision, prejudging its open questions. INT-0019 exists precisely to make
  that work possible — a durable corpus at a known path is its prerequisite, and
  `test_open_at_round_trip_persists` demonstrates the capability now exists.

## Raw result
```
Running tests\db_override.rs (diver-cli)
test test_cli_diver_db_override_creates_db_at_path ... ok
test test_cli_diver_db_override_leaves_default_db_unmodified ... ok
test result: ok. 2 passed; 0 failed
```
