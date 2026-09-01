# Plan Critique — Sprint 11

## Concerns

### C-001: `dive` surfaces nothing until the user has run `diver extract`
- **Where:** `build-plan.md` T-1103 / `INT-0012` AC4
- **Quote:** "Papers with no extracted assertions do not appear as `dive` seeds"
- **Failure mode:** intent-drift (surprising empty result)
- **Why it matters:** a user with an ingested corpus but no extractions runs
  `diver dive X` and sees nothing, which could read as a bug.
- **Suggested response:** defer-with-rationale — intentional and correct: `dive`
  traverses the *epistemic* layer (extracted, validated, persisted assertions),
  which is the engine's output, not the raw abstract corpus (that is `diver find`).
  The empty path is explicitly handled with an actionable message ("Run `diver
  extract <id>` first"), tested by `test_dive_pipeline`'s empty case and the
  `e2e_dive_no_data` smoke, so the UX is clear rather than silent.

### C-002: `LIKE '%concept%'` matches substrings inside words
- **Where:** `build-plan.md` T-1102 Notes
- **Quote:** "`a.claim LIKE '%' || ?1 || '%'`"
- **Failure mode:** weak-assertion (over-matching)
- **Why it matters:** `dive cat` would match a claim containing "category" or
  "concatenate".
- **Suggested response:** defer-with-rationale — concept search is intentionally
  fuzzy for v1 (a user exploring "attention" wants "self-attention",
  "attention-based" too), and word-boundary/stemmed matching is a search-quality
  refinement, not a graph-layer concern. The seed query is parameterized (no
  injection) and case-insensitive per the AC; `test_papers_asserting_matches`
  pins the intended substring behavior so a future tightening is a conscious change.

### C-003: a node's `related` list can be large for common categories
- **Where:** `build-plan.md` T-1101 (`build_dive`) / `display.rs` `display_dive`
- **Quote:** "`related` = every relation whose `from`/`to` equals the node id"
- **Failure mode:** granularity (display noise)
- **Why it matters:** a paper in a popular category (e.g. `cs.LG`) relates to every
  other `cs.LG` paper, so the neighborhood could be long.
- **Suggested response:** fix-in-plan — `build_dive` returns the full edge set
  (correct and complete data), and `display_dive` renders a **bounded** view: show
  each node's claims, then up to a small cap (e.g. 10) of related papers with a
  "(+N more)" suffix when exceeded. This keeps the CLI readable without dropping
  data from the API. Recorded as a Build-phase display detail.

## Confidence
proceed-with-caveats
