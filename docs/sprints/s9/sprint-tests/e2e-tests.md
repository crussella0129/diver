# Sprint 9 End-to-End Tests

- **Status:** possible (offline)
- **Tested head:** `cb613a38d985818e08ca2ed22b8781fa28411424`
- **Runner:** scripted CLI smokes via `cargo run -p diver-cli`.

## `e2e_assertions_help`
- **Intent:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) — AC5
- **Command:** `diver assertions --help`
- **Observed:** "Show the assertions previously extracted and stored for a paper";
  `Usage: diver.exe assertions <ARXIV_ID>`. `diver --help` lists `assertions`.
- **Result:** pass — the command is wired and documented.

## `e2e_assertions_unknown`
- **Intent:** [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) — AC5 (empty path)
- **Command:** `diver assertions 9999.99999`
- **Observed:** "Stored assertions for 9999.99999" / "No stored assertions for
  9999.99999."; exit code `0`.
- **Result:** pass — an unknown/empty paper prints a clean message and exits 0
  (empty is not an error), matching the AC5 requirement.

## Coverage note
The persist-then-read loop (`extract` → `save_assertions` → `assertions` →
`get_assertions`) is exercised deterministically at the library level by
`test_persist_pipeline`; the `Extract` handler's added
`store.save_assertions(...)?` call is over that same tested API. A full binary
round-trip (`diver extract <seeded-id> --deterministic` then `diver assertions
<id>`) would need a seeded real database and is not scripted here (critique
C-001); the CLI surface + empty path (binary) + library round-trip cover AC5.
