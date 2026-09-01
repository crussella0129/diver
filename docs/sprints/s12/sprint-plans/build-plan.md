Finalized - DO NOT EDIT

# Sprint 12 Build Plan

## Intents
- [INT-0013](../../../intents/INT-0013-coassertion-relations.md) — state: planned; acceptance criteria covered: AC1/AC2/AC3 (T-1201), AC4 (T-1202), AC5/AC6 (T-1203)

## Schema Tree
- Sprint Goal: co-assertion edges linking papers by shared claim terms
  - Graph core (INT-0013)
    - T-1201: `compute_coassertion_relations` + `significant_terms` + `CoAssertion` (+ display arm)
    - T-1202: `Store::all_claims`
  - Wiring (INT-0013)
    - T-1203: `diver dive` unions co-assertion edges + docs + integration test

## Execution Sequence

Pure graph additions + store query first, then the CLI composes them.

### T-1201: `graph.rs` co-assertion edges (+ display arm)
- **Intent:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md)
- **Touches:** diver-core/src/graph.rs, diver-core/src/display.rs (`relation_reason`)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0013 AC1 (variant + display), AC2 (co-assertion
  edges), AC3 (significant_terms tokenization).
- **Success criterion (EARS):**
  - **WHEN** two distinct papers' claims share a significant term, **THEN**
    `compute_coassertion_relations` **SHALL** emit one `CoAssertion(term)` edge for
    that term (once per pair even if the term repeats in a paper's claims), with no
    self-edges.
  - **WHEN** `significant_terms` is given a claim, **THEN** it **SHALL** return its
    alphanumeric tokens lowercased, length ≥ 3, excluding `STOPWORDS`, ignoring
    punctuation and case.
- **Notes:** add `CoAssertion(String)` to `RelationKind`. `significant_terms`:
  split on non-alphanumeric (`char::is_alphanumeric`), lowercase, keep length ≥ 3
  and not in `STOPWORDS`. `STOPWORDS`: small curated function-word slice (the, and,
  for, are, was, were, with, from, our, this, that, which, into, than, they, their,
  use, using, based, …); domain words intentionally not stopped in v1.
  `compute_coassertion_relations`: group `claims` by `arxiv_id` preserving
  first-seen order, `dedup_preserve_order` each paper's terms; for pairs `i < j`
  (skip equal ids), emit `CoAssertion(term)` for each term in the **sorted**
  intersection (deterministic output). `relation_reason` gains
  `CoAssertion(term) => format!("co-asserts {term}")`. `build_dive` needs no change.

### T-1202: `Store::all_claims`
- **Intent:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md)
- **Touches:** diver-core/src/store.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0013 AC4 (returns all claims; empty when none).
- **Success criterion (EARS):**
  - **WHEN** `all_claims()` runs, **THEN** it **SHALL** return `(arxiv_id, claim)`
    for every persisted assertion, ordered by paper then insertion; empty when none.
- **Notes:** mirror `papers_asserting` without the `WHERE ... LIKE ... ESCAPE`
  clause: `SELECT p.arxiv_id, a.claim FROM assertions a JOIN papers p ON p.id =
  a.paper_id ORDER BY p.arxiv_id, a.id`. Tests seed via `save_assertions`.

### T-1203: wire into `diver dive` + docs + integration test
- **Intent:** [INT-0013](../../../intents/INT-0013-coassertion-relations.md)
- **Touches:** diver-cli/src/main.rs (`Dive` handler), README.md,
  diver-core/tests/coassertion.rs (new)
- **Depends on:** T-1201, T-1202
- **Acceptance criterion:** INT-0013 AC5 (dive includes co-assertion edges), AC6
  (docs + no regression).
- **Success criterion (EARS):**
  - **WHEN** `diver dive <concept>` runs, **THEN** the relations **SHALL** include
    co-assertion edges (from `all_claims`) alongside category/author edges, so a
    seed paper co-asserting a term with another paper shows that relation.
- **Notes:** in the `Dive` else-branch, change `let relations =
  compute_relations(&facts);` to `let mut relations = compute_relations(&facts);
  relations.extend(compute_coassertion_relations(&store.all_claims()?));`. Import
  `compute_coassertion_relations` in main.rs. README: document that `dive` also
  links papers by shared claim terms (co-assertion). Integration test: save two
  papers with assertions whose claims share a significant term → the pipeline
  (`all_claims` → `compute_coassertion_relations` → `build_dive`) yields a
  `CoAssertion(term)` edge between them.
