# Test Critique — Sprint 16

## Concerns

### C-001: the `diver extract --all` handler is verified manually, not by an automated binary test
- **Where:** `unit-tests.md` T-1601 / `build-plan.md` T-1601 EARS
- **Quote:** "WHEN `diver extract --all` runs, THEN it SHALL extract every stored paper …"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** the `--all` handler (iterate `Store::list`, per-paper extract, the summary
  line, and the empty-store message) is only exercised by manual runs; no automated test invokes
  the binary with `--all`.
- **Suggested response:** defer-with-rationale — the handler is a thin loop over `Store::list`
  calling `extract_and_save`, whose deterministic path `test_real_corpus_dive` drives over an
  entire real corpus at the library level, and clap enforces the argument contract. No
  populated-DB binary-invocation harness exists (consistent with every prior CLI handler). The
  behavioral core — extract every stored paper → persisted assertions → weighted graph — is
  proven; a fixture-DB CLI E2E is future hardening.

### C-002: the fixture is reconstructed from ingested content, not a raw live-API capture
- **Where:** `integration-tests.md` "Fixture provenance" / `tests/fixtures/real_corpus_feed.xml`
- **Quote:** "re-serialized offline into the Atom feed shape `parse_feed` consumes"
- **Failure mode:** evidence-drift
- **Why it matters:** the intent asked for a "real … feed fixture"; the committed XML is the real
  arXiv content (seven papers ingested live during the probe) re-emitted into the Atom format
  offline, because arXiv rate-limited fresh raw captures at build time. A reader should not
  mistake it for a byte-for-byte API response.
- **Suggested response:** no-change (transparently documented) — the *content* (titles, abstracts,
  authors, categories, versions) is genuine arXiv data, and the fixture is parsed by the real
  `parse_feed` and drives the real pipeline, so it validates exactly what the intent wanted:
  behavior on real content. The provenance is stated in the test module doc and the integration
  record. Re-capturing a raw response when the rate limit clears is optional polish.

## Confidence
proceed-with-caveats
