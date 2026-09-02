# Sprint 16 End-to-End Tests

- **Tested head:** `bcf520f5204158e45125c89db0950df7d4d5acab`
- **Status:** possible (offline) — and additionally exercised live during research.

## `test_real_corpus_dive` IS the offline end-to-end
It runs the entire pipeline (ingest → deterministic extract → persist → weighted dive graph)
over real captured arXiv content and asserts the graph — the offline, reproducible end-to-end
of the whole engine on real data.

## Executed live (research probe, on-network)
The Sprint 16 research phase ran the real flow against the live arXiv API and confirmed it
end-to-end:
- `diver ingest 1706.03762` → *Attention Is All You Need*;
- `diver collect "attention transformer neural machine translation" --max-results 6` → 6 papers;
- `diver extract --all --deterministic` → **Extracted 7 paper(s).**;
- `diver dive attention` → real structural (`shared category cs.CL`) and weighted co-assertion
  (`co-asserts decoder (w=0.68)`, `co-asserts task (w=1.00)`) edges, with the display cap
  summarizing overflow.

## Manual verification (this sprint)
- `diver extract --help` lists `[ARXIV_ID]` (optional) + `--all` + `--deterministic`;
- `diver extract` (no args) is rejected by clap ("required arguments were not provided");
- `diver extract --all --deterministic` extracted the whole stored corpus.

## Coverage note
Real **LLM** extraction quality (grounding on model output) still needs a provider key and stays
a manual check; the deterministic path (real abstract sentences) is what the automated E2E uses.
