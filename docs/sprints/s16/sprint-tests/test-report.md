# Sprint 16 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC1: `extract --all` batch + single unchanged | T-1601 / clap-enforced args + `test_real_corpus_dive` (extract loop at library level) | pass (CLI handler manually verified, critique C-001) | Test evidence links this report |
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC2: real multi-paper feed fixture | T-1602 / committed `real_corpus_feed.xml` (7 papers) | pass (reconstructed from real ingested content, critique C-002) | Test evidence links this report |
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC3: E2E weighted graph on real content | T-1602 / `test_real_corpus_dive` | pass | Test evidence links this report |
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC4: docs | T-1603 / README corpus workflow | pass | Test evidence links this report |
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC5: no regression | all | pass (130/130) | Test evidence links this report |

## Summary
- Unit tests: 119 passed (`diver_core` lib) + 1 passed (`diver-cli` bin); no new pure units
- Integration/E2E: `real_corpus` 1 new (offline real-content pipeline) + 9 existing, all pass
- E2E: `test_real_corpus_dive` (offline) + live research probe (`ingest`/`collect`/`extract
  --all`/`dive`) + manual arg checks
- Clippy: 0 warnings
- CI status: not-configured

## CI Confirmation
- **Head SHA:** `bcf520f5204158e45125c89db0950df7d4d5acab`
- **CI run:** N/A
- **Conclusion:** success (local)
- **Confirmations:** CI not configured — local only. `cargo test --workspace` →
  `test result: ok` for every binary (`diver_core` lib 119, `diver-cli` bin 1, `coassertion` 2,
  `dive_graph` 1, `dive_pipeline` 1, `extract_pipeline` 1, `ingest_pipeline` 2,
  `llm_extract_pipeline` 1, `persist_pipeline` 1, `real_corpus` 1) = 130 total. `cargo build`
  clean; `cargo clippy --workspace --all-targets` → 0. Records: [unit](unit-tests.md),
  [integration](integration-tests.md), [e2e](e2e-tests.md).

## Failures
(none)

## Technical Debt Identified
- `diver extract --all` handler verified manually (thin loop over the tested `extract_and_save`);
  no populated-DB binary E2E — critique C-001.
- Fixture reconstructed from real ingested content (arXiv rate-limited raw captures); provenance
  documented — critique C-002.
- Real-world findings logged as backlog: co-assertion common-term noise (T-1610, INT-0014
  follow-on) and a `DIVER_DB` store-path override (T-1611).
- Real **LLM** extraction quality on real abstracts still a manual, keyed check.

## Coverage Observations
- Every acceptance criterion has a named check; the headline AC (E2E weighted graph on real
  content) is a robust, offline, deterministic test asserting edge kinds/counts/invariants.
- `test_real_corpus_dive` proves the whole engine on genuine arXiv papers and guards it against
  regressions; the live probe additionally confirmed the on-network flow end-to-end.
