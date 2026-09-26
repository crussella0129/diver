Finalized - DO NOT EDIT

# Sprint 19 Build Plan

## Intents
- [INT-0020](../../../intents/INT-0020-first-class-concepts.md) — state: planned; acceptance criteria covered: AC1 (persisted concepts with stable id, label, surface forms; claims linked, not substring-matched), AC2 (resolution reaches other surface forms; unresolved terms name containing concepts), AC3 (byte-deterministic formation), AC4 (multi-word concepts distinct from their words), AC5 (co-assertion names a concept; temperature semantics and tests hold, with the `t = 1.0` refinement recorded in the chapter), AC6 (traceability to claims and papers), AC7 (never stale).

## Schema Tree
- Sprint Goal — replace substring concept matching with deterministic, persisted concept identity
  - Vocabulary layer
    - T-1901: extract tokenization into `concept.rs`; split the stoplist into phrase-breaking and head-eligible categories
    - T-1902: pure, deterministic concept formation — folding, folded classification, phrases, labels, traceability, rules digest
  - Persistence and resolution
    - T-1903: persist concepts behind a freshness fingerprint; resolve, trace, relate, suggest and count through them
  - Graph
    - T-1904: re-key co-assertion onto concepts, with phrase subsumption
  - Surface
    - T-1905: `diver dive` resolves through concepts; resolved header, unresolved suggestions, empty-corpus hint
    - T-1906: document concept resolution in the README

