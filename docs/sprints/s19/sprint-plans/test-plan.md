# Sprint 19 Test Plan

Per user direction (2026-09-24): **operate first, test after, and keep tests lean.** The
primary E2E evidence is a proving-ground drive of the real binary against a multi-topic
corpus of real arXiv papers (see End-to-End Tests). The named tests below are written in
the Test Phase, after that drive has stopped finding defects. They are consolidated: each
test covers several EARS clauses, and the traceability table maps every clause to one.

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | Intent: stoplist split | T-1901 / WHEN `significant_terms` is called at this task's boundary THEN it SHALL return exactly what it returned before | `test_significant_terms`, `test_significant_terms_stoplist` (moved verbatim); existing co-assertion tests unmodified |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | Intent: stoplist split | T-1901 / WHEN the two files are loaded THEN disjoint, union equals the pre-split vocabulary | `test_stoplist_categories`; union equality by one-off recorded check against `git show HEAD:diver-core/src/stopwords.txt` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | Intent: stoplist split | T-1901 / WHEN a word is classified THEN web tokens, the eight doubly-listed words and `the`/`of`/`with` common; `model`/`mechanism`/`learning`/`score` filler | `test_stoplist_categories` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | Intent: inflection only | T-1902 / WHEN `fold` receives a token THEN first matching rule: `ies` → `sses` → `ics` unchanged → `s`; WHEN no rule matches THEN unchanged | `test_fold` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | Consequences: folded classification | T-1902 / WHEN two tokens fold to the same key THEN the same category everywhere | `test_folded_classification_consistent` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC1, AC4 | T-1902 / WHEN a claim contains a significant term THEN a `Term` concept `fold(term)` with the surface form and count | `test_concept_formation` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC4 | T-1902 / phrase candidates form for (content, content)/(content, filler); do not form across filler modifier, common/web token, token-rule failure, or punctuation | `test_phrase_rule` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC4 | T-1902 / WHEN a phrase is in fewer than two papers THEN no concept; two or more THEN a `Phrase` | `test_concept_formation` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC5 precondition | T-1902 / WHEN a `Phrase` is formed THEN its papers ⊆ each constituent `Term`'s papers | `test_formation_invariants`; `test_real_corpus_concepts` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC4 | T-1902 / WHEN two papers contain `machine translation` THEN three distinct concepts | `test_concept_formation`; `test_real_corpus_concepts` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | Intent: no derivational or head merge | T-1902 / WHEN claims contain `attention`, `attentional`, `attention mechanism` THEN three distinct concepts | `test_concept_formation` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC1 (label) | T-1902 / WHEN several forms THEN label = most frequent, ties lexicographically smallest | `test_concept_formation` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC3 | T-1902 / WHEN run twice THEN equal with identical `Debug`; WHEN reversed THEN equal after mapping `i` → `n − 1 − i` | `test_formation_invariants`; `test_real_corpus_concepts` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC6 | T-1902 / WHEN a concept is formed THEN it records every contributing claim index and paper | `test_formation_invariants` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC2 | T-1902 / WHEN `query_key` normalizes THEN tokenize, drop common, fold, join; `None` when empty | `test_query_key` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC7 (rules changes) | T-1902 / `digest_words` stable under spacing; sensitive to add, remove, move | `test_digest_words` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC1 | T-1903 / WHEN a store opens THEN the four tables exist, links cascading from `assertions` | `test_concept_resolution` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC1, AC2, AC6 | T-1903 / `resolve_concept` returns id, label, kind, count, forms or `None`; `claims_for_concept` returns exactly the linked claims, ordered | `test_concept_resolution` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC1, AC2 | T-1903 / WHEN `papers_asserting(q)` THEN the resolved concept's claims, or empty | `test_papers_asserting_matches`, `test_papers_asserting_empty` (unmodified); `test_concepts_pipeline`; `test_real_corpus_concepts` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC1 (not substring) | T-1903 / WHEN a query is a substring of a stored word but not a concept THEN nothing | `test_concept_resolution` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC1 (not substring) | T-1903 / WHEN a query contains `%` or `_` THEN no match through them | `test_papers_asserting_wildcards_inert` (replaces `test_papers_asserting_escapes_like_wildcards`, in the T-1903 commit) |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC7 | T-1903 / WHEN claims are written after a build THEN the next read reflects them, including a removal; WHEN version or digest differs THEN rebuild and store the current fingerprint | `test_concepts_freshness`; `test_concepts_pipeline` (across `open_at` reopen) |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | Intent: linked, not merged; AC2 | T-1903 / `concepts_related_to`, `concept_suggestions` (all-token then any-token, ordered, capped, empty store → empty), `concept_count` | `test_concept_navigation` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC5 | T-1904 / WHEN two papers share a concept THEN an edge naming its label with the unchanged IDF weight, unless subsumed | existing co-assertion tests in `graph.rs`, `tests/coassertion.rs`, `tests/real_corpus.rs` with **unmodified assertions**; `test_coassertion_concepts` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC5, AC2 | T-1904 / `networks` + `network` → exactly one edge | `test_coassertion_concepts`; `test_concepts_pipeline` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC5, AC4 | T-1904 / shared phrase → phrase edge, subject to the gate; shared phrase + its word → no word edge for that pair at any temperature, other pairs keep it subject to the gate | `test_coassertion_concepts` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC5 | T-1904 / WHEN temperature increases, incl. `hidden state`/`hidden states` THEN non-decreasing | `test_coassertion_monotonic_with_phrases`; `test_real_corpus_dive` subset check (unmodified) |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC2, AC6 | T-1905 / resolved header (label, kind, count, forms, linked concepts); unresolved → suggestions; no concepts → `diver extract` hint | `test_format_dive_concept`; `test_cli_dive_concepts`; `test_cli_dive_empty_and_help` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC2 | T-1905 / `diver dive networks` lists papers saying `networks` and `network` | `test_cli_dive_concepts`; proving-ground drive |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC7 | T-1905 / claims added between two runs appear in the second, no rebuild command | `test_cli_dive_concepts`; proving-ground drive |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | AC2 (help text) | T-1905 / `diver dive --help` describes concept resolution and the refined `t = 1.0` | `test_cli_dive_empty_and_help` |
| [INT-0020](../../../intents/INT-0020-first-class-concepts.md) | Consequences made visible | T-1906 / README `dive` section explains resolution, folding, phrases, suggestions, derivational change, subsumption; `--temperature` text states refined `1.0` | Recorded documentation review at Test Phase |

