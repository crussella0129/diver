# Plan Critique — Sprint 12

## Concerns

### C-001: co-assertion over-links on common non-stopword domain words
- **Where:** `build-plan.md` T-1201 (`STOPWORDS`, `significant_terms`)
- **Quote:** "domain words intentionally not stopped in v1"
- **Failure mode:** weak-relation (noisy graph)
- **Why it matters:** ubiquitous content words in ML abstracts — "model", "method",
  "results", "training", "performance" — are not stopped, so nearly every pair of
  papers co-asserts one of them, producing a dense, low-signal neighborhood.
- **Suggested response:** defer-with-rationale — intended for v1 per INT-0013's
  non-goals. The noise is bounded two ways: the curated stopword list removes
  function words, and the INT-0012 display cap (10 related + "(+N more)") caps the
  per-node output. Term significance weighting (TF-IDF, document-frequency
  thresholds, phrase detection) is the documented next refinement and plugs into
  the same `RelationKind`/`significant_terms` seam without changing the pipeline.
  `test_coassertion_shared_term` pins the intended behavior so a future tightening
  is a conscious change.

### C-002: `length >= 3` drops two-character acronyms that are significant
- **Where:** `build-plan.md` T-1201 (`significant_terms` len ≥ 3)
- **Quote:** "length ≥ 3 and not in `STOPWORDS`"
- **Failure mode:** weak-relation (false negatives)
- **Why it matters:** "AI", "ML", "RL" are meaningful terms but are dropped, so
  two papers sharing only a 2-char acronym won't co-assert.
- **Suggested response:** defer-with-rationale — the len ≥ 3 rule is a deliberate
  noise/coverage trade: 2-char tokens are overwhelmingly function-word fragments,
  while 3-char acronyms (GPT, RNN, CNN, LLM) are kept. It only ever drops a real
  term (false negative), never fabricates a link, which is the safe direction for a
  relation graph. A short-acronym allowlist is a cheap future refinement; the
  boundary is pinned by `test_significant_terms`.

### C-003: co-assertion is another O(P^2 * T) pass and loads every claim per dive
- **Where:** `build-plan.md` T-1201 / T-1203 (`all_claims` + pairwise)
- **Quote:** "for pairs `i < j` … for each term in the sorted intersection"
- **Failure mode:** efficiency
- **Why it matters:** each `diver dive` now loads all claims and does an O(P^2)
  pass over papers (times term-set size) on top of the existing O(P^2)
  `compute_relations` — quadratic work per invocation.
- **Suggested response:** defer-with-rationale — consistent with the O(n^2) note
  already accepted in INT-0012 for local-corpus scale; the fix (an inverted
  term→papers index, or seed-scoped computation) is the same future optimization as
  the structural graph. Not a correctness issue; deferred as documented scale work.

## Confidence
proceed-with-caveats
