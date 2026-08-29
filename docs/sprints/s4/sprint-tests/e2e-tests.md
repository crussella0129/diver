# Sprint 4 End-to-End Test Results

**Status:** not-yet-possible

The `diver collect` command requires live ArXiv API access. Automated E2E
testing is blocked by the same network dependency constraint as `diver search`
and `diver ingest`.

**Unlocked by:** a future intent introducing API mocking or recorded HTTP
fixtures for the ArXiv client.

**Manual verification plan:**
1. `diver collect "attention mechanisms" --max-results 3` — verify per-paper status + summary
2. `diver collect "xyznonexistent"` — verify "No papers found." message
3. `diver collect "attention" --sort-by submitted --max-results 2` — verify sort ordering
