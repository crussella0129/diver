# INT-0018 — Reduce co-assertion noise with a common-word stoplist

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0018
- **State:** active
- **Work evidence:** [Sprint 17 build plan](../sprints/s17/sprint-plans/build-plan.md) (T-1701, T-1702)
- **Completion evidence:** (pending)
- **Code evidence:** [diver-core/src/graph.rs](../../diver-core/src/graph.rs)
- **Test evidence:** [Sprint 17 test report](../sprints/s17/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md) (co-assertion links by distinctive terms)

## Intent

Cut the co-assertion noise the Sprint 16 real dive exposed ([[real-corpus-validation]],
INT-0017): `dive` links papers by shared significant claim terms
([[weighted-coassertion-temperature]], INT-0014), but generic English/academic words
(`eight`, `existing`, `literature`, `model`, `results`) and URL tokens (`https`, `github`)
produce spurious edges. A probe over the real 13-paper corpus showed the 189 shared terms are
dominated by such filler, while genuine signal (`transformer`, `attention`, `diffusion`,
`convolutional`, `translation`) is a minority.

- **Replace the tiny function-word `STOPWORDS`** (~65 words) with a substantial **common-word
  stoplist**: general high-frequency English words + generic research filler + web/URL tokens.
  `significant_terms` drops these before weighting, so only distinctive/technical terms remain
  and IDF then ranks *those*.
- **O(1) membership.** Load the stoplist once into a `HashSet` (from an embedded `stopwords.txt`
  via `include_str!`), replacing the linear `&[&str].contains` scan.
- **Weighting unchanged.** IDF + the `--temperature` dial are untouched; they now operate on a
  de-noised term set. IDF alone cannot fix this — a generic-but-corpus-rare word like `eight`
  (df 2) still scores `w = 1.0`; removing it before weighting is the only fix.

Non-goals:
- No change to the IDF weight formula, the `--temperature` dial, grounding, or storage.
- No phrase/bigram (multi-word) co-assertion — a distinct, larger enhancement, deferred.
- Not aiming for perfect precision — a large, validated noise reduction, not zero noise.

## Acceptance criteria

1. `significant_terms` drops a curated set of common English words, generic research filler
   (e.g. `model`, `results`, `method`, `approach`, `propose`, `novel`, `experiments`, `existing`,
   `new`, `large`), near-function words the old list missed (`even`, `however`, `between`, `due`),
   and web/URL tokens (`https`, `http`, `www`, `github`), while retaining domain terms
   (`attention`, `transformer`, `diffusion`, `convolutional`, `translation`, `bleu`).
2. Stoplist membership is O(1) (a `HashSet` built once), not a linear scan.
3. On the real corpus, the set of terms shared by ≥2 papers flips from filler-dominated to
   technical-dominated (validated by re-running the probe / the real-corpus test still finding a
   weighted co-assertion edge on a *technical* term).
4. All previously passing tests still pass (existing co-assertion tests' made-up terms are not
   in the stoplist, or are adjusted); the README notes the improved co-assertion signal.

## Rationale

The co-assertion graph's value is linking papers by *shared concepts*; generic vocabulary makes
that noisy, as the real dive showed side-by-side (`co-asserts convolutional` next to
`co-asserts eight (w=1.00)`). IDF handles corpus-common words but not general-common,
corpus-rare ones, so a general-frequency stoplist is the correct, low-risk lever — and it is
directly validated by real data.

## Alternatives

- **Phrase/bigram co-assertion** (link on `machine translation`, not `machine` + `translation`) —
  deferred: more distinctive but a larger algorithmic change; a good follow-on on this seam.
- **TF weighting** — rejected as the fix: within-paper term frequency does not distinguish
  generic from technical words.
- **Raise the default temperature threshold** — rejected: it drops signal along with noise
  (both live at similar weights); the stoplist removes noise while keeping signal.

## Consequences

- `graph.rs` gains an embedded `stopwords.txt` and a `LazyLock<HashSet<&str>>`; `significant_terms`
  checks it. Co-assertion edges become higher-signal on real corpora.
- Weighting/temperature semantics are unchanged; only the input term set is cleaner.
- Bigram/phrase co-assertion remains a documented future enhancement on this seam.

## Transition history
- 2026-09-02: created as `proposed` (after a probe over the real 13-paper corpus quantified the noise).
- 2026-09-02: `proposed` → `planned`; linked to Sprint 17 build plan (T-1701 stoplist + significant_terms + tests, T-1702 docs).
- 2026-09-02: `planned` → `active` (Sprint 17 build started; T-1701 first).
