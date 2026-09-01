Finalized - DO NOT EDIT

# Sprint 14 Build Plan

## Intents
- [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) — state: planned; acceptance criteria covered: AC1, AC2, AC3, AC4, AC5

## Schema Tree
- Sprint Goal: give the extractor's HTTP boundary automated coverage
  - Injectable seam (diver-core)
    - T-1401: injectable base URL in `LlmExtractor`
  - Transport tests (diver-core)
    - T-1402: wiremock transport tests + dev-dependencies
  - Docs + reconciliation
    - T-1403: document `ANTHROPIC_BASE_URL` + amend INT-0009

## Execution Sequence

### T-1401: make the extractor's API base URL injectable
- **Intent:** [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md)
- **Touches:** `diver-core/src/extract.rs`
- **Depends on:** (none)
- **Acceptance criterion:** AC1 (injectable base_url + from_env override), AC5 (no regression)
- **Success criterion (EARS):**
  - **WHEN** `build` is called with `Some(base)`, **THEN** the extractor **SHALL** store `base` as its base URL; **WHEN** called with `None` or a blank string, **THEN** it **SHALL** use `DEFAULT_BASE_URL` (`https://api.anthropic.com`).
  - **WHEN** `extract` sends its request, **THEN** it **SHALL** POST to `{base_url}/v1/messages`.
  - **WHEN** `from_env` runs, **THEN** it **SHALL** use `ANTHROPIC_BASE_URL` if set and non-blank, otherwise the default, by delegating to `build`.
- **Notes:** Replace the `MESSAGES_ENDPOINT` const with `DEFAULT_BASE_URL`; add a `base_url` field; thread the new third `build` parameter through `from_env` (reading `ANTHROPIC_BASE_URL`). `from_env`'s public signature is unchanged, so `diver-cli` is unaffected. Update the two compile-forced unit tests (`test_build_missing_key_errors`, `test_build_model_default_and_override`) to the 3-arg `build`. The redacting `Debug`, grounding, `parse_claims`, and `--deterministic` path are untouched.

### T-1402: wiremock-backed transport tests for `extract`
- **Intent:** [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md)
- **Touches:** `diver-core/Cargo.toml`, `diver-core/src/extract.rs` (test module)
- **Depends on:** T-1401
- **Acceptance criterion:** AC2 (2xx happy path + request shape), AC3 (non-2xx error path), AC4 (dev-deps only)
- **Success criterion (EARS):**
  - **WHEN** the mock server returns a 2xx response carrying a valid Messages envelope, **THEN** `extract` **SHALL** return the grounded candidate(s), **AND** the request the mock received **SHALL** carry the `x-api-key` and `anthropic-version` headers and a body containing the configured model, the system prompt, and the paper's abstract.
  - **WHEN** the mock server returns a non-2xx response, **THEN** `extract` **SHALL** return `Err` whose message contains the HTTP status and the response body.
- **Notes:** Add `[dev-dependencies]` `wiremock` and `tokio = { version = "1", features = ["rt-multi-thread", "macros"] }` (matching diver-cli's tokio 1.53.1). Two `#[tokio::test]`s in extract.rs's `#[cfg(test)] mod tests`, using the private `build(..., Some(server.uri()))` so wiremock's dynamic URI can be injected without widening the public API or mutating process env. Reuse `fact_with_summary`/`envelope` helpers. Verify `cargo build` (non-test) gains no new runtime dependency.

### T-1403: document `ANTHROPIC_BASE_URL` and amend INT-0009
- **Intent:** [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md)
- **Touches:** `README.md`, `docs/intents/INT-0009-llm-claim-extractor.md`
- **Depends on:** T-1401, T-1402
- **Acceptance criterion:** AC5 (docs + no regression)
- **Success criterion (EARS):**
  - **WHEN** the README documents extractor configuration, **THEN** it **SHALL** mention the optional `ANTHROPIC_BASE_URL` override and that it defaults to the Anthropic endpoint.
- **Notes:** Document `ANTHROPIC_BASE_URL` alongside `ANTHROPIC_API_KEY`/`DIVER_MODEL`. Amend INT-0009's Consequences note (currently "verified manually … not in the automated suite") to record that the HTTP boundary now has automated wiremock coverage, linking to INT-0015. This is a reconciliation of a realized intent's provenance, not a change to its acceptance criteria.
