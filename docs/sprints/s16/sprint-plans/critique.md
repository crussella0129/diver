# Plan Critique — Sprint 16

## Concerns

### C-001: the E2E co-assertion assertion must not hinge on a specific edge surviving temp 0.5
- **Where:** `test-plan.md` `test_real_corpus_dive` (initial "…, 0.5) … ≥ 1 CoAssertion edge")
- **Quote:** "≥ 1 `CoAssertion { term, weight }` edge exists between two **distinct** paper ids"
- **Failure mode:** hidden-dep (test robustness)
- **Why it matters:** whether a co-assertion edge clears the 0.5 threshold depends on the exact
  IDF weights of terms in the captured real abstracts; asserting a survivor at 0.5 could make the
  test brittle to the specific fixture content.
- **Suggested response:** fix-in-plan — **applied.** The existence assertion now runs at
  temperature **1.0** (threshold 0.0), which yields a co-assertion edge for *any* significant
  term two same-topic papers share — guaranteed for an attention/NMT corpus — while still
  asserting the edges are weighted with finite weights in `[0.0, 1.0]`. Temperature-0.5 is checked
  only as a **subset** of the 1.0 set (monotonicity on real data), not for a specific survivor.

### C-002: `diver extract --all` CLI handler is not directly tested
- **Where:** `build-plan.md` T-1601 EARS / `test-plan.md` Unit Tests
- **Quote:** "WHEN `diver extract --all` runs, THEN it SHALL extract every stored paper …"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** the `--all` handler (iterate `Store::list`, per-paper extract, summary,
  empty-store message) is only manually verified; no automated test invokes the binary with `--all`.
- **Suggested response:** defer-with-rationale — the handler is a thin loop over `Store::list`
  calling the same `extract_and_save` deterministic path that `test_real_corpus_dive` exercises
  over the whole corpus at the library level, and clap enforces the arg contract
  (`required_unless_present`/`conflicts_with`). No populated-DB binary-invocation harness exists
  (consistent with every prior CLI handler); a fixture-DB CLI E2E is future hardening. The
  behavioral core (extract every stored paper deterministically → persisted assertions → graph)
  is proven.

### C-003: the fixture must actually produce the asserted edges
- **Where:** `build-plan.md` T-1602 (captured fixture)
- **Quote:** "fetching the live arXiv API for a multi-paper attention/NMT query"
- **Failure mode:** hidden-dep
- **Why it matters:** the committed fixture must contain papers that share a category (structural
  edge) and share ≥ 1 significant claim term (co-assertion edge), or the test can't pass.
- **Suggested response:** fix-in-build — the Sprint 16 live probe already demonstrated a real
  attention/NMT collection yields both (multiple `cs.CL` papers; `co-asserts decoder/task/…`
  edges). The fixture is captured from such a query and the test verified green at build time
  before commit; if a draw lacks shared terms, capture a larger/narrower query. Bounded.

## Confidence
proceed-with-caveats
