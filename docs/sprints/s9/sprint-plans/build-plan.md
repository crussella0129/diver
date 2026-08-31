Finalized - DO NOT EDIT

# Sprint 9 Build Plan

## Intents
- [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) — state: planned; acceptance criteria covered: AC1/AC2/AC3/AC6-FK (T-901), AC4 (T-902), AC5/AC6 (T-903)

## Schema Tree
- Sprint Goal: persist supported assertions + their supporting observations
  - Storage layer (INT-0010)
    - T-901: schema + `save_assertions` (idempotent, validated-only)
    - T-902: `get_assertions` (retrieve display data)
  - Wiring (INT-0010)
    - T-903: `diver extract` persists + `diver assertions` command + display + docs

## Execution Sequence

Storage layer first (fully tested at the store level), then the CLI consumes it.

### T-901: Schema + `save_assertions`
- **Intent:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md)
- **Touches:** diver-core/src/store.rs (schema in `init_schema`, `StoredAssertion`, `save_assertions`)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0010 AC1 (tables created idempotently), AC2 (save
  persists claim + support, validated-only), AC3 (idempotent replace), AC6 (FK).
- **Success criterion (EARS):**
  - **WHEN** `Store::open` runs, **THEN** the `assertions` and `assertion_support`
    tables **SHALL** exist (`CREATE TABLE IF NOT EXISTS`).
  - **WHEN** `save_assertions(arxiv_id, version, &supported)` runs, **THEN** each
    assertion's `claim()` and every `support()` quote **SHALL** be persisted under
    that paper+version.
  - **WHEN** `save_assertions` runs again for the same `(paper, version)`, **THEN**
    the prior assertion rows and their support **SHALL** be replaced (no
    duplicates, no orphaned support rows).
- **Notes:** add the two tables to the `init_schema` batch (after
  `paper_versions_fts`); `assertion_support.assertion_id REFERENCES assertions(id)
  ON DELETE CASCADE`. `StoredAssertion { claim: String, version: String, support:
  Vec<String> }`. `save_assertions` mirrors `save()` (store.rs:121): `BEGIN` →
  `INSERT OR IGNORE INTO papers` + paper_id lookup → `DELETE FROM assertions WHERE
  paper_id=?1 AND version=?2` (cascade drops support) → per assertion `INSERT INTO
  assertions (paper_id, version, claim, created_at)` + `conn.last_insert_rowid()` →
  per `support()` observation `INSERT INTO assertion_support (assertion_id, quote)`
  with `obs.text()` → `COMMIT` (rollback on error). `created_at =
  chrono::Utc::now().to_rfc3339()`. The `&[Assertion<Supported>]` parameter is the
  storage gate. Tests query `conn` directly for row counts + FK.

### T-902: `get_assertions`
- **Intent:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md)
- **Touches:** diver-core/src/store.rs (`get_assertions`)
- **Depends on:** T-901
- **Acceptance criterion:** INT-0010 AC4 (returns stored data; empty for unknown).
- **Success criterion (EARS):**
  - **WHEN** `get_assertions(arxiv_id)` runs for a paper with stored assertions,
    **THEN** it **SHALL** return one `StoredAssertion` per stored assertion (claim,
    version, supporting quotes), newest first.
  - **WHEN** `get_assertions` runs for an unknown id or a paper with no assertions,
    **THEN** it **SHALL** return an empty vec.
- **Notes:** query `assertions` joined to `papers` by `paper_id` filtered on
  `arxiv_id`, `ORDER BY created_at DESC`; for each, a second query collects its
  `assertion_support.quote` rows. Group into `StoredAssertion`. Tests seed via
  `save_assertions`.

### T-903: CLI persist + `diver assertions` + display + docs + integration test
- **Intent:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md)
- **Touches:** diver-cli/src/main.rs (`Extract` persists; new `Assertions { arxiv_id }`),
  diver-core/src/display.rs (`display_stored_assertions`), README.md,
  diver-core/tests/persist_pipeline.rs (new)
- **Depends on:** T-901, T-902
- **Acceptance criterion:** INT-0010 AC5 (extract persists; `diver assertions`
  displays), AC6 (existing tests pass).
- **Success criterion (EARS):**
  - **WHEN** `diver extract <id>` finishes computing supported assertions (LLM or
    `--deterministic`), **THEN** it **SHALL** persist them via `save_assertions`
    before displaying.
  - **WHEN** `diver assertions <id>` runs, **THEN** it **SHALL** display the stored
    assertions for that paper; an unknown id **SHALL** print a clean "no
    assertions" message (not an error / non-zero exit).
- **Notes:** in the `Extract` handler, after building `supported`, call
  `store.save_assertions(&fact.arxiv_id, &fact.arxiv_version, &supported)?` before
  `display_extract`. New `Assertions { arxiv_id: String }` command →
  `store.get_assertions(&arxiv_id)?` → `display::display_stored_assertions(&arxiv_id,
  &stored)` (empty → "No stored assertions for {id}."). `display_stored_assertions`
  mirrors `display_extract` (owo-colors). Integration test composes save→extract→
  validate→save_assertions→get_assertions. README: document `diver assertions` and
  that `diver extract` now persists.
