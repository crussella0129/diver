Finalized - DO NOT EDIT

# Sprint 11 Build Plan

## Intents
- [INT-0012](../../../intents/INT-0012-graph-dive.md) — state: planned; acceptance criteria covered: AC1/AC3 (T-1101), AC2 (T-1102), AC4/AC5 (T-1103)

## Schema Tree
- Sprint Goal: `diver dive <concept>` — graph traversal over persisted assertions
  - Graph core (INT-0012)
    - T-1101: `graph.rs` — `compute_relations` + `build_dive` (pure)
    - T-1102: `Store::papers_asserting` (seed query)
  - Wiring (INT-0012)
    - T-1103: `diver dive` command + display + docs + integration test

## Execution Sequence

Pure graph module + store query first (fully unit-tested), then the CLI composes them.

### T-1101: `graph.rs` — relations + `build_dive`
- **Intent:** [INT-0012](../../../intents/INT-0012-graph-dive.md)
- **Touches:** diver-core/src/graph.rs (new), diver-core/src/lib.rs (`pub mod graph`)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0012 AC1 (relation edges), AC3 (`build_dive` assembly).
- **Success criterion (EARS):**
  - **WHEN** `compute_relations` is given papers sharing a category code (resp.
    author), **THEN** it **SHALL** emit a `SharedCategory` (resp. `SharedAuthor`)
    edge for that pair and no self-edges; papers with nothing in common **SHALL**
    yield no edge.
  - **WHEN** `build_dive` is given facts, asserting `(id, claim)` pairs, and
    relations, **THEN** it **SHALL** return one `DiveNode` per distinct asserting
    paper carrying its title, its matching claims, and the relations touching it
    (mapped to the other endpoint).
- **Notes:** `RelationKind { SharedCategory(String), SharedAuthor(String) }`,
  `ComputedRelation { from: String, to: String, kind: RelationKind }` (all
  `#[derive(Debug, Clone, PartialEq, Eq)]`). `compute_relations`: iterate pairs
  `i < j` in input order; per pair, `HashSet` intersection of category codes
  (`ArxivCategory::code()`) → `SharedCategory` edges, and of authors →
  `SharedAuthor` edges; `from = facts[i].arxiv_id`, `to = facts[j].arxiv_id`.
  `DiveNode { arxiv_id, title, claims: Vec<String>, related: Vec<(String,
  RelationKind)> }`. `build_dive`: group `asserting` by `arxiv_id` (preserve
  first-seen order), title from a `facts` lookup (fallback to the id), `related` =
  every relation whose `from`/`to` equals the node id, mapped to the other
  endpoint + kind.

### T-1102: `Store::papers_asserting`
- **Intent:** [INT-0012](../../../intents/INT-0012-graph-dive.md)
- **Touches:** diver-core/src/store.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0012 AC2 (seed query; empty for unknown).
- **Success criterion (EARS):**
  - **WHEN** `papers_asserting(concept)` runs, **THEN** it **SHALL** return
    `(arxiv_id, claim)` for every persisted assertion whose `claim` contains
    `concept` (case-insensitive).
  - **WHEN** no assertion matches (or the concept is unknown), **THEN** it **SHALL**
    return an empty vec.
- **Notes:** `prepare("SELECT p.arxiv_id, a.claim FROM assertions a JOIN papers p
  ON p.id = a.paper_id WHERE a.claim LIKE '%' || ?1 || '%' ORDER BY p.arxiv_id,
  a.id")`, bound `concept` (parameterized — no injection; SQLite `LIKE` is
  ASCII-case-insensitive). `query_map` → `Vec<(String, String)>`. Tests seed via
  `save_assertions`.

### T-1103: `diver dive` command + display + docs + integration test
- **Intent:** [INT-0012](../../../intents/INT-0012-graph-dive.md)
- **Touches:** diver-cli/src/main.rs (`Dive { concept }`), diver-core/src/display.rs
  (`display_dive`), README.md, diver-core/tests/dive_graph.rs (new)
- **Depends on:** T-1101, T-1102
- **Acceptance criterion:** INT-0012 AC4 (dive displays / empty message), AC5
  (docs + no regression).
- **Success criterion (EARS):**
  - **WHEN** `diver dive <concept>` runs and some paper asserts about the concept,
    **THEN** it **SHALL** display each such paper, its matching claim(s), and its
    related papers.
  - **WHEN** no paper asserts about the concept, **THEN** it **SHALL** print a
    clean actionable message (suggesting `diver extract`) and exit 0.
- **Notes:** `Dive { concept: String }` in `Commands`. Handler: `let store =
  Store::open()?; let facts = store.list()?; let asserting =
  store.papers_asserting(&concept)?;` if `asserting.is_empty()` →
  `display::display_dive(&concept, &[])` (which prints the actionable empty
  message); else `let rels = compute_relations(&facts); let nodes =
  build_dive(&facts, &asserting, &rels); display::display_dive(&concept, &nodes)`.
  `display_dive(concept, &[DiveNode])` is a **new** owo-colors function — the empty
  case prints "No papers assert about '{concept}'. Run `diver extract <id>` first."
  Per critique C-003 it renders a **bounded** view: each node's claims, then up to
  10 related papers with a "(+N more)" suffix when exceeded (the `build_dive` API
  keeps the full edge set).
  Do **not** reuse `display_dive_results` (renders `find` FTS results). README:
  replace the "`diver dive` is reserved" note with usage. Integration test in
  `dive_graph.rs` composes the pipeline over two category-sharing papers.
