# Agent Tasks (Persistent Backlog)

- [ ] T-1310 (backlog) [intent: INT-0014]: close the two deferred test gaps — a direct assertion that a structural (category/author) edge survives `--temperature 0.0`, and a seeded-DB binary run asserting the applied default 0.5 (test critique C-001/C-002) — touches: diver-core/tests/coassertion.rs (or a new CLI E2E harness)
- [ ] T-1410 (backlog) [intent: INT-0015]: close the deferred extractor-HTTP test gaps — a `from_env` test that honors `ANTHROPIC_BASE_URL` (needs a safe env-mutation approach), and a transport test for a 2xx response with a malformed body (test critique C-001/C-002) — touches: diver-core/src/extract.rs

(T-1420 subsumed by INT-0016 / this sprint.)
- [ ] T-1510 (backlog) [intent: INT-0016]: add a function-calling `tools` fallback for OpenAI-compatible targets that lack `response_format` json_schema (the direct `parse_openai_claims` error test was completed in the s15 review fixes) — touches: diver-core/src/extract.rs
- [ ] T-1601 (sprint 16) [intent: INT-0017]: `diver extract --all` batch mode (extract every stored paper; factor extract_and_save helper) — touches: diver-cli/src/main.rs
- [ ] T-1602 (sprint 16) [intent: INT-0017]: real arXiv feed fixture + offline end-to-end test (ingest→deterministic extract→weighted dive graph asserts structural + co-assertion edges) — touches: diver-core/tests/fixtures/real_corpus_feed.xml, diver-core/tests/real_corpus.rs
- [ ] T-1603 (sprint 16) [intent: INT-0017]: docs — README corpus workflow (collect → extract --all → dive) + backlog notes (co-assertion noise, DIVER_DB override) — touches: README.md, docs/work/tasks.md
