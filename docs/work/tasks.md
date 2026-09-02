# Agent Tasks (Persistent Backlog)

- [ ] T-1310 (backlog) [intent: INT-0014]: close the two deferred test gaps — a direct assertion that a structural (category/author) edge survives `--temperature 0.0`, and a seeded-DB binary run asserting the applied default 0.5 (test critique C-001/C-002) — touches: diver-core/tests/coassertion.rs (or a new CLI E2E harness)
- [ ] T-1410 (backlog) [intent: INT-0015]: close the deferred extractor-HTTP test gaps — a `from_env` test that honors `ANTHROPIC_BASE_URL` (needs a safe env-mutation approach), and a transport test for a 2xx response with a malformed body (test critique C-001/C-002) — touches: diver-core/src/extract.rs

(T-1420 subsumed by INT-0016 / this sprint.)
- [ ] T-1510 (backlog) [intent: INT-0016]: add a function-calling `tools` fallback for OpenAI-compatible targets that lack `response_format` json_schema (the direct `parse_openai_claims` error test was completed in the s15 review fixes) — touches: diver-core/src/extract.rs
- [ ] T-1611 (backlog) [intent: INT-0017]: add a `DIVER_DB` store-path override to `Store::open()` so corpora/tests/front-ends can use a scratch DB instead of the fixed `dirs::data_dir()/diver/diver.db` — touches: diver-core/src/store.rs
- [ ] T-1702 (sprint 17) [intent: INT-0018]: docs — README note that co-assertion links by distinctive shared terms (common-word stoplist) — touches: README.md

(T-1610 subsumed by INT-0018 / this sprint.)
