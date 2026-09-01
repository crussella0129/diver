# INT-0014 — Weighted co-assertion with adjustable temperature

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0014
- **State:** active
- **Work evidence:** [Sprint 13 build plan](../sprints/s13/sprint-plans/build-plan.md) (T-1301, T-1302, T-1303)
- **Completion evidence:** (pending)
- **Code evidence:** [diver-core/src/graph.rs](../../diver-core/src/graph.rs), [diver-cli/src/main.rs](../../diver-cli/src/main.rs), [diver-core/src/display.rs](../../diver-core/src/display.rs)
- **Test evidence:** (pending)
- **Documentation evidence:** (pending — README `diver dive --temperature`)

## Intent

Refine the co-assertion edges ([[coassertion-relations]], INT-0013) so they are
**weighted by TF-IDF** and gated by a user-facing **temperature** dial, turning
`diver dive` from an all-or-nothing shared-term graph into a tunable one where
rare, distinctive terms dominate and ubiquitous vocabulary is down-ranked.

- **IDF weighting.** Treating each paper as a document and each significant term
  as a term, weight a shared term `t` by its inverse document frequency over the
  persisted claim corpus: `idf(t) = ln(N / df(t))`, where `N` is the number of
  distinct papers with claims and `df(t)` is the number of those papers whose
  (deduplicated) significant terms contain `t`. A *shared* term has `df >= 2`, so
  `idf(t)` lies in `[0, ln(N/2)]`; the normalized weight is
  `idf(t) / ln(N/2)` in `[0.0, 1.0]` (1.0 for a term shared by exactly two papers,
  0.0 for a term shared by every paper).
- **Temperature dial.** `diver dive <concept> --temperature <t>` with `t` in
  `[0.0, 1.0]`. An edge for term `t` is kept iff `normalized_weight(t) >= 1.0 - temperature`:
  - `temperature = 0.0` → threshold `1.0` → only `df == 2` terms link (maximally
    selective, sparse).
  - `temperature = 1.0` → threshold `0.0` → every shared term links (permissive;
    identical to the unweighted INT-0013 behavior).
  - Intermediate values interpolate: higher temperature ⇒ more, weaker edges.
- **Small-corpus guard.** When `N <= 2`, `ln(N/2)` is `0` (undefined weight) and
  every shared term has `df == N` (no discriminating power); all shared terms are
  treated as weight `1.0` and kept, regardless of temperature.

Deterministic and LLM-free: TF-IDF and the threshold are pure functions of the
claim corpus and the temperature argument.

Non-goals:
- No TF (within-paper term frequency) weighting — claims are short; binary presence
  with IDF carries the signal. TF is a later refinement on the same seam.
- No semantic similarity, stemming, or phrase detection (INT-0013 non-goals stand).
- Temperature gates **only** the epistemic co-assertion edges; structural
  (category/author) edges from `compute_relations` are unaffected.
- No graph persistence; no change to extraction, the `validate` gate, or storage.

## Acceptance criteria

1. `compute_coassertion_relations` weights each shared term by normalized IDF over
   the claim corpus and takes a `temperature` parameter; it keeps an edge iff
   `normalized_weight >= 1.0 - temperature`.
2. `temperature = 1.0` reproduces the INT-0013 edge set exactly (every shared term);
   `temperature = 0.0` keeps only terms shared by exactly two papers; behavior is
   monotonic in temperature (a term kept at `t` is kept at every `t' >= t`).
3. When `N <= 2`, all shared terms are kept at every temperature (no division-by-
   zero / NaN), preserving the small-corpus pipeline.
4. `diver dive <concept> --temperature <t>` exposes the dial, validated/clamped to
   `[0.0, 1.0]`, with a documented default; the structural edges are not gated by it.
5. The display renders co-assertion edges with their weight so the effect of
   temperature is visible (subject to the plan's carry-the-weight decision).
6. All previously passing tests still pass (with the default preserving useful
   output); the README documents `--temperature`.

## Rationale

INT-0013 links papers by any shared significant term, so common domain words can
over-link. IDF is the standard, deterministic way to rank a shared term's
distinctiveness, and a single temperature dial gives the user direct control over
the precision/recall trade-off without re-running extraction. Keeping
`temperature = 1.0` equal to the old behavior makes the refinement strictly
additive and backward-recoverable.

## Alternatives

- **Top-K strongest edges per node** — rejected as the primary dial: a continuous
  temperature threshold is more intuitive and composes with the existing
  `DIVE_RELATED_CAP` display cap.
- **Softmax "temperature" over term weights** — rejected: mathematically closer to
  the generative-model term but yields no clean keep/drop and is harder to explain.
- **Normalize by observed max IDF (not `ln(N/2)`)** — deferred: adaptive but makes
  the dial's meaning drift with the corpus draw and lets one ultra-rare term flatten
  the rest. Theoretical-max normalization keeps the dial stable.
- **TF·IDF (with within-paper counts)** — deferred: negligible signal on short
  claims; a later refinement on the same function.

## Consequences

- `compute_coassertion_relations` gains a `temperature` parameter; all call sites
  (the `dive` handler and tests) update. If the weight is carried on the edge, the
  `CoAssertion` variant and its match arms change.
- A default temperature below `1.0` changes `dive`'s co-assertion output relative to
  Sprint 12 (fewer, higher-signal edges); `--temperature 1.0` restores it exactly.
- IDF is computed per `dive` from `all_claims` (O(claims); pairwise remains O(P²),
  consistent with INT-0012/INT-0013 scale notes).

## Transition history
- 2026-09-01: created as `proposed`.
- 2026-09-01: `proposed` → `planned`; linked to Sprint 13 build plan (T-1301
  IDF weighting + temperature threshold + `CoAssertion { term, weight }`, T-1302
  `--temperature` CLI flag, T-1303 docs + pipeline test). User-confirmed decisions:
  default temperature `0.5`; weight carried on the edge and shown in `dive` output.
- 2026-09-01: `planned` → `active` (Sprint 13 build started; T-1301 first).
