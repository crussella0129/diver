# Sprint 1 — End-to-End Test Results

- **Status:** not-yet-possible (automated)
- **Unlocked by:** A future sprint adding recorded-response (wiremock) E2E
  testing. Live ArXiv API access in CI is unreliable due to rate limits.

## Manual E2E verification

Performed manually during build phase at head `6d68d19`.

| Scenario | Command | Result |
|----------|---------|--------|
| Basic search | `diver search "transformer attention" --max-results 3` | 3 papers displayed with title, authors, abstract, category, link. "Showing 3 of 18414 results." |
| Sort by submitted | `diver search "quantum computing" --max-results 2 --sort-by submitted` | 2 recent papers sorted by submission date |
| Missing query | `diver search` (no arg) | Clap usage error, non-zero exit |
