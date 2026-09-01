# INT-0012 — Graph layer: ComputedRelation + `diver dive`

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0012
- **State:** active
- **Work evidence:** [Sprint 11 build plan](../sprints/s11/sprint-plans/build-plan.md) (T-1101, T-1102, T-1103)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** [Sprint 11 test report](../sprints/s11/sprint-tests/test-report.md)
- **Documentation evidence:** none

## Intent

Implement the long-reserved `diver dive <concept>` as the first graph traversal
over the persisted epistemic layer ([[persist-epistemic-layer]], INT-0010). In
`diver-core`, introduce:

- **`ComputedRelation { from, to, kind }`** with **`RelationKind`** =
  `SharedCategory(code)` | `SharedAuthor(name)` — deterministic edges between
  stored papers, computed from their taxonomy categories and authors.
- **`compute_relations(&[SourceFact]) -> Vec<ComputedRelation>`** — one edge per
  shared category code and per shared author between each pair of papers.
- **`Store::papers_asserting(concept) -> Vec<(arxiv_id, claim)>`** — the seed
  query: papers whose persisted assertion claims mention the concept.
- **`build_dive(concept, facts, asserting, relations) -> Vec<DiveNode>`** — a pure
  assembler producing a concept-centered neighborhood: each seed paper, its
  matching claims, and its related papers.
- **`diver dive <concept>`** — run the pipeline and display the neighborhood.

Non-goals:
- No LLM or semantic similarity — edges are exact category/author matches.
- No multi-hop/transitive traversal — a 1-hop neighborhood.
- No graph persistence — relations are computed on the fly from stored papers +
  assertions.
- No visualization / TypeScript frontend.
- No change to extraction, the `validate` gate, or the storage schema.

## Acceptance criteria

1. `diver-core` exposes `ComputedRelation` + `RelationKind`
   (`SharedCategory`, `SharedAuthor`) and `compute_relations(&[SourceFact])` that
   emits one edge per shared category code and per shared author between each pair
   of papers (deterministic order; no self-edges).
2. `Store::papers_asserting(concept)` returns `(arxiv_id, claim)` for every
   persisted assertion whose claim contains the concept (case-insensitive), and an
   empty vec for an unknown concept / no matches.
3. `build_dive` assembles, for each distinct asserting paper, its title, its
   matching claim(s), and its related papers (the edges touching it, as
   `(other_arxiv_id, RelationKind)`).
4. `diver dive <concept>` displays the concept's neighborhood; when no paper
   asserts about the concept it prints a clean, actionable message (suggesting
   `diver extract`) and exits 0.
5. All previously passing tests still pass; the README's "`diver dive` is reserved"
   note is replaced with documentation of the command.

## Rationale

The architecture vision's pipeline ends in graph construction and `dive`. Sprints
7–9 built the typed, validated, persisted assertion layer; this sprint turns it
into a navigable graph. `diver dive <concept>` shows which papers assert about a
concept and how they connect, so the extracted knowledge becomes explorable rather
than only per-paper. Deterministic category/author edges keep v1 LLM-free and
reproducible; richer semantic edges are a later sprint that plug into the same
`RelationKind` enum.

## Alternatives

- **Seed via abstract FTS (existing `find`)** — rejected: `dive` should traverse
  the *epistemic* layer (persisted assertions), which is the engine's output and
  what Sprint 9 persisted; abstract FTS already exists as `diver find`.
- **Persist the graph** — deferred: relations are cheap to recompute from stored
  papers; a persisted `graph_version` is a later concern.
- **Semantic / LLM edges** — deferred: v1 uses exact category/author matches for
  determinism; `RelationKind` is designed to extend.
- **Multi-hop traversal** — deferred: a 1-hop neighborhood is a meaningful,
  bounded first `dive`.

## Consequences

- `diver-core` gains a `graph` module; `diver-cli` gains a `dive` command; the
  README's reserved-command note is replaced.
- Relation computation is O(n²) over stored papers — fine at local-corpus scale;
  an indexed/persisted graph is a future optimization.
- Papers with no extracted assertions do not appear as `dive` seeds — `dive`
  reflects the *extracted* knowledge by design (run `diver extract` first).

## Transition history
- 2026-09-01: created as `proposed`.
- 2026-09-01: `proposed` → `planned`; linked to Sprint 11 build plan (T-1101
  graph core, T-1102 `papers_asserting`, T-1103 `diver dive`).
- 2026-09-01: `planned` → `active` (Sprint 11 build started; T-1101 first).
