Finalized - DO NOT EDIT

# Sprint 8 Build Plan

## Intents
- [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) — state: planned; acceptance criteria covered: AC2/AC3 (T-801), AC1/AC4 (T-802), AC5/AC6 (T-803)

## Schema Tree
- Sprint Goal: LLM-backed claim extractor feeding the existing validate() gate
  - Extraction core (INT-0009)
    - T-801: `parse_claims` + grounding (pure, tested)
    - T-802: `LlmExtractor` (env config + async HTTP)
  - Wiring (INT-0009)
    - T-803: `diver extract --deterministic` + docs + pipeline integration test

## Execution Sequence

Bottom-up, non-determinism isolated at the network edge: build and test the pure
parser first, then the thin async HTTP wrapper, then the CLI.

### T-801: `extract.rs` core — `parse_claims` + grounding
- **Intent:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md)
- **Touches:** diver-core/src/extract.rs (new), diver-core/src/lib.rs (`pub mod extract`)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0009 AC2 (pure fixture-tested parser), AC3
  (grounding drops hallucinations).
- **Success criterion (EARS):**
  - **WHEN** `parse_claims` receives a Messages-API body whose claims have quotes
    present in the abstract, **THEN** it **SHALL** return one
    `Assertion<Candidate>` per grounded claim (claim text + supporting
    `Observation` of the quote).
  - **WHEN** a claim's quote is not found in the abstract, **THEN** `parse_claims`
    **SHALL** drop that claim.
  - **WHEN** the model text is wrapped in ``` fences or surrounding prose, **THEN**
    `parse_claims` **SHALL** still parse the JSON array, and **WHEN** the text is
    unparseable it **SHALL** return `Err` (never panic).
- **Notes:** serde structs `MessagesResponse { content: Vec<ContentBlock> }`,
  `ContentBlock { #[serde(rename="type")] kind: String, #[serde(default)] text:
  String }`, `ClaimJson { claim: String, quote: String }`. Extract the first
  `text`-kind block (via serde, not heuristic). Parse strategy (per critique
  C-002): try `serde_json::from_str::<Vec<ClaimJson>>` on the block text
  **directly first**; only if that fails, strip ``` fences and slice the outer
  `[`...`]`, then parse again; if still unparseable, return `Err` (no panic).
  Direct-parse-first means brackets inside claim/quote text are handled by serde,
  not the slicer. Grounding helper `is_grounded(quote,
  summary)` = case-insensitive, whitespace-normalized substring. Provenance:
  `ArxivId::new(fact.arxiv_id.clone())`, `ArxivVersion::parse(&fact.arxiv_version)`.
  System/user prompt text as `const`s.

### T-802: `LlmExtractor` — env config + async HTTP call
- **Intent:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md)
- **Touches:** diver-core/src/extract.rs
- **Depends on:** T-801
- **Acceptance criterion:** INT-0009 AC1 (calls Messages API, returns candidates),
  AC4 (model default + key from env).
- **Success criterion (EARS):**
  - **WHEN** `LlmExtractor::from_env` runs with `ANTHROPIC_API_KEY` unset or blank,
    **THEN** it **SHALL** return an actionable error naming `ANTHROPIC_API_KEY`.
  - **WHEN** a key is present, **THEN** it **SHALL** select `DIVER_MODEL` if set,
    else `claude-opus-5`.
- **Notes:** testability split — `fn build(api_key: Option<String>, model:
  Option<String>) -> Result<LlmExtractor>` holds the logic (unit-tested with
  args, no global env mutation); `from_env()` = `build(env::var("ANTHROPIC_API_KEY")
  .ok(), env::var("DIVER_MODEL").ok())`; blank key treated as absent. `extract()`:
  build request body via `serde_json::to_string`, `http.post("https://api.anthropic.com/v1/messages")
  .header("x-api-key", &self.api_key).header("anthropic-version","2023-06-01")
  .header("content-type","application/json").body(body).send().await`; on non-2xx
  return an error including the status; else `.text().await` → `parse_claims`.
  `http` built like `client.rs` (`Client::builder().timeout(...).build()`).

### T-803: CLI wiring (`--deterministic`) + docs + pipeline integration test
- **Intent:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md)
- **Touches:** diver-cli/src/main.rs (`Extract { arxiv_id, deterministic }`),
  README.md, diver-core/tests/llm_extract_pipeline.rs (new)
- **Depends on:** T-801, T-802
- **Acceptance criterion:** INT-0009 AC5 (CLI default LLM + `--deterministic` +
  error path), AC6 (existing `validate` gate unchanged; all tests pass).
- **Success criterion (EARS):**
  - **WHEN** `diver extract <id>` runs without `--deterministic`, **THEN** it
    **SHALL** use `LlmExtractor` (network); with `--deterministic` it **SHALL** use
    the INT-0008 sentence-splitter; an unknown id **SHALL** error like `inspect`.
  - **WHEN** a fixture Messages-API body flows through `parse_claims` then
    `validate`, **THEN** the grounded claims **SHALL** become
    `Vec<Assertion<Supported>>` with provenance intact.
- **Notes:** `Extract { arxiv_id: String, #[arg(long)] deterministic: bool }`.
  Handler mirrors `Commands::Inspect` load/`bail!("Paper not found: {arxiv_id}")`.
  Default branch: `let extractor = LlmExtractor::from_env()?; let candidates =
  extractor.extract(&fact).await?;`. `--deterministic` branch: `candidate_assertions(&extract_observations(&fact))`.
  Both: `.into_iter().filter_map(|c| c.validate().ok()).collect()` →
  `display::display_extract(&fact.arxiv_id, &supported)`. README: document `diver
  extract`, `--deterministic`, `ANTHROPIC_API_KEY`, `DIVER_MODEL`.