## Verification gates
Every task boundary runs, locally: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo fmt --check`. CI (`.github/workflows/rust.yml`: build + test only) triggers on pull requests to and pushes on `main`, not on `dev`, so it cannot see a task boundary; its authoritative result arrives with the sprint's checkpoint PR and is recorded then.

## Sequencing (per user direction, 2026-09-24)
Operate first, test after. Each task lands as a commit that keeps the **existing** suite, clippy and fmt green, and its EARS clauses are checked at the boundary by driving the real `diver` binary against scratch corpora (`DIVER_DB`). After T-1906, a **proving-ground drive** takes place: a multi-topic corpus of real arXiv papers built in an isolated directory, then operated adversarially through every command, with defects fixed as they surface (fix commits are attributed to the task whose clause they repair). The new named tests in the test plan are written in the Test Phase, only once the drive has stopped finding defects, and they are deliberately consolidated — one test may cover several clauses. The drive log becomes the sprint's primary E2E evidence.

## Execution Sequence

### T-1901: Extract the vocabulary layer into `concept.rs` and split the stoplist into two categories
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- **Touches:** `diver-core/src/concept.rs` (new), `diver-core/src/stopwords_common.txt` (new), `diver-core/src/stopwords_filler.txt` (new), `diver-core/src/stopwords.txt` (deleted), `diver-core/src/graph.rs`, `diver-core/src/lib.rs`
- **Depends on:** (none)
- **Acceptance criterion:** INT-0020 Intent, "Phrases form from adjacent words by a category rule" — common words break a phrase; research filler may head one. Prerequisite for AC4 and AC5.
- **Success criterion (EARS):**
  - **WHEN** `significant_terms` is called on any input at this task's boundary, **THEN** it **SHALL** return exactly what it returned before the split (its stopword set is the union of the two files).
  - **WHEN** the two stoplist files are loaded, **THEN** they **SHALL** be disjoint, and their union **SHALL** equal the vocabulary of the pre-split `stopwords.txt`.
  - **WHEN** a word is classified, **THEN** the web tokens `https`, `http`, `www`, `github`, `com` **SHALL** be common; the eight words listed in both pre-split blocks — `propose`, `provides`, `recent`, `show`, `simple`, `widely`, `work`, `works` — **SHALL** be common; `the`, `of`, `with` **SHALL** be common; and `model`, `mechanism`, `learning`, `score` **SHALL** be filler.
- **Notes:** `STOPWORDS` and `significant_terms` move from `graph.rs` into `concept.rs` together with `test_significant_terms` and `test_significant_terms_stoplist`, **verbatim** — assertion bodies byte-identical. That, with every co-assertion test running unmodified, is the proof for the first clause. The pre-split file has two uncommented alphabetical blocks (common English, then research filler from `abstract`); the split follows that boundary, dedupes, assigns the eight doubly-listed words to common (research validated its phrase rule with common taking precedence, and assigning them to filler would let them head phrases like `future work`), and moves the web tokens to common. Union equality is verified once against `git show HEAD:diver-core/src/stopwords.txt` and the command and output recorded in the unit-test record; no permanent vocabulary-size test, which would fire on every deliberate edit. `graph.rs` imports `significant_terms` from `concept`, so behaviour is unchanged until T-1902/T-1904 and no dead code appears.

### T-1902: Form concepts deterministically from `(paper, claim)` pairs
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- **Touches:** `diver-core/src/concept.rs`
- **Depends on:** T-1901
- **Acceptance criterion:** AC3 (deterministic), AC4 (`machine translation` distinct from `machine` and `translation`), AC6 (traceable); Intent "many-to-one — for inflection only"; Consequences "tokens are classified by their folded form" and "stoplist edits force a rebuild".
- **Success criterion (EARS):**
  - **WHEN** `fold` receives a token, **THEN** it **SHALL** apply the first matching rule in this order: longer than 4 and ending in `ies` → replace `ies` with `y`; longer than 4 and ending in `sses` → drop `es`; ending in `ics` → unchanged; longer than 3 and ending in `s` but not `ss`, `us` or `is` → drop `s`.
  - **WHEN** `fold` receives a token matching no rule (e.g. `analysis`, `corpus`, `process`, `loss`, `gas`, `attention`, `semantics`, `metrics`), **THEN** it **SHALL** return it unchanged.
  - **WHEN** two tokens fold to the same key, **THEN** `significant_terms`, the phrase rule and `query_key` **SHALL** give them the same category — the category of the folded key against the folded stoplists, common taking precedence over filler.
  - **WHEN** a claim contains a significant term, **THEN** `form_concepts` **SHALL** produce a `Term` concept whose id is `fold(term)` and whose surface forms include the lowercased term with its occurrence count — so `network` and `networks` are one concept carrying both forms.
  - **WHEN** two adjacent tokens in the same clause are (content, content) or (content, filler), **THEN** they **SHALL** form a phrase candidate with id `fold(a) fold(b)`.
  - **WHEN** the first token is filler, **OR** either token is common, **OR** either fails the token rule (fewer than 3 characters, or no letter), **OR** a character from `. , ; : ! ? ( ) [ ] { } "` separates them, **THEN** they **SHALL NOT** form a phrase candidate. Hyphens and slashes do not separate (`encoder-decoder` forms `encoder decoder`).
  - **WHEN** a phrase candidate occurs in claims of fewer than two distinct papers, **THEN** it **SHALL NOT** become a concept; **WHEN** it occurs in two or more, **THEN** it **SHALL** become a `Phrase` concept.
  - **WHEN** a `Phrase` concept is formed, **THEN** for each of its words that is itself a `Term` concept, the phrase's papers **SHALL** be a subset of that term's papers.
  - **WHEN** claims from two papers contain `machine translation`, **THEN** `form_concepts` **SHALL** produce three distinct concepts: `machine translation`, `machine` and `translation`.
  - **WHEN** claims contain `attention`, `attentional` and `attention mechanism` (the phrase shared by two papers), **THEN** `form_concepts` **SHALL** produce three distinct concepts — no derivational merge and no merge of a phrase into its head.
  - **WHEN** a concept has several surface forms, **THEN** its label **SHALL** be the form with the most occurrences, ties broken by the lexicographically smallest form.
  - **WHEN** `form_concepts` is run twice on the same claims, **THEN** the results **SHALL** be equal with identical `Debug` output; **WHEN** it is run on the claims in reverse order, **THEN** the result **SHALL** equal the forward result once each claim index `i` is mapped to `n − 1 − i`.
  - **WHEN** a concept is formed, **THEN** it **SHALL** record the index of every input claim that contributed to it and the set of papers those claims belong to.
  - **WHEN** `query_key` normalizes a query, **THEN** it **SHALL** apply the same tokenization, drop common tokens, fold each remaining token and join them with single spaces (`"The Diffusion Models"` → `"diffusion model"`); **WHEN** nothing remains, **THEN** it **SHALL** return `None`.
  - **WHEN** `digest_words(common, filler)` receives the same two word lists, **THEN** it **SHALL** return the same value regardless of line endings or spacing; **WHEN** a word is added to, removed from, or moved between the lists, **THEN** it **SHALL** return a different value. `rules_digest()` applies it to the two embedded stoplists.
