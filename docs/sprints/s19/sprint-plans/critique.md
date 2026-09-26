# Plan Critique — Sprint 19

Two adversarial read-only rounds were run. Both are retained because round 1 changed the
design, not only the prose.

## Concerns

### Round 1 — verdict: block (11 concerns)

- **C-001 — order-independence contradicted positional claim indices** (plan-test-mismatch). Reversing input remaps index `i` to `n−1−i`, so a correct AC6 would fail the AC3 test as written. **Fixed:** reversed-input equality is defined after that remapping.
- **C-002 — inflections split across the stoplist gave half-concepts** (missing-risk). The stoplist often lists one inflection only (`state` listed, `states` not), so `dive detail` missed `detail` claims and `dive part` failed while `dive parts` resolved. **Fixed in the design:** tokens are classified by their *folded* key against folded stoplists, common taking precedence, in all three consumers. INT-0020 records the resulting reclassifications.
- **C-003 — the monotonicity argument for phrase subsumption was false** (EARS-vague). With a filler head (`hidden state` / `hidden states`), raw classification could give the word a higher weight than the phrase. **Fixed:** folded classification restores the invariant (a phrase's papers ⊆ each content word's papers), now an explicit EARS clause with a test; suppression is unconditional; INT-0020's justification corrected.
- **C-004 — a T-1903 integration test needed T-1904** (hidden-dep). **Fixed**, and later made moot: under the user's operate-first direction all new tests are written in the Test Phase.
- **C-005 — stoplist edits did not change the fingerprint** (missing-risk). Persisted and in-pass concepts could disagree after an unversioned list edit. **Fixed:** the fingerprint includes a whitespace-insensitive digest of both lists.
- **C-006 — eight doubly-listed stopwords had no stated category** (EARS-vague). **Fixed:** they are common, matching the rule research validated, and are in the membership test.
- **C-007 — fold rules overlapped without precedence** (EARS-vague). **Fixed:** order stated and tested; the `bias`/`biases` limitation recorded.
- **C-008 — clause 1 contradicted subsumption; `t = 1.0` semantics silently changed; `N` undefined** (intent-drift). **Fixed:** clause 1 says "unless subsumed"; `N` keeps its meaning; INT-0020 records the refinement of INT-0014's endpoint (INT-0014, being realized, is not rewritten).
- **C-009 — taxonomy vocabulary neither delivered nor deferred** (intent-drift). **Deferred with rationale:** the Intent bullet is narrowed and the deferral recorded in Alternatives.
- **C-010 — the no-concepts branch had no API** (hidden-dep). **Fixed:** `concept_count()`, with an E2E check against an empty corpus.
- **C-011 — the research's CI/local-gate finding was absent** (missing-risk). **Fixed:** verification-gates sections in both plans.

### Round 2 — verdict: proceed-with-caveats (3 new concerns)

Round 1 disposition: all eleven closed.

- **C-101 — the fold rule makes derivational merges and misses common plurals, while the chapter claimed it only misses** (missing-risk). `semantics`→`semantic`, `news`→common `new`; `batches`, `gpus` not merged. **Fixed:** an `ics` exclusion (preferring a missed merge over a derivational one, per the chapter's own principle), and INT-0020 now lists the remaining missed-merge and wrong-merge classes explicitly; `test_fold` pins `semantics` and `metrics`.
- **C-102 — the `--temperature` help and README still state the old `t = 1.0` rule** (intent-drift). **Fixed:** T-1905 gains a help-text clause (concept argument and `--temperature`); T-1906 gains a clause that no README text says `1.0` admits every shared term.
- **C-103 — the subsumption clause omitted the gate, and its test's control edge had weight 0** (EARS-vague). **Fixed:** "subject to the gate" added; the test corpus gains a fourth paper and pins the control edge's temperature.

### Changes after round 2, and why no third round

Besides the three fixes above, both plans were changed under an explicit user direction (2026-09-24): operate the real binary in a proving ground first, write official tests afterwards, and keep them lean. The test plan now consolidates the new tests from roughly fifty to eighteen, still mapping every EARS clause, and names the proving-ground drive as primary E2E evidence; the build plan gained a Sequencing section and dropped test files from task Touches. No EARS clause or acceptance-criterion coverage was removed. A third round was not run: the round-2 fixes implement the critic's own suggested responses, and the remaining change is a user-directed restructuring of *when* and *how compactly* tests are written, not of what they prove.

## Confidence
proceed-with-caveats

Caveats carried to Build and Test:
1. Folding is a suffix heuristic with recorded missed- and wrong-merge classes (INT-0020 Consequences); the proving-ground drive should look for others on a larger, multi-topic corpus.
2. New tests land in the Test Phase, not per task; per-task commits are verified by the existing suite plus driving the binary.
3. INT-0020's documentation clauses (T-1906) are verified by recorded review, not an automated test.
