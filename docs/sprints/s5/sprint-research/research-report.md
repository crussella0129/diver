# Sprint 5 Research Report

## Intents Reviewed
- [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) — created; relevance: this is the primary deliverable for Sprint 5; current state: planned

## 1. Sprint Goal

Sprint 5 is the first of a five-sprint arc that will eventually deliver `diver dive` as real knowledge-graph traversal. Before any semantic layer can be built, Diver's factual substrate must be rigorous: identifiers should be typed so the compiler rejects category/ID confusion, all arXiv categories should be preserved (not just the first one), the taxonomy should be explicit and versioned, and paper versions should be stored as immutable records so that evidence extracted from `v2` remains reproducible after `v3` is ingested. The sprint also corrects a product-vocabulary mistake: the current `Dive` command is FTS search and should be called `Find`.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| `src/main.rs` | high | Defines CLI commands; `Dive` must be renamed to `Find`; orchestrates `store.search` |
| `src/fact.rs` | high | `SourceFact` uses `String` for every identifier; gains `categories: Vec<ArxivCategory>` and typed `arxiv_id`/`arxiv_version` fields |
| `src/model.rs` | high | `Paper` uses `String` for `primary_category`; must add `categories: Vec<String>` to pass all tags through from parser |
| `src/parse.rs` | high | Parser stops at first `primary_category`/`category` tag (line 72: `&& primary_category.is_empty()`); must collect all category tags |
| `src/store.rs` | high | `source_facts` table uses `arxiv_id TEXT PRIMARY KEY` with `INSERT OR REPLACE`; schema must split into `papers` + `paper_versions` with `UNIQUE(arxiv_id, version)` |
| `src/display.rs` | medium | `display_fact` must show primary category name (from taxonomy) + all secondary categories + versions list |
| `src/query.rs` | low | URL construction not changed this sprint; flagged for future structured-query refactor |
| `src/client.rs` | low | Network layer unchanged this sprint |
| `tests/` | medium | All existing tests must pass; new tests needed for multi-version storage, taxonomy validation, category collection |

## 3. External Sources

- [arXiv Category Taxonomy](https://arxiv.org/category_taxonomy) — official list of all arXiv category codes, names, and parent groups; will be snapshot as `taxonomy/arxiv_categories.json`

## 4. Risks, Unknowns, Dependencies

- **Risk:** The `paper_versions` schema split is a breaking change to any existing local `diver.db`. We must either provide a migration path or document that pre-Sprint-5 databases are incompatible and must be deleted. Given this is pre-1.0 local tooling, documenting the incompatibility is acceptable.
- **Risk:** Embedding the taxonomy JSON in the binary via `include_str!` works cleanly but ties the taxonomy version to the binary version. Acceptable for now; a runtime-upgradeable taxonomy path can be introduced later.
- **Unknown:** Exact format of the arXiv taxonomy page — needs to be scraped or transcribed into JSON format once, then embedded. A curated representative subset is sufficient for Sprint 5.
- **Dependency:** `ArxivCategory::parse` needs the taxonomy file present in the repository at `taxonomy/arxiv_categories.json` before compile time (if embedded via `include_str!`).

## 5. Recommended Approach

**Primary:** Incremental changes in the order that minimizes test breakage:

1. Add `taxonomy/arxiv_categories.json` with a representative snapshot of arXiv categories.
2. Add `ArxivId`, `ArxivVersion`, `ArxivCategory` newtypes in a new `src/id.rs` module; implement `ArxivCategory::parse()` with taxonomy validation.
3. Update `model.rs` (`Paper`) to add `categories: Vec<String>`.
4. Update `parse.rs` to collect all category tags.
5. Update `fact.rs` (`SourceFact`) to use `ArxivCategory` for categories; keep `arxiv_id` and `arxiv_version` as `String` for now (full newtype propagation through store/display is Sprint 5 scope, but the newtype for compilation checks lives in `id.rs`).
6. Update `store.rs`: create `papers` + `paper_versions` schema, update `save`/`get`/`list`/`search`/`exists`; add `get_versions()` method.
7. Rename `Commands::Dive` → `Commands::Find` in `main.rs`.
8. Update `display.rs` to show taxonomy-resolved category names, secondary categories, and versions in `display_fact`.
9. Update `SUMMARY.md` and `docs/intents/README.md` to reference INT-0005.

**Alternative considered:** Full newtype propagation through store and display in one pass. Rejected as too noisy — it changes every test fixture and all display code simultaneously, making review harder.

**Rationale:** The taxonomy and multi-category changes are logically independent of the schema split, but the schema split is the highest-risk item and should be done with a clean transaction boundary and migration note.

## Artifacts

_(none — no scratch files needed; all changes are to existing source files and one new taxonomy file)_
