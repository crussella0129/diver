# Agent Tasks (Persistent Backlog)

- [ ] T-1310 (backlog) [intent: INT-0014]: close the two deferred test gaps — a direct assertion that a structural (category/author) edge survives `--temperature 0.0`, and a seeded-DB binary run asserting the applied default 0.5 (test critique C-001/C-002) — touches: diver-core/tests/coassertion.rs (or a new CLI E2E harness)
- [ ] T-1401 (sprint 14) [intent: INT-0015]: make the extractor's API base URL injectable (base_url field + build param + ANTHROPIC_BASE_URL in from_env) — touches: diver-core/src/extract.rs
- [ ] T-1402 (sprint 14) [intent: INT-0015]: wiremock-backed transport tests for extract + dev-dependencies — touches: diver-core/Cargo.toml, diver-core/src/extract.rs
- [ ] T-1403 (sprint 14) [intent: INT-0015]: document ANTHROPIC_BASE_URL (README) + amend INT-0009 Consequences — touches: README.md, docs/intents/INT-0009-llm-claim-extractor.md
