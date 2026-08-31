# Plan Critique — Sprint 10

## Concerns

### C-001: T-1001's EARS clauses are verified by tooling/review, not a new named test
- **Where:** `build-plan.md` T-1001 / `test-plan.md` (no new tests)
- **Quote:** "No new unit/integration tests (lint-only; the existing 94-test suite
  is the regression guard)."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** the acceptance criteria map to `cargo clippy` (AC1), a diff
  review (AC2), and the existing suite (AC3) rather than to newly authored tests.
- **Suggested response:** defer-with-rationale — this is correct for a lint-only
  maintenance sprint. The changes are mechanical, semantics-preserving `clippy`
  rewrites (`redundant_closure`, `useless_vec`, `uninlined_format_args`); authoring
  a new test for "the closure was inlined" would be meaningless. AC1 is a clippy
  gate, AC2 is a bounded two-file diff review, and AC3 is the full 94-test
  regression suite — all objective and executed. Adding tests would be noise.

### C-002: the fixes were produced by a separate (background) session
- **Where:** `build-plan.md` T-1001 Notes
- **Quote:** "the fixes are already applied in the working tree by the background
  maintenance session"
- **Failure mode:** intent-drift (provenance)
- **Why it matters:** adopting another session's uncommitted edits risks pulling
  in unintended changes.
- **Suggested response:** fix-in-plan — the Build phase must confirm, before
  committing, that `git diff` is confined to `diver-core/src/store.rs` and
  `diver-core/src/display.rs` and contains only lint rewrites (no logic change),
  and that `cargo clippy` = 0 and `cargo test` is green. Recorded as an explicit
  Build-phase gate; the commit uses exactly those two explicit paths.

## Confidence
proceed-with-caveats
