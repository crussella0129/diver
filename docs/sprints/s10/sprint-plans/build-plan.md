Finalized - DO NOT EDIT

# Sprint 10 Build Plan

## Intents
- [INT-0011](../../../intents/INT-0011-clippy-hygiene.md) — state: planned; acceptance criteria covered: AC1/AC2/AC3 (T-1001)

## Schema Tree
- Sprint Goal: clear the 7 pre-existing clippy warnings in `diver-core` (maintenance)
  - Hygiene (INT-0011)
    - T-1001: adopt the clippy fixes (store.rs + display.rs)

## Execution Sequence

### T-1001: Adopt the clippy fixes
- **Intent:** [INT-0011](../../../intents/INT-0011-clippy-hygiene.md)
- **Touches:** diver-core/src/store.rs, diver-core/src/display.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0011 AC1 (clippy zero warnings), AC2 (lint-only,
  two files), AC3 (no regression).
- **Success criterion (EARS):**
  - **WHEN** `cargo clippy --workspace --all-targets` runs after the change,
    **THEN** it **SHALL** report zero warnings.
  - **WHEN** `cargo test --workspace` runs, **THEN** all tests **SHALL** pass (no
    behavior change).
  - **WHEN** the diff is reviewed, **THEN** it **SHALL** be confined to `store.rs`
    and `display.rs` and be lint-only (no logic change).
- **Notes:** the fixes are already applied in the working tree by the background
  maintenance session — `store.rs` `list()` `redundant_closure`, and four
  `useless_vec` + two `uninlined_format_args` rewrites in the `display.rs` test
  module. T-1001 verifies (clippy 0 + full suite green + lint-only diff review)
  and commits the two files.
