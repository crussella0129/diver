# Sprint 15 Meta

- **Sprint number:** 15
- **Book schema version:** 2
- **Start timestamp:** 2026-09-01T18:31:20Z
- **End timestamp:** 2026-09-01T20:03:57Z
- **Model:** claude-opus-4-8
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Make the extractor agent-agnostic: a provider substrate with two compiled shapes — `anthropic` (Messages API tool-use) and `openai` (Chat Completions structured outputs) — selected by hot-loadable runtime config. Covers Claude, OpenAI, Grok, and Animus_Ferric (OpenAI-compatible via `ferric server`); grounding + validate unchanged; fence heuristics removed.
- **Intents:** [INT-0016](../../intents/INT-0016-structured-claim-extraction.md)
- **Completion evidence:** INT-0016 realized: agent-agnostic extractor substrate (Anthropic tool-use + OpenAI structured shapes) with hot-loadable providers config; covers Claude/OpenAI/Grok/Animus_Ferric; 128 tests pass, clippy 0
