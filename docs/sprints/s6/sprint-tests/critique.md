# Test Critique — Sprint 6

## Concerns

### C-001: INT-0006 AC1 evidence (code-evidence link + 8-fix enumeration) is not yet in the chapter
- **Where:** `INT-0006` Acceptance criteria AC1 / `test-plan.md` Intent
  Traceability row "AC1"
- **Quote:** "This chapter links commit `dd69859` and PR #4 as code evidence for
  all eight review fixes, and enumerates them."
- **Failure mode:** intent-coverage
- **Why it matters:** AC1 is a documentation criterion with no EARS clause or
  automated test. At the tested head the INT-0006 chapter references `dd69859` in
  prose but its **Code evidence** field is still `none` and it lacks an explicit
  enumeration of the eight fixes, so AC1 is not yet provable by doc review.
- **Suggested response:** defer-with-rationale — AC1's code-evidence link and the
  enumeration are realization evidence, attached when INT-0006 moves `active →
  realized` in the Loop Phase (mirroring how Sprint 5 realized INT-0005 in its
  close commit). The Test Phase adds Test evidence only and explicitly does not
  mark intents realized. Tracked as a required Loop-Phase step.

### C-002: Command-body E2E (search/ingest/collect execution) not exercised through the binary
- **Where:** `e2e-tests.md` "Coverage note"
- **Quote:** "Full behavioral E2E of `search`/`ingest`/`collect` command bodies
  requires a live arXiv network call and writes to the real data dir"
- **Failure mode:** e2e-cop-out
- **Why it matters:** the E2E smokes prove the CLI *surface* (`--help`) but not
  that a subcommand's body still runs end-to-end after the crate split.
- **Suggested response:** defer-with-rationale — INT-0007's acceptance boundary is
  structural (same behavior, new crate layout), not new behavior. The command
  bodies call library functions that the moved integration tests
  (`test_ingest_pipeline` full parse→save→get round-trip, `test_find_pipeline`
  search) exercise deterministically under `diver_core::`. A binary-level network
  E2E would add an `assert_cmd`/network dependency, an explicit sprint non-goal,
  and introduce flake. Surface parity + library integration is sufficient
  evidence for a no-behavior-change restructure.

## Confidence
proceed-with-caveats
