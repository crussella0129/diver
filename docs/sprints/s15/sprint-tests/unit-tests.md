# Sprint 15 Unit Tests

- **Tested head:** `19ee28db6feff91e27f6f522f138078053011b53`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets`
- **Result:** `diver_core` lib unittests — **118 passed; 0 failed**; `diver-cli` bin — 1
  passed. Clippy: 0 warnings.

## Migrated / new (INT-0016)

### T-1501 — substrate + structured parsing (diver-core/src/extract.rs)
- **Intent:** [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) — AC1/AC2/AC3
- `test_parse_claims_grounded`, `_drops_hallucinated`, `_empty` — migrated to the Anthropic
  `tool_use` structured envelope; grounding admits only grounded quotes. **pass**
- `test_parse_claims_requires_structured`: non-JSON, a text-only block (no `tool_use`), and a
  `tool_use` whose input fails the schema all → `Err` (never panics). **pass** (AC3)
- **removed:** `test_parse_claims_tolerates_fences`, `_tolerates_prose` (heuristics deleted).
- `test_anthropic_config_from_env`: key required (missing/blank → actionable error); model +
  base URL defaults/overrides/blank-fallback; shape is `Anthropic`. **pass**
- `test_is_grounded_whitespace_and_case` — unchanged. **pass**

### T-1502 — provider config resolution (diver-core/src/extract.rs)
- **Intent:** [INT-0016](../../../intents/INT-0016-structured-claim-extraction.md) — AC4
- Resolution is a **pure helper** `resolve_provider(file_contents, selection, env_lookup)` —
  env/file injected, so no global-env mutation.
- `test_resolve_provider_select`: active-from-file → `anthropic`/claude; `DIVER_PROVIDER`-style
  override → `openai`/animus (`http://127.0.0.1:8080`, `local-gguf`). **pass**
- `test_resolve_provider_key_env`: `api_key` read from the entry's `api_key_env`. **pass**
- `test_resolve_provider_no_file_fallback`: `None` file → Anthropic env fallback (model from
  `DIVER_MODEL`); missing key → `Err`. **pass**
- `test_resolve_provider_missing_key_errors`: selected provider whose `api_key_env` is unset →
  `Err` naming the var. **pass**

## Raw result
```
cargo clippy --workspace --all-targets  →  0 warnings
Running unittests src\lib.rs (diver_core)  →  118 passed; 0 failed
Running unittests src\main.rs (diver-cli)  →    1 passed; 0 failed
```
