# Sprint 14 Meta

- **Sprint number:** 14
- **Book schema version:** 2
- **Start timestamp:** 2026-09-01T17:55:27Z
- **End timestamp:** (filled at Loop Phase)
- **Model:** claude-opus-4-8
- **Exit status:** in-progress
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Harden the LLM extractor's HTTP boundary: make the API base URL injectable (default `https://api.anthropic.com`, optional `ANTHROPIC_BASE_URL`) and add `wiremock`-backed transport tests over the real `reqwest` round-trip (2xx happy path + request shape, non-2xx error path). Deterministic, offline; tool-use/structured outputs deferred.
- **Intents:** [INT-0015](../../intents/INT-0015-harden-extractor-http-boundary.md)
- **Completion evidence:** (filled at Loop Phase)
