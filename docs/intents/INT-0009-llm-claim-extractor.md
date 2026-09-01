# INT-0009 — LLM-backed claim extractor

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0009
- **State:** realized
- **Work evidence:** [Sprint 8 build plan](../sprints/s8/sprint-plans/build-plan.md) (T-801, T-802, T-803)
- **Completion evidence:** [T-801/T-802/T-803 completion](../work/completed-tasks.md#t-801--sprint-8)
- **Code evidence:** [diver-core/src/extract.rs](../../diver-core/src/extract.rs), [diver-cli/src/main.rs](../../diver-cli/src/main.rs)
- **Test evidence:** [Sprint 8 test report](../sprints/s8/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md) (`diver extract`, `--deterministic`, `ANTHROPIC_API_KEY`, `DIVER_MODEL`)

## Intent

Deliver the real extractor the typestate gate ([[typestate-assertion-core]],
INT-0008) was built to admit: replace the deterministic sentence-splitter with
claim extraction via the Claude API. In `diver-core`, introduce an
`LlmExtractor` that, given a stored `SourceFact`, asks Claude to extract the
factual claims the paper's abstract actually makes — each with a supporting quote
from that abstract — and produces `Assertion<Candidate>`s that flow through the
**existing** `Assertion::<Candidate>::validate()` gate.

Epistemic integrity is enforced by **grounding**: a claim becomes a candidate
only if its supporting quote actually appears in the abstract; hallucinated or
ungrounded quotes are dropped at extraction. Non-determinism is isolated to the
network boundary — a pure `parse_claims` function turns the API's JSON response
into grounded candidates and is fully unit-tested with fixtures.

Wire `diver extract <arxiv_id>` to use LLM extraction by default (requiring
`ANTHROPIC_API_KEY`), with a `--deterministic` flag preserving the offline
sentence-splitter path from INT-0008.

Non-goals:
- No streaming, tool use, or agent loop — a single Messages API call per paper.
- No Anthropic SDK — Rust has no official SDK; use raw HTTP over the existing
  `reqwest` client. No new crate dependencies.
- No new crate — the extractor lives in `diver-core`.
- No persistence of extracted assertions (still computed on the fly; a later
  sprint persists). No graph.
- No change to the typestate gate's core — `validate` is unchanged; grounding is
  enforced at extraction.

## Acceptance criteria

1. `diver-core` exposes an `LlmExtractor` that calls the Anthropic Messages API
   (`POST https://api.anthropic.com/v1/messages`, raw HTTP via `reqwest`) with the
   paper's abstract and returns `Vec<Assertion<Candidate>>` (each claim + its
   supporting `Observation`).
2. A pure `parse_claims(response_body, fact)` function turns a Messages API JSON
   response into grounded candidates and is covered by unit tests using fixture
   JSON — no network in the test suite.
3. Grounding: a claim is turned into a candidate only if its supporting quote is
   found in the paper's abstract; ungrounded quotes are dropped. Covered by tests
   (grounded quote kept, hallucinated quote dropped).
4. The model defaults to `claude-opus-5`, overridable via the `DIVER_MODEL`
   environment variable; the API key is read from `ANTHROPIC_API_KEY` and never
   hard-coded. A missing key yields a clear, actionable error.
5. `diver extract <arxiv_id>` uses LLM extraction by default and displays the
   supported assertions; `--deterministic` uses the INT-0008 sentence-splitter
   path; an unknown id errors like `inspect`.
6. Extracted candidates pass through the existing
   `Assertion::<Candidate>::validate()` gate unchanged; all previously passing
   tests still pass.

## Rationale

INT-0008 made epistemic integrity a compile-time property specifically so that
LLM output could be admitted only after validation — "downstream graph builders
literally cannot accept unvalidated LLM output." Deterministic sentence-splitting
produced trivial observations; this sprint delivers genuine knowledge units:
Claude reads each abstract and returns the claims it makes, grounded in quotes
from the source, which then pass the same gate. That is what makes the assertion
layer valuable and what the future graph and synthesis layers will consume.

## Alternatives

- **Use an Anthropic SDK** — rejected: there is no official Rust SDK; raw HTTP
  over the existing `reqwest` is the documented path for unsupported languages,
  and adds no dependency.
- **Tool use / structured outputs (`output_config.format`) for the response** —
  deferred: prompt-for-JSON plus tolerant parsing is adequate for v1; schema-
  constrained output is a future robustness refinement that does not change the
  pipeline.
- **Enforce grounding inside `validate()` (passing the source text)** — rejected
  for v1: grounding at extraction keeps the candidate constructor honest and the
  `validate` gate unchanged, and the type gate still guarantees only validated
  candidates reach downstream consumers.
- **Default to a cheaper model (Haiku/Sonnet)** — not the default: Anthropic
  guidance is to default to the most capable model and leave cost downgrades to
  the user; `DIVER_MODEL` makes the choice the user's.

## Consequences

- `diver extract` now performs a network call and needs `ANTHROPIC_API_KEY` by
  default; `--deterministic` preserves the offline path. Per-paper cost is the
  user's to manage (model configurable via `DIVER_MODEL`).
- `diver-core` gains an extractor module using the existing `reqwest` client; no
  new crate dependencies (`reqwest` + `serde_json` already present).
- LLM non-determinism is confined to the HTTP boundary; parsing and grounding are
  pure and tested. Live end-to-end extraction is verified manually (a real run
  with a key), not in the automated suite.
  - **Update (INT-0015, Sprint 14):** the HTTP boundary now has automated coverage.
    `extract`'s request construction (endpoint, `x-api-key`/`anthropic-version`
    headers, body) and its non-2xx error path are exercised by `wiremock`-backed
    transport tests against a mock server via an injectable `base_url`
    (`ANTHROPIC_BASE_URL`). Only real-endpoint reachability/credentials remain a
    manual check. See [[harden-extractor-http-boundary]].

## Transition history
- 2026-08-31: created as `proposed`.
- 2026-08-31: `proposed` → `planned`; linked to Sprint 8 build plan (T-801
  `parse_claims` + grounding, T-802 `LlmExtractor`, T-803 CLI `--deterministic`).
- 2026-08-31: `planned` → `active` (Sprint 8 build started; T-801 first).
- 2026-08-31: `active` → `realized` (Sprint 8: `LlmExtractor` calls the Claude
  Messages API over raw HTTP, `parse_claims` grounds claims against the abstract
  and feeds the existing `validate()` gate, `diver extract` defaults to LLM with a
  `--deterministic` fallback; 86 tests pass. Live HTTP call covered by the
  parse→validate integration test plus a documented manual run — see test
  report / critique C-001).