- **Notes:** std only — no new dependency. `significant_terms` becomes the unigram rule of formation and switches to folded classification here; its two moved tests still pass unmodified (each input traced in planning: `improves` folds to listed `improve`/`improves`, `epochs` to unlisted `epoch`, and every noise word in the stoplist test stays listed). Reclassified by this change on the current lists: plurals of listed singulars (e.g. `states`, `inputs`, `details` → filler; `parts`, `ones` → common) and six singulars whose plural was listed (`application`, `condition`, `consist`, `contain`, `contribution`, `effect` → filler); the full list is recorded in the unit-test record. Ordered collections (`BTreeMap`/`BTreeSet`) throughout, since Rust randomizes `HashMap` iteration per instance. `digest_words` is FNV-1a 64 over the whitespace-split words (so CRLF checkouts do not change it), with a separator between the two lists. `pub const CONCEPT_FORMATION_VERSION: u32 = 1` covers code-level rule changes; the digest covers list edits. Formation uses claim text only — arXiv taxonomy vocabulary is deferred per the chapter's Alternatives.

### T-1903: Persist concepts behind a freshness fingerprint, and resolve, trace, relate, suggest and count through them
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- **Touches:** `diver-core/src/store.rs`
- **Depends on:** T-1902
- **Acceptance criterion:** AC1 (persisted; claims linked, not substring-matched), AC2 (other surface forms reached; unresolved terms name containing concepts), AC6 (traceable), AC7 (never stale).
- **Success criterion (EARS):**
  - **WHEN** a store is opened, **THEN** tables `concepts`, `concept_forms`, `assertion_concepts` and `meta` **SHALL** exist, with `assertion_concepts` rows cascading on delete from `assertions`.
  - **WHEN** `resolve_concept(q)` is called and `query_key(q)` names a formed concept, **THEN** it **SHALL** return that concept's id, label, kind, paper count and surface forms with counts; **WHEN** it names none, **THEN** it **SHALL** return `None`.
  - **WHEN** `claims_for_concept(id)` is called, **THEN** it **SHALL** return `(arxiv_id, claim)` for exactly the claims linked to that concept, ordered by paper then assertion id.
  - **WHEN** `papers_asserting(q)` is called, **THEN** it **SHALL** return `claims_for_concept` of `resolve_concept(q)`, or an empty vec when `q` does not resolve — keeping its signature, so its existing callers exercise concept resolution.
  - **WHEN** a query is a substring of a stored word but not itself a concept (`gan` against a claim containing `organized`; `net` against `network`), **THEN** `papers_asserting` **SHALL** return nothing.
  - **WHEN** a query contains the characters `%` or `_`, **THEN** `papers_asserting` **SHALL NOT** match any claim through them.
  - **WHEN** claims are written by `save_assertions` after concepts were built, **THEN** the next concept read **SHALL** reflect the new claims with no explicit rebuild call — including a replacement that removes the last claim forming a concept, after which that concept **SHALL NOT** resolve.
  - **WHEN** the stored fingerprint differs from the current one in its formation version or its rules digest, **THEN** the next concept read **SHALL** rebuild concepts and store the current fingerprint.
  - **WHEN** `concepts_related_to(id)` is called for a `Term`, **THEN** it **SHALL** return the `Phrase` concepts containing it; for a `Phrase`, the `Term` concepts of its words that exist — ordered by paper count descending then id, capped at 10.
  - **WHEN** `concept_suggestions(q)` is called, **THEN** it **SHALL** return concepts whose id contains every folded token of `q` as a whole word, or, if there are none, those containing any of them — ordered by paper count descending then id, capped at 10; on an empty store, **THEN** it **SHALL** return an empty vec.
  - **WHEN** `concept_count()` is called, **THEN** it **SHALL** return the number of formed concepts after ensuring freshness — `0` for an empty store or one whose claims form no concept.
