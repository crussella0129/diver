# Sprint 19 Test Report

## Intent Verification

| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|--------------|--------|------------------------|
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC1: persisted concept with stable id, label, surface forms; claims linked, not substring-matched | T-1902/4, 11; T-1903/1–6 / `test_concept_formation`, `test_concept_resolution`, `test_papers_asserting_wildcards_inert`, `test_papers_asserting_matches` (unmodified) | pass | Test evidence links this report |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC2: resolution reaches other surface forms; unresolved terms name containing concepts | T-1902/14; T-1903/2, 4, 10; T-1905/1–4 / `test_query_key`, `test_concept_navigation`, `test_concepts_pipeline`, `test_real_corpus_concepts`, `test_cli_dive_concepts`, `test_cli_dive_empty_and_help`; drive | pass (fallback ordering deviates from T-1903/10 by design — see unit-tests.md) | Test evidence links this report |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC3: byte-deterministic formation | T-1902/12 / `test_formation_invariants`, `test_real_corpus_concepts` | pass | Test evidence links this report |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC4: multi-word concepts distinct from their words | T-1902/5–7, 9 / `test_phrase_rule`, `test_concept_formation`, `test_real_corpus_concepts` | pass | Test evidence links this report |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC5: co-assertion names a concept; temperature semantics and tests hold | T-1904/1–5 / existing co-assertion tests unmodified, `test_coassertion_concepts`, `test_coassertion_monotonic_with_phrases`, `test_coassertion_matches_naive_reference`, `test_touching_variants_match_full` | pass (`t = 1.0` refined by subsumption, recorded in the chapter) | Test evidence links this report |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC6: traceable to claims and papers | T-1902/13; T-1903/3 / `test_formation_invariants`, `test_concept_resolution` | pass | Test evidence links this report |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC7: never stale | T-1903/7–8; T-1905/5 / `test_concepts_freshness`, `test_concepts_pipeline`, `test_cli_dive_concepts`; drive (live stoplist edit, formation-version bump, concurrent rebuild) | pass | Test evidence links this report |

## Summary

- **Method:** operate first, test after (user direction). The real binary was driven
  against real arXiv corpora of 480 and 2,068 papers in an isolated proving ground; six
  walls were found and repaired; the official tests were written afterwards and kept lean.
  Drive log: [e2e-tests.md](e2e-tests.md).
- **Unit:** `diver_core` lib 146 passed (17 new this sprint) + `diver-cli` bin 1.
- **Integration:** `tests/concepts.rs` 2 (new); every pre-existing binary unmodified and green.
- **E2E:** proving-ground drive (primary) + `diver-cli/tests/dive_concepts.rs` 2 (new).
- **Total:** **163 passed; 0 failed.** Clippy 0 (`-D warnings`); fmt clean.
- **Mutation checks:** nine mutations, each caught by its named test (see unit-tests.md).
- **Performance:** `dive` at 2,068 papers 0.58–1.46 s (was 3.5–12.3 s).

## CI Confirmation

- **Head SHA:** `d08eaa61cbccb32a468e72b06b3588874a638637`
- **CI run:** pending — `.github/workflows/rust.yml` (build + test) runs on pull requests to
  `main`, not on `dev`; its conclusion is recorded when the sprint checkpoint PR runs.
- **Conclusion:** success (local)
- **Confirmations:** `cargo test --workspace --no-fail-fast` → 163 passed, 0 failed across 14
  binaries; `cargo clippy --workspace --all-targets -- -D warnings` → 0; `cargo fmt --check`
  → clean. Records: [unit](unit-tests.md), [integration](integration-tests.md),
  [e2e / drive](e2e-tests.md), [documentation review](documentation-review.md),
  [critique](critique.md).

## Failures

(none)

## Technical Debt Identified

- **The claim layer is sentence splitting.** With no LLM key, "claims" are abstract
  sentences, including non-claims and LaTeX fragments. Concepts are robust to it; the
  epistemic layer (typed relations, INT-0021) cannot be built on it.
- **Generic vocabulary is reduced, not removed**, and **word concepts are polysemous**
  (`code` spans genetic, error-correcting and source code). Phrases carry most of the
  signal. Both recorded in INT-0020.
- **Known fold limits**: `bias`/`biases` separate; `news` folds to common `new`.
- **Deviations from locked clauses** (T-1902/3, T-1903/10, T-1905/3) are recorded in
  unit-tests.md and INT-0020 rather than rewritten into the locked plan.
- **arXiv politeness across processes**: the 3 s throttle is per process; back-to-back
  `collect` runs are not spaced.
- **`CoAssertion.term`** now carries a concept label; the rename is deferred to the backlog.
- **CI** covers build and test only, and only on `main`-bound PRs.
