# INT-0006 — Reconcile out-of-loop review hardening

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0006
- **State:** planned
- **Work evidence:** [Sprint 6 build plan](../sprints/s6/sprint-plans/build-plan.md) (T-601, T-602, T-603)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Eight hardening fixes were made during a GPT/Opus code review after Sprint 5
closed (commit `dd69859`, "Fix 8 review findings: FTS consistency, metadata
upsert, taxonomy caching"), then merged to `main` via PR #4 — entirely outside
the sprint loop. The Project Book has no intent chapter recording them, three of
the eight invariants carry no regression test, and `origin/dev` sits behind
`origin/main` by those commits.

Reconcile the loop with the released substrate:

1. Make this chapter the durable Book record of the eight fixes, with the commit
   and PR as code evidence.
2. Add regression tests for the invariants that landed untested — foreign-key
   enforcement, stale-FTS-on-older-version reingest, and taxonomy-cache
   determinism.
3. Bring `dev` into line with `main` so the two branches no longer diverge.

Non-goals:
- No new product behavior, schema change, or CLI change.
- No `Observation`/`Assertion`/semantic-layer types (deferred to a later sprint).
- Do not rewrite the Sprint 5 record; sprint provenance is append-only.

## Acceptance criteria

1. This chapter links commit `dd69859` and PR #4 as code evidence for all eight
   review fixes, and enumerates them.
2. A regression test asserts the `paper_versions → papers` foreign-key constraint
   is enforced (inserting a version whose parent paper is absent fails), locking
   in `PRAGMA foreign_keys=ON`.
3. A regression test asserts that re-ingesting an *older* version does not
   overwrite the FTS index with stale data (fix #1: FTS refresh reads the latest
   stored version, not the incoming fact).
4. A regression test covers taxonomy-cache determinism: repeated
   `ArxivCategory::parse` calls return equal, correct results (fix #4: `OnceLock`
   memoization).
5. `dev` contains every commit reachable from `main`; the branches are
   reconciled with no divergence.
6. All previously passing tests still pass.

## Rationale

Fixes merged outside the loop create a provenance gap: the Book's authority
(intents plus attached evidence) no longer reflects what the code does. Untested
invariants — especially the FK constraint and the subtle stale-FTS-on-reingest
case — can silently regress under future edits. Leaving `dev` behind `main`
guarantees the next sprint re-diverges from the released substrate. Recording the
fixes and locking their invariants restores the Book-as-authority invariant and
keeps the branches coherent.

## Alternatives

- **Leave the fixes unrecorded** — rejected: it permanently breaks the
  Book-as-single-source-of-truth invariant and makes the untested invariants
  easy to regress.
- **Amend the Sprint 5 record to absorb the fixes** — rejected: sprint records
  are append-only provenance; a terminal record must not be rewritten. A
  follow-on intent is the prescribed mechanism.
- **Trust the CI-green PR and skip added tests** — rejected: three invariants
  have no direct test, so "green" only proves the untouched paths still work.

## Consequences

- The Book now treats out-of-loop review hotfixes as first-class intents,
  reconciled at the next sprint boundary. Future review-driven fixes inherit the
  same small amount of ceremony.
- Local `dev` is fast-forwarded to `main`; pushing `origin/dev` to match is a
  remote-affecting step gated on explicit request / the remote profile.

## Transition history
- 2026-08-29: created as `proposed`.
- 2026-08-29: `proposed` → `planned`; linked to Sprint 6 build plan (T-601 FK
  enforcement, T-602 stale-FTS reingest, T-603 taxonomy-cache determinism).
