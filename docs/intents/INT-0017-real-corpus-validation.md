# INT-0017 — Persist and validate a real dive corpus

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0017
- **State:** planned
- **Work evidence:** [Sprint 16 build plan](../sprints/s16/sprint-plans/build-plan.md) (T-1601, T-1602, T-1603)
- **Completion evidence:** (pending)
- **Code evidence:** [diver-cli/src/main.rs](../../diver-cli/src/main.rs)
- **Test evidence:** (pending)
- **Documentation evidence:** (pending)

## Intent

Prove the whole pipeline on real content and make building a real dive corpus a first-class,
reproducible workflow. A live probe (Sprint 16 research) confirmed ingest → `--deterministic`
extract → weighted `dive` works end-to-end on real arXiv papers, producing structural and
weighted co-assertion edges; the friction was that extraction is per-paper. This intent:

- **Batch extraction.** `diver extract --all [--deterministic]` extracts every stored paper
  (offline for `--deterministic`), so `collect` + `extract --all` builds a corpus in two
  commands instead of a manual per-id loop. `diver extract <id>` (single) is unchanged.
- **Real-corpus end-to-end test.** A committed real multi-paper arXiv **feed fixture** plus an
  **offline** integration test that parses+saves it, extracts each paper deterministically,
  and asserts the weighted dive graph over the corpus — at least one structural edge and at
  least one `CoAssertion { term, weight }` edge between two real papers.
- **Workflow docs.** README documents the `collect → extract --all → dive` corpus workflow.

Non-goals:
- No change to grounding, the `validate` gate, the provider substrate, or the co-assertion
  weighting (the observed common-term noise is the deferred INT-0014 TF/phrase follow-on).
- No live network in the automated test (uses a committed fixture, like `ingest_pipeline`).
- No new `DIVER_DB` store-path override (logged as backlog, not required here).
- Batch extraction over the **LLM** path is allowed but not exercised by the test (cost/key);
  the corpus/test path is `--deterministic`.

## Acceptance criteria

1. `diver extract --all` extracts every stored paper (iterating `Store::list`), honoring
   `--deterministic`; `diver extract <arxiv_id>` still extracts a single paper. With no stored
   papers, `--all` reports that clearly and exits 0.
2. A committed real multi-paper arXiv feed fixture exists under `diver-core/tests/fixtures/`.
3. An offline integration test ingests that fixture (parse → `save`), extracts each paper via
   the deterministic path (`extract_observations` → `candidate_assertions` → `validate` →
   `save_assertions`), builds the dive graph over the corpus, and asserts it contains at least
   one structural edge and at least one weighted `CoAssertion` edge between two distinct real
   papers.
4. The README documents the corpus-building workflow (`collect` → `extract --all` → `dive`).
5. All previously passing tests still pass.

## Rationale

The engine had strong unit/integration coverage of each stage but had never been proven on a
real multi-paper corpus, and there was no one-command way to extract a batch — the exact
things needed to actually use `dive` as a knowledge graph. Validating on real arXiv content
both exercises the pipeline against real vocabulary (surfacing e.g. co-assertion noise) and
locks a reproducible real-corpus regression test. Batch extraction removes the friction to
persisting a corpus.

## Alternatives

- **Live-network E2E test** — rejected: non-deterministic and offline-hostile; a committed
  captured feed fixture is reproducible and matches the existing pipeline tests.
- **`collect --extract` instead of `extract --all`** — considered; `extract --all` composes
  more cleanly (collect is ingest-only; extract-all is a separate, re-runnable step) and also
  covers papers ingested via `ingest`.
- **Fix co-assertion noise here** — deferred: it is the INT-0014 TF/phrase-weighting follow-on,
  a distinct outcome.

## Consequences

- `diver extract` gains an `--all` batch mode (single-id path unchanged); README gains the
  corpus workflow.
- A real feed fixture + offline E2E test guard the whole pipeline against regressions on real
  content.
- Real-world findings logged as backlog: co-assertion common-term noise (INT-0014 follow-on)
  and a `DIVER_DB` store-path override.

## Transition history
- 2026-09-02: created as `proposed` (after a live pipeline probe on real arXiv papers).
- 2026-09-02: `proposed` → `planned`; linked to Sprint 16 build plan (T-1601 `extract --all`,
  T-1602 real-feed fixture + offline E2E, T-1603 docs + backlog).
