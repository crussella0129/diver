# INT-0011 — Resolve pre-existing clippy warnings (maintenance)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0011
- **State:** realized
- **Work evidence:** [Sprint 10 build plan](../sprints/s10/sprint-plans/build-plan.md) (T-1001)
- **Completion evidence:** [T-1001 completion](../work/completed-tasks.md#t-1001--sprint-10)
- **Code evidence:** [diver-core/src/store.rs](../../diver-core/src/store.rs), [diver-core/src/display.rs](../../diver-core/src/display.rs)
- **Test evidence:** [Sprint 10 test report](../sprints/s10/sprint-tests/test-report.md)
- **Documentation evidence:** none

## Intent

Clear the 7 pre-existing `clippy` warnings that accumulated in `diver-core` and
were flagged as out-of-scope technical debt across Sprints 6–9. These are
lint-only cleanups with **no behavior change**:

- `diver-core/src/store.rs` — `list()`'s `query_map([], |row| row_to_fact(row))`
  → `query_map([], row_to_fact)` (redundant closure).
- `diver-core/src/display.rs` (tests) — `useless_vec` (`vec![…]` → `[…]`) in four
  display tests, and uninlined/empty format-string args in two `writeln!` calls.

The fixes were applied in a background maintenance session (spawned from the
tracking chip created in Sprint 6) and are formalized here so the Book records
them as deliberate maintenance rather than incidental churn.

Non-goals:
- No behavior change, no new features, no dependency change — purely lint hygiene.
- No new tests (the existing suite is the regression guard).

## Acceptance criteria

1. `cargo clippy --workspace --all-targets` reports **zero** warnings.
2. All changes are lint-only (no logic change) and confined to
   `diver-core/src/store.rs` and `diver-core/src/display.rs`.
3. `cargo test --workspace` stays green (no regression).

## Rationale

The warnings were repeatedly flagged as debt (Sprints 6–9) and deliberately left
out of feature sprints to keep each task commit a coherent, single-concern diff.
A dedicated maintenance sprint clears them in one coherent change without
polluting a feature sprint's history, and restores clippy output to signal
(new warnings are visible) rather than noise.

## Alternatives

- **Fix inline during a feature sprint** — rejected across Sprints 6–9: it pulls
  unrelated files into a feature commit and breaks task-commit coherence.
- **Leave the warnings** — rejected: standing warnings mask newly introduced ones.

## Consequences

- `diver-core` is clippy-clean; subsequent clippy warnings are actionable signal.
- Establishes the pattern of periodic maintenance sprints for accumulated lint
  debt.

## Transition history
- 2026-08-31: created as `proposed`.
- 2026-08-31: `proposed` → `planned`; linked to Sprint 10 build plan (T-1001).
- 2026-08-31: `planned` → `active` (Sprint 10 build; T-1001).
- 2026-08-31: `active` → `realized` (Sprint 10: all 7 clippy warnings cleared —
  `cargo clippy --workspace --all-targets` = 0; lint-only, 94 tests still green).
