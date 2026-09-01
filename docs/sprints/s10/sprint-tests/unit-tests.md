# Sprint 10 Unit Tests

- **Tested head:** `084e652ca1332ada3746ecfbe614f555b976cc0c`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets`
- **Result:** `diver_core` lib unittests — **88 passed; 0 failed**; `diver-cli` bin
  — 0 tests.

## No new tests (lint-only maintenance)
This sprint (INT-0011) makes no behavior change — the fixes are mechanical,
semantics-preserving `clippy` rewrites. No new unit tests are warranted; the
existing suite is the regression guard (AC3). The touched code is covered by:
- `store.rs::list()` (the inlined `row_to_fact` closure) — exercised by
  `test_store_list`.
- `display.rs` test rewrites (`vec!` → array, `writeln!` arg inlining) — the
  affected tests are themselves the display tests, all still passing.

## Lint verification (AC1)
`cargo clippy --workspace --all-targets` → **0 warnings** (down from 7). This is
the primary acceptance gate for the sprint.

## Raw result
```
cargo clippy --workspace --all-targets  →  0 warnings
Running unittests src\lib.rs (diver_core)  →  88 passed; 0 failed
```
