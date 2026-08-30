# Plan Critique — Sprint 6

## Concerns

### C-001: Structural tasks (T-604, T-605) have no dedicated unit test
- **Where:** `build-plan.md` T-604/T-605 / `test-plan.md` "T-604 / T-605
  verification"
- **Quote:** "no new unit tests; existing suite is the guard"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** T-604's "`diver-core` SHALL compile" and T-605's
  "SHALL produce a binary named `diver`" are EARS clauses whose verification is a
  build/e2e check, not a `test_*` function. If read strictly, an EARS clause
  lacks a named unit test.
- **Suggested response:** defer-with-rationale — a restructure's correct
  verification is that the *entire pre-existing suite plus the three new tests*
  compile and pass under the new crate layout (AC4/AC6), plus `cargo build` for
  the binary-name clause and the two named e2e smokes (`e2e_cli_help_lists_
  subcommands`, `e2e_cli_list_runs`) for CLI parity. The traceability table maps
  every clause to a concrete verification; adding a synthetic unit test for
  "the crate compiles" would be redundant with the build itself.

### C-002: T-604 bundles the workspace manifest, `diver-core` creation, and the test move into one task
- **Where:** `build-plan.md` T-604 Touches
- **Quote:** "move `tests/` (incl. `tests/fixtures/`) → `diver-core/tests/` with
  `use diver::` → `use diver_core::`"
- **Failure mode:** granularity
- **Why it matters:** a single task touches the root manifest, creates a crate,
  moves nine source files, and rewrites integration-test imports — a large diff.
- **Suggested response:** defer-with-rationale — the pieces are not independently
  compilable. Renaming the crate to `diver-core` without simultaneously
  repointing the integration tests' `use diver::` leaves the workspace in a
  non-building intermediate state. Bundling yields one coherent, atomically
  green diff, which is the granularity rule's intent (coherent diff), not a
  violation of it. T-605 (the CLI crate) is correctly split out because it *is*
  independently sequenced after core exists.

### C-003: `Cargo.lock` / resolver drift under the workspace is an open unknown
- **Where:** `research-report.md` §4 / `build-plan.md` T-604 Notes
- **Quote:** "whether `Cargo.lock` / dependency resolution shifts under the
  workspace. Expected identical"
- **Failure mode:** missing-risk
- **Why it matters:** a virtual workspace with `resolver = "2"` on edition-2024
  members could in principle re-resolve features and perturb the lockfile beyond
  member entries, which would be an unreviewed change riding along with the
  restructure.
- **Suggested response:** fix-in-plan — the Build phase must inspect the
  `Cargo.lock` diff after T-604 and confirm it only drops the old `diver` package
  entry and adds `diver-core` + `diver-cli`, with **no third-party dependency
  version or feature churn**; treat any wider diff as a signal to reconcile before
  committing. Recorded in `build-plan.md` T-604 as an explicit Build-phase check.

## Confidence
proceed-with-caveats
