# Plan Critique — Sprint 8

## Concerns

### C-001: AC1's live HTTP call (and the default CLI path) is not exercised by an automated test
- **Where:** `build-plan.md` T-802 / T-803 default branch; `test-plan.md` AC1 row
- **Quote:** "manual live run (documented); `test_llm_extract_pipeline` covers
  parse→validate"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** `LlmExtractor::extract` (the actual POST) and the default
  `diver extract <id>` branch make a real network call; nothing in the suite
  proves the request is well-formed against the live API or that a 2xx body flows
  through end to end.
- **Suggested response:** defer-with-rationale — a live-API test would be
  non-deterministic, networked, and bill real money on every CI run; that is why
  the design isolates all logic in the pure `parse_claims` seam (fixture-tested
  for grounding, fences, malformed input) and keeps `extract` a thin wrapper
  (build request → POST → `.text()` → `parse_claims`). AC1's evidence is
  `test_llm_extract_pipeline` (a fixture Messages-API body → `parse_claims` →
  `validate` → supported) plus a **documented manual run** with a real key
  recorded in `e2e-tests.md`. A future sprint could add an opt-in `#[ignore]`
  live test or a mocked HTTP server (`wiremock`) if the request shape needs
  regression protection — deliberately out of scope here (no new test-infra dep).

### C-002: `parse_claims` tolerance rules (fence-stripping, `[...]` slicing) risk silent mis-parses
- **Where:** `build-plan.md` T-801 Notes
- **Quote:** "strip optional ``` fences; slice the outer `[` … `]`"
- **Failure mode:** granularity / correctness edge
- **Why it matters:** heuristic slicing (outer `[` to last `]`) could grab the
  wrong span if the claim/quote text itself contains brackets, yielding a parse
  error or dropped claims.
- **Suggested response:** fix-in-plan — the parse strategy is: (1) deserialize the
  Messages-API envelope with serde and take the first text block (structured, not
  heuristic); (2) on the block's text, try `serde_json::from_str::<Vec<ClaimJson>>`
  directly first, and only fall back to fence-stripping + outer-`[`…`]` slicing if
  that fails. `test_parse_claims_tolerates_fences` covers the fenced case and
  `test_parse_claims_malformed_errors` covers the unparseable case (clean `Err`,
  no panic). Bracket-in-text is thus handled by the direct-parse-first path.

### C-003: T-801 and T-802 both edit `diver-core/src/extract.rs`
- **Where:** `build-plan.md` T-801 Touches / T-802 Touches
- **Failure mode:** hidden-dep
- **Why it matters:** two sequential tasks write the same new file.
- **Suggested response:** reject (the critique is wrong because …) — T-801 creates
  `extract.rs` with the pure parser; T-802 appends the `LlmExtractor` struct +
  `build`/`from_env`/`extract` to the same file in dependency order. The additions
  are disjoint items; each task's commit is a coherent slice of one new module.
  No shared-symbol or ordering hazard.

## Confidence
proceed-with-caveats
