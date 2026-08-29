Finalized - DO NOT EDIT

# Sprint 5 Build Plan

## Intents
- [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md) — state: planned; acceptance criteria covered: AC1 (Find rename), AC2 (existing commands work), AC3 (inspect shows taxonomy + versions), AC4 (multi-version storage), AC5 (ArxivId newtype), AC6 (ArxivCategory newtype), AC7 (taxonomy validation), AC8 (tests)

## Schema Tree
- Sprint 5 — Harden the factual substrate
  - Taxonomy snapshot
    - T-501: Create `taxonomy/arxiv_categories.json`
  - Identifier newtypes
    - T-502: Add `src/id.rs` with `ArxivId`, `ArxivVersion`, `ArxivCategory` newtypes
  - Parser — preserve all categories
    - T-503: Update `model.rs` and `parse.rs` to collect all category tags
  - Fact — typed categories
    - T-504: Update `fact.rs` to carry `categories: Vec<ArxivCategory>`
  - Store — immutable paper versions
    - T-505: Rewrite `store.rs` schema to `papers` + `paper_versions`; add `get_versions()`
  - CLI — rename Dive → Find
    - T-506: Rename `Commands::Dive` → `Commands::Find` in `main.rs`
  - Display — taxonomy names + versions
    - T-507: Update `display.rs` `display_fact` to show taxonomy-resolved names, secondary categories, and version list
  - Docs
    - T-508: Update `docs/SUMMARY.md` and `docs/intents/README.md`

## Execution Sequence

### T-501: Create arXiv taxonomy snapshot
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- **Touches:** `taxonomy/arxiv_categories.json` (new file)
- **Depends on:** (none)
- **Acceptance criterion:** AC7 — `ArxivCategory::parse("invalid.XX")` returns `Err`
- **Success criterion (EARS):**
  - **WHEN** `taxonomy/arxiv_categories.json` is present in the repository, **THEN** it **SHALL** contain a JSON object mapping every known arXiv category code (e.g., `"cs.CV"`) to an object with at least `"name"` and `"group"` fields.
  - **WHEN** an unknown category code (e.g., `"invalid.XX"`) is looked up against the taxonomy, **THEN** the lookup **SHALL** return an error.
- **Notes:** Snapshot all groups from https://arxiv.org/category_taxonomy. Include at minimum: cs.*, math.*, stat.*, physics.*, eess.*, econ.*, q-bio.*, q-fin.*. Store as a flat `{ "code": { "name": "...", "group": "..." } }` object.

---

### T-502: Add `src/id.rs` with identifier newtypes
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- **Touches:** `src/id.rs` (new), `src/lib.rs`
- **Depends on:** T-501
- **Acceptance criterion:** AC5, AC6, AC7
- **Success criterion (EARS):**
  - **WHEN** `ArxivId` is defined as a newtype wrapping `String`, **THEN** a function `fn f(id: ArxivId)` **SHALL** not accept a bare `String` argument without explicit construction.
  - **WHEN** `ArxivCategory::parse("cs.CV")` is called, **THEN** it **SHALL** return `Ok(ArxivCategory)` if `"cs.CV"` exists in the taxonomy.
  - **WHEN** `ArxivCategory::parse("not.valid")` is called, **THEN** it **SHALL** return `Err` with a message indicating the code is not in the taxonomy.
  - **WHEN** `ArxivVersion` wraps a `u32`, **THEN** `ArxivVersion(2).to_string()` **SHALL** return `"v2"`.
- **Notes:** Taxonomy JSON is embedded via `include_str!("../taxonomy/arxiv_categories.json")`. Use `serde_json` (already a transitive dep) or a hand-rolled lookup. Expose `ArxivCategory::code()` returning `&str`.

---

### T-503: Update `model.rs` and `parse.rs` to collect all category tags
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- **Touches:** `src/model.rs`, `src/parse.rs`
- **Depends on:** (none) — independent of T-501/T-502
- **Acceptance criterion:** AC3 (secondary categories visible in inspect), AC8
- **Success criterion (EARS):**
  - **WHEN** an arXiv Atom feed entry contains a `primary_category` element and two additional `category` elements, **THEN** `parse_feed` **SHALL** return a `Paper` whose `categories` field contains all three distinct codes.
  - **WHEN** `primary_category` appears in `categories`, **THEN** it **SHALL** appear exactly once (no duplication).
  - **WHEN** the feed entry has only a `primary_category` and no extra `category` elements, **THEN** `paper.categories` **SHALL** have length 1.
- **Notes:** Remove the `&& primary_category.is_empty()` guard on line 72 of `parse.rs`. Collect into a `Vec<String>` on `Paper`; dedup by code.

---

