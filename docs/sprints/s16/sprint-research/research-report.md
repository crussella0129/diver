# Sprint 16 Research Report

## Intents Reviewed
- [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) — created; relevance: primary; current state: proposed
- [INT-0004](../../../intents/INT-0004-batch-collection.md) — selected; relevance: `diver collect` batch-ingests; extraction is still per-paper; current state: realized
- [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) — selected; relevance: the weighted dive graph validated here; current state: realized

## 1. Sprint Goal

Shift from plumbing to content: make **persisting and validating a real dive corpus** a
first-class, reproducible workflow, and prove the whole pipeline (ingest → extract →
persist → dive) on real arXiv papers. Add **batch extraction** so building a corpus is one
command, commit a **real multi-paper arXiv feed fixture + an offline end-to-end test** that
exercises the weighted dive graph, and document the workflow. Advances **INT-0017**.
Baseline: `362b297`, `cargo test --workspace` green (129), clippy 0.

## 2. Live probe (this research actually ran the pipeline)

Network to `export.arxiv.org` works from this environment, so I exercised the real pipeline:

- `diver ingest 1706.03762` → **Ingested: Attention Is All You Need**.
- `diver extract 1706.03762 --deterministic` → 5 grounded claims (the offline
  sentence-splitter: each abstract sentence becomes a claim whose quote is that sentence, so
  it always grounds — no API key needed).
- `diver collect "attention transformer neural machine translation" --max-results 6` →
  6 more real papers ingested in one call.
- Extracting each `--deterministic` (a **manual loop** — see the gap below) then
  `diver dive attention` produced a real weighted graph:
  - structural edges: `shared category cs.CL`, `shared category cs.LG`;
  - weighted co-assertion edges: `co-asserts decoder (w=0.68)`, `co-asserts convolutional
    (w=0.68)`, `co-asserts task (w=1.00)`, `co-asserts large (w=1.00)`, with the display cap
    summarizing the overflow (`(+31 more)`).

**The engine works end-to-end on real content.** The 7-paper corpus is now persisted in the
user's `diver.db` — the objective, fulfilled operationally.

### Real-world findings (surfaced by the probe)
1. **No batch extraction.** `collect` batch-ingests, but `extract` is per-`arxiv_id`, so
   building a corpus means a manual shell loop. This is the main friction to "extract a real
   batch" — the enabling feature this sprint should add.
2. **Co-assertion noise on common terms.** `large` and `task` score `w=1.00` (df==2) and link
   papers; these are the ubiquitous-word links the deferred TF/phrase weighting (INT-0014
   follow-on) targets. Observed, out of scope here — logged.
3. **Fixed DB path.** `Store::open()` uses `dirs::data_dir()/diver/diver.db` with no override,
   so probing/tests write the user's real DB. A `DIVER_DB` override would make corpora
   reproducible and keep tests/front-ends off the real DB (candidate deliverable / backlog).
4. **Deterministic claims are abstract sentences** (grounded trivially). Good enough to
   exercise the graph on real vocabulary; real *LLM* extraction (grounding quality) still needs
   a key and stays a manual check.

## 3. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-cli/src/main.rs | high | `Extract { arxiv_id, deterministic }` handler — extend for batch (e.g. `--all`, extract every stored paper). `Collect` already batch-ingests; `Dive` renders the graph. |
| diver-core/src/store.rs | high | `Store::list()` gives all stored papers to iterate for batch extraction; `save_assertions` persists; `Store::open()` fixed path (finding 3). |
| diver-core/src/assertion.rs, observation.rs | med | `extract_observations` + `candidate_assertions` = the deterministic path used for the offline corpus. |
| diver-core/tests/*_pipeline.rs, tests/fixtures/*.xml | high | Existing fixture-driven pipeline tests + the `parse` path show how to ingest a committed feed XML offline. The new E2E test commits a real multi-paper feed fixture and runs ingest→deterministic-extract→dive over it. |
| diver-core/src/graph.rs | med | `compute_relations` + `compute_coassertion_relations` + `build_dive` — the graph the E2E test asserts on real data. |

Baseline: workspace at `362b297`. green (129); clippy 0.

### Design sketch
- **Batch extraction:** `diver extract --all [--deterministic]` iterates `store.list()`,
  extracts each (offline for `--deterministic`), and persists — `collect` + `extract --all`
  builds a corpus in two commands. (LLM batch is possible but costs per paper; the corpus/test
  path uses `--deterministic`.)
- **Real-corpus E2E test:** commit a captured real arXiv **multi-paper feed fixture**
  (`tests/fixtures/`), plus a test that parses+saves it, extracts each paper deterministically,
  and asserts the weighted dive graph over the corpus — structural edges AND at least one
  `CoAssertion { term, weight }` edge between two real papers. Fully offline/reproducible.
- **Docs:** README documents the `collect → extract --all → dive` corpus workflow.

## 4. External Sources
- Live `export.arxiv.org` Atom API (probed directly) — the real feed for the fixture and the
  ingest/collect paths; confirmed reachable and parsed correctly by the existing pipeline.
- [arXiv API user manual](https://info.arxiv.org/help/api/user-manual.html) — `id_list` /
  `search_query` response shape (already handled by `diver`'s parser).

## 5. Risks / Unknowns / Dependencies
- **No network in tests.** The E2E test must be offline → it ingests a **committed** feed
  fixture (captured once from the live API), not a live call. Matches the existing
  `ingest_pipeline` fixture pattern.
- **Batch `extract --all` scope.** Keep it to the local store (extract papers already ingested)
  so it composes with `collect` and needs no new network path; `--deterministic` keeps the
  corpus build free/offline.
- **Real-DB pollution.** Batch extraction and the probe write the user's real `diver.db`. The
  E2E test uses `Store::open_in_memory()` (no real-DB writes). A `DIVER_DB` override is a
  candidate deliverable to fully decouple, else backlog.
- **Co-assertion noise** (finding 2) is real but a documented INT-0014 follow-on, not this
  sprint's fix.
- No new dependencies.

## 6. Recommended Approach

Add `diver extract --all [--deterministic]` (batch over the local store) to make "extract a
real batch" one command. Commit a real multi-paper arXiv feed fixture and an offline E2E test
that runs ingest→deterministic-extract→dive over it and asserts the weighted graph (structural
+ ≥1 weighted co-assertion edge) on real content. Document the `collect → extract --all → dive`
workflow. Log the real-world findings (co-assertion noise, `DIVER_DB` override) as backlog.

### Referenced artifacts
- [INT-0017 chapter](../../../intents/INT-0017-real-corpus-validation.md)
- Build/test plans: `../sprint-plans/`
- Baseline evidence: `cargo test --workspace` 129/129, clippy 0 at `362b297`
