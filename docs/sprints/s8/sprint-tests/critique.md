# Test Critique — Sprint 8

## Concerns

### C-001: AC1's live HTTP call (and the default `diver extract` branch) is not exercised by the automated suite
- **Where:** `integration-tests.md` / `e2e-tests.md` "Live LLM run"; `INT-0009` AC1
- **Quote:** "only the socket call itself is outside the suite"
- **Failure mode:** e2e-cop-out / intent-coverage
- **Why it matters:** `LlmExtractor::extract` (the real POST) and the default CLI
  branch make a network call that nothing in `cargo test` proves end to end — a
  malformed request URL/header or a response-shape mismatch would not fail CI.
- **Suggested response:** defer-with-rationale — carried from the accepted plan
  (plan critique C-001). The design deliberately isolates all logic in the pure,
  fixture-tested `parse_claims` and keeps `extract` a thin wrapper (build request →
  POST → `.text()` → `parse_claims`). `test_llm_extract_pipeline` feeds the exact
  Messages-API response shape through `parse_claims` → `validate`, so everything
  from the response body onward is covered; the request headers/body and the
  socket are verified by a **documented manual run** (`e2e-tests.md`). A live or
  mocked-HTTP (`wiremock`) test is intentionally out of scope (no network in CI,
  no new test-infra dependency) and noted as future hardening.

### C-002: grounding uses a normalized substring — a paraphrased quote would be dropped
- **Where:** `unit-tests.md` T-801 / `extract.rs` `is_grounded`
- **Quote:** "normalized case/whitespace substring"
- **Failure mode:** weak-assertion (false negatives, not false positives)
- **Why it matters:** the model is told to copy quotes verbatim, but if it
  paraphrases, a true claim could be dropped as "ungrounded".
- **Suggested response:** defer-with-rationale — this is a deliberately
  conservative gate: it can only **drop** a real claim (false negative), never
  **admit** a hallucination (false positive), which is the epistemically safe
  direction for a provenance engine. The prompt instructs verbatim copying, and
  `test_parse_claims_grounded` confirms the intended path. Fuzzy/semantic grounding
  is a future refinement that plugs into the same seam without weakening safety.

## Confidence
proceed-with-caveats
