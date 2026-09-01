# Sprint 15 Research Report

## Intents Reviewed
- [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) — created, then broadened; relevance: primary; current state: proposed
- [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) — selected/revised; relevance: the extractor being generalized; current state: realized
- [INT-0015](../../../intents/INT-0015-harden-extractor-http-boundary.md) — selected; relevance: the wiremock harness + injectable base_url this builds on; current state: realized

## 1. Sprint Goal

Turn the Anthropic-only, fence-tolerant extractor into an **agent-agnostic structured
extractor substrate**: providers are selected by **runtime config** (hot-loadable), and the
extractor speaks two compiled *shapes* — **`anthropic`** (Messages API + forced `record_claims`
tool-use) and **`openai`** (Chat Completions + `response_format` json_schema structured
outputs). The OpenAI shape, via config, covers OpenAI, Grok, and the user's **Animus_Ferric**
local models. Grounding and the typestate `validate` gate are unchanged. Advances **INT-0016**.
Baseline: `5f6a7d7`, `cargo test --workspace` green (126), clippy 0.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/extract.rs | high | `LlmExtractor { http, model, api_key, base_url }` (INT-0015) + async `extract` builds the Anthropic request and parses a text block via `parse_claim_array`/`strip_fences`. Generalize: `ProviderShape` + `ProviderConfig` + `from_config`; per-shape request/parse; delete the fence heuristics + `text`. Grounding (`is_grounded`/`normalize`), `ClaimJson`, redacting `Debug`, and `--deterministic` unchanged. |
| diver-core/src/extract.rs (tests) | high | wiremock harness (INT-0015) + `envelope()` helper. Add per-shape structured envelopes + config-resolution tests; remove fence/prose tolerance tests. |
| diver-core/tests/llm_extract_pipeline.rs | high | Canned text-block body → migrate to a structured (tool_use) envelope; grounding assertions stay. |
| diver-core/Cargo.toml | low | `serde_json` + `dirs` already present → JSON providers-config needs **no new dependency**. |
| docs/intents/INT-0009-*.md | medium | Parsing-related Consequences amended to the structured, multi-provider contract. |

Baseline: workspace at `5f6a7d7`. green (126); clippy 0.

## 3. Animus_Ferric investigation (the decisive finding)

The user asked me to determine, from GitHub, what integrating **Animus_Ferric** requires.
Findings (`crussella0129/Animus_Ferric`, docs read via the GitHub API):

- Animus_Ferric is *"a local-first agentic coding harness written in Rust, purpose-built for
  small local models (1B–14B GGUF)"* with **harness-owned constrained decoding (JSON-Schema /
  regex / CFG grammars)**.
- Its recommended, default inference path is the **`backend-openai`** feature — *"the
  OpenAI-compatible HTTP valve — the constrained-decoding path. Talks to llama.cpp
  (llama-server), Ollama, or vLLM."*
- `ferric server up --engine llama-server --model <gguf>` launches **llama-server on
  `127.0.0.1:8080`**, an OpenAI-compatible server (writes `.ferric/server.json`).

**Conclusion:** Animus_Ferric needs **no bespoke adapter**. It is reached through the same
**OpenAI-compatible** shape as OpenAI (`api.openai.com`) and Grok (`api.x.ai`) — just pointed
at the local `ferric` server (`http://127.0.0.1:8080`, `openai` shape, model = the loaded
GGUF, key usually empty). llama.cpp's OpenAI server honors `response_format` json_schema /
grammar constraints, which is exactly Animus_Ferric's constrained-decoding design — so the
OpenAI shape's structured-output request is the right contract for it. This makes the
extractor's "main value-add" (works with Claude, OpenAI/Codex, Grok, or Animus_Ferric) a
matter of config over two compiled shapes.

## 4. External Sources
- [crussella0129/Animus_Ferric README + docs/getting-started.md](https://github.com/crussella0129/Animus_Ferric) — the OpenAI-compatible `ferric server` / llama-server-on-127.0.0.1:8080 facts above.
- [Anthropic tool use](https://docs.anthropic.com/en/docs/build-with-claude/tool-use) — `tools` + forced `tool_choice` → a `tool_use` block whose `input` is schema-validated JSON.
- [OpenAI structured outputs (`response_format: json_schema`)](https://platform.openai.com/docs/guides/structured-outputs) — strict JSON-schema-constrained assistant output in `choices[0].message.content`; the Chat Completions contract Grok and llama.cpp/Ollama/vLLM also implement.

## 5. Risks / Unknowns / Dependencies
- **Two contracts to build + all envelope-shaped tests migrate.** Bounded; the wiremock
  harness (INT-0015) makes both shapes testable offline. A per-shape structured-envelope
  helper centralizes the shapes.
- **Removing fence/prose tolerance is a deliberate behavior change** (structured output makes
  a non-conforming response an error, not something to salvage). `--deterministic` remains the
  offline fallback.
- **OpenAI structured-output portability.** `response_format: json_schema` is supported by
  OpenAI, Grok, and llama.cpp/Ollama/vLLM (Animus_Ferric); function-calling `tools` is the
  fallback if a target lacks it. Chosen: `response_format` (cleanest; `message.content` is the
  JSON). Recorded as a decision.
- **Config = no new dependency.** JSON via `serde_json`; path via `dirs::config_dir()` (both
  already deps). Keys never stored in the file — each provider names an `api_key_env`.
- **Back-compat.** No providers config ⇒ `from_env` behaves exactly as today (anthropic shape
  from `ANTHROPIC_API_KEY`/`DIVER_MODEL`/`ANTHROPIC_BASE_URL`).
- **Scope.** Substrate + two shapes + config + tests + docs is a large sprint; the plan notes
  a clean split point (config/docs → Sprint 16) if desired.

## 6. Recommended Approach

Introduce `ProviderShape` (`Anthropic`, `OpenAiCompatible`) + `ProviderConfig` + `from_config`;
build/parse per shape over shared `ClaimJson`/grounding/`validate`; delete the fence heuristics.
Add a hot-loadable JSON providers config (`{config_dir}/diver/providers.json`, `DIVER_PROVIDER`/
`DIVER_PROVIDERS_CONFIG`, `api_key_env`) with a no-file fallback to today's Anthropic env path.
Cover both shapes with wiremock (the OpenAI-shape-against-mock test doubles as the
Animus_Ferric/Grok/local-server proof) plus config-resolution tests. Document the config with a
worked example per provider, including Animus_Ferric via `ferric server up`.

### Referenced artifacts
- [INT-0016 chapter](../../../intents/INT-0016-structured-claim-extraction.md)
- Build/test plans: `../sprint-plans/`
- Baseline evidence: `cargo test --workspace` 126/126, clippy 0 at `5f6a7d7`
