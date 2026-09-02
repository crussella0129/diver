# Sprint 18 Integration Tests

- **Tested head:** `f651bb6d9353dda127bbfcfc223300ccf719225a`
- **Runner:** `cargo test --workspace`
- **Result:** all integration binaries green; **142 tests total** across the workspace
  (132 baseline + 10 new).

## Scope note: Sprint 18 adds no integration-only test

Said plainly, because the previous draft of this file obscured it (test critic C-004).
The one result cited below, `test_open_at_round_trip_persists`, is **also** counted among
the eight new unit tests in [unit-tests.md](unit-tests.md), and the only other new binary
(`diver-cli tests/db_override.rs`) is E2E, covered in [e2e-tests.md](e2e-tests.md). So the
142 total reconciles as 132 baseline + 8 new unit + 2 new E2E — the round-trip test appears
under two headings because it satisfies both roles, not because it is counted twice.

The locked test plan pre-authorized exactly this: "No separate integration file is added —
the surrounding store tests live in `diver-core/src/store.rs`, and splitting this one out
would break that convention for no gain." The reasoning holds: the test genuinely crosses a
drop-and-reopen boundary over a real on-disk SQLite file, which is integration behaviour by
substance even though it lives in a unit-test module by convention.

## Store persistence integration (INT-0019)

- **Intent:** [INT-0019](../../../intents/INT-0019-configurable-store-path.md) — AC4
- `store::tests::test_open_at_round_trip_persists` doubles as the integration check. It
  composes `open_at` → `save` → `save_assertions` → *drop* → `open_at` → `get` →
  `get_assertions` across a real on-disk SQLite file and a store lifecycle boundary —
  the one thing `open_in_memory()` structurally cannot exercise, since it discards
  everything on drop. **pass**
- No separate integration file was added: the surrounding store tests live in
  `diver-core/src/store.rs`'s `mod tests`, and splitting this one out would break that
  convention for no gain.
- This is the test that makes durable fixture corpora possible, which is the reason
  INT-0019 exists at all and the dependency
  [INT-0022](../../../intents/INT-0022-relation-evaluation-harness.md) is waiting on.

## Pre-existing integration binaries (regression guard)

All green and unchanged; none required modification for this sprint. The new
`diver-cli tests/db_override.rs` binary is deliberately **not** listed here — it is new,
and it is E2E; see [e2e-tests.md](e2e-tests.md).

| Binary | Result |
|--------|--------|
| `tests/coassertion.rs` | 2 passed |
| `tests/dive_graph.rs` | 1 passed |
| `tests/dive_pipeline.rs` | 1 passed |
| `tests/extract_pipeline.rs` | 1 passed |
| `tests/ingest_pipeline.rs` | 2 passed |
| `tests/llm_extract_pipeline.rs` | 1 passed |
| `tests/persist_pipeline.rs` | 1 passed |
| `tests/real_corpus.rs` | 1 passed |

## Raw result
```
cargo test --workspace
  diver-cli bin                 1 passed
  diver-cli db_override         2 passed
  diver_core lib              129 passed
  coassertion                   2 passed
  dive_graph                    1 passed
  dive_pipeline                 1 passed
  extract_pipeline              1 passed
  ingest_pipeline               2 passed
  llm_extract_pipeline          1 passed
  persist_pipeline              1 passed
  real_corpus                   1 passed
  doc-tests                     0 passed
                              ------------
                              142 passed; 0 failed
```
