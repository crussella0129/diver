# Plan Critique — Sprint 7

## Concerns

### C-001: AC2/AC4 (the compile-time gate) have no automated compile-fail test
- **Where:** `build-plan.md` T-702 / `test-plan.md` Intent Traceability AC2, AC4
- **Quote:** "no `trybuild` dev-dep is added this sprint"
- **Failure mode:** intent-drift (acceptance criterion without automated verification)
- **Why it matters:** AC4 — "`Assertion<Supported>` is unconstructable outside
  validation" — is the central promise of the sprint, yet it is verified by design
  review, not an executed negative test. A future refactor could add a public
  `Supported` constructor and no test would fail.
- **Suggested response:** defer-with-rationale — the guarantee is enforced
  structurally (private fields + no public `Supported` constructor; `validate`
  consumes a `Candidate` and is the only path). The AC3 runtime tests prove the
  gate *logic*, and the `diver-cli` handler exercising `validate().ok()` proves
  the external happy path compiles. A `trybuild` ui test is the canonical negative
  proof but adds a brittle dev-dependency (error-text matching drifts across
  compiler versions) — a poor trade for a ~2.3k-line project. Recorded as an
  explicit design decision; a future sprint may add `trybuild` if the assertion
  API grows public surface.

### C-002: T-701 and T-702 both edit `diver-core/src/lib.rs`
- **Where:** `build-plan.md` T-701 Touches / T-702 Touches
- **Quote:** "diver-core/src/lib.rs (`pub mod observation`)" … "diver-core/src/lib.rs (`pub mod assertion`)"
- **Failure mode:** hidden-dep
- **Why it matters:** two sequential tasks touch the same file, which can look like
  an ordering hazard.
- **Suggested response:** reject (the critique is wrong because …) — the edits are
  additive, non-overlapping `pub mod` lines on distinct lines, applied in
  dependency order (T-701 before T-702). There is no shared symbol or ordering
  conflict; each task's commit includes only its own module declaration.

### C-003: `is_supported` v1 rule (`!support.is_empty()`) makes every observation "supported"
- **Where:** `build-plan.md` T-702 Notes
- **Quote:** "v1: one candidate per observation … `is_supported` v1 rule:
  `!support.is_empty()`"
- **Failure mode:** intent-drift (does the gate mean anything if it always passes?)
- **Why it matters:** if every candidate validates, `test_validate_rejects_unsupported`
  can only fail an artificially empty-support candidate, not a realistic one.
- **Suggested response:** defer-with-rationale — this is intended and stated in
  INT-0008 ("the deterministic support rule is intentionally simple in v1; later
  sprints refine it … without changing the typestate gate"). This sprint's value
  is the *type architecture*, not the sophistication of the rule. The empty-support
  case is a real API state (a candidate can be built with no support), so the
  negative test is meaningful, and the gate's shape is what future rules plug into.

## Confidence
proceed-with-caveats