## Verification gates
Every task boundary and the Test Phase run, locally: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`. CI builds and tests only, on pull requests to and pushes on `main` — not on `dev` — so the Test Phase records local results, and the authoritative CI conclusion is recorded when the checkpoint PR runs.

## Unit Tests

### `diver-core/src/concept.rs`
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- `test_significant_terms`, `test_significant_terms_stoplist` — moved verbatim from `graph.rs`.
- `test_stoplist_categories` — the files are disjoint; `https`, `http`, `www`, `github`, `com`, `propose`, `provides`, `recent`, `show`, `simple`, `widely`, `work`, `works`, `the`, `of`, `with` are common; `model`, `mechanism`, `learning`, `score` are filler.
- `test_fold` — `networks`→`network`, `rnns`→`rnn`, `families`→`family` (not `familie`), `classes`→`class` (not `classe`), `studies`→`study`; unchanged: `analysis`, `corpus`, `process`, `loss`, `gas`, `attention`, `semantics`, `metrics`.
- `test_folded_classification_consistent` — `state`/`states`, `detail`/`details`, `input`/`inputs`, `part`/`parts`, `effect`/`effects`, `condition`/`conditions`, `network`/`networks` each classify identically.
- `test_phrase_rule` — forms: `language model`, `machine translation`, `encoder-decoder`. Breaks: `model establishes`, `attention of translation`, `page https`, `gpt 4 model`, `50 layers`, `translation, attention`.
- `test_concept_formation` — `"Neural networks learn."` + `"A network generalizes."` → one `network` concept with both forms and both papers; a phrase in one paper (even repeated) → no concept, in two → `Phrase`; `machine translation` in two papers → `machine translation`, `machine`, `translation` distinct; `attention` / `attentional` / shared `attention mechanism` → three distinct; label `{networks: 3, network: 1}` → `networks`, a 1–1 tie → the smaller form.
- `test_formation_invariants` — on a corpus including `hidden state` (P1–P3) / `hidden states` (P4–P5) and `machine translation`/`translation` papers: every `Phrase`'s papers ⊆ each constituent `Term`'s papers; forward twice → equal with identical `Debug`; reversed → equal after remapping; each concept's claim indices and papers are exactly those containing it.
- `test_query_key` — `"The Diffusion Models"` → `"diffusion model"`, `"NETWORKS"` → `"network"`; `""`, `"the of"`, `"50"` → `None`.
- `test_digest_words` — `"a b\r\nc"` and `"a  b\nc"` equal; adding, removing, or moving a word between lists changes it.

### `diver-core/src/store.rs`
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- `test_papers_asserting_matches`, `test_papers_asserting_empty` — **unmodified**.
- `test_papers_asserting_wildcards_inert` — replaces `test_papers_asserting_escapes_like_wildcards` in the T-1903 commit: with claims stored, `%`, `_`, `atten%`, `50%` match nothing.
- `test_concept_resolution` — four tables exist and deleting an assertion removes its links; `resolve_concept("Diffusion Models")` → id `diffusion model`, phrase, count 2, both forms; unknown → `None`; `claims_for_concept` exact and ordered, excluding an unlinked claim in the same paper; `gan` and `net` against claims containing `organized` and `network` → nothing.
- `test_concepts_freshness` — resolve, save a second paper, resolve → included; replace a paper's claims without a term → the term stops resolving; forge a fingerprint differing in version, then one differing only in digest, each after deleting a concept row → the next read restores it and stores the current fingerprint.
- `test_concept_navigation` — `attention` → related includes `attention mechanism`, and the reverse; `model` → suggestions include `diffusion model`; a query whose tokens never share a concept falls back to any-token; at most 10; empty store → empty; `concept_count` is `0` on an empty store and on an all-stopword corpus, positive otherwise.

### `diver-core/src/graph.rs`
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- **All existing co-assertion tests with unmodified assertions.** Two fixtures gain comma separators because they now form shared phrases (`"rare mid common"` / `"mid common"`; `"Zebra apple mango."` / `"Mango zebra apple."`).
- `test_coassertion_concepts` — forms `{networks: 2, network: 1}` → edge `term` is the label `networks`; `networks` (A) + `network` (B) → one edge; shared `diffusion model` → a phrase edge; A–B share `machine translation`, C shares only `translation` with A, D has neither → A–B carries the phrase and no word edge at t = 0.0, 0.5, 1.0; A–C carries `translation` at t = 1.0.
- `test_coassertion_monotonic_with_phrases` — corpus with shared phrases at several document frequencies including `hidden state` / `hidden states` → edge sets t = 0.0 ⊆ 0.5 ⊆ 1.0.

### `diver-core/src/display.rs`
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- `test_format_dive_concept` — a term header shows label, `term`, count, forms, `narrower:`; a phrase header shows `broader:`; unresolved with suggestions lists them; unresolved with no concepts gives the `diver extract` hint.

## Integration Tests

### `diver-core/tests/concepts.rs` (new)
- **Intents:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- `test_concepts_pipeline` — on a scratch path via `Store::open_at`: papers saying `networks` and `network` → `papers_asserting("networks")` returns both; `compute_coassertion_relations` links them by one edge naming the concept and `build_dive` lists each as the other's related paper; drop, reopen, save a third paper → it resolves with no rebuild call.
- `test_real_corpus_concepts` — the checked-in 7-paper fixture, offline: `machine translation` resolves as a phrase in at least two papers; `network` carries both forms; `papers_asserting("networks")` includes *"We propose a new simple network architecture, the Transformer…"*, which substring matching misses; every phrase's papers ⊆ its terms' papers; forward and reversed formation equal after remapping.

### Existing integration binaries (regression guard)
- `tests/coassertion.rs`, `tests/dive_graph.rs`, `tests/real_corpus.rs` run **unmodified**, now exercising concept resolution and concept-keyed edges.

## End-to-End Tests
- **Status:** possible

### Proving-ground drive (primary evidence)
An isolated directory outside the repository holds scratch corpora (`DIVER_DB`) so the user's real corpus is never touched. The real release binary is used to `collect` several hundred real arXiv papers across unrelated fields (e.g. ML, quantum physics, number theory, biology), extract them, and then operate every command adversarially: `dive` over common, rare, plural, phrasal, stoplisted, misspelled, empty, Unicode and wildcard-laden terms at several temperatures; `find`, `list`, `inspect`, `assertions`; re-extraction and additions between runs; and timing on the large corpus. Every defect found is fixed on `dev` and re-driven. The drive log — commands, observed output, defects, and fixes — is recorded in `sprint-tests/e2e-tests.md` and is the sprint's primary E2E evidence.

### Hermetic CLI tests (`diver-cli/tests/dive_concepts.rs`, new)
Each test seeds a scratch corpus through `diver_core` and runs the real binary via `env!("CARGO_BIN_EXE_diver")` with `DIVER_DB` set by `Command::env`, comparing output after stripping ANSI escapes. Corpus: A `"Diffusion models generate images."`, B `"A diffusion model denoises networks."`, C `"Neural network training converges."` — checked under folded classification: `model`/`models` and `training` are filler, `a` is common, the rest content, so `diffusion model` forms (A, B) and `model` alone does not.
- `test_cli_dive_concepts` — `dive networks` lists B and C with a header showing both forms (C is the paper substring matching missed); `dive model` says it is not a concept and suggests `diffusion model`; adding D `"Diffusion sampling accelerates."` through the library → the next `dive diffusion` lists D.
- `test_cli_dive_empty_and_help` — an empty scratch corpus → `dive anything` gives the `diver extract` hint; `dive --help` describes concept resolution and the refined `t = 1.0`.
