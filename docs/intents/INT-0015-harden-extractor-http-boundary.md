# INT-0015 — Harden the LLM extractor's HTTP boundary

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0015
- **State:** planned
- **Work evidence:** [Sprint 14 build plan](../sprints/s14/sprint-plans/build-plan.md) (T-1401, T-1402, T-1403)
- **Completion evidence:** (pending)
- **Code evidence:** [diver-core/src/extract.rs](../../diver-core/src/extract.rs), [diver-core/Cargo.toml](../../diver-core/Cargo.toml)
- **Test evidence:** (pending — `diver-core/tests/extract_http.rs`)
- **Documentation evidence:** (pending)

## Intent

Close the automated-coverage gap in the LLM extractor ([[llm-claim-extractor]],
INT-0009): its pure `parse_claims`/`build` logic is well unit-tested, but the
`reqwest` round-trip in `LlmExtractor::extract` — request construction and
response/error handling — has **no automated test** (INT-0009 Consequences: the
HTTP boundary is "verified manually … not in the automated suite"). This sprint
makes that boundary testable and tests it.

- **Injectable base URL.** Replace the hardcoded `MESSAGES_ENDPOINT` const with a
  `base_url` on `LlmExtractor` (default `https://api.anthropic.com`), overridable
  via `build` and an optional `ANTHROPIC_BASE_URL` env in `from_env`. `extract`
  posts to `{base_url}/v1/messages`.
- **wiremock transport tests.** With `wiremock` + `tokio` as dev-dependencies,
  point `extract` at a mock server and assert:
  - a 2xx response carrying a valid Messages envelope yields the grounded
    candidate(s), and the request that was sent carried the `x-api-key` and
    `anthropic-version` headers and a body containing the model, system prompt,
    and user content;
  - a non-2xx response produces an error whose message includes the HTTP status
    and the response body.

The request/response *contract* (fence-tolerant text parsing, grounding, the
`--deterministic` offline path, the redacting `Debug`) is unchanged — only the
endpoint becomes injectable and the round-trip becomes covered.

Non-goals:
- No migration to Anthropic **tool-use / structured outputs** (a request/response
  contract redesign) — deferred to a future intent; the harness built here is its
  prerequisite.
- No retry/backoff policy, no streaming, no change to grounding or the typestate
  `validate` gate.
- No new runtime/binary dependency — `wiremock`/`tokio` are dev-dependencies only.

## Acceptance criteria

1. `LlmExtractor` carries an injectable `base_url` (default `https://api.anthropic.com`);
   `extract` posts to `{base_url}/v1/messages`; `from_env` honors an optional
   `ANTHROPIC_BASE_URL`, falling back to the default when unset/blank.
2. A `wiremock`-backed `#[tokio::test]` proves the 2xx happy path: `extract`
   returns the grounded candidate(s) from a mocked Messages envelope, and the
   request the mock received carried the `x-api-key` + `anthropic-version` headers
   and a body containing the configured model, the system prompt, and the paper's
   abstract.
3. A `wiremock`-backed test proves the error path: a non-2xx mock response makes
   `extract` return `Err` whose message contains the HTTP status and the body.
4. `wiremock` and `tokio` are added as **dev-dependencies** of `diver-core` only;
   `cargo build` (non-test) gains no new runtime dependency.
5. All previously passing tests still pass; `build`'s new `base_url` parameter is
   threaded through its callers and tests.

## Rationale

The extractor is the epistemic foundation: every downstream stage consumes the
claims it produces, yet its network boundary is the one pipeline stage with no
automated test — a silent-failure risk if the API contract, headers, or
error-handling regress. A base-URL seam plus wiremock gives deterministic,
offline coverage of the exact request the client sends and the errors it surfaces,
and is the prerequisite for any future structured-output migration.

## Alternatives

- **Trait-abstract the HTTP client and mock the trait** — rejected: heavier
  indirection; a base-URL seam + a real `reqwest` round-trip against wiremock tests
  more of the actual code (serialization, headers, status handling).
- **Record/replay (VCR-style) fixtures** — rejected: wiremock is simpler for a
  single endpoint and asserts the outbound request directly.
- **Do tool-use/structured outputs now** — deferred: a contract redesign that
  should ride on top of this test harness, not block it.

## Consequences

- `diver-core` gains `wiremock` + `tokio` dev-dependencies (test-only; no binary
  impact). The extractor gains a `base_url` field and `build` a parameter.
- The HTTP boundary is now covered by the automated suite; INT-0009's
  "manual-only" Consequences note is amended accordingly.
- An `ANTHROPIC_BASE_URL` override exists (defaulting to the real endpoint); it is
  a standard test/proxy seam, and the redacting `Debug` and key handling are
  unchanged.

## Transition history
- 2026-09-01: created as `proposed`.
- 2026-09-01: `proposed` → `planned`; linked to Sprint 14 build plan (T-1401
  injectable base URL, T-1402 wiremock transport tests + dev-deps, T-1403 docs +
  amend INT-0009). Scope: transport harness only; tool-use/structured outputs deferred.
