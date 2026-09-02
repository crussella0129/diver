# Test Critique — Sprint 18

Two adversarial read-only rounds were run against the test evidence. Round 1 is retained
in full because its findings changed the implementation, not just the prose.

## Concerns

### Round 1 — verdict: block (4 concerns)

- **C-001 — `test_default_db_path_matches_legacy` did not pin the composition it advertised**
  (weak-assertion). It asserted `resolve_db_path(None, dirs::data_dir())`, re-deriving the
  call-site argument instead of reading it, so it constrained only the helper — duplicating
  `test_resolve_db_path_default` — while three artifacts claimed it guarded the call site.
  A double-join written at the call site would have shipped 142/142 green and silently
  relocated every existing corpus. **Response: fixed in code** — the composition was
  extracted into a named function that both `Store::open()` and the test call, and the fix
  was verified by fault injection rather than inspection (see round 2, C-101, which
  tightened it further).
- **C-002 — INT-0019 AC5's documentation review was planned but had no execution record**
  (evidence-drift). A manual verification step with no recorded outcome is indistinguishable
  from an unverified one. **Response: fixed** — [documentation-review.md](documentation-review.md)
  records reviewer, date, head, verdict, and a clause-by-clause check of all three SHALLs
  against quoted README text.
- **C-003 — the shipped default-db guard was narrower than the locked test plan, undisclosed**
  (evidence-drift). The plan specified an existence + mtime + `-wal` snapshot; the shipped
  test observes existence only. **Response: fixed** — [e2e-tests.md](e2e-tests.md) states the
  deviation and why the dropped observations could not have delivered what the plan hoped.
- **C-004 — the integration record contradicted itself** (integration-drift). The new
  `db_override.rs` binary was listed under a heading asserting nothing was new or modified,
  and the 142 total only reconciled if the reader knew one test was cited twice.
  **Response: fixed** — [integration-tests.md](integration-tests.md) opens with a scope note
  reconciling 132 + 8 unit + 2 E2E and stating plainly that this sprint adds no
  integration-only test; the E2E row was removed from the pre-existing table.

### Round 2 — verdict: proceed-with-caveats (5 concerns)

Round 1 disposition: C-001, C-002, C-003, C-004 all closed.

- **C-101 — the C-001 fix introduced a test that self-disables** (flake-risk). The extracted
  function still read the environment, so the test needed an `if DIVER_DB is set { return }`
  guard — meaning the workspace's only guard on the default composition became a silent
  no-op in exactly the environment this sprint teaches developers to create, with no
  `#[ignore]` and no output. **Response: fixed in code** — the environment read was split out
  (`current_db_path_for(override)` holds the composition; `current_db_path()` supplies
  `var_os`), so the test asserts unconditionally and needs no guard. Re-verified by fault
  injection against the new seam.
- **C-102 — `Store::open()`'s own delegation is still unpinned** (weak-assertion). The test
  constrains `current_db_path_for`; that `open()` reaches it is asserted by nothing, and the
  same double-join written inside `open()` would leave the suite green.
  **Response: deferred with rationale, and the artifacts corrected.** The residual lines
  contain no path composition, and testing them directly would require calling
  `Store::open()` with `DIVER_DB` unset — writing to the developer's real corpus. The gap is
  now stated as a bounded gap in [unit-tests.md](unit-tests.md) rather than papered over
  with a full-coverage claim.
- **C-103 — the default-db guard's recorded "pass" asserted nothing on this machine**
  (evidence-drift). `%APPDATA%\diver\diver.db` already exists here, so `existed_before` was
  true and the guarded branch never executed. **Response: fixed** — [e2e-tests.md](e2e-tests.md)
  now labels that result "pass (vacuous on this run)" and says the guard carries signal only
  on a clean machine or in CI. AC2's evidence never depended on it.
- **C-104 — the durable round-trip did not assert the support quotes it persisted**
  (weak-assertion). A reopened corpus that had lost its `assertion_support` rows would still
  have passed, and AC4 is the criterion INT-0022 depends on. **Response: fixed in code** —
  version and support are now asserted.
- **C-105 — `test-report.md` was still empty** (evidence-drift). **Response: fixed** — the
  report is written; this concern flagged an in-progress placeholder, not a defect.

### Final disposition

Nine concerns across two rounds: seven fixed (four in code, three in the evidence records),
one deferred with rationale (C-102), one an in-progress artifact since completed (C-105).
No concern was rejected.

Round 2's fixes were verified the same way round 1's were — by injecting the exact fault
each test claims to catch and confirming it fails — then reverting and re-running the full
suite green. A third round was not run: C-101 and C-104 were narrow, directly implemented
the critic's own suggested responses, and were re-verified empirically; C-102 and C-103 were
resolved by correcting overclaims in the records rather than by changing behaviour.

## Confidence
proceed-with-caveats

Caveats carried to Loop Phase:
1. **C-102:** `Store::open()` reaches the pinned composition through two delegating lines
   that no test asserts. Deliberate — they hold no composition, and covering them would
   write to the real corpus.
2. **C-103:** `test_cli_diver_db_override_leaves_default_db_unmodified` is vacuous on any
   machine that already has a corpus, including this one. It is a clean-machine/CI guard,
   not part of AC2's evidence.
3. INT-0019 AC5 is verified by recorded human review, not by an automated test.
