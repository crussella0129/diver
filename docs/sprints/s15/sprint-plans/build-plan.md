Finalized - DO NOT EDIT

# Sprint 15 Build Plan

## Intents
- [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) — state: planned; acceptance criteria covered: AC1, AC2, AC3, AC4, AC5

## Schema Tree
- Sprint Goal: agent-agnostic structured extractor substrate
  - Provider substrate + its tests (diver-core)
    - T-1501: `ProviderShape`/`ProviderConfig` + per-shape request/parse + migrated/added tests
  - Hot-loadable config + its tests (diver-core)
    - T-1502: runtime provider config + `from_env` back-compat + config tests
  - Docs
    - T-1503: provider config + per-provider examples (incl. Animus_Ferric)

> Note: tests are folded into the code task they cover (T-1501, T-1502) so every commit
> boundary stays green — the contract change and the migration of its now-broken existing
> tests must land together.

## Execution Sequence

### T-1501: provider substrate — shapes + per-shape dispatch (+ migrate/extend extract tests)
- **Intent:** [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md)
- **Touches:** `diver-core/src/extract.rs`, `diver-core/tests/llm_extract_pipeline.rs`
- **Depends on:** (none)
- **Acceptance criterion:** AC1 (Anthropic tool-use), AC2 (OpenAI structured), AC3 (error paths + grounding)
- **Success criterion (EARS):**
  - **WHEN** the shape is `Anthropic`, **THEN** `extract` **SHALL** POST `{base_url}/v1/messages` with `x-api-key`/`anthropic-version` headers and a forced `record_claims` tool, and **SHALL** parse claims from the response `tool_use` block `input`.
  - **WHEN** the shape is `OpenAiCompatible`, **THEN** `extract` **SHALL** POST `{base_url}/v1/chat/completions` with an `Authorization: Bearer` header and a `json_schema` `response_format`, and **SHALL** parse claims from `choices[0].message.content`.
  - **WHEN** either response is unparseable or missing its structured payload, **THEN** `extract` **SHALL** return `Err` (never panic); grounding and `validate` **SHALL** be applied identically for both shapes.
- **Notes:** Add `ProviderShape { Anthropic, OpenAiCompatible }`, `ProviderConfig { shape, base_url, model, api_key }`, and `LlmExtractor::from_config`. Refactor `extract` to build/parse per shape over a shared `ClaimJson`/claims-schema/`is_grounded`. Delete `parse_claim_array`/`strip_fences`/`text` (dead once structured). **Tests land here** (the contract change breaks the existing text-envelope tests, so they migrate in the same commit): migrate the `parse_claims` unit tests + `llm_extract_pipeline.rs` to structured envelopes; **remove** `test_parse_claims_tolerates_fences`/`_tolerates_prose`; add per-shape wiremock happy + error `#[tokio::test]`s (`test_extract_anthropic_tool_use`, `test_extract_openai_structured` — the latter also the Animus_Ferric/Grok/local-server proof, built via `from_config` with `base_url = server.uri()`) and per-shape "structurally-missing payload → Err" tests. Keep injectable `base_url`, redacting `Debug`, grounding, `--deterministic`.

### T-1502: hot-loadable runtime provider config + `from_env` fallback (+ config tests)
- **Intent:** [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md)
- **Touches:** `diver-core/src/extract.rs`
- **Depends on:** T-1501
- **Acceptance criterion:** AC4 (config resolution + back-compat + actionable key error)
- **Success criterion (EARS):**
  - **WHEN** a providers config file exists, **THEN** the active provider (from `DIVER_PROVIDER` or the file's `active`) **SHALL** resolve to a `ProviderConfig` whose `api_key` is read from the provider's named `api_key_env`.
  - **WHEN** no providers config file exists, **THEN** `from_env` **SHALL** fall back to the `anthropic` shape from `ANTHROPIC_API_KEY`/`DIVER_MODEL`/`ANTHROPIC_BASE_URL`.
  - **WHEN** the selected provider's key env is unset or blank, **THEN** construction **SHALL** return an actionable error naming the env var.
- **Notes:** JSON providers file via `serde_json`; path `dirs::config_dir()/diver/providers.json`, overridable by `DIVER_PROVIDERS_CONFIG`. Small config structs (`ProvidersFile { active, providers }`, `ProviderEntry { shape, base_url, model, api_key_env }`); map `shape` string → `ProviderShape`. Keys stay in env, never in the file. **Tests land here:** a `resolve_provider(config_path, selection, key_lookup)` pure helper (env/file passed in, not read globally) tested with a temp JSON file — selection via `active` and via override; `api_key_env` resolution; no-file fallback; missing-key error. (Pure-function seam avoids racy global-env mutation, per the plan-critic.)

### T-1503: docs — provider config format + per-provider examples
- **Intent:** [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md)
- **Touches:** `README.md`, `docs/intents/INT-0009-llm-claim-extractor.md`, `diver-core/src/extract.rs` (module doc)
- **Depends on:** T-1501, T-1502
- **Acceptance criterion:** AC5 (docs)
- **Success criterion (EARS):**
  - **WHEN** the README documents extraction, **THEN** it **SHALL** show the `providers.json` format and a worked example per provider — Claude (`anthropic`), OpenAI, Grok, and **Animus_Ferric** (`openai` shape, `http://127.0.0.1:8080` via `ferric server up`).
- **Notes:** Amend INT-0009's parsing/single-provider Consequences to point to INT-0016. Update the extract.rs `//!` module doc to describe the substrate (shapes + grounding + config).
