# Sprint 14 Research Report

## Intents Reviewed
- [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) — created; relevance: primary; current state: proposed
- [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) — selected/revised; relevance: the extractor whose HTTP boundary this hardens; current state: realized (its Consequences note about manual-only HTTP verification is the debt this pays down)

## 1. Sprint Goal

Harden the LLM extractor's HTTP boundary — today the only pipeline stage with **no
automated transport-level test**. Make the API base URL injectable and add
`wiremock`-backed tests that exercise the real `reqwest` round-trip in
`LlmExtractor::extract`: request shape (endpoint, headers, body), the 2xx → grounded
candidates happy path, and the non-2xx error path. Deterministic and offline (no
network, no API key). Advances **INT-0015**. Baseline: `8ba2caf`,
`cargo test --workspace` green (123), clippy 0.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/extract.rs | high | `LlmExtractor { http, model, api_key }` + async `extract` (extract.rs:88) builds the request (headers `x-api-key`/`anthropic-version`/`content-type`, body `model`/`max_tokens`/`system`/`messages`) and handles status/error (the `bail!` at extract.rs:115). `MESSAGES_ENDPOINT` (extract.rs:25) is a **hardcoded const** — must become injectable. `parse_claims` and `build` are already well unit-tested; only the HTTP glue is untested. |
| diver-core/tests/llm_extract_pipeline.rs | high | Exercises `parse_claims` on a canned body only — never the HTTP round-trip. The new wiremock test complements this at the transport layer. |
| diver-core/Cargo.toml | high | No `tokio`/`wiremock`. Add both as **dev-dependencies** (async test + mock server). `reqwest` uses `rustls-tls`; posting to a plain-HTTP localhost mock works (TLS only engages for https). |
| diver-cli/Cargo.toml | medium | Already depends on `tokio = { version = "1", features = ["rt-multi-thread", "macros"] }` (lockfile: tokio 1.53.1) — reuse the same version/features for the dev-dep so no new lock churn. |
| docs/intents/INT-0009-*.md | high | Consequences: "Live end-to-end extraction is verified manually (a real run with a key), not in the automated suite." — the exact debt INT-0015 pays down. |

Baseline: workspace at `8ba2caf`. `cargo test --workspace` green (123); clippy 0.

### Design

```rust
// diver-core/src/extract.rs
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com"; // replaces MESSAGES_ENDPOINT

pub struct LlmExtractor {
    http: reqwest::Client,
    model: String,
    api_key: String,
    base_url: String,          // NEW — injectable API root
}

// extract() posts to `format!("{}/v1/messages", self.base_url)` instead of the const.

impl LlmExtractor {
    // build() gains a base_url arg; a blank/None falls back to DEFAULT_BASE_URL.
    fn build(api_key: Option<String>, model: Option<String>, base_url: Option<String>) -> Result<Self>;
    // from_env() reads an optional ANTHROPIC_BASE_URL override (unset → default).
    pub fn from_env() -> Result<Self>;
}
```

```rust
// diver-core/tests/extract_http.rs (new) — #[tokio::test], wiremock
// - happy path: Mock POST /v1/messages -> 200 {content:[{type:text,text:"[{claim,quote}]"}]}
//   Build extractor with base_url = mock.uri(); assert extract() returns the grounded
//   candidate, AND assert the received request had the x-api-key / anthropic-version
//   headers and a body carrying the model + system + user content.
// - error path: Mock -> 500 (body "upstream boom"); assert extract() errors with a
//   message containing the status and body.
```

The change is additive: `parse_claims`, grounding, and the `--deterministic` path are
untouched; the request/response *contract* (fence-tolerant text parsing) is unchanged —
only the endpoint becomes injectable and the round-trip becomes testable.

## 3. External Sources
- [wiremock (Rust) — HTTP mock server for tests](https://docs.rs/wiremock/latest/wiremock/) — `MockServer::start().await`, `Mock::given(method("POST")).and(path("/v1/messages")).respond_with(ResponseTemplate::new(200).set_body_string(..))`, `.mount(&server)`; `server.uri()` yields the base URL; `server.received_requests()` (or a matcher) inspects the sent request/headers/body.
- [reqwest 0.12 client](https://docs.rs/reqwest/latest/reqwest/) — posting to an arbitrary base URL; plain-HTTP localhost works with the `rustls-tls` build.
- [tokio test macro](https://docs.rs/tokio/latest/tokio/attr.test.html) — `#[tokio::test]` for the async transport tests (dev-dep, reusing diver-cli's version/features).
- [Anthropic Messages API — request/response shape](https://docs.anthropic.com/en/api/messages) — confirms the `x-api-key` + `anthropic-version` headers and the `content: [{type, text}]` response envelope the mock must emulate.

## 4. Risks / Unknowns / Dependencies
- **New dev-dependencies (`wiremock`, `tokio`).** Both are dev-only (no runtime/binary impact). tokio is already transitively built; wiremock pulls a small server stack used only under `cargo test`. Mitigation: dev-dependencies only; pin to a current release; verify `cargo build` (non-test) is unchanged.
- **Injectable base URL must not weaken the default or leak the key.** `ANTHROPIC_BASE_URL` override defaults to the real endpoint when unset; the redacting `Debug` impl stays; tests use a throwaway `sk-test` key against localhost. No real secret is ever sent anywhere but the real endpoint. The base-url override is a standard test seam (same pattern as many API clients), not a security downgrade.
- **`build` signature change ripples.** Adding the `base_url` param breaks `test_build_missing_key_errors` and `test_build_model_default_and_override` (compile-forced) — update them in the same task.
- **wiremock request-body assertions.** Asserting the exact JSON body is brittle; assert the *presence* of key fields (model, system, the user content) and the required headers, not a byte-exact body.
- **Async test runtime.** `#[tokio::test]` needs the macros feature; wiremock needs a reactor. Reuse `rt-multi-thread` + `macros` (diver-cli's features) to avoid single-threaded-runtime surprises.
- **Out of scope (named for a follow-up):** migrating the request/response contract to Anthropic **tool-use / structured outputs** (replacing fence-tolerant text parsing). That is a contract redesign deserving its own intent; the transport harness built here is the prerequisite that makes such a migration safely verifiable.

## 5. Recommended Approach

Add `base_url` to `LlmExtractor` (default `https://api.anthropic.com`, overridable via
`build`'s new arg and an optional `ANTHROPIC_BASE_URL` env in `from_env`); post to
`{base_url}/v1/messages`. Add `wiremock` + `tokio` dev-dependencies and a new
`tests/extract_http.rs` with `#[tokio::test]` cases: a 200 happy path (asserting both the
returned grounded candidate and the request's headers/body) and a non-2xx error path
(asserting the error carries status + body). Update the two compile-forced `build` unit
tests. Keep `parse_claims`, grounding, the redacting `Debug`, and `--deterministic`
unchanged. Revise INT-0009's Consequences note to reflect that the HTTP boundary now has
automated coverage. Structured-output/tool-use migration is explicitly deferred to a
future intent.

### Referenced artifacts
- [INT-0015 chapter](../../../intents/INT-0015-harden-extractor-http-boundary.md)
- Build/test plans: `../sprint-plans/` (authored in the plan phase)
- Baseline evidence: `cargo test --workspace` 123/123, clippy 0 at `8ba2caf`
