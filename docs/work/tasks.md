# Agent Tasks (Persistent Backlog)

- [ ] T-1310 (backlog) [intent: INT-0014]: close the two deferred test gaps — a direct assertion that a structural (category/author) edge survives `--temperature 0.0`, and a seeded-DB binary run asserting the applied default 0.5 (test critique C-001/C-002) — touches: diver-core/tests/coassertion.rs (or a new CLI E2E harness)
- [ ] T-1410 (backlog) [intent: INT-0015]: close the deferred extractor-HTTP test gaps — a `from_env` test that honors `ANTHROPIC_BASE_URL` (needs a safe env-mutation approach), and a transport test for a 2xx response with a malformed body (test critique C-001/C-002) — touches: diver-core/src/extract.rs
- [ ] T-1503 (sprint 15) [intent: INT-0016]: docs — providers.json format + per-provider examples (Claude/OpenAI/Grok/Animus_Ferric) + amend INT-0009 + module doc — touches: README.md, docs/intents/INT-0009-llm-claim-extractor.md, diver-core/src/extract.rs

(T-1420 subsumed by INT-0016 / this sprint.)
