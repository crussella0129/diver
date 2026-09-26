# Sprint 19 Unit Tests

- **Tested head:** `d08eaa61cbccb32a468e72b06b3588874a638637`
- **Runner:** `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --check`
- **Result:** `diver_core` lib **146 passed; 0 failed** (121 pre-existing + 8 Sprint 18 + 17
  new this sprint, one pre-existing test replaced); `diver-cli` bin 1. Clippy 0; fmt clean.
- Written in the Test Phase, after the proving-ground drive, and consolidated: each test
  covers several EARS clauses (see the test plan's traceability table).

## `diver-core/src/concept.rs` — [INT-0020](../../../intents/INT-0020-first-class-concepts.md)
| Test | Clauses | Result |
|------|---------|--------|
| `test_significant_terms`, `test_significant_terms_stoplist` | T-1901/1 — moved verbatim from `graph.rs` | pass |
| `test_stoplist_categories` | T-1901/2–3; drive repair 4 (adverbs filler; `anomaly`, `family`, `assembly` content) | pass |
| `test_fold` | T-1902/1–2 incl. precedence and `semantics`/`metrics` unchanged | pass |
| `test_folded_classification_consistent` | T-1902/3 | pass |
| `test_phrase_rule` | T-1902/5–6 | pass |
| `test_concept_formation` | T-1902/4, 7, 9, 10, 11 | pass |
| `test_formation_invariants` | T-1902/8, 12, 13 | pass |
| `test_query_key` | T-1902/14 | pass |
| `test_digest_words` | T-1902/15 (incl. comment lines ignored) | pass |

## `diver-core/src/store.rs`
| Test | Clauses | Result |
|------|---------|--------|
| `test_papers_asserting_matches`, `test_papers_asserting_empty` | T-1903/4 — **unmodified** | pass |
| `test_papers_asserting_wildcards_inert` | T-1903/6 — replaces `test_papers_asserting_escapes_like_wildcards` (T-1903 commit) | pass |
| `test_concept_resolution` | T-1903/1, 2, 3, 5 | pass |
| `test_concepts_freshness` | T-1903/7, 8 (forged version and forged digest; and no rebuild when the fingerprint matches) | pass |
| `test_concept_navigation` | T-1903/9, 10 (caps; paper-count order, with a fixture where count order and id order disagree), 11; drive repairs 2 and 3 | pass |

## `diver-core/src/graph.rs`
| Test | Clauses | Result |
|------|---------|--------|
| all pre-existing co-assertion tests | T-1904/1 — assertions unmodified; two fixtures gained comma separators (T-1904 commit) | pass |
| `test_coassertion_concepts` | T-1904/1 (label), 2, 3, 4 (incl. gate on the control pair) | pass |
| `test_coassertion_monotonic_with_phrases` | T-1904/5 (incl. `hidden state`/`hidden states`) | pass |
| `test_touching_variants_match_full` | drive repair 5 — scoped == full ∩ seeds, co-assertion and structural (fixture includes a non-seed pair) | pass |
| `test_coassertion_matches_naive_reference` | drive repair 5 — the gate pre-filter changes no output: equal to an unoptimized reference as ordered `Vec`s at five temperatures | pass |
| `test_group_related_ranking` | drive repair 1 | pass |

## `diver-core/src/display.rs`
| Test | Clauses | Result |
|------|---------|--------|
| `test_format_dive_concept` | T-1905/2, 3, 4 (formatting); drive repairs 1 (incl. top-5 cap line), 3, 6 (incl. echo truncation) | pass |

## Mutation check
Each mutation was applied, its named test run, and the file restored. All caught.
| Mutation | Test | Outcome |
|----------|------|---------|
| subsumption disabled (`\|\| true`) | `test_coassertion_concepts` | caught |
| freshness ignores the fingerprint | `test_concepts_freshness` | caught |
| seed scope `\|\|` → `&&` (co-assertion) | `test_touching_variants_match_full` | caught |
| structural scoping removed | `test_touching_variants_match_full` | caught |
| phrases gated on two *claims* instead of two papers | `test_concept_formation` | caught |
| gate pre-filter `>=` → `>` | `test_coassertion_matches_naive_reference` | caught |
| always rebuild (both fingerprint checks disabled) | `test_concepts_freshness` | caught |
| list cap removed | `test_concept_navigation` | caught |
| paper-count key dropped from list ordering | `test_concept_navigation` | caught |

A first "always rebuild" mutation disabled only the outer fingerprint check and was *not*
caught — because the check is repeated inside the rebuild transaction, so that mutation
never forced a rebuild. Disabling both checks is the real mutation, and it is caught.

## Deviations from locked EARS clauses (proving-ground repairs)
Recorded here, and in INT-0020 Consequences, so the pass marks above are not read as
proving the locked wording:
- **T-1903/10** — the any-token fallback was locked as "ordered by paper count
  descending then id". Repair 2 (`f1f3341`) ranks by query words covered first, then
  paper count, then id. The all-token path and the cap are unchanged and tested as locked.
- **T-1902/3** — the category is no longer *only* "the category of the folded key against
  the folded stoplists": repair 4 (`8cd1cc1`) also makes a folded key of 6+ characters
  ending in `ly` filler (except a short noun allowlist). Consistency across inflections —
  the property the clause exists for — still holds, because the rule runs on the folded
  key; `test_stoplist_categories` asserts the adverb rule.
- **T-1905/3** — for a query with *no searchable words* (only common words, numbers,
  short tokens), the output says so instead of "is not a concept" (repair 6). Terms that
  are searchable but unresolved still get the locked "is not a concept" message.

## One-off check (T-1901/2)
`union(stopwords_common, stopwords_filler) == set(git show HEAD:diver-core/src/stopwords.txt)`
at the T-1901 boundary: 525 = 525, disjoint. (Both lists have since grown by design.)
