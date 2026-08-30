# Sprint 6 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC1: chapter records 8 fixes + links dd69859/PR#4 | doc criterion (no EARS) | deferred to realization | Code-evidence link + enumeration added at Loop-Phase `active → realized` (critique C-001) |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC2: FK constraint enforced | T-601 / `test_fk_constraint_enforced` | pass | Test evidence links this report |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC3: no stale FTS on older reingest | T-602 / `test_reingest_older_version_keeps_latest_in_fts` | pass | Test evidence links this report |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC4: taxonomy cache determinism | T-603 / `test_taxonomy_parse_repeated_consistent` | pass | Test evidence links this report |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC5: dev reconciled with main | `git merge-base --is-ancestor origin/main dev` | pass (0 behind, 12 ahead) | verified at test head |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC6: existing tests still pass | full `cargo test` suite | pass (68/68) | Test evidence links this report |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC1: workspace, members core+cli | `cargo build` / `Cargo.toml` members | pass | Test evidence links this report |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC2: core=lib, cli=bin `diver` | T-604,T-605 / build → `target/debug/diver.exe` | pass | Test evidence links this report |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC3: taxonomy include_str resolves | T-604 / `test_taxonomy_valid_code` | pass | Test evidence links this report |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC4: cargo build/test succeed | full `cargo test` suite | pass (68/68) | Test evidence links this report |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC5: CLI parity | `e2e_cli_help_lists_subcommands`, `e2e_cli_subcommand_help_parses` | pass | Test evidence links this report |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC6: integration tests use `diver_core::` | T-604 / `test_find_pipeline`, `test_ingest_pipeline*` | pass | Test evidence links this report |

## Summary
- Unit tests: 65 passed / 0 failed / 65 total (`diver_core`); 0 in `diver-cli`
- Integration tests: 3 passed / 0 failed / 3 total
- E2E tests: 2 passed / 0 failed / 2 total (CLI `--help` smokes)
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `6b4dbef6fcc81bbb187a45d6a5e533a3e26845c3`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local confirmations only. Canonical
  runner `cargo test --workspace` reported `test result: ok` for every binary
  (`diver_core` lib 65, `dive_pipeline` 1, `ingest_pipeline` 2, `diver-cli` bin 0,
  doc-tests 0). `cargo build` produced `target/debug/diver.exe`. `cargo clippy
  --workspace --all-targets` produced only 7 pre-existing warnings (no new ones).
  Records: [unit](unit-tests.md), [integration](integration-tests.md),
  [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- 7 pre-existing clippy warnings in `diver-core` (`store.rs:311` redundant
  closure; `display.rs` empty format-strings and `useless vec!` in tests) predate
  Sprint 6 and are out of the locked plan's scope. Flagged as a background task;
  not part of any Sprint 6 intent.
- INT-0006 AC1 realization evidence (code-evidence link `dd69859`/PR #4 +
  explicit 8-fix enumeration) is completed at Loop-Phase realization (critique
  C-001).

## Coverage Observations
- Every code-testable acceptance criterion has a named, executed test asserting
  the SHALL response, including negative paths (FK violation, invalid taxonomy
  code, empty-search result).
- Tests are deterministic: in-memory SQLite, fixed timestamps, embedded taxonomy,
  and side-effect-free `--help` E2E (no network, no real data dir).
- AC1 (documentation) and AC5 (git reconciliation) are non-code criteria verified
  by doc review and a git ancestry check respectively, per the locked test plan.
