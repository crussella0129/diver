# Plan Critique — Sprint 13

## Concerns

### C-001: enum change in T-1301 forces the integration test to migrate, but the plan assigned that file to T-1303
- **Where:** `build-plan.md` T-1301 vs T-1303 / `tests/coassertion.rs`
- **Quote:** "Change `RelationKind::CoAssertion(String)` → `CoAssertion { term: String, weight: f64 }`"
- **Failure mode:** hidden-dep
- **Why it matters:** the `CoAssertion` variant is matched/constructed in `tests/coassertion.rs::test_coassertion_pipeline`. If that file is only touched in T-1303, the workspace would not compile after the T-1301 and T-1302 commits — a broken intermediate tree and a false "task complete" for a change whose blast radius is crate-wide.
- **Suggested response:** fix-in-plan — **applied.** T-1301's Touches now includes `tests/coassertion.rs` (compile-forced migration of the existing `test_coassertion_pipeline` to the new signature), and its Notes state the enum change migrates all references together. T-1303 now only **adds** the new `test_coassertion_temperature_pipeline`. Each commit leaves the tree green.

### C-002: "structural edges remain ungated by temperature" is asserted but not directly tested
- **Where:** `build-plan.md` T-1302 EARS / `test-plan.md` Integration
- **Quote:** "compute co-assertion edges at `t` while leaving the structural (category/author) edges ungated"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** the pipeline test uses papers with distinct categories/authors, so it proves temperature gates co-assertion but never exercises a structural edge surviving a low temperature.
- **Suggested response:** defer-with-rationale — the separation is structural, not behavioral: `compute_relations` has no `temperature` parameter in its signature, so it is impossible for the dial to affect structural edges. The existing `compute_relations` unit tests (graph.rs) already assert category/author edges unconditionally. A dedicated "structural edge survives at t=0.0" assertion would be redundant with the type signature; not worth a task. (Low-cost to add inside `test_coassertion_temperature_pipeline` if desired during build.)

### C-003: the default temperature (0.5) is not covered by an automated test
- **Where:** `build-plan.md` T-1302 EARS / `test-plan.md` T-1302 unit tests
- **Quote:** "WHEN `--temperature` is omitted, THEN the handler SHALL use `0.5`"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** `test_parse_temperature` covers the value parser but not the clap `default_value_t = 0.5`, so a regression in the default would not be caught by a unit test.
- **Suggested response:** defer-with-rationale — the default is a single declarative clap attribute (`default_value_t = 0.5`) verified by the offline E2E check (`diver dive --help` shows the default) and by manual runs in the plan's Verification section. A binary-invocation harness that asserts the applied default does not exist yet (no populated-DB binary test — the same deferral as INT-0012/INT-0013 E2E). Changing the default is a conscious one-token edit; the cost of a full CLI harness exceeds the risk here.

## Confidence
proceed-with-caveats
