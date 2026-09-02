# INT-0022 — Relation evaluation harness and gold set

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0022
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none

## Intent

Diver's tests prove the pipeline *runs*. Nothing measures whether what it
produces is *right*. Extraction quality has so far been judged by reading output
and finding it plausible ([[real-corpus-validation]], INT-0017) — which is how a
project ends up confidently wrong at scale.

Build the measurement layer before the thing it measures:

- **A versioned gold set**, checked into the repo: a fixed corpus of papers with
  hand-labelled expected claims and, once [[typed-epistemic-relations]] (INT-0021)
  exists, hand-labelled expected relations between them. Small and honest beats
  large and guessed — dozens of carefully labelled relations are worth more than
  thousands of assumed ones.
- **A harness that scores a run against the gold set** and reports per-relation-type
  precision and recall, so `contradicts` performing badly is visible instead of
  averaged away by `supports` performing well.
- **Labelling provenance.** Who labelled a relation, when, and against which
  paper version. Gold sets rot as silently as code.
- **Disagreement is data.** Where a human labeller was unsure, or two labellers
  disagreed, record that rather than forcing a label. An item humans cannot agree
  on is not a fair test of the machine, and knowing which items those are is
  itself a finding.

Non-goals:
- No benchmark leaderboard, no target score. The harness reports; humans judge.
- No automatic gold-set generation from model output. A gold set labelled by the
  system it evaluates measures nothing.
- Not a replacement for the qualitative north-star test in the [charter](../README.md#north-star) —
  they measure different things and both are needed.

## Acceptance criteria

1. A fixture corpus and its labels live in the repo, versioned, reproducible via
   `DIVER_DB` ([[configurable-store-path]], INT-0019).
2. The harness scores a pipeline run against the gold set and reports precision
   and recall **per relation type**, not just in aggregate.
3. Re-running against an unchanged corpus and unchanged extractor produces an
   identical score.
4. Labelling provenance and inter-labeller disagreement are representable and
   preserved.
5. The gold set covers, at minimum, one confirmed instance of each relation type
   in INT-0021's starting vocabulary, plus negative cases — claim pairs that
   share vocabulary but stand in *no* epistemic relation. Negatives are what
   catch a system that relates everything to everything.

## Rationale

The [external review](../history/2026-09-02-external-review-gpt-5-6.md) lists "evaluate relations, not
just extraction" ninth in a list of principles. It belongs closer to first. Every
ambition above it — contradiction detection, independent rediscovery, tracing an
idea's history — is a claim about *accuracy*, and accuracy claims without
measurement are marketing.

There is also a sequencing argument. Building INT-0021 first and evaluating it
afterwards means every design question in that intent — the relation vocabulary,
the SPO-versus-claim-text shape, the confidence threshold — gets settled by
whoever argues most persuasively, because no evidence exists to settle it. The
harness converts those arguments into experiments. That is worth more than the
sprint it costs.

The honest cost: labelling is slow, unglamorous, and requires domain judgement.
That is exactly why it gets skipped, and exactly why it should not be.

## Alternatives

- **Evaluate after building INT-0021** — the conventional order; rejected above.
  The harness is cheap to build and expensive to retrofit, and its absence turns
  design questions into opinion contests.
- **LLM-as-judge scoring** — deferred, not rejected. Useful later for cheap
  regression signal over a larger sample. Cannot be the primary standard: it
  evaluates a model's relation judgements with a model's relation judgements, and
  the correlated errors are invisible.
- **Reuse an external benchmark** (SciFact, SciTail, citation-intent corpora) —
  worth investigating when this intent is scheduled, and it might save
  significant labelling. Deferred on the understanding that those corpora are
  shaped around sentence-level entailment over their own document sets rather
  than claim-to-claim relations over an arXiv corpus, so adapting them may cost
  more than it saves. **That characterization is from memory and was not verified
  against the actual datasets** — check it first, because if it is wrong this
  alternative is much cheaper than the primary approach. An open question, not a
  closed one.
- **Score only aggregate accuracy** — rejected: it hides exactly the failure that
  matters, since `supports` is common and easy while `contradicts` is rare and
  hard.

## Consequences

- New fixture corpus, label format, and a scoring binary or test harness.
- Depends on INT-0019 for a reproducible store path.
- Gates INT-0021's `realized` state: that intent is not done on a working
  pipeline, it is done on a measured one.
- Ongoing maintenance cost — the gold set must be re-checked when extraction
  prompts or paper versions change. Accepted deliberately.
- Makes it possible to say something falsifiable about Diver's quality, which is
  a precondition for anyone outside the project trusting it.

## Transition history
- 2026-09-02: created as `proposed` during Sprint 18 roadmap realignment; deliberately elevated above the external review's own priority ordering, with reasoning recorded above.
