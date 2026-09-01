# Sprint 15 End-to-End Tests

- **Tested head:** `19ee28db6feff91e27f6f522f138078053011b53`
- **Status:** possible (offline)

## The per-shape wiremock tests are the offline end-to-end of each provider contract

`test_extract_anthropic_tool_use` and `test_extract_openai_structured` drive the full
`extract` path (per-shape request build → send → status/error handling → structured parse →
grounding) against a local `MockServer`. The OpenAI-shape test is, by construction, the
end-to-end proof for **every OpenAI-compatible provider** — OpenAI, Grok, and
**Animus_Ferric** (its `ferric server` is an OpenAI-compatible llama.cpp server): the only
difference in production is the configured `base_url`, which the test supplies as the mock's.

## Executed (manual, offline)
- **No regression / heuristics removed cleanly.** `cargo clippy --workspace --all-targets` → 0
  warnings (no dead-code from the deleted `parse_claim_array`/`strip_fences`); `cargo build`
  clean. **pass** (AC5)
- **Offline path unaffected.** `diver extract --help` still lists `--deterministic` (offline
  sentence-splitter, no key/network). **pass**

## Coverage note
A live run against a real endpoint (Anthropic/OpenAI keys, or a real `ferric server up`
serving a GGUF model) remains a manual check — but it is now a small residual: only real
endpoint reachability/credentials/model behavior, not the request construction, headers,
structured-response parsing, error handling, or provider selection, all covered offline.
