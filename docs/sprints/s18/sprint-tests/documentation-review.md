# Sprint 18 Documentation Review

- **Tested head:** `f651bb6d9353dda127bbfcfc223300ccf719225a`
- **Reviewer:** Claude Opus 5 (primary agent), Sprint 18 Test Phase, 2026-09-02
- **Subject:** [INT-0019](../../../intents/INT-0019-configurable-store-path.md) acceptance
  criterion 5 — "The README documents `DIVER_DB`."
- **Verdict:** **pass**

## Why this artifact exists

AC5 is the sprint's one acceptance criterion with no automated verification: the locked
test plan records it as "Documentation review at Test Phase (no automated test; asserted by
reading the rendered section)." The test critic (C-002) correctly observed that a manual
step with no recorded outcome is indistinguishable from an unverified one. This file is that
record.

## Reviewed content

`README.md`, new section `### Corpus location (DIVER_DB)`, placed under
`## Claim extraction → Building a corpus` — where the other environment-variable
configuration (`DIVER_PROVIDER`, `DIVER_PROVIDERS_CONFIG`) already lives, rather than under
the pre-Sprint-5 `## Database compatibility` migration warning. That placement was an
explicit T-1803 decision after plan-critic round 1 (C-007) flagged the original pinning.

## Clause-by-clause verification

T-1803's EARS clause carries three SHALLs. Each is checked against the committed text:

1. **SHALL state that `DIVER_DB` overrides the default corpus path.**
   > "By default the corpus lives at the platform data directory — `~/.local/share/diver/diver.db`
   > on Linux, `%APPDATA%\diver\diver.db` on Windows. Set **`DIVER_DB`** to keep a corpus
   > somewhere else…"

   Present, with both platform defaults named and a three-command worked example.
   **pass**

2. **SHALL state that an unset or empty value selects the default.**
   > "An **unset or empty** `DIVER_DB` selects the default — `DIVER_DB=` is deliberately
   > treated as unset, because SQLite reads an empty filename as a private temporary
   > database that would be discarded on exit."

   Present, and it gives the reason rather than just the rule — which matters, because the
   behaviour is otherwise surprising. **pass**

3. **SHALL warn that a stray value silently redirects the corpus.**
   > "**Warning:** `DIVER_DB` silently redirects the corpus. A stray value left exported in
   > your shell will make `list`, `find`, and `dive` look empty — the papers are not lost,
   > you are simply pointed at a different database. Check the variable before concluding a
   > corpus has disappeared."

   Present as a blockquote warning. It names the observable symptom (commands look empty)
   and corrects the wrong conclusion a user would otherwise draw (data loss), which is the
   specific hazard INT-0019's Consequences section identified. **pass**

## Notes

- The documented behaviour matches the implementation: `resolve_db_path` in
  `diver-core/src/store.rs` filters an empty `OsString` to the default branch, and
  `Store::open_at` creates missing parent directories, as the section claims.
- INT-0019's Consequences names a second mitigation — a `diver inspect`-style path echo —
  which is **not** in this sprint. It is recorded as a deliberate deferral in the build
  plan and is filed to the backlog at Loop Phase.
