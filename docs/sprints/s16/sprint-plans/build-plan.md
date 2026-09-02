Finalized - DO NOT EDIT

# Sprint 16 Build Plan

## Intents
- [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) — state: planned; acceptance criteria covered: AC1, AC2, AC3, AC4, AC5

## Schema Tree
- Sprint Goal: persist and validate a real dive corpus
  - Batch extraction (diver-cli)
    - T-1601: `diver extract --all`
  - Real-corpus validation (diver-core)
    - T-1602: real-feed fixture + offline end-to-end test
  - Docs
    - T-1603: corpus workflow + backlog

## Execution Sequence

### T-1601: `diver extract --all` batch mode
- **Intent:** [INT-0017](../../../intents/INT-0017-real-corpus-validation.md)
- **Touches:** `diver-cli/src/main.rs`
- **Depends on:** (none)
- **Acceptance criterion:** AC1 (batch + single extract)
- **Success criterion (EARS):**
  - **WHEN** `diver extract --all` runs, **THEN** it **SHALL** extract every stored paper (iterating `Store::list`), honoring `--deterministic`, printing a per-paper result and a summary.
  - **WHEN** `diver extract <arxiv_id>` runs, **THEN** it **SHALL** extract that single paper (unchanged), erroring "Paper not found" for an unknown id.
  - **WHEN** `--all` runs with no stored papers, **THEN** it **SHALL** print a clear message and exit 0; **WHEN** neither an id nor `--all` is given, **THEN** clap **SHALL** reject the invocation.
- **Notes:** `arxiv_id: Option<String>` (`required_unless_present = "all"`); `all: bool` (`conflicts_with = "arxiv_id"`). Factor the current single-paper body into an async `extract_and_save(&store, &fact, deterministic)` helper reused by both the single and `--all` paths. Args are clap-enforced; the extract-each-paper logic is exercised at library level by T-1602.

### T-1602: real-feed fixture + offline end-to-end test
- **Intent:** [INT-0017](../../../intents/INT-0017-real-corpus-validation.md)
- **Touches:** `diver-core/tests/fixtures/real_corpus_feed.xml` (new, captured), `diver-core/tests/real_corpus.rs` (new)
- **Depends on:** (none)
- **Acceptance criterion:** AC2 (fixture), AC3 (E2E graph over real content)
- **Success criterion (EARS):**
  - **WHEN** the real-feed fixture is ingested (`parse::parse_feed` → `SourceFact::from_paper` → `Store::save`) and each paper is extracted deterministically (`extract_observations` → `candidate_assertions` → `validate` → `save_assertions`), **THEN** the dive graph over the corpus (`compute_relations` + `compute_coassertion_relations(_, 0.5)`) **SHALL** contain at least one structural edge **AND** at least one weighted `CoAssertion { term, weight }` edge between two distinct real papers.
- **Notes:** Capture the fixture during build by fetching the live arXiv API for a multi-paper attention/NMT query (the probe showed those abstracts share significant terms → co-assertion edges) and committing the raw XML. The test is fully offline (`Store::open_in_memory`, reads the committed fixture). Assert on edge kinds/counts, not exact strings, for robustness.

### T-1603: docs — corpus workflow + backlog
- **Intent:** [INT-0017](../../../intents/INT-0017-real-corpus-validation.md)
- **Touches:** `README.md`, `docs/work/tasks.md`
- **Depends on:** T-1601, T-1602
- **Acceptance criterion:** AC4 (docs)
- **Success criterion (EARS):**
  - **WHEN** the README documents `dive`, **THEN** it **SHALL** show the corpus workflow `collect <query>` → `extract --all --deterministic` → `dive <concept>`.
- **Notes:** Add the workflow to the README extraction/dive section. Log the probe's real-world findings as `(backlog)` tasks: co-assertion common-term noise (INT-0014 TF/phrase follow-on) and a `DIVER_DB` store-path override.
