# Test Critique — Sprint 13

## Concerns

### C-001: "structural edges remain ungated by temperature" (AC4) is not directly asserted
- **Where:** `integration-tests.md` `test_coassertion_temperature_pipeline` / `INT-0014` AC4
- **Quote:** "Structural (category/author) edges are unaffected"
- **Failure mode:** intent-coverage
- **Why it matters:** the pipeline test seeds papers with distinct categories/authors
  and passes only the co-assertion relation set to `build_dive`, so no test exercises
  a category/author edge surviving a low temperature. A reader wanting proof that the
  dial cannot suppress structural edges will not find a direct assertion.
- **Suggested response:** defer-with-rationale — the separation is guaranteed by the
  type signature: `compute_relations` takes no `temperature` parameter, and the `dive`
  handler always unions `compute_relations(&facts)` before extending with the
  temperature-gated co-assertion edges. A direct test would restate the type system.
  The existing `compute_relations` unit tests already assert category/author edges
  unconditionally. Matches the plan critique's C-002 deferral.

### C-002: the `--temperature` default (0.5) has no library/unit assertion
- **Where:** `unit-tests.md` T-1302 / `INT-0014` AC4
- **Quote:** "WHEN `--temperature` is omitted, THEN the handler SHALL use `0.5`"
- **Failure mode:** EARS-coverage
- **Why it matters:** `test_parse_temperature` covers the parser, not the clap
  `default_value_t = 0.5`, so a regression that changed the default would not fail a
  unit test.
- **Suggested response:** defer-with-rationale — the default is a declarative clap
  attribute and is verified by executed E2E evidence: `diver dive --help` prints
  `[default: 0.5]` (recorded in `e2e-tests.md`). A binary-invocation harness asserting
  the applied default does not exist yet (the same populated-DB E2E deferral as
  INT-0012/INT-0013). Changing the default is a conscious one-token edit.

## Confidence
proceed-with-caveats
