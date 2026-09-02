# Plan Critique — Sprint 18

Three adversarial read-only rounds were run against the plans before locking.
Rounds 1 and 2 are retained in full because their concerns changed the design, not
just the prose. The final verdict is at the bottom.

## Concerns

### Round 1 — verdict: block (10 concerns)

- **C-001 — the `DIVER_DB` override was never executed by any test** (plan-test-mismatch).
  `resolve_db_path` and `open_at` were each covered in isolation, but the one composing
  line reading the environment had no coverage; a `var`/`var_os` slip would ship green.
  **Response: fixed in plan** — added T-1802, a CLI subprocess test using
  `Command::env` (safe; no `set_var`) and `env!("CARGO_BIN_EXE_diver")`.
- **C-002 — "platform data directory untouched" was verified by nothing** (missing-risk).
  Today `open()` calls `create_dir_all(&data_dir)` before computing the path
  (`store.rs` l.37-40); an implementer leaving that in place would violate criterion 2
  while passing every planned test. **Response: fixed in plan** — `create_dir_all` moves
  into `open_at`, `resolve_db_path` is required to be side-effect-free, and
  `test_resolve_db_path_is_side_effect_free` proves it. The critic's suggested mechanism
  (redirect `APPDATA`/`XDG_DATA_HOME`) was **rejected**: on Windows `dirs::data_dir()`
  resolves through `SHGetKnownFolderPath`, not `%APPDATA%`, so the redirect would not
  apply on this sprint's own platform and the test would pass vacuously. Parameterizing
  the data directory achieves the same end platform-independently.
- **C-003 — `DIVER_DB=""` was unspecified** (EARS-vague). `std::env::var_os` returns
  `Some("")`, and SQLite treats an empty filename as a private temporary database that
  vanishes on close — silent data loss. **Response: fixed in plan** — empty is treated as
  unset, with an EARS clause and `test_resolve_db_path_empty_override`.
- **C-004 — `tempfile` dev-dependency undeclared** (hidden-dep). **Response: fixed in
  plan** — declared in Touches and Notes.
- **C-005 — E2E `not-yet-possible` was misapplied** (e2e-drift). The rationale (fixture
  format belongs to INT-0022) was sound for an evaluation-quality E2E but not for an
  env-var override, which needs no fixture at all. **Response: fixed in plan** — E2E is
  now `possible`, with `not-yet-possible` re-scoped to the evaluation run.
- **C-006 — the no-data-directory fallback was untestable as designed**
  (plan-test-mismatch). **Response: fixed in plan** — same parameterization as C-002.
- **C-007 — T-1803 pinned the new doc to a stale schema-migration section**
  (granularity). **Response: fixed in plan** — the EARS clause pins no section.
- **C-008 — a mitigation named in INT-0019 Consequences had no task or deferral**
  (missing-risk). **Response: deferred with rationale** — recorded under `## Deferrals`
  and assigned to Loop Phase.
- **C-009 — sprint metadata still held Plan-Phase placeholders** (intent-drift).
  **Response: fixed in plan** — Summary and Intents filled.
- **C-010 — unverified prior-art claims lost their qualifier when they moved into the
  durable chapters** (intent-drift). In a project whose charter says "extraction is never
  truth", a qualifier living only in disposable sprint provenance will read as fact later.
  **Response: fixed in plan** — qualifiers carried into INT-0021, INT-0022, and INT-0024.

### Round 2 — verdict: proceed-with-caveats (5 new concerns)

Round 1 disposition: C-001, C-003, C-005, C-006, C-007, C-008, C-009, C-010 closed;
C-002 and C-004 partially closed (see below).

- **C-201 — the default path through `open()` had zero executable coverage**
  (plan-test-mismatch). `Store::open()` has no caller outside `diver-cli/src/main.rs`,
  and every existing test uses `open_in_memory()`, so the claimed "regression guard" could
  not guard `open()` at all. The natural refactor error is silent and compiles: carrying
  `.map(|d| d.join("diver"))` into both the helper and the call site yields
  `<data>/diver/diver/diver.db`, relocating every user's corpus with a green suite.
  **Response: fixed in plan** — new EARS clause and `test_default_db_path_matches_legacy`;
  the regression-guard paragraph rewritten to state precisely what the suite does and does
  not prove.
