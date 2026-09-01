# INT-0013 — Co-assertion relations

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0013
- **State:** active
- **Work evidence:** [Sprint 12 build plan](../sprints/s12/sprint-plans/build-plan.md) (T-1201, T-1202, T-1203)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Deepen the graph ([[graph-dive]], INT-0012) so `diver dive` connects papers by
**what they assert**, not only their category/author. Add an epistemic edge type
over the persisted claims ([[persist-epistemic-layer]], INT-0010):

- **`RelationKind::CoAssertion(term)`** — two papers are linked when their stored
  assertion claims share a significant term.
- **`compute_coassertion_relations(&[(arxiv_id, claim)])`** — emits a
  `CoAssertion(term)` edge for each significant term shared between two distinct
  papers' claims.
- **`Store::all_claims()`** — returns `(arxiv_id, claim)` for every persisted
  assertion (the corpus of claims to relate).
- The `diver dive` pipeline combines co-assertion edges with the structural
  (category/author) edges from `compute_relations`, and the display renders them.

"Significant term" is deterministic and LLM-free: a claim's alphanumeric tokens,
lowercased, length ≥ 3, excluding a curated stopword set (articles, prepositions,
conjunctions, pronouns, auxiliaries, and a few ubiquitous filler words).

Non-goals:
- No LLM or semantic similarity — exact shared-term overlap only.
- No stemming/lemmatization, no TF-IDF weighting (later refinements on the same
  `RelationKind`).
- No graph persistence; no change to extraction, the `validate` gate, or the
  storage schema.

## Acceptance criteria

1. `RelationKind` gains a `CoAssertion(String)` variant (the shared term); the
   display renders it (e.g. "co-asserts <term>").
2. `compute_coassertion_relations(&[(arxiv_id, claim)])` emits a
   `CoAssertion(term)` edge for each significant term shared between two **distinct**
   papers' claims; a term is counted once per paper (deduped) and there are no
   self-edges.
3. Significant terms are the claims' alphanumeric tokens, lowercased, length ≥ 3,
   excluding the curated stopword set; punctuation and case are ignored.
4. `Store::all_claims()` returns `(arxiv_id, claim)` for every persisted assertion.
5. `diver dive <concept>` includes co-assertion edges alongside category/author
   edges; when a seed paper co-asserts a term with another paper, `dive` shows that
   relation.
6. All previously passing tests still pass; the README documents co-assertion edges.

## Rationale

Category and author edges are structural; the interesting graph links papers by
shared **knowledge**. Now that claims are persisted (INT-0010) and the graph
exists (INT-0012), co-assertion edges turn `dive` into a knowledge graph over the
extracted claims. Deterministic term overlap keeps v1 reproducible and LLM-free;
TF-IDF or semantic edges are later refinements that plug into the same
`RelationKind` enum and the same `dive` pipeline.

## Alternatives

- **TF-IDF / semantic similarity** — deferred: exact shared-term is deterministic
  and simple; weighting/embedding is a refinement, not the foundation.
- **Stemming / lemmatization** — deferred: adds a dependency; raw token overlap is
  adequate for v1 ("attention" vs "attention-based" already share "attention").
- **Extend `compute_relations` to also compute co-assertion** — rejected: keep the
  structural primitive (over `SourceFact`) separate from the epistemic one (over
  claims); the `dive` handler composes both edge sets.

## Consequences

- The graph gains an epistemic edge type; the `dive` pipeline now unions two edge
  sources.
- Common domain words that are not in the stopword set can over-link papers —
  bounded by the display cap (INT-0012) and the curated stopword set; refining
  term significance (weighting, phrase detection) is future work.
- `Store::all_claims` loads every claim per `dive` invocation — O(claims), fine at
  local-corpus scale (consistent with the O(n²) relation note in INT-0012).

## Transition history
- 2026-09-01: created as `proposed`.
- 2026-09-01: `proposed` → `planned`; linked to Sprint 12 build plan (T-1201
  co-assertion edges, T-1202 `all_claims`, T-1203 `diver dive` wiring).
- 2026-09-01: `planned` → `active` (Sprint 12 build started; T-1201 first).
