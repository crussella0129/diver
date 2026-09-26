# Test Critique — Sprint 19

Two adversarial read-only rounds. Both are kept, because round 1 changed tests and records,
not just wording.

## Concerns

### Round 1 — verdict: block (8 concerns)

- **C-001 — README `--temperature 1.0` example comment still gave the old meaning, and the
  documentation review passed it** (EARS-coverage). The review grepped only "shared term".
  **Fixed:** comment now states the subsumption exception; the review checks every
  statement of `1.0`.
- **C-002 — deviations from locked clauses cited the wrong source** (evidence-drift): the
  suggestion-ordering change replaced T-1903's locked ordering (not AC2's, which has none),
  and the no-searchable-words message replaced part of T-1905/3. **Fixed:** a Deviations
  section in `unit-tests.md`; INT-0020 corrected.
- **C-003 — the CLI AC2 check could pass with the key paper missing** (weak-assertion): a
  bare id also appears in related-paper lines. **Fixed:** asserts the header's paper count
  and both claim bullets.
- **C-004 — the phrase-recurrence fixture could not tell two papers from two claims**
  (weak-assertion). **Fixed:** two claims of one paper; mutation caught.
- **C-005 — caps and paper-count ordering untested** (weak-assertion). **Fixed:** 12
  `<word> vortex` phrases exercise both caps (mutation caught); ordering finished in round 2.
- **C-006 — gate pre-filter equivalence rested on the out-of-repo probe; the structural
  fixture had no non-seed pair** (evidence-drift). **Fixed:** committed
  `test_coassertion_matches_naive_reference` (ordered `Vec`s, five temperatures; `>=`→`>`
  caught); non-seed pair added (scoping removal caught); probe output logged at head.
- **C-007 — drive evidence labelled with the tested head was produced by older binaries**
  (evidence-drift). **Fixed:** battery, timings, race and equivalence re-run on the head
  binary and logged; walls labelled as pre-repair observations with commits.
- **C-008 — records claimed coverage the assertions lacked** (evidence-drift). **Fixed:**
  freshness test asserts no rebuild on a matching fingerprint (the "always rebuild"
  mutation caught); display test asserts the top-5 cap line and echo truncation.

### Round 2 — verdict: proceed-with-caveats (6 concerns)

Round 1 disposition: C-001–C-004 and C-006–C-008 closed; C-005 partially closed.

- **C-101 — paper-count order was indistinguishable from id order** (weak-assertion).
  **Fixed:** the third paper moved to `transformer model`, so the two orders disagree;
  dropping the paper-count key is now caught.
- **C-102 — README said structural edges are "always shown"**, false since repair 1.
  **Fixed:** "do not depend on temperature"; the review re-read the section for other
  statements the repairs invalidated.
- **C-103 — the adverb rule's change to T-1902/3 was missing from the Deviations section.**
  **Fixed:** added.
- **C-104 — INT-0020 bookkeeping stale** ("four things"; no transition line). **Fixed.**
- **C-105 — the head race log did not show the forced staleness or final state.**
  **Fixed:** `race.sh` logs fingerprint before (`stale`) and after (current), exits,
  distinct outputs (1) and orphan links (0).
- **C-106 — generic words still reach top related lists at head; the records implied wall
  4 was fully repaired.** **Deferred with rationale:** recorded under Findings and in
  INT-0020 as a known limitation — no acceptance criterion promises zero generic edges, and
  a stoplist cannot enumerate them.

### Final disposition

Fourteen concerns over two rounds: thirteen fixed, one deferred with rationale (C-106). None
rejected. Round 2's fixes are narrow (one test fixture, documentation, logged evidence) and
each was verified — the new ordering assertion by mutation, the race by a scripted and logged
re-run — so a third round was not run. Mutation checks: nine mutations applied, all caught by
their named tests.

## Confidence
proceed-with-caveats

Caveats carried to Loop:
1. The claim layer is deterministic sentence splitting (no LLM key in this environment);
   concept formation is robust to it, but the epistemic layer is not doing epistemic work.
2. Generic vocabulary is reduced, not removed (C-106); word concepts are polysemous.
3. CI (`rust.yml`, build + test only) runs on the checkpoint PR, not on `dev`; clippy and fmt
   are local gates.
