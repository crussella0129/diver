Finalized - DO NOT EDIT

# Sprint 13 Build Plan

## Intents
- [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) — state: planned; acceptance criteria covered: AC1, AC2, AC3, AC4, AC5, AC6

## Schema Tree
- Sprint Goal: weight co-assertion edges by TF-IDF and expose an adjustable temperature dial
  - Core weighting (diver-core)
    - T-1301: IDF weighting + temperature threshold in `compute_coassertion_relations` (+ `CoAssertion { term, weight }` enum + display arm)
  - CLI surface (diver-cli)
    - T-1302: `--temperature` flag on `diver dive`
  - Docs + pipeline verification
    - T-1303: README + integration test

## Execution Sequence

### T-1301: IDF-weight co-assertion terms and gate them by a temperature threshold
- **Intent:** [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md)
- **Touches:** `diver-core/src/graph.rs`, `diver-core/src/display.rs`, `diver-core/tests/coassertion.rs` (compile-forced migration of the existing `test_coassertion_pipeline` to the new signature/enum — keeps the workspace compiling)
- **Depends on:** (none)
- **Acceptance criterion:** AC1 (IDF weight + keep rule), AC2 (temperature endpoints + monotonic), AC3 (N≤2 guard), AC5 (weight shown)
- **Success criterion (EARS):**
  - **WHEN** two distinct papers share a significant term whose normalized IDF weight `w = ln(N/df)/ln(N/2)` satisfies `w >= 1.0 - temperature`, **THEN** `compute_coassertion_relations` **SHALL** emit exactly one `CoAssertion { term, weight: w }` edge for that term (terms sorted, no self-edges, one edge per term per pair).
  - **WHEN** `temperature == 1.0`, **THEN** `compute_coassertion_relations` **SHALL** emit an edge for every shared term; **WHEN** `temperature == 0.0`, **THEN** it **SHALL** emit an edge only for terms with `df == 2`.
  - **WHEN** a term is emitted at temperature `t`, **THEN** it **SHALL** also be emitted at every `t' >= t` (monotonic non-decreasing edge set).
  - **WHEN** the corpus has `N <= 2` distinct papers, **THEN** every shared term **SHALL** be emitted with `weight = 1.0` at any temperature, with no NaN or infinity.
  - **WHEN** `relation_reason` is given a `CoAssertion { term, weight }`, **THEN** it **SHALL** return a string containing `term` and the weight formatted to two decimals.
- **Notes:** Change `RelationKind::CoAssertion(String)` → `CoAssertion { term: String, weight: f64 }`; update the exhaustive `relation_reason` match arm and every `CoAssertion(..)` match/construction (the graph.rs unit tests **and** the existing `test_coassertion_pipeline` in `tests/coassertion.rs`) in the same task — the enum change is compile-forced crate-wide, so all references migrate together to keep the workspace green. Build `df: HashMap<&str, usize>` counting each deduped term once per paper; reuse the existing first-seen grouping + `dedup_preserve_order`. Defensively clamp `temperature` to `[0.0, 1.0]`. `significant_terms`/`STOPWORDS` unchanged.

### T-1302: expose the temperature dial on `diver dive`
- **Intent:** [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md)
- **Touches:** `diver-cli/src/main.rs`
- **Depends on:** T-1301
- **Acceptance criterion:** AC4 (CLI flag, validated/clamped, structural edges ungated)
- **Success criterion (EARS):**
  - **WHEN** `diver dive <concept> --temperature <t>` runs with `t` in `[0.0, 1.0]`, **THEN** the handler **SHALL** compute co-assertion edges at `t` while leaving the structural (category/author) edges ungated.
  - **WHEN** `--temperature` is omitted, **THEN** the handler **SHALL** use `0.5`.
  - **WHEN** `t` is outside `[0.0, 1.0]` or is NaN, **THEN** the command **SHALL** exit with a non-zero status and a clear error message.
- **Notes:** Add `#[arg(long, default_value_t = 0.5, value_parser = parse_temperature)] temperature: f64` to the `Dive` variant; `parse_temperature(&str) -> Result<f64, String>`. Thread `temperature` into `compute_coassertion_relations` at main.rs:203. Only the co-assertion `.extend(...)` call takes the dial.

### T-1303: document the flag and verify the low-vs-high-temperature pipeline
- **Intent:** [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md)
- **Touches:** `README.md`, `diver-core/tests/coassertion.rs`
- **Depends on:** T-1301, T-1302
- **Acceptance criterion:** AC4 (documented default), AC6 (docs + no regression)
- **Success criterion (EARS):**
  - **WHEN** the corpus has papers sharing both a rare term (low `df`) and a corpus-ubiquitous term (high `df`), **THEN** a low-temperature run **SHALL** yield only the rare-term edge while a high-temperature run **SHALL** yield both.
  - **WHEN** the README is read, **THEN** it **SHALL** document `--temperature` (meaning, default `0.5`, and `1.0` = the Sprint 12 unweighted behavior).
- **Notes:** Add the NEW `test_coassertion_temperature_pipeline` (the existing `test_coassertion_pipeline` was already migrated in T-1301). Seeds ≥3 papers so document frequency separates the rare from the common term; assert the edge sets at a low and a high temperature through `compute_coassertion_relations` + `build_dive`.
