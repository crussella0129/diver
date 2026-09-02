Finalized - DO NOT EDIT

# Sprint 16 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC1: batch `--all` + single unchanged | T-1601 / WHEN --all THEN iterate store; WHEN <id> THEN single | clap-enforced args (manual `extract --all --deterministic` / `extract` rejected); the per-paper extract loop is exercised by `test_real_corpus_dive` |
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC2: real-feed fixture | T-1602 / committed fixture | fixture present under tests/fixtures/ and parsed by `test_real_corpus_dive` |
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC3: E2E weighted graph on real content | T-1602 / WHEN ingest+extract THEN structural + weighted co-assertion edge | `test_real_corpus_dive` |
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC4: docs | T-1603 / README corpus workflow | README updated |
| [INT-0017](../../../intents/INT-0017-real-corpus-validation.md) | AC5: no regression | all | `cargo test --workspace`; clippy 0 |

## Unit Tests
- No new pure-unit tests. `diver extract --all`'s arg contract is enforced by clap
  (`required_unless_present`/`conflicts_with`); its extract-each-paper logic is the same
  deterministic path `test_real_corpus_dive` drives over the whole corpus.

## Integration / End-to-End Tests (`diver-core/tests/real_corpus.rs`, offline)
- **Intent:** [INT-0017](../../../intents/INT-0017-real-corpus-validation.md)
- `test_real_corpus_dive`: read the committed `tests/fixtures/real_corpus_feed.xml`;
  `parse::parse_feed` → for each paper `SourceFact::from_paper` + `Store::save`
  (`open_in_memory`); extract each deterministically (`candidate_assertions(&extract_observations(&fact))`
  → `validate` → `save_assertions`); compute `compute_relations(&store.list()?)`. Assert:
  - the corpus has ≥ 2 papers and every paper produced ≥ 1 stored assertion;
  - ≥ 1 structural edge (SharedCategory/SharedAuthor) exists;
  - co-assertion **existence at temperature 1.0** (guaranteed for a same-topic corpus, so the
    test is not brittle): `compute_coassertion_relations(&all_claims, 1.0)` has ≥ 1
    `CoAssertion { term, weight }` edge between two **distinct** paper ids, every weight finite
    in `[0.0, 1.0]`;
  - the weighting is monotonic on the real data: the temperature-0.5 co-assertion edge set is a
    subset of the temperature-1.0 set;
  - `build_dive` over a real seed (`papers_asserting` of a common term) yields a node whose
    `related` lists another paper.
  Assertions are on kinds/counts/invariants, not exact strings, so the test is robust to the
  real content.

## End-to-End (manual, offline-capable)
- `diver collect "<query>" && diver extract --all --deterministic && diver dive attention` —
  the exact flow the Sprint 16 probe ran; produces the persisted real corpus and the weighted
  graph. (Live `collect` needs network; the automated `test_real_corpus_dive` is the offline
  equivalent over the captured fixture.)
