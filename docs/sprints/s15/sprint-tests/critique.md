# Test Critique — Sprint 15

## Concerns

### C-001: the OpenAI-shape structured-parse error path is not directly tested
- **Where:** `integration-tests.md` transport tests / `INT-0016` AC3
- **Quote:** "return `Err` (never panic) on an unparseable or structurally-missing response"
- **Failure mode:** negative-path
- **Why it matters:** `parse_anthropic_claims`'s error paths are pinned by
  `test_parse_claims_requires_structured`, and both shapes' non-2xx paths by the `_error`
  tests, but there is no test for a 2xx OpenAI response whose `choices[0].message.content` is
  absent or not valid claims JSON (the `parse_openai_claims` error branch).
- **Suggested response:** defer-with-rationale — `parse_openai_claims` is the structural
  mirror of the tested `parse_anthropic_claims` (both `serde_json::from_*` into `ClaimsInput`
  with `.context(...)`), the OpenAI happy path + transport-error path are covered, and a
  non-conforming body yields a clean `Err` by construction. Logged as backlog (a direct
  `parse_openai_claims` unit test — no-choices / non-JSON content). Low value vs. the symmetric
  coverage already present.

### C-002: `from_env`'s file read + `DIVER_PROVIDER` wiring is covered only via the pure helper
- **Where:** `unit-tests.md` T-1502 / `INT-0016` AC4
- **Quote:** "the active provider (from `DIVER_PROVIDER` or the file's `active`)"
- **Failure mode:** intent-coverage
- **Why it matters:** `resolve_provider` (pure) is thoroughly tested, but the thin `from_env`
  wrapper that reads `providers.json` from disk and `DIVER_PROVIDER` from the env, then calls
  `resolve_provider`, is not exercised end-to-end.
- **Suggested response:** defer-with-rationale — `from_env` is a few lines that only read a
  file + env vars and delegate to the fully-tested pure helper; testing it directly needs disk
  + `std::env::set_var` (unsafe/racy in edition 2024). Same residual shape as INT-0015's
  `ANTHROPIC_BASE_URL` read; the pure seam is exactly the mitigation.

### C-003: OpenAI `response_format: json_schema` may not be honored by every OpenAI-compatible target
- **Where:** `build-plan.md` T-1501 (OpenAI shape) / research §5 (carried from plan critique C-003)
- **Quote:** "supported by OpenAI, Grok, and llama.cpp/Ollama/vLLM"
- **Failure mode:** intent-drift (portability assumption)
- **Why it matters:** a minimal/older OpenAI-compatible server could ignore `response_format`
  and return prose, which would then fail parsing.
- **Suggested response:** defer-with-rationale — structured/grammar-constrained output is
  exactly Animus_Ferric's design point and is broadly supported by the named targets; a
  non-conforming response errors cleanly (AC3), not silently. Function-calling `tools` is the
  recorded fallback contract (INT-0016 Alternatives) for a future target lacking structured
  outputs; not needed for the named set now.

## Confidence
proceed-with-caveats
