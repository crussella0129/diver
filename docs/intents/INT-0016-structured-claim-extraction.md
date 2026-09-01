# INT-0016 — Agent-agnostic structured claim extraction

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0016
- **State:** realized
- **Work evidence:** [Sprint 15 build plan](../sprints/s15/sprint-plans/build-plan.md) (T-1501, T-1502, T-1503)
- **Completion evidence:** [T-1501/T-1502/T-1503 completion](../work/completed-tasks.md#t-1501--sprint-15)
- **Code evidence:** [diver-core/src/extract.rs](../../diver-core/src/extract.rs)
- **Test evidence:** [Sprint 15 test report](../sprints/s15/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md) (providers config + per-provider examples)

## Intent

Generalize the Anthropic-only, fence-tolerant extractor ([[llm-claim-extractor]], INT-0009)
into an **agent-agnostic structured extractor substrate**, so extraction works with Claude,
OpenAI/Codex, Grok, and the user's local **Animus_Ferric** ([[reference-animus-ferric]]) — the
main value-add — with providers **hot-loadable** from runtime config. Builds on the injectable
`base_url` + wiremock harness ([[harden-extractor-http-boundary]], INT-0015).

- **Two compiled shapes.** `ProviderShape::Anthropic` (Messages API `{base_url}/v1/messages`,
  `x-api-key`+`anthropic-version` headers, a forced `record_claims` **tool**; claims read from
  the `tool_use` block `input`) and `ProviderShape::OpenAiCompatible` (Chat Completions
  `{base_url}/v1/chat/completions`, `Authorization: Bearer`, `response_format` json_schema
  structured output; claims read from `choices[0].message.content`). Investigation shows
  Animus_Ferric's `ferric server` is an OpenAI-compatible llama.cpp server, so the OpenAI shape
  covers OpenAI, Grok, and Animus_Ferric by config alone.
- **Hot-loadable runtime config.** A `ProviderConfig { shape, base_url, model, api_key }` built
  by a front-end via `LlmExtractor::from_config`, or resolved from a JSON providers file
  (`{config_dir}/diver/providers.json`, override `DIVER_PROVIDERS_CONFIG`; active provider via
  `DIVER_PROVIDER` or the file's `active`). API keys are read from a named `api_key_env`, never
  stored in the file.
- **Structured, grounded, gated.** Both shapes yield `{ claim, quote }` objects validated
  against a shared JSON schema; grounding (quote must appear in the abstract) and the typestate
  `validate` gate are applied identically and unchanged.
- **Retire the heuristics.** The fence/prose-tolerant `parse_claim_array`/`strip_fences` are
  deleted — dead once output is structured.
- **Back-compat.** With no providers config, `from_env` behaves exactly as today (anthropic
  shape from `ANTHROPIC_API_KEY`/`DIVER_MODEL`/`ANTHROPIC_BASE_URL`).

Non-goals:
- No change to grounding, the `validate` gate, storage, the redacting `Debug`, or
  `--deterministic`.
- No fully data-driven provider templates — runtime config selects among *compiled* shapes
  (user-confirmed).
- No retries/streaming; no new dependency (`serde_json`/`dirs` already present).

## Acceptance criteria

1. `ProviderShape::Anthropic` builds a Messages request with a forced `record_claims` tool and
   parses claims from the response `tool_use` block `input`.
2. `ProviderShape::OpenAiCompatible` builds a Chat Completions request with a `Bearer` key and a
   `json_schema` `response_format`, and parses claims from `choices[0].message.content`.
3. Both shapes apply grounding (drop any claim whose quote is not in the abstract) and return
   `Err` (never panic) on an unparseable or structurally-missing response.
4. A providers config resolves the active provider (via `DIVER_PROVIDER`/`active`) to a
   `ProviderConfig` with `api_key` from the named `api_key_env`; with no config file, `from_env`
   falls back to today's Anthropic env behavior; a missing/blank required key errors actionably.
5. The fence/prose parser is removed; grounding + all previously passing behavior hold after
   migrating tests to structured envelopes; the README documents the config with a per-provider
   example including Animus_Ferric (`openai` shape → `ferric server` at `127.0.0.1:8080`).

## Rationale

The extractor's value multiplies if it is not tied to one vendor: the same grounded,
typestate-gated pipeline can run on a frontier API or a local constrained-decoding model. Two
compiled shapes (Anthropic Messages + OpenAI Chat Completions) cover the entire named set
because Grok and Animus_Ferric are OpenAI-compatible; runtime config over those shapes makes
providers swappable from a front-end without a rebuild. Structured outputs (tool-use /
`response_format`) also remove the fence heuristics INT-0009 shipped as a stopgap.

## Alternatives

- **Fully data-driven provider templates** — rejected (user-confirmed): a request/response
  templating + extraction engine is far more complex and less type-safe than compiled shapes
  chosen by config.
- **A bespoke Animus_Ferric adapter** — unnecessary: it is OpenAI-compatible via `ferric
  server`; the OpenAI shape covers it by config.
- **Keep a text-parsing fallback** — rejected: with structured output forced, a non-conforming
  response should error, not be salvaged; `--deterministic` is the offline fallback.
- **Function-calling `tools` for the OpenAI shape** — deferred alternative to `response_format`
  json_schema (chosen for the cleaner `message.content` payload); usable if a target lacks
  structured outputs.

## Consequences

- `extract` dispatches on `ProviderShape`; `ProviderConfig`/`from_config` are the hot-loadable
  seam; a JSON providers config + `DIVER_PROVIDER`/`DIVER_PROVIDERS_CONFIG` select providers;
  `parse_claim_array`/`strip_fences`/`text` are deleted.
- Extraction is provider-portable and schema-validated at the API boundary; non-conforming
  responses error cleanly. INT-0009's parsing/single-provider Consequences are amended.
- Keys remain in environment variables (named per provider), never in the config file.

## Transition history
- 2026-09-01: created as `proposed` (initial scope: Anthropic tool-use migration).
- 2026-09-01: broadened (still `proposed`) to an **agent-agnostic provider substrate**
  (Anthropic + OpenAI-compatible shapes, hot-loadable runtime config) after the user asked for
  OpenAI/Grok/Animus_Ferric support and a GitHub investigation showed Animus_Ferric is
  OpenAI-compatible via `ferric server`.
- 2026-09-01: `proposed` → `planned`; linked to Sprint 15 build plan (T-1501 shapes +
  dispatch + tests, T-1502 hot-loadable config + tests, T-1503 docs).
- 2026-09-01: `planned` → `active` (Sprint 15 build started; T-1501 first).
- 2026-09-01: `active` → `realized` (Sprint 15: provider substrate — `ProviderShape`
  Anthropic/OpenAiCompatible + `ProviderConfig`/`from_config` + hot-loadable `providers.json`
  (`resolve_provider`, `DIVER_PROVIDER`, `api_key_env`, Anthropic env fallback); structured
  output via Anthropic tool-use and OpenAI `response_format` json_schema (covers OpenAI/Grok/
  Animus_Ferric); fence heuristics deleted; grounding + validate unchanged. 128 tests pass,
  clippy 0. Deferred: OpenAI parse-error unit test, `from_env` file/env wiring test,
  `response_format` portability fallback — see test critique C-001/C-002/C-003).
