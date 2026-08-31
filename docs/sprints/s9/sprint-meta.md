# Sprint 9 Meta

- **Sprint number:** 9
- **Book schema version:** 2
- **Start timestamp:** 2026-08-31T18:30:41Z
- **End timestamp:** (filled at Loop Phase)
- **Model:** claude-opus-4-8
- **Exit status:** in-progress
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Persist the epistemic layer: `assertions` + `assertion_support` SQLite tables, `Store::save_assertions(&[Assertion<Supported>])` (idempotent per paper+version, validated-only) and `get_assertions`; `diver extract` now persists and `diver assertions <id>` reads them back.
- **Intents:** [INT-0010](../../intents/INT-0010-persist-epistemic-layer.md)
- **Completion evidence:** (filled at Loop Phase)
