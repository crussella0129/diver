# Sprint 17 Meta

- **Sprint number:** 17
- **Book schema version:** 2
- **Start timestamp:** 2026-09-02T04:46:29Z
- **End timestamp:** 2026-09-02T06:44:38Z
- **Model:** claude-opus-4-8
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Reduce co-assertion noise (INT-0014 follow-on): replace the tiny function-word STOPWORDS with a curated common-word stoplist (general English + generic research filler + web/URL tokens) in an O(1) HashSet, so `significant_terms` keeps only distinctive/technical terms before IDF weighting. Quantified and validated on the real 13-paper corpus.
- **Intents:** [INT-0018](../../intents/INT-0018-coassertion-stoplist.md)
- **Completion evidence:** INT-0018 realized: common-word co-assertion stoplist; real-corpus shared terms 189->92, technical-dominated; 132 tests pass, clippy 0
