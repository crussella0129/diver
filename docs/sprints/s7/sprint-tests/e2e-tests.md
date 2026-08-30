# Sprint 7 End-to-End Tests

- **Status:** possible
- **Tested head:** `df09a674f89676b92c3abdbc2f8384f27df8c5fe`
- **Runner:** scripted CLI smokes via `cargo run -p diver-cli` (no `assert_cmd`
  dependency added).

## `e2e_extract_help`
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) — AC5
- **Commands / observed:**
  - `diver --help` → Commands list includes
    `extract  Extract supported assertions from a stored paper's abstract`.
  - `diver extract --help` → `Usage: diver.exe extract <ARXIV_ID>`, argument
    `<ARXIV_ID>`.
- **Result:** pass — the subcommand is wired and its argument surface is correct.
  Side-effect-free (`--help` only).

## `e2e_extract_unknown_errors`
- **Intent:** [INT-0008](../../../intents/INT-0008-typestate-assertion-core.md) — AC5 (error path)
- **Command:** `diver extract 9999.99999`
- **Observed:** `Error: Paper not found: 9999.99999`; exit code `1`.
- **Result:** pass — matches `inspect`'s `bail!` behavior on an unknown id.
  Read-only DB access (schema init only), no new stored state.

## Coverage note
The happy-path `diver extract <stored-id>` display is exercised at the library
level by `test_extract_pipeline` (the same
extract→candidate→validate→supported flow the handler runs) plus the
`display_extract` renderer; a full binary run against a stored paper would
require a pre-seeded real database and is not scripted here to keep the E2E
deterministic and side-effect-free. No unlocking intent is needed — the CLI
surface and error path are covered above.
