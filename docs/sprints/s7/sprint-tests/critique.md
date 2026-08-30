# Test Critique — Sprint 7

## Concerns

### C-001: AC2/AC4 (the compile-time gate) are evidenced structurally, not by an executed test
- **Where:** `unit-tests.md` "AC2 / AC4 — structural evidence" / `INT-0008` AC2, AC4
- **Quote:** "No `trybuild` compile-fail test added — see plan critique C-001."
- **Failure mode:** intent-coverage
- **Why it matters:** AC4 — `Assertion<Supported>` is unconstructable outside
  `validate` — is the sprint's central promise, yet no executed test fails if a
  public `Supported` constructor is later added.
- **Suggested response:** defer-with-rationale — carried from the accepted plan
  decision. The guarantee is structural (private fields; `new`/`validate` only on
  `impl Assertion<Candidate>`; `validate` consumes a `Candidate`). The AC3 tests
  prove the gate logic and the `diver-cli` handler compiling against
  `validate().ok()` proves the external happy path. A `trybuild` ui test is the
  canonical negative proof but adds a brittle dev-dependency; deferred until the
  assertion API grows more public surface. Recorded, not silently dropped.

### C-002: happy-path `diver extract <stored-id>` is not run through the binary
- **Where:** `e2e-tests.md` "Coverage note"
- **Quote:** "a full binary run against a stored paper would require a pre-seeded
  real database and is not scripted here"
- **Failure mode:** e2e-cop-out
- **Why it matters:** the E2E smokes cover the CLI surface and the error path but
  not a real extract-and-display over a stored paper.
- **Suggested response:** defer-with-rationale — the exact
  extract→candidate→validate→supported flow the handler runs is covered
  deterministically by `test_extract_pipeline`, and `display_extract` is pure
  formatting. A binary happy-path run needs a seeded real DB (network ingest or a
  fixture DB), which would make the E2E stateful/flaky for no additional logic
  coverage. The surface + error path are covered; the pipeline is covered at the
  library level.

## Confidence
proceed-with-caveats
