# Sprint 8 Research Report

## Intents Reviewed
- [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) — created; relevance: primary; current state: proposed
- [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) — selected; relevance: the gate this extractor feeds; current state: realized

## 1. Sprint Goal

Deliver the LLM-backed claim extractor that INT-0008's typestate gate was built to
admit. In `diver-core`, add an `LlmExtractor` that asks Claude (raw HTTP over the
existing `reqwest`, no SDK — Rust has none) to extract the factual claims a
paper's abstract makes, each with a supporting quote, and returns
`Assertion<Candidate>`s that flow through the existing `validate()` gate.
Epistemic integrity comes from **grounding**: a claim becomes a candidate only if
its quote appears in the abstract. Non-determinism is confined to the HTTP call; a
pure `parse_claims` function is unit-tested with fixtures. `diver extract` uses
LLM extraction by default (needs `ANTHROPIC_API_KEY`), with `--deterministic` for
the INT-0008 offline path.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/client.rs | high | Reqwest pattern to mirror: `Client::builder().user_agent(...).timeout(...).build()`; `async fn` doing `.get(url).send().await` then `.text().await`. `LlmExtractor` mirrors this with `.post(url).header(...).body(json).send().await`. |
| diver-core/src/assertion.rs | high | `Assertion::<Candidate>::new(claim, support)` + `validate()` (the shared gate). `candidate_assertions()` is the deterministic seeder; the LLM extractor builds candidates directly (claim = extracted claim, support = grounded `Observation`). |
| diver-core/src/observation.rs | high | `Observation::new(ArxivId, ArxivVersion, text)`; the LLM quote becomes the supporting observation's text. `extract_observations` remains the `--deterministic` path. |
| diver-core/src/fact.rs | high | `SourceFact.summary` is the abstract fed to the model; `arxiv_id` + `arxiv_version` supply provenance. |
| diver-core/src/id.rs | medium | `ArxivId::new`, `ArxivVersion::parse` for provenance. |
| diver-core/Cargo.toml | medium | Already has `reqwest { default-features=false, features=["rustls-tls"] }`, `serde`, `serde_json`. **No new deps**: send body via `serde_json::to_string` + `.body(...)` (avoids needing reqwest's `json` feature); read via `.text().await` + `serde_json::from_str`. |
| diver-cli/src/main.rs | high | `Commands::Extract { arxiv_id }` gains a `--deterministic` flag; handler branches: LLM (async, needs key) vs the existing deterministic pipeline. `#[tokio::main]` already drives async. |
| diver-core/src/display.rs | low | `display_extract` is reused unchanged (renders `Vec<Assertion<Supported>>`). |

Baseline: workspace at `c6a343b`. `cargo test --workspace` green (73 unit + 4
integration = 77).

### Anthropic Messages API (raw HTTP) — from the `claude-api` skill

- **Endpoint:** `POST https://api.anthropic.com/v1/messages`
- **Headers:** `x-api-key: $ANTHROPIC_API_KEY`, `anthropic-version: 2023-06-01`,
  `content-type: application/json`.
- **Request body:** `{ "model": <id>, "max_tokens": 2048, "system": <extraction
  instructions>, "messages": [{"role":"user","content": <abstract + ask for JSON
  claims>}] }`.
- **Response body:** `{ "content": [{"type":"text","text": <model output>}],
  "stop_reason": "...", "usage": {...} }`. `parse_claims` reads the first `text`
  block and parses the model's JSON.
- **Model:** default `claude-opus-5` (skill guidance: default to most capable;
  user downgrades via `DIVER_MODEL`). Exact ID string, no date suffix. Thinking is
  adaptive-by-default on Opus 5 — omit the `thinking` param. No prefill (400 on
  Opus 5). `max_tokens` modest (2048) for a bounded claim list.
- **Output contract:** instruct the model to return ONLY a JSON array of
  `{"claim": "...", "quote": "..."}`; `parse_claims` strips optional ``` fences,
  slices the outer `[...]`, and `serde_json`-parses. Tolerant by design.

### Extractor design (testable seam)

```rust
// diver-core/src/extract.rs
pub struct LlmExtractor { http: reqwest::Client, model: String, api_key: String }

impl LlmExtractor {
    pub fn from_env() -> Result<Self>;                 // reads ANTHROPIC_API_KEY, DIVER_MODEL
    pub async fn extract(&self, fact: &SourceFact)     // builds request, POSTs, awaits body,
        -> Result<Vec<Assertion<Candidate>>>;          //   then calls parse_claims (below)
}

// PURE + UNIT-TESTED (no network):
pub fn parse_claims(response_body: &str, fact: &SourceFact) -> Result<Vec<Assertion<Candidate>>>;
//   - parse Messages API JSON -> first text block -> Vec<{claim, quote}>
//   - GROUNDING: keep a claim only if `quote` is found in fact.summary (normalized
//     whitespace, case-insensitive substring); build Observation(quote), candidate = (claim, [obs])
```

Testing `parse_claims` with fixture JSON (a grounded claim + a hallucinated one)
covers AC2/AC3 without a live call. The async `extract` is thin glue over
`parse_claims`; live behavior is verified manually with a real key (AC1/AC5).

## 3. External Sources
- [Anthropic Messages API](https://docs.anthropic.com/en/api/messages) — endpoint, headers (`x-api-key`, `anthropic-version: 2023-06-01`), request/response shape.
- `claude-api` skill (bundled) — Rust is unsupported → raw HTTP; default `claude-opus-5`; adaptive thinking; no prefill; `max_tokens` guidance.
- [reqwest](https://docs.rs/reqwest) — `Client::post`, `.header`, `.body`, `.send().await`, `.text().await`; `.json()` needs the `json` feature (avoided here).

## 4. Risks, Unknowns, Dependencies

- **Risk:** live API in tests would be non-deterministic, networked, and cost
  money. Mitigation: the tested seam is the pure `parse_claims`; the HTTP call is
  not in the automated suite. A real run is a manual/documented check.
- **Risk:** the model wraps JSON in prose or ``` fences. Mitigation: `parse_claims`
  strips fences and slices the outer `[...]` before parsing; on unparseable output
  it errors cleanly (no panic) so `diver extract` reports a useful message.
- **Risk:** hallucinated quotes. Mitigation: grounding — a candidate is created
  only if the quote is a normalized substring of the abstract; tested.
- **Risk:** missing/blank `ANTHROPIC_API_KEY`. Mitigation: `from_env` returns a
  clear actionable error ("set ANTHROPIC_API_KEY or use --deterministic").
- **Unknown:** whether `diver-core` needs a `tokio` dependency. Expected no:
  `reqwest`'s async is driven by `diver-cli`'s `#[tokio::main]` (same as
  `client.rs`), and `parse_claims` tests are synchronous (no `#[tokio::test]`).
  Verify at build.
- **Dependency:** none new. `reqwest`, `serde`, `serde_json` already in
  `diver-core`; body serialized with `serde_json::to_string` to avoid needing
  reqwest's `json` feature.
- **Cost note (for the user):** `diver extract` now calls the model once per
  paper; default `claude-opus-5` is the most capable/expensive. `DIVER_MODEL`
  (e.g. `claude-haiku-4-5`, `claude-sonnet-5`) lets the user trade cost/quality.

## 5. Recommended Approach

Primary: build the extractor around the pure, tested `parse_claims` seam, then the
thin async HTTP wrapper, then the CLI flag.

- `extract.rs`: `parse_claims` (pure, grounding, tested) + `LlmExtractor`
  (`from_env`, async `extract` = build request → POST → `text()` → `parse_claims`)
  + a small `ClaimJson { claim, quote }` serde struct + request/response serde
  structs. Prompt/system text as consts.
- `id.rs`/`observation.rs`: reuse `ArxivId`/`ArxivVersion`/`Observation::new`.
- `main.rs`: `Extract { arxiv_id, #[arg(long)] deterministic }`; default branch
  builds `LlmExtractor::from_env()?`, `.extract(&fact).await?`, validates,
  displays; `--deterministic` runs the INT-0008 pipeline. Missing key → error.
- README/docs: document `diver extract`, the `--deterministic` flag,
  `ANTHROPIC_API_KEY`, and `DIVER_MODEL`.

Tests: `parse_claims` — grounded claim kept, hallucinated quote dropped, fenced
JSON tolerated, malformed body errors (no panic); `from_env` missing-key error;
an integration test composing a fixture Messages-API body →
`parse_claims` → `validate()` → `Vec<Assertion<Supported>>` with provenance. E2E:
`diver extract --help` shows `--deterministic`; `diver extract <id>
--deterministic` runs offline (no key, no network); unknown id errors.

Alternative considered: put the extractor behind an async `trait Extractor` with a
fake impl for tests — deferred; the pure `parse_claims` seam gives the same test
coverage without async-trait/dyn-safety complexity. Rationale: isolate
non-determinism at the network edge and unit-test everything up to it.

## Artifacts
- No standalone snippet files; the design and API shape are inline in §2.
