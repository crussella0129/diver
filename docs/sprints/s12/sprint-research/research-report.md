# Sprint 12 Research Report

## Intents Reviewed
- [INT-0013](../../../intents/INT-0013-coassertion-relations.md) — created; relevance: primary; current state: proposed
- [INT-0012](../../../intents/INT-0012-graph-dive.md) — selected; relevance: the graph/dive this extends; current state: realized

## 1. Sprint Goal

Add a `CoAssertion(term)` edge type over the persisted claims so `diver dive`
links papers by shared assertion vocabulary, not only category/author. Add
`compute_coassertion_relations`, `Store::all_claims`, and combine the two edge
sources in the `dive` handler. Deterministic term overlap (tokenize + lowercase +
length ≥ 3 + stopword filter), no LLM.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/graph.rs | high | `RelationKind` enum (add `CoAssertion(String)`), `ComputedRelation`, `compute_relations` (structural, unchanged), `build_dive` (unchanged — already unions any relation kind). `dedup_preserve_order` helper reusable. Add `compute_coassertion_relations` + `significant_terms` + `STOPWORDS`. |
| diver-core/src/store.rs | high | `papers_asserting` (store.rs) shows the assertions→papers join and the collect loop to mirror for `all_claims` (drop the WHERE/ESCAPE). |
| diver-core/src/display.rs | high | `relation_reason` (display.rs) matches `RelationKind` **exhaustively** — adding the variant requires a new arm here (`CoAssertion(term) => "co-asserts {term}"`) or the crate won't compile; this arm ships in the same task as the variant. |
| diver-cli/src/main.rs | medium | `Dive` handler composes the pipeline; add `store.all_claims()?` and `relations.extend(compute_coassertion_relations(&claims))`. |

Baseline: workspace at `d4d084b`. `cargo test --workspace` green (106); clippy 0.

### Design

```rust
// graph.rs
pub enum RelationKind { SharedCategory(String), SharedAuthor(String), CoAssertion(String) }

/// Papers whose stored claims share a significant term. `claims` is
/// (arxiv_id, claim) for every persisted assertion.
pub fn compute_coassertion_relations(claims: &[(String, String)]) -> Vec<ComputedRelation> {
    // 1. group claims by arxiv_id (first-seen order) -> ordered-unique significant terms
    // 2. for each pair (i<j) of distinct papers, for each term in intersection (sorted),
    //    push ComputedRelation { from, to, kind: CoAssertion(term) }
}

/// Alphanumeric tokens of a claim, lowercased, length >= 3, minus STOPWORDS.
fn significant_terms(claim: &str) -> Vec<String>;
const STOPWORDS: &[&str] = &["the","and","for","are","was","were","has","have","had",
    "this","that","with","from","our","its","can","will","not","but","all","use",
    "using","based","which","into","than","then","they","their","als", ...];
```
`build_dive` needs no change — it collects every relation touching a node
regardless of kind. `display_dive`/`relation_reason` gain the `CoAssertion` arm.

```rust
// store.rs
pub fn all_claims(&self) -> Result<Vec<(String, String)>>;
//   SELECT p.arxiv_id, a.claim FROM assertions a JOIN papers p ON p.id = a.paper_id
//   ORDER BY p.arxiv_id, a.id
```

`dive` handler: `let mut relations = compute_relations(&facts);
relations.extend(compute_coassertion_relations(&store.all_claims()?));`

## 3. External Sources
- [Rust char::is_alphanumeric / split_whitespace](https://doc.rust-lang.org/std/primitive.char.html#method.is_alphanumeric) — tokenize on non-alphanumeric, lowercase with `to_lowercase()`.
- [Common English stop words](https://en.wikipedia.org/wiki/Stop_word) — reference for the curated function-word list (kept small; domain words intentionally not stopped in v1).

## 4. Risks, Unknowns, Dependencies

- **Risk:** over-linking on ubiquitous domain words ("model", "method", "results").
  Mitigation: curated stopwords + length ≥ 3 + the INT-0012 display cap bound the
  noise; TF-IDF/phrase detection is documented future work. A fixed-input test
  pins the intended term set.
- **Risk:** duplicate co-assertion edges when a term repeats across a paper's
  claims. Mitigation: per-paper terms are order-preserving deduped (reuse
  `dedup_preserve_order`), so each shared term yields one edge per pair.
- **Risk:** non-determinism from HashSet iteration when emitting shared terms.
  Mitigation: sort the shared terms before emitting edges, so output order is
  stable and testable.
- **Dependency:** none new. Reuses `ComputedRelation`, `build_dive`,
  `dedup_preserve_order`, the `assertions` table.

## 5. Recommended Approach

Primary: pure graph additions + store query first, then wire the handler.
- `graph.rs`: `CoAssertion` variant, `STOPWORDS`, `significant_terms`,
  `compute_coassertion_relations` (group-by-paper, pairwise shared-term, sorted);
  `display.rs`: `relation_reason` `CoAssertion` arm (same task, for compilation).
- `store.rs`: `all_claims`.
- `main.rs`: extend the `dive` relations with co-assertion edges; README documents
  the new edge type.

Tests: `significant_terms` (tokenize/lowercase/stopword/length); `compute_coassertion_relations`
(shared term → edge, no self-edge, dedup repeated term, disjoint → none, sorted
determinism); `store::all_claims` (returns all, empty when none); integration
(save two papers + assertions sharing a term → dive pipeline yields a CoAssertion
edge); e2e (dive still works; help unchanged).

Alternative considered: fold co-assertion into `compute_relations` — rejected;
keep structural vs epistemic edge computation separate (different inputs).

## Artifacts
- No standalone snippet files; design inline in §2.
