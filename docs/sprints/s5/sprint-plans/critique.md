# Sprint 5 Plan Critique

## Summary

The build plan and test plan for Sprint 5 are well-scoped and internally consistent. The eight tasks follow a sound dependency order: taxonomy JSON first, newtypes second (which depend on the taxonomy), parser and fact updates third and fourth, store schema split fifth (the highest-risk change), then CLI rename, display update, and docs. Every acceptance criterion in INT-0005 is mapped to at least one named EARS clause and at least one named test in the test plan.

## Concerns

- **Schema migration**: The `papers` + `paper_versions` schema split destroys all pre-Sprint-5 local databases. The plan documents this as acceptable (pre-1.0), which is correct, but T-505 and T-508 must both carry the migration note — currently only T-508 mentions README. The implementation should also add a clear error message if the binary detects an old schema at startup, rather than silently corrupting it. Acceptable to address during Build.
- **`display_fact` signature change**: T-507 adds a `versions: &[String]` parameter, which cascades into every call site in `main.rs`. This is a small but easily overlooked ripple. The build sequence handles it last (after T-505), which is correct.
- **Taxonomy coverage**: "A curated representative subset is sufficient for Sprint 5" is a pragmatic call. The taxonomy file should document its snapshot date so future sprints can reason about staleness.

No blocking issues. All concerns are addressable during Build without replanning.

## Confidence: `proceed-with-caveats`
