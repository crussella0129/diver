# Sprint 16 Unit Tests

- **Tested head:** `bcf520f5204158e45125c89db0950df7d4d5acab`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets`
- **Result:** `diver_core` lib — **119 passed; 0 failed**; `diver-cli` bin — 1 passed. Clippy: 0 warnings.

## New (INT-0017)

No new pure-unit tests this sprint. The two behavioral additions are covered as follows:

- **T-1601 `diver extract --all`:** its argument contract is enforced declaratively by clap
  (`arxiv_id: Option<String>` with `required_unless_present = "all"`; `all: bool` with
  `conflicts_with = "arxiv_id"`), verified manually (`extract --help`, `extract` with no args
  rejected, `extract --all --deterministic` run over the live 7-paper corpus → "Extracted 7
  paper(s)."). Its per-paper `extract_and_save` deterministic path is exercised over a whole
  real corpus by the integration test below.

All 119 prior lib unit tests remain green (no regressions from the `Extract` command change).

## Raw result
```
cargo clippy --workspace --all-targets  →  0 warnings
Running unittests src\lib.rs (diver_core)  →  119 passed; 0 failed
Running unittests src\main.rs (diver-cli)  →    1 passed; 0 failed
```
