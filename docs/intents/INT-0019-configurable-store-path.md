# INT-0019 — Configurable store path for reproducible corpora

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0019
- **State:** active
- **Work evidence:** [Sprint 18 build plan](../sprints/s18/sprint-plans/build-plan.md) (T-1801, T-1802, T-1803)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

`Store::open()` hardcodes `dirs::data_dir()/diver/diver.db`. There is exactly one
on-disk corpus per machine, and the only escape is `open_in_memory()`, which
discards everything on drop. Let an explicit path override that default.

- **`DIVER_DB` environment variable.** When set, `Store::open()` uses that path
  instead of the platform data directory, creating parent directories as needed.
  When unset, behaviour is byte-for-byte what it is today.
- **A path-taking constructor.** `Store::open_at(path)` carries the real logic;
  `Store::open()` becomes "resolve the path, then `open_at`". Tests and future
  tooling get a first-class API rather than having to mutate process environment.

Non-goals:
- No multi-corpus management, no named profiles, no `--db` flag on every command.
  One override, one constructor. A CLI flag can come later if it earns its way in.
- No schema, migration, or query change.
- No change to `open_in_memory()`.

## Acceptance criteria

1. With `DIVER_DB` unset, the resolved path is unchanged from current behaviour.
2. With `DIVER_DB` set to a path in a temporary directory, `Store::open()` reads
   and writes there, and the platform data directory is untouched.
3. `Store::open_at(path)` creates missing parent directories and initializes the
   schema exactly as `open()` does.
4. A test persists a corpus to a scratch path, reopens it, and reads the same
   papers and assertions back — proving durable fixture corpora are now possible.
5. The README documents `DIVER_DB`.

## Rationale

This is a small maintenance item ([backlog](../work/tasks.md) T-1611) that turns out to sit
on the critical path. Every evaluation ambition in [[relation-evaluation-harness]]
(INT-0022) needs a fixed, versioned corpus that a test can open, assert against,
and throw away — one that survives a process boundary, so a corpus can be built
once and scored repeatedly.

Today there is no way to hold one. `open_in_memory()` dies on drop, and `open()`
resolves to a single fixed path per machine. The existing real-corpus test
([[real-corpus-validation]], INT-0017) works around this by parsing a checked-in
Atom fixture into an in-memory store on every run — which is genuinely
reproducible, but caps what can be tested at whatever fits in one fixture built
from scratch each time. A durable, versioned corpus on disk is a different
capability, and it is the one INT-0022 needs.

There is a second, sharper gap: `Store::open()` is reachable only from the CLI,
so nothing in the test suite exercises it at all. Making the path resolvable and
injectable is what lets that change.

Landing it now, in a sprint that otherwise writes no logic, keeps it from being
rediscovered as a blocker halfway through the work that needs it.

## Alternatives

- **A `--db` global CLI flag** — deferred, not rejected. It is the better *user*
  affordance, but it does not help library-level tests, which are where the
  reproducibility problem actually bites. The env var plus `open_at` covers both
  callers; a flag can be layered on later without redesign.
- **Always require an explicit path** — rejected: it breaks every existing
  invocation for a benefit only tests need.
- **Test-only `#[cfg(test)]` constructor** — rejected: it would not help the
  corpus-building and evaluation tooling that runs as a normal binary.

## Consequences

- `Store` gains one constructor and one env lookup; the default path resolution
  moves behind `open_at` but does not change.
- Fixture corpora become possible, unblocking [[relation-evaluation-harness]].
- A stray `DIVER_DB` in a developer's shell silently redirects their corpus. The
  README note is the mitigation; a future `diver inspect`-style path echo would
  be a cheap further guard.

## Transition history
- 2026-09-02: created as `proposed` during Sprint 18 roadmap realignment, promoted from backlog T-1611 after the code survey found it gates INT-0022.
- 2026-09-02: `proposed` → `planned`; linked to the Sprint 18 build plan (T-1801 `resolve_db_path` + `open_at` + tests, T-1802 README).
- 2026-09-02: plan re-scoped after plan-critic round 1 (still `planned`, no state change). `resolve_db_path` now takes the data directory as a parameter so every branch is pure and testable without env mutation, and `create_dir_all` moves into `open_at` so an override leaves the platform data directory untouched. A CLI subprocess test was added as T-1802 to cover the `DIVER_DB` env read itself, which no unit test reaches; the README task became T-1803. Round 2 additionally corrected this chapter's Rationale, which had claimed the real-corpus test shares the developer's database — it does not; it parses a checked-in fixture into an in-memory store.
- 2026-09-02: `planned` → `active` (Sprint 18 build started; T-1801 first).
