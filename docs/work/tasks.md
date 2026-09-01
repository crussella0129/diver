# Agent Tasks (Persistent Backlog)

- [ ] T-1310 (backlog) [intent: INT-0014]: close the two deferred test gaps — a direct assertion that a structural (category/author) edge survives `--temperature 0.0`, and a seeded-DB binary run asserting the applied default 0.5 (test critique C-001/C-002) — touches: diver-core/tests/coassertion.rs (or a new CLI E2E harness)
- [ ] T-1410 (backlog) [intent: INT-0015]: close the deferred extractor-HTTP test gaps — a `from_env` test that honors `ANTHROPIC_BASE_URL` (needs a safe env-mutation approach), and a transport test for a 2xx response with a malformed body (test critique C-001/C-002) — touches: diver-core/src/extract.rs

(T-1420 subsumed by INT-0016 / this sprint.)
- [ ] T-1510 (backlog) [intent: INT-0016]: close the deferred extractor gaps — a direct `parse_openai_claims` error unit test (no-choices / non-JSON content), and a function-calling `tools` fallback for OpenAI-compatible targets that lack `response_format` json_schema (test critique C-001/C-003) — touches: diver-core/src/extract.rs
