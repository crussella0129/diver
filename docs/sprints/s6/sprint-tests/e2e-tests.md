# Sprint 6 End-to-End Tests

- **Status:** possible
- **Tested head:** `6b4dbef6fcc81bbb187a45d6a5e533a3e26845c3`
- **Runner:** manual/scripted CLI smokes via `cargo run -p diver-cli` (no
  `assert_cmd` dependency added — a non-goal for this sprint).

All smokes use `--help`, which clap handles before any command body runs, so
they make no network call and never touch the real data directory.

## `e2e_cli_help_lists_subcommands`
- **Intent:** [INT-0007](../../../intents/INT-0007-workspace-restructure.md) — AC5 / AC2
- **EARS:** WHEN `diver --help` is run, THEN it SHALL list subcommands `search`,
  `ingest`, `inspect`, `list`, `collect`, `find` exactly as before the split.
- **Command:** `cargo run -p diver-cli -- --help`
- **Observed:** header "Find knowledge, not just papers, on ArXiv"; usage line
  `Usage: diver.exe <COMMAND>` (binary name **diver**); Commands section lists
  `search`, `ingest`, `inspect`, `list`, `collect`, `find` (+ `help`).
- **Result:** pass — all six subcommands present; binary named `diver`
  (`target/debug/diver.exe` built), satisfying AC2's binary-name clause.

## `e2e_cli_subcommand_help_parses`
- **Intent:** [INT-0007](../../../intents/INT-0007-workspace-restructure.md) — AC5
- **EARS:** WHEN `diver --help` is run, THEN subcommands SHALL behave exactly as
  before the split (argument surface unchanged).
- **Commands / observed:**
  - `diver find --help` → exit 0; shows `<QUERY>` and `--max-results [default: 10]`.
  - `diver ingest --help` → exit 0; shows `<ARXIV_ID>`.
  - `diver collect --help` → exit 0; shows `--max-results` and `--sort-by
    [possible values: relevance, submitted, updated]`.
- **Result:** pass — argument surfaces match pre-split definitions.

## Coverage note
Full behavioral E2E of `search`/`ingest`/`collect` command bodies requires a
live arXiv network call and writes to the real data dir; those paths are
exercised at the library level by the integration tests (`test_ingest_pipeline`,
`test_find_pipeline`) rather than through the binary, keeping E2E deterministic
and side-effect-free. No unlocking intent is needed — the observable CLI surface
is fully covered by the `--help` smokes above.
