# Sprint 8 Meta

- **Sprint number:** 8
- **Book schema version:** 2
- **Start timestamp:** 2026-08-31T04:51:31Z
- **End timestamp:** (filled at Loop Phase)
- **Model:** claude-opus-4-8
- **Exit status:** in-progress
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** LLM-backed claim extractor: `LlmExtractor` calls the Claude Messages API (raw HTTP) to extract grounded factual claims from a paper's abstract as `Assertion<Candidate>`s that flow through the existing validate() gate; `diver extract` uses it by default with a `--deterministic` fallback.
- **Intents:** [INT-0009](../../intents/INT-0009-llm-claim-extractor.md)
- **Completion evidence:** (filled at Loop Phase)
