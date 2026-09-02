# Test Critique — Sprint 17

## Concerns

### C-001: the "189 → 92" real-corpus flip is measured by a research script, not a committed test
- **Where:** `e2e-tests.md` real-corpus validation / `INT-0018` AC3
- **Quote:** "Terms shared by ≥2 papers: 189 → 92"
- **Failure mode:** evidence-drift
- **Why it matters:** the headline AC3 metric comes from a Python probe over the local DB, not a
  committed Rust test, so it is a one-time measurement rather than a guarded invariant.
- **Suggested response:** defer-with-rationale — the *committed* guard is
  `test_real_corpus_dive`, which after de-noising still requires ≥1 weighted co-assertion edge
  (now necessarily on a surviving technical term), proving the stoplist tightened the graph
  without emptying it; and `test_significant_terms_stoplist` pins the drop/keep behavior directly
  on named words. The 189→92 count is corroborating evidence over a specific corpus snapshot, not
  a stable invariant to assert. Re-implementing the probe as a Rust test would duplicate the
  real-corpus guard's purpose.

### C-002: AC2 ("O(1) membership") has no runtime test
- **Where:** `test-plan.md` AC2 row
- **Quote:** "membership SHALL be O(1) (a `HashSet` built once)"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** big-O is structural, not unit-assertable.
- **Suggested response:** defer-with-rationale — the guarantee is the type
  (`LazyLock<HashSet<&str>>`, verifiable by reading the code), which also retires the prior
  review's "linear `STOPWORDS.contains`" finding. A timing test would be flaky and meaningless.

### C-003: a few generic stragglers survive the stoplist
- **Where:** `e2e-tests.md` (shared-term list still shows `multi`, `low`)
- **Quote:** "189 → 92 (−51%)"
- **Failure mode:** weak-relation (residual noise)
- **Why it matters:** the stoplist is curated, not exhaustive; fragments like `multi` (from
  `multi-head`/`multilingual`, split on the hyphen) and `low` remain.
- **Suggested response:** defer-with-rationale — the intent explicitly targets a large, validated
  reduction, not perfection (INT-0018 non-goals). The result is technical-dominated; the residuals
  are minor and mostly tokenizer fragments. Phrase/bigram co-assertion (the deferred follow-on)
  would address hyphenated fragments more cleanly than piling onto the word list.

## Confidence
proceed-with-caveats
