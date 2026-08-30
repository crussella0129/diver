Finalized - DO NOT EDIT

# Sprint 6 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC1: chapter records 8 fixes + links dd69859/PR#4 | Book/provenance step | `check-book.sh` valid + doc review of INT-0006 |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC2: FK constraint enforced | T-601 / WHEN insert bad `paper_id` THEN FK error | `test_fk_constraint_enforced` |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC3: no stale FTS on older reingest | T-602 / WHEN v2 then older v1 THEN find v2, not v1 | `test_reingest_older_version_keeps_latest_in_fts` |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC4: taxonomy cache determinism | T-603 / WHEN repeated parse THEN correct per code | `test_taxonomy_parse_repeated_consistent` |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC5: dev reconciled with main | git step | `git merge-base --is-ancestor origin/main dev` exits 0 |
| [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md) | AC6: existing tests still pass | all tasks | full `cargo test` green (≥65 pre-existing) |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC1: workspace, members core+cli | T-604 / WHEN `cargo build -p diver-core` THEN lib compiles | `cargo metadata` lists 2 members; build green |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC2: core=lib, cli=bin `diver` | T-604, T-605 / WHEN `cargo build` THEN `diver` binary | binary named `diver` present under `target/debug` |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC3: taxonomy include_str resolves | T-604 / WHEN build THEN include_str resolves | `test_taxonomy_valid_code` passes post-move |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC4: cargo build/test succeed, tests pass | T-604, T-605 | `cargo test` green at root (65 pre-existing + 3 new) |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC5: CLI parity | T-605 / WHEN `diver --help` THEN 6 subcommands | e2e smoke `test`-phase |
| [INT-0007](../../../intents/INT-0007-workspace-restructure.md) | AC6: integration tests use `diver_core::` and pass | T-604 / WHEN tests compile THEN `diver_core::` | `test_find_pipeline`, `test_ingest_pipeline`, `test_ingest_pipeline_multi_category` |

## Unit Tests

### T-601 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md)
- `test_fk_constraint_enforced`: direct INSERT into `paper_versions` with
  `paper_id = 99999` (no parent) → `Err`, message contains `FOREIGN KEY`.
- Stubs: none (uses `Store::open_in_memory` + private `conn`).

### T-602 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md)
- `test_reingest_older_version_keeps_latest_in_fts`: save v2 (`ingested_at`
  Tb=`2026-08-28T02:00:00Z`, summary `latestquantumfoo`), then save v1
  (`ingested_at` Ta=`2026-08-28T01:00:00Z`, summary `olderclassicbar`) →
  `search("latestquantumfoo")` len 1; `search("olderclassicbar")` empty.
- Stubs: none.

### T-603 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-reconcile-review-hardening.md)
- `test_taxonomy_parse_repeated_consistent`: 3 iterations asserting `cs.CV`,
  `math.NA`, `stat.ML` resolve to their taxonomy names and `invalid.XX` → `Err`
  on every pass.
- Stubs: none (embedded taxonomy).

### T-604 / T-605 verification (no new unit tests; existing suite is the guard)
- **Intent:** [INT-0007](../../../intents/INT-0007-workspace-restructure.md)
- The full pre-existing unit suite (62 tests) and the three new tests above must
  compile and pass under the new `diver-core` crate. `test_taxonomy_valid_code`
  specifically proves AC3 (include_str resolves post-move).

## Integration Tests

### Library pipeline integration (moves to `diver-core/tests/`)
- **Intents:** [INT-0007](../../../intents/INT-0007-workspace-restructure.md) (AC6)
- `test_find_pipeline`: save 3 facts → `search("attention")` returns expected
  ids; `max_results` honored; unknown term empty. Must compile under
  `use diver_core::`.
- `test_ingest_pipeline`: parse `tests/fixtures/sample_feed.xml` → `extract_paper`
  → `SourceFact::from_paper` → `store.save` → `store.get` round-trips. Fixture
  path resolves under `diver-core/`.
- `test_ingest_pipeline_multi_category`: parsed paper preserves `cs.CL` + `cs.AI`.

## End-to-End Tests
- **Status:** possible
- `e2e_cli_help_lists_subcommands` (scripted smoke, test phase): `cargo run -p
  diver-cli -- --help` — output contains `search`, `ingest`, `inspect`, `list`,
  `collect`, `find`, and the top-level name `diver`; the built binary is
  `target/debug/diver(.exe)`. Pass/fail: all six subcommand names + binary name
  present.
- `e2e_cli_subcommand_help_parses` (scripted smoke): `cargo run -p diver-cli --
  <sub> --help` for a representative subset (`find`, `ingest`, `collect`) — each
  exits 0 and prints the same args as before the split (e.g. `find` shows
  `--max-results`, `collect` shows `--max-results`/`--sort-by`). Pass/fail: exit
  0 and expected arg flags present.
- **Side-effect-free by design:** every E2E smoke uses `--help`, which clap
  handles before any command body runs — no network call and no touch of the
  real data dir (`Store::open()` is never reached). This is why the earlier
  `diver list` smoke was dropped: `list` would create `%APPDATA%/diver/diver.db`.
- Rationale for scripting rather than an `assert_cmd` test: adding a test-harness
  dependency is a non-goal for this sprint (no dep changes beyond the split);
  the smokes are executed and recorded in `e2e-tests.md` during the Test phase.
