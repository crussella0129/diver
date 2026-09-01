# Sprint 11 End-to-End Tests

- **Status:** possible (offline)
- **Tested head:** `de281c76affc76e696f03f55331b8ae3de0c8aeb`
- **Runner:** scripted CLI smokes via `cargo run -p diver-cli`.

## `e2e_dive_help`
- **Intent:** [INT-0012](../../../intents/INT-0012-graph-dive.md) — AC5
- **Commands / observed:**
  - `diver --help` → Commands list includes
    `dive  Explore a concept: papers that assert about it and how they connect`.
  - `diver dive --help` → `Usage: diver.exe dive <CONCEPT>`, argument `<CONCEPT>`.
- **Result:** pass — the command is wired and documented.

## `e2e_dive_no_data`
- **Intent:** [INT-0012](../../../intents/INT-0012-graph-dive.md) — AC4 (empty path)
- **Command:** `diver dive zzznonexistentconcept`
- **Observed:** `Dive: zzznonexistentconcept` / `No papers assert about
  'zzznonexistentconcept'. Run \`diver extract <id>\` first.`; exit code `0`.
- **Result:** pass — when nothing asserts about the concept, `dive` prints the
  actionable message and exits 0 (empty is not an error).

## Coverage note
The populated-neighborhood path (`dive` over a corpus with extracted assertions)
is exercised deterministically at the library level by `test_dive_pipeline`
(seed → `compute_relations` → `build_dive` → a node with claim + related paper).
A full binary run would require a seeded real DB; the CLI surface + empty path
(binary) + library neighborhood assembly cover AC4/AC5.
