# Test Critique — Sprint 9

## Concerns

### C-001: the binary persist-then-read round-trip is not E2E-tested
- **Where:** `e2e-tests.md` "Coverage note"; `INT-0010` AC5
- **Quote:** "A full binary round-trip … would need a seeded real database and is
  not scripted here"
- **Failure mode:** e2e-cop-out
- **Why it matters:** the E2E smokes cover the CLI surface and the empty path, but
  no test runs `diver extract <seeded-id> --deterministic` then `diver assertions
  <id>` through the binary to prove the on-disk persist→read loop.
- **Suggested response:** defer-with-rationale — the exact
  `save`→`extract`→`validate`→`save_assertions`→`get_assertions` loop is covered
  deterministically at the library level by `test_persist_pipeline`, and the
  `Extract` handler's single added line (`store.save_assertions(...)?`) is over
  that tested API. A binary round-trip needs a seeded real DB (network ingest or a
  fixture DB), which would make the E2E stateful/flaky for no additional logic
  coverage. Surface + empty path (binary) + library round-trip is sufficient; a
  fixture-DB binary test is possible future hardening.

### C-002: `save_assertions` uses one `created_at` for the whole batch — order within a save relies on `id`
- **Where:** `unit-tests.md` T-901; `store.rs` `save_assertions` / `get_assertions`
- **Quote:** "`ORDER BY a.created_at DESC, a.id DESC`"
- **Failure mode:** weak-assertion (ordering within a single save is by `id`, not time)
- **Why it matters:** all assertions from one `save_assertions` call share a
  `created_at`, so "newest first" only distinguishes *between* saves; within a save
  the tiebreak is `id DESC`.
- **Suggested response:** defer-with-rationale — this is intentional and correct:
  a single extraction is one logical event, so its assertions sharing a timestamp
  is accurate. The `id DESC` tiebreak gives a stable, deterministic order (verified
  by `test_get_assertions_round_trip`, which checks membership rather than a
  brittle positional order). Cross-save recency (the meaningful axis) is preserved
  by `created_at DESC`.

## Confidence
proceed-with-caveats
