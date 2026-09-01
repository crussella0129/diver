# Sprint 15 Integration Tests

- **Tested head:** `19ee28db6feff91e27f6f522f138078053011b53`
- **Runner:** `cargo test --workspace`

## Per-shape transport integration (INT-0016)

Four `wiremock`-backed `#[tokio::test]`s (in the `diver_core` lib binary) drive the real
`reqwest` round-trip against a mock, one pair per shape:

- `test_extract_anthropic_tool_use`: `from_config(shape=Anthropic, base_url=server.uri())`;
  mock `POST /v1/messages` → 200 `tool_use` envelope (grounded + hallucinated claim). Asserts
  the grounded candidate is returned and the request declared the forced `record_claims` tool
  with the `x-api-key`/`anthropic-version` headers and the model. **pass** (AC1)
- `test_extract_openai_structured`: `from_config(shape=OpenAiCompatible, base_url=server.uri())`;
  mock `POST /v1/chat/completions` → 200 with `choices[0].message.content` = the claims JSON.
  Asserts the grounded candidate and that the request carried `Authorization: Bearer`, a
  `json_schema` `response_format`, and the model. **This is also the Animus_Ferric / Grok /
  local-server proof** (identical contract, base_url = mock). **pass** (AC2)
- `test_extract_anthropic_error` (500) / `test_extract_openai_error` (401): non-2xx → `Err`
  carrying the status and body. **pass** (AC3)

## Existing integration binaries (migrated / unchanged, all green)
- `llm_extract_pipeline` (1) and `persist_pipeline` (1) — migrated their canned bodies to
  `tool_use` structured envelopes; still assert grounding admits only the grounded claim and it
  validates to `Supported`. `coassertion` 2, `dive_graph` 1, `dive_pipeline` 1,
  `extract_pipeline` 1, `ingest_pipeline` 2 — unchanged. **9 passed.**

## Isolation / determinism
- Each transport test owns a fresh `MockServer` on a random port; `base_url` injected via
  `from_config`; fixed request/response bodies; throwaway `sk-test`; no real network or key.
- Config resolution tested through the pure `resolve_provider` (no process-env mutation).

## Raw result
```
Running unittests src\lib.rs (diver_core)  →  the four test_extract_* ok
Running tests\llm_extract_pipeline.rs / persist_pipeline.rs  →  ok (migrated)
```
