Finalized - DO NOT EDIT

# Sprint 15 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC1: Anthropic tool-use request + parse | T-1501 / WHEN Anthropic THEN /v1/messages + tool_use | `test_extract_anthropic_tool_use` |
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC2: OpenAI structured request + parse | T-1501 / WHEN OpenAiCompatible THEN /v1/chat/completions + response_format | `test_extract_openai_structured` |
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC3: error paths + grounding both shapes | T-1501 / WHEN unparseable/missing THEN Err; grounding drops ungrounded | `test_extract_anthropic_error`, `test_extract_openai_error`, `test_parse_claims_grounded`, `_drops_hallucinated`, `test_parse_claims_requires_structured` |
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC4: config resolution + back-compat | T-1502 / WHEN config exists/absent, key env | `test_resolve_provider_select`, `test_resolve_provider_key_env`, `test_resolve_provider_no_file_fallback`, `test_resolve_provider_missing_key_errors` |
| [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) | AC5: heuristics gone + docs + no regression | T-1501/T-1503 + full suite | fence/prose tests removed; `cargo test --workspace`; README/INT-0009 updated |

## Unit Tests

### T-1501 unit tests (`diver-core/src/extract.rs`)
- **Intent:** [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md)
- `test_parse_claims_grounded`, `_drops_hallucinated`, `_empty` — migrated to structured
  payloads; grounding admits only grounded quotes.
- `test_parse_claims_requires_structured`: a response missing the tool_use / message payload → `Err` (never panics).
- **removed:** `test_parse_claims_tolerates_fences`, `_tolerates_prose` (behavior deleted).
- Unchanged: `test_is_grounded_whitespace_and_case`.

### T-1502 unit tests (`diver-core/src/extract.rs`)
- **Intent:** [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md)
- Config resolution is a **pure helper** — `resolve_provider(file_contents: Option<&str>,
  selection: Option<&str>, key_lookup: impl Fn(&str)->Option<String>)` — so env/file are passed
  in, not read from the racy process environment.
- `test_resolve_provider_select`: a JSON providers doc + selection (via arg standing in for
  `DIVER_PROVIDER`/`active`) resolves the right `shape`/`base_url`/`model`.
- `test_resolve_provider_key_env`: `api_key` comes from the entry's `api_key_env` via `key_lookup`.
- `test_resolve_provider_no_file_fallback`: `None` file → anthropic-shape fallback from the
  `ANTHROPIC_*` lookups.
- `test_resolve_provider_missing_key_errors`: selected provider whose `api_key_env` yields
  `None` → actionable `Err` naming the env var.

## Integration / Transport Tests (`diver-core/src/extract.rs` tests, `#[tokio::test]` + wiremock)
- **Intents:** [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md)
- `test_extract_anthropic_tool_use`: `from_config(shape=Anthropic, base_url=server.uri())`; mock
  `POST /v1/messages` → 200 with a `tool_use` envelope (grounded + hallucinated claim). Assert the
  grounded candidate is returned and the request declared the `record_claims` tool with the
  `x-api-key`/`anthropic-version` headers.
- `test_extract_openai_structured`: `from_config(shape=OpenAiCompatible, base_url=server.uri())`;
  mock `POST /v1/chat/completions` → 200 with `choices[0].message.content` = the claims JSON.
  Assert the grounded candidate is returned and the request carried `Authorization: Bearer` and a
  `json_schema` `response_format`. **This doubles as the Animus_Ferric / Grok / local-server proof.**
- `test_extract_anthropic_error` / `test_extract_openai_error`: non-2xx → `Err` with status + body.

## Integration (`diver-core/tests/llm_extract_pipeline.rs`)
- Canned body → a structured (tool_use) envelope; still asserts grounding admits only the
  grounded claim and it validates to `Supported`.

## End-to-End Tests
- **Status:** possible (offline)
- The per-shape wiremock tests are the offline end-to-end of each provider contract; the
  OpenAI-shape test doubles as the Animus_Ferric/Grok/local-server end-to-end (same contract,
  `base_url` = mock). Live runs (real Anthropic/OpenAI keys, or a real `ferric server`) remain a
  manual check.
