# Sprint 12 End-to-End Tests

- **Status:** possible (offline)
- **Tested head:** `473197e821fdbd1a6b5106c5a810338c5caa2030`
- **Runner:** scripted CLI smoke via `cargo run -p diver-cli`.

## `e2e_dive_unchanged_surface`
- **Intent:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md) — AC6 (no regression)
- **Command:** `diver dive somethingzzz`
- **Observed:** `Dive: somethingzzz` / `No papers assert about 'somethingzzz'. Run
  \`diver extract <id>\` first.`; exit code `0`.
- **Result:** pass — the `dive` command still runs and its no-data path is
  unchanged after wiring in co-assertion edges. `diver dive --help` surface is
  unchanged (still `<CONCEPT>`).

## Coverage note
The populated co-assertion path (`dive` over a corpus where papers co-assert a
term) is exercised deterministically at the library level by
`test_coassertion_pipeline` — the exact `all_claims` →
`compute_coassertion_relations` → `build_dive` flow the handler runs. A full
binary run over a seeded corpus is out of the automated suite (needs a seeded real
DB — same rationale as INT-0012).
