# Sprint 19 Documentation Review

- **Tested head:** `d08eaa61cbccb32a468e72b06b3588874a638637`
- **Reviewer:** Claude Opus 5.5 (primary agent), Sprint 19 Test Phase, 2026-09-24
- **Subject:** T-1906 — `README.md` `## Concept exploration (diver dive)`; T-1905's help-text
  clause is automated (`test_cli_dive_empty_and_help`).
- **Verdict:** **pass**

## T-1906 clause 1 — the `dive` section explains:
| Required | Where / quoted | Result |
|----------|----------------|--------|
| terms resolve to concepts | "It resolves the term to a **concept**…" | pass |
| plural forms are one concept | "Plural forms are the same concept: `diver dive networks` also finds the papers that only say `network`" | pass |
| shared two-word phrases are concepts | "Two adjacent words that at least two papers share are a concept in their own right" | pass |
| unresolved terms list containing concepts | "A term that is not a concept says so and lists the concepts containing it" (plus coverage ranking and *did you mean*, from the drive) | pass |
| derivational variants distinct; a change from substring matching | "Earlier versions matched any claim *containing* the text, so `gan` matched 'or**gan**ized'… `attention` does not reach `attentional`" | pass |
| phrase subsumption | "When two papers share a phrase, they are linked by the phrase alone — not additionally by each of its words" | pass |

## T-1906 clause 2 — `--temperature`
- States: "`1.0` links every shared concept, except words already covered by a phrase the
  same two papers share." **pass**
- Every statement of `1.0` carries the exception — the prose bullet and the example
  comment (`# every shared concept, minus words a shared phrase covers`). Checked by
  listing every README line mentioning `1.0`, not by grepping one phrase: an earlier
  version of this review grepped only for "shared term" and missed a stale example
  comment ("every shared concept links"), which test-critique C-001 caught and which
  was then fixed. **pass**

## Also documented (drive repairs)
- Grouped, ranked related papers with the category-only count — matches
  `display::format_related` as tested in `test_format_dive_concept`.
- The whole section was re-read for statements the drive repairs made false. One was
  found (test-critique C-102): "structural (category/author) edges are always shown",
  written before repair 1, when category-only papers stopped being listed. Reworded to
  "structural … edges do not depend on temperature". **pass** after the fix.