- **C-202 — the data-directory test could pass vacuously** (plan-test-mismatch).
  **Response: fixed in plan** (see C-301, which corrected the fix).
- **C-203 — INT-0019's Rationale asserted something the code contradicts**
  (intent-drift). It claimed the real-corpus test "has to share the developer's own
  database"; verified false — `real_corpus.rs` l.34-44 parses a checked-in fixture into
  `Store::open_in_memory()`, and the fixture holds 7 entries, not 13. **Response: fixed in
  plan** — Rationale rewritten around the accurate gap (no *durable* corpus survives a
  process boundary; `open()` is untested because nothing but the CLI calls it), and the
  research-report row corrected.
- **C-204 — `dirs` is not reachable from `diver-cli`** (hidden-dep). `diver-core/src/lib.rs`
  has no `pub use`, so the dependency is not transitive. **Response: fixed in plan** —
  T-1802 Notes name `tempfile` and `dirs = "6"`, pinned to match `diver-core` so the test
  resolves the same directory the binary does.
- **C-205 — INT-0019's evidence and history were stale after the re-scope**
  (intent-drift). **Response: fixed in plan** — Work evidence now (T-1801, T-1802, T-1803);
  a new transition line records the re-scope without rewriting the earlier accurate one.

### Round 3 — verdict: proceed-with-caveats (2 new concerns)

Round 2 disposition: C-201, C-203, C-204, C-205 closed; C-202 partially closed — the
rename and stated limits were real improvements, but the *stated reason* the check
discriminates was wrong.

- **C-301 — the "unmodified default DB" clause promised more than any available
  observation could decide** (plan-test-mismatch). The claim that modified-time plus WAL
  presence discriminates is **false**: SQLite deletes the `-wal`/`-shm` sidecars when the
  last connection closes cleanly, and against an already-initialized corpus
  `PRAGMA journal_mode=WAL` plus `CREATE TABLE IF NOT EXISTS` write nothing, so mtime need
  not change. **Response: fixed in plan** — the EARS clause narrowed to what is decidable
  ("SHALL NOT be created" when absent); the snapshot demoted to an explicitly best-effort
  clean-machine guard; the real regression signal named (an ignored override leaves the
  scratch path absent, failing `test_cli_diver_db_override_creates_db_at_path`, backed by
  the hermetic `test_resolve_db_path_is_side_effect_free`); and the concurrent-write flake
  risk disclosed with a standing instruction to delete the guard rather than weaken the
  two tests that carry the signal.
- **C-302 — no task or phase owned closing backlog T-1611, whose intent tag is wrong**
  (intent-drift). It is the item this sprint realizes, is still `- [ ]`, and is tagged
  `[intent: INT-0017]`. **Response: fixed in plan** — a new `## Loop Phase bookkeeping`
  section assigns the closure, the tag correction, and the new deferred backlog entry
  explicitly, rather than leaving them to an assumed default.

### Final disposition

All seventeen concerns raised across three rounds are addressed: fifteen fixed in plan,
one deferred with rationale (C-008), one rejected with reasoning (the round-1 mechanism
suggested for C-002, superseded by a platform-independent fix that closed the same gap).

Round 3's two concerns were closed by narrow, targeted edits that implement the critic's
own suggested responses — a weakened EARS clause, relocated coverage claims, and an
explicit bookkeeping assignment. No new design surface was introduced, so a fourth round
was not run.

## Confidence
proceed-with-caveats

Caveats carried into the Build and Test phases:
1. `test_cli_diver_db_override_leaves_default_db_unmodified` is a best-effort
   clean-machine guard and must not be read as proof that the override was honored.
2. INT-0019 acceptance criterion 5 has no automated verification; it is asserted by
   documentation review at Test Phase.
3. Loop Phase owns three backlog edits that no build task touches.
