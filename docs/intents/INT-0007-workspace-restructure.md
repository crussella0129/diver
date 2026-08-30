# INT-0007 — Restructure into a Cargo workspace (diver-core + diver-cli)

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0007
- **State:** planned
- **Work evidence:** [Sprint 6 build plan](../sprints/s6/sprint-plans/build-plan.md) (T-604, T-605)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Split the single `diver` crate into two crates under a Cargo workspace:

- **`diver-core`** — the epistemic engine library: the current `client`,
  `display`, `fact`, `id`, `model`, `parse`, `query`, and `store` modules plus
  the embedded arXiv taxonomy.
- **`diver-cli`** — the thin binary crate holding the clap CLI (`main.rs`),
  producing a binary still named `diver`.

Preserve every current CLI behavior and the `diver` binary name. This carves the
core/CLI seam now — while the surface is small — so the coming semantic layer
(`Observation`, `Assertion`) grows inside `diver-core` without bloating the CLI.

Non-goals:
- No crates beyond `diver-core` and `diver-cli`. The architecture roadmap's
  later crates (`diver-arxiv`, `diver-parser`, `diver-graph`, `diver-server`, …)
  are deliberately *not* created yet — grow into them.
- No behavior, output, or dependency changes beyond what the split mechanically
  requires.
- No semantic-layer types.

## Acceptance criteria

1. The repository is a Cargo workspace whose members are `diver-core` and
   `diver-cli`.
2. `diver-core` is a library crate containing the current `src/` modules except
   `main.rs`; `diver-cli` is a binary crate that produces a binary named
   `diver`.
3. The embedded taxonomy (`include_str!`) resolves correctly from its new
   location inside `diver-core`.
4. `cargo build` and `cargo test` succeed from the workspace root, and all tests
   that passed before the split still pass (65 at sprint start).
5. `diver --help` and every subcommand — `search`, `ingest`, `inspect`, `list`,
   `collect`, `find` — behave exactly as before the split.
6. Integration tests under `tests/` reference the library as `diver_core::` and
   pass.

## Rationale

The architecture vision stages the crate split as `diver-core` + `diver-cli`
first, then evolves toward finer crates. Performing the split while the code is
~2.3k lines across nine modules is cheap; retrofitting a crate boundary after the
semantic layer is layered on top is expensive and error-prone. Doing it in the
same sprint as the provenance reconciliation ([[reconcile-review-hardening]])
keeps all "prepare the substrate before the semantic layer" work together.

## Alternatives

- **Keep a single crate with lib + bin (current layout)** — rejected: the vision
  explicitly stages the core/CLI split first, and it is cheapest to do now.
- **Split into every target crate at once** — rejected: the vision says not to
  create all crates on day one; premature boundaries add churn with no payoff
  until the modules that justify them exist.
- **Move `main.rs` into `diver-core` as a second target** — rejected: that keeps
  a single crate and does not establish the reusable library boundary the vision
  wants.

## Consequences

- Import paths change from `diver::` to `diver_core::` across `main.rs` and the
  integration tests.
- The taxonomy file moves under `diver-core/`, and `include_str!` in `id.rs` is
  repointed accordingly.
- The root `Cargo.toml` becomes a workspace manifest; `cargo build` / `cargo
  test` at the root continue to work unchanged for callers and CI.

## Transition history
- 2026-08-29: created as `proposed`.
- 2026-08-29: `proposed` → `planned`; linked to Sprint 6 build plan (T-604
  workspace + `diver-core` library, T-605 `diver-cli` binary).
