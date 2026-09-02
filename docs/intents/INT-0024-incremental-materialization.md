# INT-0024 — Incremental graph materialization

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0024
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

`diver dive` loads the whole corpus and computes every edge at query time.
`compute_relations` and `compute_coassertion_relations` are pairwise over all
papers — O(n²) per invocation, recomputed from scratch on every dive, discarded
afterwards.

**This intent is a deliberate deferral, recorded so it stays deliberate.**

When the trigger conditions below are met, move edge computation from query time
to ingest time:

- **Materialize on write.** Ingesting a paper or persisting assertions updates the
  affected indexes and recomputes only the edges that paper touches.
- **Dive becomes a read.** Query time turns into a lookup over stored edges rather
  than a recomputation of the universe.
- **Determinism is preserved exactly.** Incremental and from-scratch computation
  must agree, and a test must prove they do — the failure mode of incremental
  systems is silent drift, where the stored graph and the true graph diverge
  and nothing notices.
- **Rebuild stays available.** A full recomputation path must remain, both as the
  correctness oracle and as the recovery path when the algorithm changes.

Non-goals:
- No distributed processing, no external graph database, no service architecture.
  The moment those appear, this intent has been misread.
- No change to graph *semantics*. Identical edges, computed earlier.

## Trigger conditions

Schedule this intent when **any** of the following is observed and recorded:

1. A `diver dive` on the real corpus exceeds roughly two seconds wall-clock.
2. The corpus exceeds roughly 10,000 papers.
3. [[typed-epistemic-relations]] (INT-0021) makes relation computation expensive
   enough that recomputing per query is untenable — likely the real trigger, since
   LLM-proposed relations cannot be recomputed on every dive at any corpus size.

Until then, the O(n²) query-time implementation is the **correct** design, and
optimizing it is a mistake.

## Acceptance criteria

1. Ingest updates stored edges incrementally; dive reads them without pairwise
   recomputation.
2. A test asserts incremental and full-rebuild edge sets are identical on a
   fixture corpus.
3. A full rebuild command exists and is documented.
4. Dive latency on the corpus that triggered this work is measurably improved,
   with before-and-after numbers recorded.
5. `--temperature` semantics ([[weighted-coassertion-temperature]], INT-0014) are
   preserved. This needs care: IDF weights depend on corpus-wide document
   frequency, so adding a paper perturbs weights on edges it does not touch. The
   design must state whether it recomputes affected weights or accepts bounded
   staleness — and if the latter, bound it explicitly.

## Rationale

The [external review](../history/2026-09-02-external-review-gpt-5-6.md) flags the O(n²) and then says
plainly not to fix it yet. That is correct, and it is why this chapter exists in
`proposed` rather than as work: the current implementation is easy to reason
about, easy to validate, and fast enough at the corpus sizes Diver actually has.
Rewriting it now would trade clarity for a speed nobody needs while the genuinely
hard problems — concept identity, relation quality — are still open.

The reason to write it down anyway is that undocumented deferrals decay into
folklore. Six sprints from now, "why is dive O(n²)?" should have an answer in the
repo — with the conditions that end the deferral stated in advance, so the
decision to act is triggered by measurement rather than by whoever most recently
felt uneasy about it.

Criterion 5 exists because it is the trap: IDF is a global statistic, and
"incremental" and "corpus-wide weight" pull against each other. Discovering that
mid-implementation would be expensive.

## Alternatives

- **Optimize now** — rejected. Premature by the project's own evidence: no
  measured latency problem exists.
- **Cache computed relations with invalidation** — a lighter middle option that
  keeps query-time computation but avoids repeating it. Worth considering first
  when the trigger fires, since it is far less invasive than materialize-on-write.
- **Move to a graph database** — rejected at this scale, and probably at the next
  one. SQLite is expected to handle millions of edges comfortably, so the
  substrate is not believed to be the bottleneck, and swapping it would import
  operational complexity for no measured gain. **That expectation is unbenchmarked
  on this workload** — measure before either rejecting this alternative again or
  acting on it.
- **Approximate/sampled edges** — rejected: it trades away determinism, which is
  the property the whole project is built on.

## Consequences

- While deferred: dive latency grows quadratically, and that is accepted.
- When scheduled: new stored edge tables, ingest-path changes, a rebuild command,
  and the IDF staleness decision above.
- Interacts with [[first-class-concepts]] (INT-0020) — concept-keyed edges change
  what "affected edges" means on ingest, so scheduling this after INT-0020 avoids
  doing the work twice.

## Transition history
- 2026-09-02: created as `proposed` during Sprint 18 roadmap realignment, to record an intentional deferral with explicit, measurable trigger conditions rather than leaving it as undocumented folklore.
