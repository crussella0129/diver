# Sprint 18 Test Report

## Intent Verification

| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | AC1: `DIVER_DB` unset → path unchanged | T-1801 / `test_resolve_db_path_default`, `test_resolve_db_path_no_data_dir`, `test_resolve_db_path_empty_override`, `test_default_db_path_matches_legacy` | pass | Test evidence links this report |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | AC2: override honored, platform data dir untouched | T-1801/T-1802 / `test_resolve_db_path_override`, `test_resolve_db_path_is_side_effect_free`, `test_cli_diver_db_override_creates_db_at_path` | pass | Test evidence links this report |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | AC3: `open_at` creates parents + initializes schema | T-1801 / `test_open_at_creates_parent_dirs` | pass | Test evidence links this report |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | AC4: durable corpus survives reopen | T-1801 / `test_open_at_round_trip_persists` (paper, claim, version, support) | pass | Test evidence links this report |
| [INT-0019](../../../intents/INT-0019-configurable-store-path.md) | AC5: README documents `DIVER_DB` | T-1803 / recorded human review, three SHALLs checked against quoted text | pass (no automated test — [documentation-review.md](documentation-review.md)) | Test evidence links this report |

The five forward-looking chapters authored this sprint —
[INT-0020](../../../intents/INT-0020-first-class-concepts.md),
[INT-0021](../../../intents/INT-0021-typed-epistemic-relations.md),
[INT-0022](../../../intents/INT-0022-relation-evaluation-harness.md),
[INT-0023](../../../intents/INT-0023-full-text-evidence.md),
[INT-0024](../../../intents/INT-0024-incremental-materialization.md) — remain `proposed`
and carry no acceptance verification, by design. They describe work that has not been
accepted into execution; there is nothing to test yet.

## Summary

- **Unit tests:** 129 passed in `diver_core` (+8 new, all hermetic — no environment
  mutation anywhere in this sprint) + 1 in the `diver-cli` bin.
- **Integration:** every pre-existing binary green and unmodified.
  `test_open_at_round_trip_persists` is the integration check by substance — it crosses a
  drop-and-reopen boundary over a real on-disk SQLite file, which `open_in_memory()`
  structurally cannot. Sprint 18 adds no integration-*only* test, as the locked plan
  anticipated.
- **E2E:** 2 new CLI subprocess tests. `possible` for this sprint's deliverable;
  `not-yet-possible` for evaluation-quality corpus scoring, unlocked by INT-0022.
- **Clippy:** 0 warnings (`--workspace --all-targets -- -D warnings`). **fmt:** clean.
- **Total:** **142 passed; 0 failed** (132 baseline + 8 unit + 2 E2E).
- **CI status:** not-configured.

### Fault injection

The one test guarding the default path was verified by injecting the exact fault it claims
to catch, rather than by inspection: changing `current_db_path_for` to
`resolve_db_path(override_value, dirs::data_dir().map(|d| d.join("diver")))` — the silent
double-join the build plan named as this sprint's one dangerous refactor error — fails the
test with `…\diver\diver\diver.db` vs `…\diver\diver.db`. Run twice, against both
successive forms of the fix; reverted and re-run green each time. This matters because the
test critic found the original version of that test did **not** catch it.

## CI Confirmation

- **Head SHA:** `79a12818bb5c09787afc8e9a2f98ee3881fd8a3c`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` →
  `test result: ok` for every binary, **142 total**. `cargo clippy --workspace
  --all-targets -- -D warnings` → 0. `cargo fmt --check` → clean. Records:
  [unit](unit-tests.md), [integration](integration-tests.md), [e2e](e2e-tests.md),
  [documentation review](documentation-review.md), [critique](critique.md).

## Failures

(none)

## Technical Debt Identified

- **`Store::open()`'s delegation is unpinned** (test-critique C-102). The default-path test
  constrains `current_db_path_for`, which holds the composition; that `open()` reaches it
  through `open_at(current_db_path())` and `current_db_path_for(var_os(DB_PATH_ENV))` is
  asserted by nothing. Deliberate: those lines hold no path composition, and covering them
  directly would mean calling `Store::open()` with `DIVER_DB` unset, writing to the
  developer's real corpus.
- **`test_cli_diver_db_override_leaves_default_db_unmodified` is vacuous on a populated
  machine** (C-103), including the one that produced this evidence. It is a clean-machine
  and CI guard; AC2's evidence rests on `test_cli_diver_db_override_creates_db_at_path`
  plus the hermetic `test_resolve_db_path_is_side_effect_free`.
- **AC5 has no automated verification.** Verified by recorded human review. Making it
  executable (an `include_str!` assertion over the README) was considered and left as a
  possible future guard.
- **No CI.** Every confirmation in this report is a local run. This is pre-existing across
  the project, not introduced here, but it is the reason the clean-machine guard above has
  nowhere to run meaningfully.
- **Backlog carried forward, untouched this sprint:** T-1310, T-1410, T-1510
  (T-1710 is now absorbed by INT-0020).
