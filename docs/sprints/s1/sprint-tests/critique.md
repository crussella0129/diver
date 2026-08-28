# Test Critique — Sprint 1

## Concerns

### C-001: T-005 client non-200 status test not in automated suite
- **Where:** `test-plan.md` T-005 unit tests / `unit-tests.md`
- **Quote:** "test_client_non_200_status: mock/stub returning 503 → Err naming status code"
- **Failure mode:** EARS-coverage
- **Why it matters:** The T-005 EARS clause "WHEN non-200 status THEN Err naming status code" lacks an automated test. The code path exists and is correct by inspection, but there's no regression guard.
- **Suggested response:** defer-with-rationale — Adding a mock HTTP server (wiremock or similar) solely for one negative-path test adds a dev dependency and complexity disproportionate to the risk in Sprint 1. The code path is trivial (`if !status.is_success() { bail!(...) }`). Will be covered when a future sprint adds wiremock for E2E.

## Confidence
proceed-with-caveats
