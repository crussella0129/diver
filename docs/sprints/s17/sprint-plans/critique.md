# Plan Critique — Sprint 17

## Concerns

### C-001: the fixture-migration blast radius is only fully known at build time
- **Where:** `build-plan.md` T-1701 (adjust existing co-assertion test fixtures)
- **Quote:** "audit … for stopped words … and update each to a surviving term"
- **Failure mode:** hidden-dep
- **Why it matters:** enlarging the stoplist can silently stop a placeholder word used in an
  existing co-assertion test (`tfidf_corpus`'s `common`, or `models`/`accuracy`/`cost`/`improves`
  in `coassertion.rs`/graph tests), changing edge sets/counts and breaking assertions. The exact
  set is not knowable until the list is written and the suite is run.
- **Suggested response:** fix-in-build — all fixture adjustments live in T-1701 (compile/behavior-
  forced), and the task's boundary is a green `cargo test --workspace`, so every breakage is caught
  and fixed before commit. The plan explicitly enumerates the files to audit. Bounded.

### C-002: AC2 ("O(1) membership") has no runtime test
- **Where:** `test-plan.md` AC2 row
- **Quote:** "membership SHALL be O(1) (a `HashSet` built once)"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** big-O is a structural property, not something a unit test can assert.
- **Suggested response:** defer-with-rationale — the guarantee is the type: a
  `LazyLock<HashSet<&str>>` built once with O(1) `contains`, verifiable by reading the code. This
  also retires the prior review's "`STOPWORDS.contains` is a linear scan" finding, so the change is
  strictly better; a runtime timing test would be flaky and pointless.

### C-003: curation could over-stop a genuine domain term
- **Where:** `build-plan.md` T-1701 (`stopwords.txt` curation)
- **Quote:** "It **must not** contain domain terms (`attention`, `transformer`, `diffusion`, …)"
- **Failure mode:** intent-drift (quality)
- **Why it matters:** a word that is both common-English and technical (`network`, `model`,
  `state`, `field`) could be stopped, dropping some real bridges.
- **Suggested response:** defer-with-rationale — the list is curated from a *general* English
  frequency list (which by construction excludes domain jargon like `convolutional`/`diffusion`),
  plus the probe's observed filler; then **validated on the real corpus** (re-run `noise_probe.py`)
  and pinned by `test_significant_terms_stoplist` asserting domain terms survive. Very generic
  words that are borderline (`network`, `model`) are high-df and already IDF-down-weighted, so
  stopping them costs little signal. The goal is a large, validated reduction, not perfect recall.

## Confidence
proceed-with-caveats
