# Sprint 13 Research Report

## Intents Reviewed
- [INT-0014](../../../intents/INT-0014-weighted-coassertion-temperature.md) — created; relevance: primary; current state: proposed
- [INT-0013](../../../intents/INT-0013-coassertion-relations.md) — selected; relevance: the unweighted co-assertion this refines; current state: realized
- [INT-0012](../../../intents/INT-0012-graph-dive.md) — selected; relevance: the `dive` pipeline/display the knob threads through; current state: realized

## 1. Sprint Goal

Weight co-assertion edges by **TF-IDF** so that rare, distinctive shared terms
drive links while ubiquitous vocabulary is down-ranked, and expose an adjustable
**temperature** dial (`diver dive <concept> --temperature <0.0..=1.0>`) that trades
selectivity for recall:

- **Low temperature** → selective: only high-weight (rare, distinctive) shared
  terms produce edges → a sparse, high-signal graph.
- **High temperature** → permissive: lower-weight (more common) shared terms also
  qualify → a denser graph. `temperature = 1.0` admits every shared term, exactly
  reproducing the Sprint 12 (INT-0013) unweighted behavior.

Deterministic and LLM-free: TF-IDF is a pure function of the persisted claim
corpus; the temperature threshold is a deterministic cutoff. Advances **INT-0014**.
Baseline: `cb1ab25`, `cargo test --workspace` green (116), clippy 0.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| diver-core/src/graph.rs | high | `compute_coassertion_relations(claims)` (graph.rs) is the seam — today it emits one edge per shared term unweighted. Add DF/IDF scoring + temperature threshold here. `significant_terms`/`STOPWORDS` unchanged (numeric-token fix from the s12 review stays). `dedup_preserve_order` reused. Signature grows a `temperature` (or `CoassertionParams`) argument. |
| diver-cli/src/main.rs | high | `Dive { concept }` (main.rs:66) gains a `--temperature` clap arg (`f64`, default TBD, validated to `0.0..=1.0`); the handler (main.rs:203) passes it into `compute_coassertion_relations`. `compute_relations` (structural) is untouched — temperature only gates the epistemic edges. |
| diver-core/src/display.rs | medium | `relation_reason` (display.rs:143) renders `CoAssertion(term)`. If the edge carries its weight, this arm can annotate it (e.g. `co-asserts attention (w=0.82)`) so the user can see why an edge survived and calibrate temperature. `DIVE_RELATED_CAP`/`related_overflow` (display.rs:135) already bound per-node output and interact with the new filter (fewer edges → less overflow). |
| diver-core/src/store.rs | low | `all_claims()` (store.rs) already returns the `(arxiv_id, claim)` corpus TF-IDF needs; no schema/query change. N (document count) = distinct `arxiv_id` in `all_claims`. |
| diver-core/tests/coassertion.rs | high | `test_coassertion_pipeline` calls `compute_coassertion_relations` with 2 papers (N=2, df=2). The N≤2 guard (below) must keep that edge so this test and the unit tests stay green under the new signature. |

Baseline: workspace at `cb1ab25`. `cargo test --workspace` green (116); clippy 0.

### Design

```rust
// graph.rs — weight each candidate shared term by IDF over the claim corpus.
//
// Documents = papers (one per distinct arxiv_id in `claims`); N = document count.
// df(t) = number of papers whose deduped significant terms contain t.
// A *shared* term has df >= 2 by definition, so df in [2, N].
// idf(t)            = ln(N / df(t))            // in [0, ln(N/2)]
// normalized_weight = idf(t) / ln(N / 2)       // in [0.0, 1.0]
//   - df == 2 (rarest shareable)  -> 1.0  (most distinctive)
//   - df == N (every paper)       -> 0.0  (ubiquitous noise)
// Keep the edge iff normalized_weight >= (1.0 - temperature).
//   - temperature = 1.0 -> threshold 0.0 -> keep all shared terms (= INT-0013).
//   - temperature = 0.0 -> threshold 1.0 -> keep only df==2 terms (maximally selective).
//
// Guard: when N <= 2, ln(N/2) == 0 (division by zero) and every shared term has
// df == N, so there is no discriminating power — treat all shared terms as weight
// 1.0 (keep them). This preserves the 2-paper pipeline/unit tests.

pub fn compute_coassertion_relations(
    claims: &[(String, String)],
    temperature: f64,          // clamped to [0.0, 1.0] defensively
) -> Vec<ComputedRelation>;
```