### T-504: Update `fact.rs` to carry typed categories
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- **Touches:** `src/fact.rs`
- **Depends on:** T-502, T-503
- **Acceptance criterion:** AC6, AC8
- **Success criterion (EARS):**
  - **WHEN** `SourceFact::from_paper` is called with a `Paper` that has three category codes, **THEN** `fact.categories` **SHALL** be a `Vec<ArxivCategory>` of length 3.
  - **WHEN** a category code from the paper is not in the taxonomy, **THEN** `SourceFact::from_paper` **SHALL** log a warning and skip that code rather than panicking.
- **Notes:** `primary_category` in `SourceFact` becomes `ArxivCategory`. `categories` is `Vec<ArxivCategory>`. Serialise to store as JSON string (codes only) for SQLite storage.

---

### T-505: Rewrite `store.rs` schema to `papers` + `paper_versions`
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- **Touches:** `src/store.rs`
- **Depends on:** T-504
- **Acceptance criterion:** AC4, AC8
- **Success criterion (EARS):**
  - **WHEN** `store.save(&fact_v1)` is called for `2301.00001 v1` followed by `store.save(&fact_v2)` for `2301.00001 v2`, **THEN** `store.get_versions("2301.00001")` **SHALL** return both `["v1", "v2"]`.
  - **WHEN** `store.save(&fact_v1)` is called twice with identical data, **THEN** the second save **SHALL** succeed and `store.get_versions("2301.00001")` **SHALL** still return `["v1"]` (idempotent).
  - **WHEN** `store.get("2301.00001")` is called, **THEN** it **SHALL** return the most recently ingested version's metadata.
  - **WHEN** `store.search("attention", 10)` is called, **THEN** it **SHALL** return ranked results from the FTS index covering all paper versions.
- **Notes:** Schema: `papers (id INTEGER PK, arxiv_id TEXT UNIQUE)` and `paper_versions (id INTEGER PK, paper_id INTEGER FK, version TEXT, title TEXT, authors TEXT, summary TEXT, primary_category TEXT, categories TEXT, published TEXT, updated TEXT, pdf_url TEXT, source_url TEXT, ingested_at TEXT, UNIQUE(paper_id, version))`. FTS indexes on latest version of each paper. Local `diver.db` from pre-Sprint-5 is incompatible; document this in README.

---

### T-506: Rename `Commands::Dive` to `Commands::Find` in `main.rs`
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- **Touches:** `src/main.rs`
- **Depends on:** (none)
- **Acceptance criterion:** AC1, AC2
- **Success criterion (EARS):**
  - **WHEN** the user runs `diver find "attention"`, **THEN** the CLI **SHALL** perform local FTS search and display ranked results.
  - **WHEN** the user runs `diver dive`, **THEN** the CLI **SHALL** respond with an unknown-subcommand error (the subcommand is not registered).
  - **WHEN** `diver search`, `diver ingest`, `diver collect`, `diver inspect`, and `diver list` are invoked with valid arguments, **THEN** each **SHALL** behave identically to their pre-Sprint-5 behaviour.
- **Notes:** Simple rename; update doc string to: "Search your local corpus".

---

### T-507: Update `display.rs` to show taxonomy names, secondary categories, and versions
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- **Touches:** `src/display.rs`
- **Depends on:** T-502, T-505
- **Acceptance criterion:** AC3
- **Success criterion (EARS):**
  - **WHEN** `display_fact(&fact, &versions)` is called with a fact whose `primary_category` is `"cs.CV"`, **THEN** the output **SHALL** include `"Computer Vision and Pattern Recognition"` (the taxonomy name for `cs.CV`).
  - **WHEN** `display_fact(&fact, &versions)` is called with a fact that has two secondary categories, **THEN** the output **SHALL** list them under a "Secondary:" heading.
  - **WHEN** `display_fact(&fact, &versions)` is called and versions are `["v1", "v2", "v3"]` with current `"v3"`, **THEN** the output **SHALL** list all three versions, marking the current with a `←` indicator.
- **Notes:** `display_fact` signature gains a `versions: &[String]` parameter.

---

### T-508: Update docs
- **Intent:** [INT-0005](../../../intents/INT-0005-harden-factual-substrate.md)
- **Touches:** `docs/SUMMARY.md`, `docs/intents/README.md`, `README.md`
- **Depends on:** (none)
- **Acceptance criterion:** (documentation only)
- **Success criterion (EARS):**
  - **WHEN** `docs/SUMMARY.md` is read, **THEN** it **SHALL** contain a link to `INT-0005-harden-factual-substrate.md`.
  - **WHEN** `README.md` is read, **THEN** it **SHALL** contain a note that pre-Sprint-5 `diver.db` databases must be deleted.
- **Notes:** Also update `docs/intents/README.md` to list INT-0005.
