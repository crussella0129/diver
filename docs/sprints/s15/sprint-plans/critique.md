# Plan Critique — Sprint 15

## Concerns

### C-001: separating the contract change from its test migration would break intermediate commits
- **Where:** `build-plan.md` (initial 4-task split: code in T-1501, tests in T-1503)
- **Quote:** "Delete `parse_claim_array`/`strip_fences`/`text`"
- **Failure mode:** hidden-dep
- **Why it matters:** the moment `parse_claims` moves to structured output, every existing
  text-envelope test (`parse_claims` unit tests, the Sprint-14 transport happy path,
  `llm_extract_pipeline.rs`) fails to pass (some fail to compile). If those migrations live in a
  later task, the T-1501/T-1502 commit boundaries are red.
- **Suggested response:** fix-in-plan — **applied.** Tests are folded into the code task they
  cover: T-1501 carries the substrate **and** the migration/extension of the extractor tests;
  T-1502 carries the config code **and** its config-resolution tests; T-1503 is docs only. Every
  commit boundary stays green. (4 tasks → 3.)

### C-002: config-resolution tests must not mutate the global process environment
- **Where:** `test-plan.md` T-1502 config tests / `INT-0016` AC4
- **Quote:** "the active provider (from `DIVER_PROVIDER` or the file's `active`)"
- **Failure mode:** flake-risk
- **Why it matters:** reading `DIVER_PROVIDER`/`DIVER_PROVIDERS_CONFIG`/`api_key_env` from the
  real environment inside tests requires `std::env::set_var` (unsafe in edition 2024) and is
  racy under the parallel runner.
- **Suggested response:** fix-in-plan — **applied.** Config resolution is factored as a pure
  helper `resolve_provider(file_contents, selection, key_lookup)` with env/file injected as
  arguments; `from_env` is the thin wrapper that supplies the real env/file. Tests drive the pure
  helper deterministically (a temp/string doc + a closure `key_lookup`), no global-env mutation.
  The thin `from_env` wiring is the same kind of residual as INT-0015's `ANTHROPIC_BASE_URL` read.

### C-003: OpenAI `response_format: json_schema` may not be honored by every OpenAI-compatible target
- **Where:** `build-plan.md` T-1501 (OpenAI shape) / research §5
- **Quote:** "`response_format` json_schema structured output … supported by OpenAI, Grok, and llama.cpp/Ollama/vLLM"
- **Failure mode:** intent-drift (portability assumption)
- **Why it matters:** an older or minimal OpenAI-compatible server might ignore `response_format`
  and return prose, which would then fail parsing.
- **Suggested response:** defer-with-rationale — `response_format`/grammar-constrained output is
  exactly Animus_Ferric's design point (harness-owned JSON-Schema decoding) and is broadly
  supported by the named targets; parsing a non-conforming response yields a clean `Err` (AC3),
  not a silent wrong result. Function-calling `tools` is recorded as the fallback contract
  (INT-0016 Alternatives) for a future target that lacks structured outputs; not needed for the
  named set now.

## Confidence
proceed-with-caveats