`RelationKind::CoAssertion` sub-decision (for the plan): keep `CoAssertion(String)`
and filter silently, **or** carry the weight (`CoAssertion { term, weight }`) so
`relation_reason` can show it. Carrying the weight is more honest and makes the
temperature effect visible, at the cost of touching the enum + its match arms +
the two s12 tests. Leaning: carry the weight — small, well-contained churn, high
explanatory payoff. `build_dive` is unaffected (kind-agnostic).

CLI: `diver dive <concept> --temperature <t>` (default a moderate value that trims
obvious noise; `1.0` recovers old behavior). TF (within-paper term count) is left
at binary presence — claims are short, so IDF carries the signal; TF weighting is
a documented later refinement on the same seam.

## 3. External Sources
- [TF-IDF (inverse document frequency), smoothed/standard variants](https://en.wikipedia.org/wiki/Tf%E2%80%93idf) — `idf = ln(N/df)`; document = paper, term = significant term. We use the unsmoothed form and normalize by the theoretical shared-term max `ln(N/2)`; a shared term always has `df >= 2` so no `df == 0` case arises.
- [clap value parsing / range validation](https://docs.rs/clap/latest/clap/) — `#[arg(long, default_value_t = ..., value_parser = ...)]` for a bounded `f64`; reject out-of-range with a clear error (or clamp).
- [Rust `f64::ln`](https://doc.rust-lang.org/std/primitive.f64.html#method.ln) — deterministic for identical inputs; comparisons against the threshold are stable, preserving reproducible output.

## 4. Risks / Unknowns / Dependencies
- **Default-behavior change.** A default temperature < 1.0 makes `dive` emit fewer
  co-assertion edges than Sprint 12. This is the point (noise reduction), but it is
  a visible behavior change — document it; `--temperature 1.0` restores INT-0013
  output exactly. Mitigation: pick a default that is clearly useful and note it in
  README + intent.
- **Signature change ripples.** `compute_coassertion_relations` gains a parameter;
  every call site (main.rs, the graph.rs unit tests, coassertion.rs) must update.
  Expected, bounded churn — caught at compile time.
- **N ≤ 2 division-by-zero.** `ln(N/2)` is 0 when N == 2 and undefined for N < 2.
  The guard (weight 1.0, keep all) both avoids the NaN/∞ and keeps the small-corpus
  tests meaningful. Must have an explicit unit test.
- **Normalization choice.** Normalizing by the theoretical max `ln(N/2)` (vs. the
  observed max IDF) keeps the dial's meaning stable as the corpus grows and avoids a
  single ultra-rare term flattening all others. Documented as the chosen approach;
  observed-max normalization is an alternative recorded in the intent.
- **Temperature semantics must match intuition.** Higher temperature = more
  permissive/denser (generative-model convention). The threshold `1.0 - temperature`
  encodes exactly that. A unit test at t=0.0, t=1.0, and a mid value pins the
  monotonic behavior.
- **No new dependencies.** Pure `std` (`f64::ln`, arithmetic) + existing clap. Stays
  deterministic and offline.

## 5. Recommended Approach

Implement IDF weighting inside `compute_coassertion_relations` with a `temperature`
parameter, threshold `normalized_weight >= 1.0 - temperature`, normalized by
`ln(N/2)` with an `N <= 2` guard (keep-all). Carry the weight on the
`CoAssertion` edge so the display can show it. Add a validated `--temperature`
clap flag to `diver dive` (default moderate, `1.0` = legacy). Cover with unit tests
(monotonicity at t∈{0.0, mid, 1.0}, df-ordering, N≤2 guard, threshold boundary) and
extend the integration pipeline test to assert a low-temperature run drops a
common-term edge that a high-temperature run keeps. Update README + INT-0014.

Structural (category/author) edges are deliberately **not** temperature-gated —
the dial governs only the epistemic co-assertion layer, keeping the two edge
sources cleanly separated (consistent with INT-0013's "compose two edge sets"
decision). The exact default temperature and the carry-the-weight enum change are
the two decisions to confirm at plan approval.

### Referenced artifacts
- [INT-0014 chapter](../../../intents/INT-0014-weighted-coassertion-temperature.md)
- Build plan / test plan: `../sprint-plans/` (authored in the plan phase)
- Baseline evidence: `cargo test --workspace` 116/116, clippy 0 at `cb1ab25`