- **Notes:** The fingerprint is `v{CONCEPT_FORMATION_VERSION}:{rules_digest}:{COUNT(assertions)}:{MAX(assertions.id) or 0}`, stored in `meta`. Assertions are written only by `save_assertions`, which deletes a paper-version's rows and inserts new ones; `AUTOINCREMENT` never reuses ids, so any insert changes `MAX(id)` and any net deletion changes `COUNT`. A claim's paper is fixed at insert (`paper_id` is never updated, and no API deletes papers), so paper membership cannot change without an assertion write. `store.save` touches only `papers`, `paper_versions` and FTS, none of which formation reads. Every concept read (`resolve_concept`, `claims_for_concept`, `concepts_related_to`, `concept_suggestions`, `concept_count`, and so `papers_asserting`) first calls `ensure_concepts_fresh`, which rebuilds in full inside one transaction on mismatch. Rebuild reads claims with assertion ids `ORDER BY p.arxiv_id, a.id`, runs `form_concepts`, and maps claim indices back to assertion ids. Concept ids are folded keys, stable across rebuilds and corpus growth. Suggestions and relations use whole-word containment over ids in Rust, not `LIKE`. `test_papers_asserting_matches` and `test_papers_asserting_empty` run **unmodified**. `test_papers_asserting_escapes_like_wildcards` is **replaced** by `test_papers_asserting_wildcards_inert`: it asserted that `"50%"` matched a claim containing `50%`, substring behaviour that no longer exists (numbers are not concepts); its safety intent — wildcard characters cannot widen a match — is re-asserted against concept resolution. Because `test_papers_asserting_escapes_like_wildcards` asserts the old substring behaviour, it is replaced in this task's commit, so the existing suite stays green at the boundary.

### T-1904: Re-key co-assertion onto concepts, with phrase subsumption
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- **Touches:** `diver-core/src/graph.rs`
- **Depends on:** T-1902
- **Acceptance criterion:** AC5 — co-assertion edges name a concept; existing temperature semantics and tests hold, with the `t = 1.0` refinement recorded in INT-0020 Consequences.
- **Success criterion (EARS):**
  - **WHEN** two papers' claims share a concept, **THEN** `compute_coassertion_relations` **SHALL** emit an edge whose `term` is that concept's label and whose weight is `ln(N/df)/ln(N/2)` — `df` the concept's paper count, `N` the number of distinct papers in the claims input — unless the edge is subsumed under the fourth clause below. The temperature gate, small-corpus guard and sorted output are unchanged.
  - **WHEN** one paper's claim says `networks` and another's says `network`, **THEN** exactly one co-assertion edge **SHALL** link them, naming the concept.
  - **WHEN** a pair of papers shares a `Phrase` concept, **THEN** an edge naming the phrase **SHALL** be emitted, subject to the gate.
  - **WHEN** a pair shares a `Phrase` concept and a `Term` concept that is one of its words, **THEN** the `Term` edge **SHALL NOT** be emitted for that pair at any temperature, while a different pair sharing only the `Term` **SHALL** still receive it, subject to the gate.
  - **WHEN** temperature increases on a corpus containing shared phrases — including one whose phrase appears in both a singular and a plural form (`hidden state` / `hidden states`) — **THEN** the emitted edge set **SHALL** be non-decreasing.
- **Notes:** Calls `concept::form_concepts` on the same claims `dive` persists. The persisted set rebuilds whenever the rules digest changes, so the two cannot disagree. Suppression is unconditional. Under the invariant from T-1902 (a phrase's papers are a subset of each of its content words' papers), the phrase's weight is never below the word's, so unconditional suppression equals suppressing only when the phrase edge clears the gate — which is why the edge set stays monotonic. `N` keeps its current meaning, counting every distinct paper with a claim even if that paper forms no concept. The `compute_coassertion_relations` doc comment is updated for concepts and for the refined `t = 1.0` endpoint. The `CoAssertion { term, weight }` field name is **kept** so every existing co-assertion test runs with unmodified assertions as the AC5 proof; a rename to `concept` is deferred. Two synthetic `graph.rs` fixtures were bags of content words that now legitimately form shared phrases — the `"rare mid common"` / `"mid common"` corpus and the `"Zebra apple mango."` / `"Mango zebra apple."` pair — and gain comma separators (`"rare, mid, common"`); their assertions stay byte-identical. The other fixtures were checked against the rule and form no shared phrase.

