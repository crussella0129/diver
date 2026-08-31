# Sprint 8 End-to-End Tests

- **Status:** possible (offline paths in the suite; live LLM is a manual check)
- **Tested head:** `4d73c28491ec40da62c0d61523f63fc2cc60d2cc`
- **Runner:** scripted CLI smokes via `cargo run -p diver-cli`.

## `e2e_extract_help`
- **Intent:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) — AC5
- **Command:** `diver extract --help`
- **Observed:** `Usage: diver.exe extract [OPTIONS] <ARXIV_ID>`; option
  `--deterministic  Use the offline sentence-splitter instead of the Claude API
  (no key needed)`.
- **Result:** pass — the flag is wired and documented in help.

## `e2e_extract_deterministic_no_key`
- **Intent:** [INT-0009](../../../intents/INT-0009-llm-claim-extractor.md) — AC5 (offline branch + error path)
- **Command:** `ANTHROPIC_API_KEY= diver extract 9999.99999 --deterministic`
- **Observed:** `Error: Paper not found: 9999.99999`; exit code `1`.
- **Result:** pass — with **no** API key set, `--deterministic` makes no network
  call and reaches the store lookup (erroring on the unknown id like `inspect`),
  rather than failing with a missing-key error. Proves the offline branch bypasses
  the key requirement and the error path works.

## Live LLM run (manual — not in the automated suite)
Verifying the real HTTP call requires a valid `ANTHROPIC_API_KEY`, network, and
spends money, and returns non-deterministic output. It is therefore excluded from
`cargo test`. Manual check (for the operator):

```sh
export ANTHROPIC_API_KEY=sk-ant-...
diver ingest 1706.03762            # seed a paper
diver extract 1706.03762           # LLM extraction, default claude-opus-5
DIVER_MODEL=claude-haiku-4-5 diver extract 1706.03762   # cheaper model
```
Expected: a list of "Supported assertions" — grounded claims from the abstract,
each with its source provenance. The automated `test_llm_extract_pipeline` covers
everything from the API response body onward; only the socket call itself is
outside the suite.
