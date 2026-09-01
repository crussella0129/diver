# Test Critique — Sprint 14

## Concerns

### C-001: `from_env`'s `ANTHROPIC_BASE_URL` read is not directly tested
- **Where:** `unit-tests.md` T-1401 / `INT-0015` AC1
- **Quote:** "`from_env` … SHALL use `ANTHROPIC_BASE_URL` when set/non-blank, else the default"
- **Failure mode:** intent-coverage
- **Why it matters:** `test_build_base_url_default_and_override` covers `build`'s
  default/blank/override logic, but nothing asserts that `from_env` actually reads the
  `ANTHROPIC_BASE_URL` env var and forwards it, so dropping that read would go uncaught.
- **Suggested response:** defer-with-rationale — `from_env` is a three-line wrapper that
  only reads env vars and delegates to the tested `build`; directly testing it means
  mutating process env (`std::env::set_var`, `unsafe` in edition 2024) which is racy under
  the parallel runner. The env read mirrors the already-shipped `ANTHROPIC_API_KEY`/
  `DIVER_MODEL` reads, and `test_extract_http_happy_path` drives a non-default `base_url`
  end-to-end through `build`. Carried from plan critique C-001.

### C-002: no transport test for a 2xx response with a malformed body
- **Where:** `integration-tests.md` transport tests / `INT-0015` AC2
- **Quote:** "a 2xx response carrying a valid Messages envelope yields the grounded candidate(s)"
- **Failure mode:** negative-path
- **Why it matters:** the happy path uses a well-formed envelope; there is no transport
  test for a 200 whose body is not a valid envelope (the "server says OK but returns
  garbage" case).
- **Suggested response:** defer-with-rationale — the two halves are each covered: the 200 →
  `parse_claims` wiring is proven by `test_extract_http_happy_path`, and `parse_claims`'s
  handling of malformed/garbage bodies is exhaustively unit-tested (`test_parse_claims_malformed_errors`,
  and the envelope/no-text/non-array cases). A combined "200 + garbage" transport test
  would only re-compose two already-tested halves. Optional future hardening, not a gate.

## Confidence
proceed-with-caveats
