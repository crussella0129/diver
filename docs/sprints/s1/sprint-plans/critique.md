# Plan Critique — Sprint 1

## Concerns

### C-001: Rate-limit risk deferred without task
- **Where:** `build-plan.md` — no task addresses ArXiv rate limiting
- **Quote:** Research report: "ArXiv API availability / rate limiting — Medium"
- **Failure mode:** missing-risk
- **Why it matters:** If a future sprint adds batch or pagination, the client would hit the 3 req/s limit without protection.
- **Suggested response:** defer-with-rationale — CLI users make one request per invocation; no batching exists yet. Explicitly deferred to a future sprint that adds multi-request flows.

## Confidence
proceed-with-caveats
