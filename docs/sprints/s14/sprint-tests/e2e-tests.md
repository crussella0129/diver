# Sprint 14 End-to-End Tests

- **Tested head:** `4f956d798cbc2ca230219b61cc5fc6885455f0cf`
- **Status:** possible (offline)

## The transport tests ARE the offline end-to-end of the HTTP boundary

`test_extract_http_happy_path` / `test_extract_http_error_status` drive a real `reqwest`
client through the full `extract` path (request build → send → status/error handling →
`parse_claims` → grounding) against a local `MockServer`. This is the offline E2E of the
extractor's network boundary that was previously verified only by a manual run with a key.

## Executed (manual, offline)

- **Dev-dependencies do not enter the runtime build.** `cargo tree -p diver-core -e
  no-dev` contains no `wiremock`; `cargo build` (non-test) succeeds unchanged. **pass** (AC4)
- **Offline path unaffected.** `diver extract --help` still lists `--deterministic`
  (offline sentence-splitter, no key/network). **pass** (AC5)

## Coverage note

A live run against the real Anthropic endpoint remains a manual check (needs a real
`ANTHROPIC_API_KEY` and network) — but it is now a much smaller residual: only
real-endpoint reachability and credentials, not the request construction, header set,
response parsing, or error handling, all of which are now covered offline. An
`ANTHROPIC_BASE_URL` override makes pointing at a staging/proxy endpoint trivial.
