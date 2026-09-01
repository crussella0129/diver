# Plan Critique — Sprint 14

## Concerns

### C-001: `from_env`'s `ANTHROPIC_BASE_URL` wiring is not directly tested
- **Where:** `build-plan.md` T-1401 EARS / `test-plan.md` T-1401 unit tests
- **Quote:** "WHEN `from_env` runs, THEN it SHALL use `ANTHROPIC_BASE_URL` if set and non-blank, otherwise the default"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** the unit test covers `build`'s Some/None/blank logic, but nothing asserts that `from_env` actually reads the `ANTHROPIC_BASE_URL` env var and passes it to `build`, so a regression dropping that env read would go uncaught.
- **Suggested response:** defer-with-rationale — `from_env` is a three-line wrapper that only reads env vars and delegates to the tested `build`; directly testing it requires mutating process env (`std::env::set_var`, `unsafe` in edition 2024) which is racy under the parallel test runner. The build-level branch logic is unit-tested, and `test_extract_http_happy_path` exercises a non-default `base_url` end-to-end through `build`. The residual is one `env::var(...).ok()` call, mirrored on the already-shipped `ANTHROPIC_API_KEY`/`DIVER_MODEL` reads.

### C-002: `extract`'s exact request path (`{base_url}/v1/messages`) is only implicitly asserted
- **Where:** `build-plan.md` T-1401 EARS / `test-plan.md` transport tests
- **Quote:** "WHEN `extract` sends its request, THEN it SHALL POST to `{base_url}/v1/messages`"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** there is no dedicated assertion on the path; it is verified only by the happy-path mock's `path("/v1/messages")` matcher.
- **Suggested response:** defer-with-rationale — the matcher IS the assertion: if `extract` posted to a different path (or method), no mock would match, wiremock would return 404, `extract` would bail on the non-2xx, and `test_extract_http_happy_path`'s `.unwrap()` on the returned candidates would fail. The path/method/headers are all enforced by matcher-or-fail; making them explicit `received_requests` assertions is a readability nicety, not additional coverage.

## Confidence
proceed-with-caveats
