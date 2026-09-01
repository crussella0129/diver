# Sprint 10 Research Report

## Intents Reviewed
- [INT-0011](../../../intents/INT-0011-clippy-hygiene.md) — created; relevance: primary; current state: proposed

## 1. Sprint Goal

A maintenance sprint: clear the 7 pre-existing `clippy` warnings in `diver-core`
that were flagged as out-of-scope debt across Sprints 6–9 and tracked via a
background task chip. Lint-only, no behavior change. The fixes are already applied
in the working tree (from the background maintenance session); this sprint
formalizes and records them.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/store.rs | high | One lint: `list()` (~line 335) `query_map([], \|row\| row_to_fact(row))` → `query_map([], row_to_fact)` (`redundant_closure`). |
| diver-core/src/display.rs | high | Six lints in the `#[cfg(test)]` module: four `useless_vec` (`vec![…]` → `[…]`) and two `writeln!` uninlined/empty format-string args (`clippy::uninlined_format_args`). |

Baseline: workspace at `4a0011e`. `cargo clippy --workspace --all-targets`
currently reports **0 warnings** with the working-tree fixes applied (down from
7); `cargo test --workspace` is green (94 tests).

## 3. External Sources
- [clippy: redundant_closure](https://rust-lang.github.io/rust-clippy/master/#redundant_closure) — pass the function directly instead of wrapping it in a closure.
- [clippy: useless_vec](https://rust-lang.github.io/rust-clippy/master/#useless_vec) — use an array where a `Vec` is not needed.
- [clippy: uninlined_format_args](https://rust-lang.github.io/rust-clippy/master/#uninlined_format_args) — inline captured identifiers into the format string.

## 4. Risks, Unknowns, Dependencies

- **Risk:** a "lint fix" silently changes behavior. Mitigation: all three lints
  are mechanical, semantics-preserving rewrites; the full test suite (94) is the
  regression guard and stays green.
- **Risk:** the fixes were produced by a separate session. Mitigation: they are
  confined to the two named files, verified clippy-clean and test-green before
  adoption, and reviewed as lint-only diffs.
- **Dependency:** none. No code behavior, dependency, or schema change.

## 5. Recommended Approach

Primary: adopt the working-tree fixes as a single maintenance task and verify.
- Confirm the diff is confined to `store.rs` (the `list()` closure) and
  `display.rs` (test `vec!`/`writeln!` rewrites), lint-only.
- Verify `cargo clippy --workspace --all-targets` = 0 warnings and `cargo test
  --workspace` green.
- Commit as the sprint's single build task.

Alternative considered: re-derive the fixes from scratch — unnecessary; the
working-tree changes are already correct and verified.

## Artifacts
- No standalone snippet files; the changes are the two-file working-tree diff.
