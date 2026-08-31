# INT-0010 — Persist the epistemic layer

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0010
- **State:** planned
- **Work evidence:** [Sprint 9 build plan](../sprints/s9/sprint-plans/build-plan.md) (T-901, T-902, T-903)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Give the extracted knowledge from [[llm-claim-extractor]] (INT-0009) durable,
queryable storage. Today `diver extract` computes supported assertions and then
**discards them** — every run re-hits the LLM and nothing accumulates. Extend the
SQLite store so extractions persist:

- New `assertions` table (claim + provenance to paper + version + timestamp) and
  `assertion_support` table (the supporting observation quotes), created
  idempotently on open.
- `Store::save_assertions(arxiv_id, version, &[Assertion<Supported>])` and
  `Store::get_assertions(arxiv_id) -> Vec<StoredAssertion>`.
- `diver extract <id>` persists the supported assertions it produces (both LLM and
  `--deterministic` paths); a new `diver assertions <id>` command reads them back.

The storage method accepts **only** `Assertion<Supported>`, so the typestate gate
extends to persistence: the database can only ever hold **validated** knowledge —
unvalidated content is un-persistable at compile time.

Non-goals:
- No graph / `ComputedRelation` / `diver dive` — persisted assertions are the input
  a later graph sprint consumes.
- No cross-paper observation identity — each assertion's support is stored with its
  own paper; a normalized shared-`observations` table is a graph-sprint concern.
- No change to extraction or the `validate` gate.
- No new dependencies (`rusqlite` already present).
- No migration tooling beyond `CREATE TABLE IF NOT EXISTS` (consistent with the
  project's pre-1.0 local-DB stance).

## Acceptance criteria

1. The store schema gains `assertions(id, paper_id → papers, version, claim,
   created_at)` and `assertion_support(id, assertion_id → assertions ON DELETE
   CASCADE, quote)`, created idempotently on `Store::open`.
2. `Store::save_assertions(arxiv_id, version, &[Assertion<Supported>])` persists
   each supported assertion's claim and its supporting observation quotes; the
   parameter type is `&[Assertion<Supported>]`, so unvalidated content cannot be
   stored.
3. Saving is idempotent per `(paper, version)`: re-saving replaces the prior
   assertion set for that paper+version (old assertion rows and their support are
   deleted, not duplicated).
4. `Store::get_assertions(arxiv_id)` returns the stored assertions (claim,
   version, supporting quotes) for a paper; an unknown id or a paper with none →
   empty vec.
5. `diver extract <arxiv_id>` persists the supported assertions it produces (LLM
   and `--deterministic` paths); `diver assertions <arxiv_id>` displays the stored
   assertions; an unknown id is handled cleanly.
6. The `paper_versions → papers` and `assertion_support → assertions` foreign-key
   constraints are enforced; all previously passing tests still pass.

## Rationale

`diver extract` produced validated knowledge and threw it away. Persisting it
makes the epistemic layer durable and queryable, and gives the future graph layer
persisted nodes to traverse. Storing only `Assertion<Supported>` extends the
compile-time gate to the storage boundary: the database holds only validated
knowledge, so downstream consumers of the store inherit that guarantee.

## Alternatives

- **Persist candidates too** — rejected: only validated assertions are admitted
  knowledge; candidates are transient.
- **A normalized shared `observations` table (cross-paper identity)** — deferred:
  v1 stores each assertion's support inline (quotes) keyed to the assertion's
  paper+version; cross-paper observation identity is a graph-sprint concern.
- **Append (accumulate) re-extractions** — rejected: LLM extraction is
  non-deterministic, so accumulating would duplicate/conflict; idempotent replace
  per paper+version is cleaner and matches the paper-version FTS-refresh pattern.
- **A `--save` flag instead of auto-save** — rejected: durability is the whole
  point; an extraction with a discarded result is exactly the old behavior.

## Consequences

- The store schema grows two tables; existing databases gain them on next open
  (`CREATE TABLE IF NOT EXISTS`) — no data migration.
- `diver extract` now writes to the DB (previously read-only for extraction);
  re-extracting a paper replaces its stored assertions.
- `Store::get_assertions` returns display-oriented `StoredAssertion` data this
  sprint; reconstructing the `Assertion<Supported>` typestate on load (for the
  graph builder) is a later concern — the `&[Assertion<Supported>]` save signature
  already guarantees what was stored was validated.

## Transition history
- 2026-08-31: created as `proposed`.
- 2026-08-31: `proposed` → `planned`; linked to Sprint 9 build plan (T-901 schema
  + `save_assertions`, T-902 `get_assertions`, T-903 CLI persist + `diver
  assertions`).
