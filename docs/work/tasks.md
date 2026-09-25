# Agent Tasks (Persistent Backlog)

- [ ] T-1310 (backlog) [intent: INT-0014]: close the two deferred test gaps — a direct assertion that a structural (category/author) edge survives `--temperature 0.0`, and a seeded-DB binary run asserting the applied default 0.5 (test critique C-001/C-002) — touches: diver-core/tests/coassertion.rs (or a new CLI E2E harness)
- [ ] T-1410 (backlog) [intent: INT-0015]: close the deferred extractor-HTTP test gaps — a `from_env` test that honors `ANTHROPIC_BASE_URL` (needs a safe env-mutation approach), and a transport test for a 2xx response with a malformed body (test critique C-001/C-002) — touches: diver-core/src/extract.rs

(T-1420 subsumed by INT-0016 / this sprint.)
- [ ] T-1510 (backlog) [intent: INT-0016]: add a function-calling `tools` fallback for OpenAI-compatible targets that lack `response_format` json_schema (the direct `parse_openai_claims` error test was completed in the s15 review fixes) — touches: diver-core/src/extract.rs
(T-1611 realized by INT-0019 / sprint 18. It was filed under `[intent: INT-0017]`, which was never right — the work belongs to INT-0019, which did not exist when it was filed. Correction recorded here rather than by silently rewriting the original.)

(T-1610 subsumed by INT-0018 / this sprint.)
(T-1710 realized by INT-0020 / sprint 19: two-word phrase concepts shared by 2+ papers, with co-assertion keyed on them.)

- [ ] T-1810 (backlog) [intent: INT-0019]: echo the resolved corpus path in a CLI affordance (e.g. `diver inspect`/`list` header) so a stray `DIVER_DB` is visible rather than silently redirecting the corpus — the second mitigation named in INT-0019 Consequences; the README warning (T-1803) is the first — touches: diver-cli/src/main.rs, diver-core/src/store.rs (`current_db_path` is already public)

- [ ] T-1910 (backlog) [intent: INT-0020]: rename `RelationKind::CoAssertion { term }` to `{ concept }` now that it carries a concept label (deferred in Sprint 19 so the existing co-assertion tests could prove AC5 unmodified) — touches: diver-core/src/graph.rs, diver-core/src/display.rs, diver-core/tests/coassertion.rs, diver-core/tests/real_corpus.rs
- [ ] T-1911 (backlog) [intent: INT-0004]: persist the arXiv request throttle across processes (e.g. a timestamp next to the corpus) — back-to-back `diver collect` runs are not spaced today, since the 3 s interval lives in one process (found in the Sprint 19 drive) — touches: diver-core/src/client.rs
