# Sprint 11 Research Report

## Intents Reviewed
- [INT-0012](../../../intents/INT-0012-graph-dive.md) — created; relevance: primary; current state: proposed
- [INT-0010](../../../intents/INT-0010-persist-epistemic-layer.md) — selected; relevance: the persisted assertions `dive` seeds from; current state: realized

## 1. Sprint Goal

Implement `diver dive <concept>` — the first graph traversal over the persisted
epistemic layer. Add a `graph` module (`ComputedRelation`/`RelationKind`,
`compute_relations`, a pure `build_dive` assembler), a `Store::papers_asserting`
seed query over the `assertions` table, and the `dive` CLI command. Edges are
deterministic (shared category / shared author); no LLM, no persistence, 1-hop.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/fact.rs | high | `SourceFact { arxiv_id: String, authors: Vec<String>, categories: Vec<ArxivCategory>, … }` — the input to `compute_relations`. Category code via `ArxivCategory::code()`. |
| diver-core/src/store.rs | high | `list()` returns `Vec<SourceFact>` (all latest-version papers) — the corpus for relations. `assertions(paper_id, version, claim, …)` joined to `papers` gives `papers_asserting`; mirror the `get_assertions` query pattern (store.rs). |
| diver-core/src/id.rs | medium | `ArxivCategory::code()`, `ArxivId` — categories compared by code string. |
| diver-cli/src/main.rs | high | Add `Dive { concept }` to `Commands`; no `Dive` exists today. Handler: `list` + `papers_asserting` + `compute_relations` + `build_dive` + `display_dive`. |
| diver-core/src/display.rs | medium | Add `display_dive(concept, &[DiveNode])`. NOTE: `display_dive_results` already exists but renders the FTS `find` results (legacy name) — do **not** reuse it; add a distinct function. |
| diver-cli/src/main.rs `Find` | low | `find` (FTS over abstracts) is the abstract-search path; `dive` is the assertion-graph path — distinct. |

Baseline: workspace at `443fda4`. `cargo test --workspace` green (94); clippy 0.

### Design

```rust
// diver-core/src/graph.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationKind { SharedCategory(String), SharedAuthor(String) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedRelation { pub from: String, pub to: String, pub kind: RelationKind }

/// One edge per shared category code and per shared author, for each unordered
/// pair (i < j). Deterministic; no self-edges.
pub fn compute_relations(facts: &[SourceFact]) -> Vec<ComputedRelation>;

pub struct DiveNode {
    pub arxiv_id: String,
    pub title: String,
    pub claims: Vec<String>,                    // matching claims about the concept
    pub related: Vec<(String, RelationKind)>,   // (other arxiv_id, why)
}

/// Assemble a concept neighborhood: for each distinct asserting paper, its title
/// (from `facts`), its matching claims (from `asserting`), and the relations
/// touching it (from `relations`, mapped to the other endpoint).
pub fn build_dive(
    facts: &[SourceFact],
    asserting: &[(String, String)],     // (arxiv_id, claim)
    relations: &[ComputedRelation],
) -> Vec<DiveNode>;
```

```rust
// diver-core/src/store.rs
// Papers whose persisted assertion claims contain `concept` (case-insensitive).
pub fn papers_asserting(&self, concept: &str) -> Result<Vec<(String, String)>>;
//   SELECT p.arxiv_id, a.claim FROM assertions a JOIN papers p ON p.id = a.paper_id
//   WHERE a.claim LIKE '%' || ?1 || '%' COLLATE NOCASE  ORDER BY p.arxiv_id, a.id
```

`diver dive <concept>`: `let facts = store.list()?;` `let asserting =
store.papers_asserting(&concept)?;` if empty → actionable message; else `let rels
= compute_relations(&facts);` `let nodes = build_dive(&facts, &asserting, &rels);`
`display::display_dive(&concept, &nodes)`.

## 3. External Sources
- [SQLite LIKE / case-insensitivity](https://www.sqlite.org/lang_expr.html#like) — `LIKE` is case-insensitive for ASCII by default; use `'%' || ?1 || '%'` with a bound parameter (no injection).
- [Rust HashSet](https://doc.rust-lang.org/std/collections/struct.HashSet.html) — intersect category/author sets per pair for shared-edge detection.

## 4. Risks, Unknowns, Dependencies

- **Risk:** O(n²) pairwise relation computation. Mitigation: local corpora are
  small (tens–hundreds of papers); acceptable for v1. Noted as a future
  optimization (indexed/persisted graph). A test with a small fixture corpus
  covers correctness, not scale.
- **Risk:** `dive` returns nothing because no assertions are stored. Mitigation:
  this is correct (dive reflects extracted knowledge) — the empty path prints an
  actionable "run `diver extract` first" message and exits 0.
- **Risk:** duplicate edges (a pair sharing two categories yields two edges).
  Intended: each shared attribute is its own typed edge; `build_dive` groups by
  paper for display. Deterministic ordering by input index keeps output stable.
- **Unknown:** whether to dedupe `related` per node. Decision: keep one entry per
  (other paper, kind) edge; do not collapse across kinds (a shared category and a
  shared author with the same paper are distinct, informative edges).
- **Dependency:** none new. `Store::list` + the `assertions` table (INT-0010) +
  `SourceFact`. No new crate deps.

## 5. Recommended Approach

Primary: build the pure graph module + store query, then the CLI.
- `graph.rs`: `RelationKind`, `ComputedRelation`, `compute_relations`, `DiveNode`,
  `build_dive` — all pure, unit-tested with fixture `SourceFact`s.
- `store.rs`: `papers_asserting` (LIKE query over `assertions`), tested via
  `save_assertions` seeding.
- `main.rs`: `Dive { concept }` command composing the pipeline;
  `display.rs::display_dive`. README: replace the reserved-`dive` note.

Tests: `compute_relations` (shared category, shared author, no self-edges, no
edges when disjoint); `build_dive` (assembles nodes, claims, related); `store`
(papers_asserting finds by claim substring, case-insensitive, empty for unknown);
integration (`store.save` papers + `save_assertions` + `compute_relations` +
`build_dive` neighborhood); e2e (`dive --help`, `dive <concept>` with no data →
actionable message, exit 0).

Alternative considered: compute relations lazily only among seeds + neighbors —
deferred; whole-corpus `compute_relations` is simpler and fine at scale, and
`build_dive` already filters to the seeds.

## Artifacts
- No standalone snippet files; design inline in §2.
