# Plan Critique — Sprint 2

## Concerns

### C-001: T-011 error detection not testable without fixture
- **Where:** `test-plan.md` T-011 unit tests
- **Quote:** "`test_fetch_by_id_error_entry`: parse fixture with `<title>Error</title>` → descriptive Err"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The EARS clause for error entry detection is in `fetch_by_id` (which wraps HTTP + parse), but the test tests parsing only. The error detection logic (checking for "Error" title) must exist in a function the test can call without network. If `fetch_by_id` is the only place that checks, the test either needs a mock HTTP client or a separate `validate_fetch_result` function.
- **Suggested response:** fix-in-plan — add `validate_fetch_result(feed: &FeedResult) → Result<Paper>` as a testable unit that `fetch_by_id` calls internally. The fixture test exercises this function.

### C-002: T-011 depends on existing parse module but declaration says "(none)"
- **Where:** `build-plan.md` T-011
- **Quote:** "**Depends on:** (none — uses existing parse module)"
- **Failure mode:** hidden-dep
- **Why it matters:** While the parse module already exists from Sprint 1, the error-entry detection adds new validation logic that depends on the parse output shape (`FeedResult`). The dependency notation is misleading but not blocking since `parse.rs` is not being modified.
- **Suggested response:** defer-with-rationale — the comment in parens already clarifies it. No code dependency is untracked.

## Confidence
proceed-with-caveats