### T-1905: `diver dive` resolves through concepts, with a resolved header, suggestions for unresolved terms, and an empty-corpus hint
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- **Touches:** `diver-cli/src/main.rs`, `diver-core/src/display.rs`
- **Depends on:** T-1903, T-1904
- **Acceptance criterion:** AC2 end-to-end, and AC7 end-to-end.
- **Success criterion (EARS):**
  - **WHEN** `diver dive networks` runs against a corpus where one paper's claim says `networks` and another's says only `network`, **THEN** its output **SHALL** list both papers.
  - **WHEN** a term resolves, **THEN** the output **SHALL** begin with the concept's label, kind and paper count, its surface forms with counts, and its linked concepts (narrower phrases for a term, constituent terms for a phrase).
  - **WHEN** a term does not resolve and `concept_count()` is positive, **THEN** the output **SHALL** say the term is not a concept and list the concepts `concept_suggestions` returns — so `diver dive model` names `diffusion model` when two papers share it.
  - **WHEN** a term does not resolve and `concept_count()` is `0`, **THEN** the output **SHALL** tell the user to run `diver extract` first.
  - **WHEN** a paper's claims are added to the corpus between two `diver dive` runs, **THEN** the second run's output **SHALL** include that paper with no rebuild command in between.
  - **WHEN** a user runs `diver dive --help`, **THEN** the `concept` argument **SHALL** be described as resolved to a concept (not "matched against stored assertion claims"), and `--temperature` **SHALL** state that at `1.0` every shared concept links except words subsumed by a shared phrase.
- **Notes:** `main.rs` chooses the branch by `resolve_concept`, then `concept_count`. Header and message text come from pure `format_…` functions in `display.rs` returning `Vec<String>`; the `display_…` functions only print them. The Test Phase's E2E tests seed a scratch corpus through `diver_core` (`Store::open_at` + `save` + `save_assertions`) and run the real binary with `DIVER_DB` set via `Command::env`; INT-0019's override is what makes a hermetic CLI test of `dive` possible.

### T-1906: Document concept resolution in the README
- **Intent:** [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
- **Touches:** `README.md`
- **Depends on:** T-1905
- **Acceptance criterion:** INT-0020 Consequences — the recall change, the non-resolution of stoplisted words and the refined `t = 1.0` meaning must be visible to users, not only recorded in the chapter.
- **Success criterion (EARS):**
  - **WHEN** a reader consults the `diver dive` section, **THEN** it **SHALL** explain: terms resolve to concepts; plural forms are one concept; two-word phrases shared by at least two papers are concepts in their own right; an unresolved term lists concepts containing it; derivational variants (`attention` / `attentional`) are distinct, a deliberate change from substring matching; and a pair linked by a shared phrase is not additionally linked by that phrase's individual words.
  - **WHEN** a reader consults the README's `--temperature` description and its example comments, **THEN** the `1.0` endpoint **SHALL** be stated as every shared concept except words subsumed by a shared phrase — no remaining text **SHALL** say `1.0` admits every shared term.

## Deferrals
- **Renaming `CoAssertion.term` to `concept`** — deferred so this sprint's AC5 proof is the existing tests running unmodified. Filed to the backlog at Loop Phase.
- **arXiv taxonomy vocabulary, three-word phrases, acronym linking, derivational merging, `-s`/`-ses` singulars** — out of scope; recorded in INT-0020's Alternatives and Consequences rather than filed as tasks.

## Loop Phase bookkeeping
- Mark T-1710 (phrase/bigram co-assertion) as realized by INT-0020 in `docs/work/tasks.md`.
- File the `CoAssertion.term` rename as a backlog task against INT-0020.
