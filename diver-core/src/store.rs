use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use crate::assertion::{Assertion, Supported};
use crate::concept::{
    CONCEPT_FORMATION_VERSION, ConceptKind, form_concepts, query_key, rules_digest,
};
use crate::fact::SourceFact;

/// `meta` key holding the fingerprint persisted concepts were built from.
const CONCEPT_FINGERPRINT_KEY: &str = "concept_fingerprint";

/// Most concepts any related/suggestion list returns.
pub const CONCEPT_LIST_CAP: usize = 10;

/// A persisted concept, resolved from a query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptInfo {
    pub id: String,
    pub label: String,
    pub kind: ConceptKind,
    pub paper_count: usize,
    /// Observed surface forms with occurrence counts, most frequent first.
    pub forms: Vec<(String, usize)>,
}

/// A concept's identity and reach, for related-concept and suggestion lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptSummary {
    pub id: String,
    pub label: String,
    pub kind: ConceptKind,
    pub paper_count: usize,
}

/// Order by paper count descending, then id, and keep at most [`CONCEPT_LIST_CAP`].
fn rank_and_cap(list: &mut Vec<ConceptSummary>) {
    list.sort_by(|a, b| b.paper_count.cmp(&a.paper_count).then(a.id.cmp(&b.id)));
    list.truncate(CONCEPT_LIST_CAP);
}

/// Levenshtein distance between two strings, by characters.
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let substitute = prev[j] + usize::from(ca != *cb);
            cur[j + 1] = substitute.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

fn parse_kind(kind: &str) -> Result<ConceptKind> {
    ConceptKind::parse(kind).ok_or_else(|| anyhow!("unknown concept kind {kind:?} in database"))
}

/// Environment variable that overrides the corpus location.
pub const DB_PATH_ENV: &str = "DIVER_DB";

/// Choose the corpus path from an override value and the platform data directory.
///
/// `override_value` is the raw `DIVER_DB` value (or `None` when unset), and
/// `data_dir` is the platform data directory (or `None` when unavailable). A
/// non-empty override wins outright; otherwise the default is
/// `<data_dir>/diver/diver.db`, falling back to `.diver/diver.db`.
///
/// A **set-but-empty** override is treated as unset: `std::env::var_os` yields
/// `Some("")` for `DIVER_DB=`, and SQLite reads an empty filename as a private
/// temporary database that is discarded on close — so honoring it literally would
/// silently hand every command a throwaway corpus.
///
/// This function is pure. It performs no I/O and reads no environment, which is
/// what makes every branch — including the no-data-directory fallback — testable
/// without mutating process environment. Directory creation belongs to
/// [`Store::open_at`], so resolving a path never touches the platform data
/// directory that an override was meant to bypass.
pub fn resolve_db_path(override_value: Option<OsString>, data_dir: Option<PathBuf>) -> PathBuf {
    match override_value.filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => data_dir
            .map(|dir| dir.join("diver"))
            .unwrap_or_else(|| PathBuf::from(".diver"))
            .join("diver.db"),
    }
}

/// The corpus path for an explicit override value, against the platform data
/// directory.
///
/// This carries the composition — the arguments actually handed to
/// [`resolve_db_path`] — as a named function rather than an inline expression, so
/// a test can pin it. Testing the helper alone would miss the one silent refactor
/// error available here: folding the `join("diver")` into this call site as well
/// as into the helper yields `<data_dir>/diver/diver/diver.db`, relocating every
/// existing corpus while a helper-only test still passes.
///
/// The environment read is deliberately *not* part of this function. Keeping it
/// out means the default-path test needs no `DIVER_DB`-is-unset guard, and so
/// cannot silently degrade into a no-op in the very environment this feature
/// teaches people to create. The `var_os` read is covered instead by the CLI
/// tests in `diver-cli/tests/db_override.rs`.
pub fn current_db_path_for(override_value: Option<OsString>) -> PathBuf {
    resolve_db_path(override_value, dirs::data_dir())
}

/// The corpus path [`Store::open`] will use, given the current environment.
pub fn current_db_path() -> PathBuf {
    current_db_path_for(std::env::var_os(DB_PATH_ENV))
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub primary_category: String,
    pub rank: f64,
}

/// A previously-validated assertion loaded from storage for display. Its
/// existence in the store means it was persisted as an `Assertion<Supported>`.
#[derive(Debug, Clone)]
pub struct StoredAssertion {
    pub claim: String,
    pub version: String,
    pub support: Vec<String>,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open the corpus at the path [`current_db_path`] selects: `DIVER_DB` when
    /// that variable holds a non-empty value, otherwise the platform default.
    pub fn open() -> Result<Self> {
        Self::open_at(current_db_path())
    }

    /// Open (creating if absent) the corpus stored at `path`, creating any missing
    /// parent directories and initializing the schema. This carries the real work
    /// behind [`Store::open`]; tests and corpus tooling call it directly rather
    /// than mutating process environment.
    pub fn open_at(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create data directory: {}", parent.display())
            })?;
        }

        let conn = Connection::open(path)
            .with_context(|| format!("failed to open database: {}", path.display()))?;

        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("failed to open in-memory database")?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "PRAGMA journal_mode=WAL;
                PRAGMA foreign_keys=ON;

                CREATE TABLE IF NOT EXISTS papers (
                    id       INTEGER PRIMARY KEY AUTOINCREMENT,
                    arxiv_id TEXT NOT NULL UNIQUE
                );

                CREATE TABLE IF NOT EXISTS paper_versions (
                    id               INTEGER PRIMARY KEY AUTOINCREMENT,
                    paper_id         INTEGER NOT NULL REFERENCES papers(id),
                    version          TEXT NOT NULL,
                    title            TEXT NOT NULL,
                    authors          TEXT NOT NULL,
                    summary          TEXT NOT NULL,
                    primary_category TEXT NOT NULL,
                    categories       TEXT NOT NULL,
                    published        TEXT NOT NULL,
                    updated          TEXT NOT NULL,
                    pdf_url          TEXT NOT NULL,
                    source_url       TEXT NOT NULL,
                    ingested_at      TEXT NOT NULL,
                    UNIQUE(paper_id, version)
                );

                CREATE VIRTUAL TABLE IF NOT EXISTS paper_versions_fts
                USING fts5(arxiv_id, title, authors, summary, primary_category);

                CREATE TABLE IF NOT EXISTS assertions (
                    id         INTEGER PRIMARY KEY AUTOINCREMENT,
                    paper_id   INTEGER NOT NULL REFERENCES papers(id),
                    version    TEXT NOT NULL,
                    claim      TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS assertion_support (
                    id           INTEGER PRIMARY KEY AUTOINCREMENT,
                    assertion_id INTEGER NOT NULL REFERENCES assertions(id) ON DELETE CASCADE,
                    quote        TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS concepts (
                    id          TEXT PRIMARY KEY,
                    label       TEXT NOT NULL,
                    kind        TEXT NOT NULL,
                    paper_count INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS concept_forms (
                    concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
                    form       TEXT NOT NULL,
                    count      INTEGER NOT NULL,
                    PRIMARY KEY (concept_id, form)
                );

                CREATE TABLE IF NOT EXISTS assertion_concepts (
                    assertion_id INTEGER NOT NULL REFERENCES assertions(id) ON DELETE CASCADE,
                    concept_id   TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
                    PRIMARY KEY (assertion_id, concept_id)
                );

                CREATE INDEX IF NOT EXISTS idx_assertion_concepts_concept
                    ON assertion_concepts(concept_id);

                CREATE TABLE IF NOT EXISTS meta (
                    key   TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );",
            )
            .context("failed to initialize database schema")?;

        self.backfill_fts()?;
        Ok(())
    }

    fn backfill_fts(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM paper_versions_fts", [], |row| {
                row.get(0)
            })
            .context("failed to count FTS rows")?;

        if count > 0 {
            return Ok(());
        }

        let paper_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM papers", [], |row| row.get(0))
            .context("failed to count papers rows")?;

        if paper_count == 0 {
            return Ok(());
        }

        // Backfill FTS from the latest version of each paper.
        self.conn
            .execute_batch(
                "INSERT INTO paper_versions_fts (arxiv_id, title, authors, summary, primary_category)
                 SELECT p.arxiv_id, pv.title, pv.authors, pv.summary, pv.primary_category
                 FROM papers p
                 JOIN paper_versions pv ON pv.paper_id = p.id
                 WHERE pv.ingested_at = (
                     SELECT MAX(pv2.ingested_at)
                     FROM paper_versions pv2
                     WHERE pv2.paper_id = p.id
                 );",
            )
            .context("failed to backfill FTS index")?;

        Ok(())
    }

    pub fn save(&self, fact: &SourceFact) -> Result<()> {
        let authors_json =
            serde_json::to_string(&fact.authors).context("failed to serialize authors")?;
        let categories_json = fact.categories_json();

        self.conn
            .execute_batch("BEGIN;")
            .context("failed to begin transaction")?;

        let result = (|| -> Result<()> {
            // Upsert into papers — get or create the paper row.
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO papers (arxiv_id) VALUES (?1)",
                    rusqlite::params![fact.arxiv_id],
                )
                .context("failed to upsert papers row")?;

            let paper_id: i64 = self
                .conn
                .query_row(
                    "SELECT id FROM papers WHERE arxiv_id = ?1",
                    rusqlite::params![fact.arxiv_id],
                    |row| row.get(0),
                )
                .context("failed to retrieve paper_id")?;

            self.conn
                .execute(
                    "INSERT INTO paper_versions
                     (paper_id, version, title, authors, summary, primary_category,
                      categories, published, updated, pdf_url, source_url, ingested_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                     ON CONFLICT(paper_id, version) DO UPDATE SET
                       title = excluded.title,
                       authors = excluded.authors,
                       summary = excluded.summary,
                       primary_category = excluded.primary_category,
                       categories = excluded.categories,
                       published = excluded.published,
                       updated = excluded.updated,
                       pdf_url = excluded.pdf_url,
                       source_url = excluded.source_url,
                       ingested_at = excluded.ingested_at",
                    rusqlite::params![
                        paper_id,
                        fact.arxiv_version,
                        fact.title,
                        authors_json,
                        fact.summary,
                        fact.primary_category.code(),
                        categories_json,
                        fact.published,
                        fact.updated,
                        fact.pdf_url,
                        fact.source_url,
                        fact.ingested_at,
                    ],
                )
                .context("failed to upsert paper_version")?;

            // Refresh FTS using the latest version's data (not the incoming fact,
            // which may be an older version being re-ingested).
            let (fts_title, fts_authors_json, fts_summary, fts_category): (
                String,
                String,
                String,
                String,
            ) = self
                .conn
                .query_row(
                    "SELECT pv.title, pv.authors, pv.summary, pv.primary_category
                     FROM paper_versions pv
                     WHERE pv.paper_id = ?1
                     ORDER BY pv.ingested_at DESC
                     LIMIT 1",
                    rusqlite::params![paper_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .context("failed to query latest version for FTS")?;

            let fts_authors: Vec<String> =
                serde_json::from_str(&fts_authors_json).unwrap_or_default();
            let fts_authors_text = fts_authors.join(", ");

            self.conn
                .execute(
                    "DELETE FROM paper_versions_fts WHERE arxiv_id = ?1",
                    rusqlite::params![fact.arxiv_id],
                )
                .context("failed to delete old FTS entry")?;

            self.conn
                .execute(
                    "INSERT INTO paper_versions_fts (arxiv_id, title, authors, summary, primary_category)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        fact.arxiv_id,
                        fts_title,
                        fts_authors_text,
                        fts_summary,
                        fts_category,
                    ],
                )
                .context("failed to insert FTS entry")?;

            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn
                    .execute_batch("COMMIT;")
                    .context("failed to commit transaction")?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
    }

    /// Returns the most recently ingested version's metadata for a given arxiv_id.
    pub fn get(&self, arxiv_id: &str) -> Result<Option<SourceFact>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT p.arxiv_id, pv.version, pv.title, pv.authors, pv.summary,
                        pv.primary_category, pv.categories, pv.published, pv.updated,
                        pv.pdf_url, pv.source_url, pv.ingested_at
                 FROM papers p
                 JOIN paper_versions pv ON pv.paper_id = p.id
                 WHERE p.arxiv_id = ?1
                 ORDER BY pv.ingested_at DESC
                 LIMIT 1",
            )
            .context("failed to prepare get query")?;

        let mut rows = stmt
            .query(rusqlite::params![arxiv_id])
            .context("failed to execute get query")?;

        match rows.next().context("failed to read row")? {
            Some(row) => Ok(Some(row_to_fact(row)?)),
            None => Ok(None),
        }
    }

    /// Returns all stored versions for a given arxiv_id, ordered by ingestion time.
    pub fn get_versions(&self, arxiv_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT pv.version
                 FROM papers p
                 JOIN paper_versions pv ON pv.paper_id = p.id
                 WHERE p.arxiv_id = ?1
                 ORDER BY pv.ingested_at ASC",
            )
            .context("failed to prepare get_versions query")?;

        let versions = stmt
            .query_map(rusqlite::params![arxiv_id], |row| row.get(0))
            .context("failed to execute get_versions query")?
            .collect::<std::result::Result<Vec<String>, _>>()
            .context("failed to collect versions")?;

        Ok(versions)
    }

    pub fn list(&self) -> Result<Vec<SourceFact>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT p.arxiv_id, pv.version, pv.title, pv.authors, pv.summary,
                        pv.primary_category, pv.categories, pv.published, pv.updated,
                        pv.pdf_url, pv.source_url, pv.ingested_at
                 FROM papers p
                 JOIN paper_versions pv ON pv.paper_id = p.id
                 WHERE pv.ingested_at = (
                     SELECT MAX(pv2.ingested_at)
                     FROM paper_versions pv2
                     WHERE pv2.paper_id = p.id
                 )
                 ORDER BY pv.ingested_at DESC",
            )
            .context("failed to prepare list query")?;

        let facts = stmt
            .query_map([], row_to_fact)
            .context("failed to execute list query")?;

        let mut result = Vec::new();
        for fact_result in facts {
            result.push(fact_result.context("failed to read paper row")?);
        }
        Ok(result)
    }

    pub fn exists(&self, arxiv_id: &str) -> Result<bool> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM papers WHERE arxiv_id = ?1",
                rusqlite::params![arxiv_id],
                |row| row.get(0),
            )
            .context("failed to check existence")?;
        Ok(count > 0)
    }

    pub fn search(&self, query: &str, max_results: u32) -> Result<Vec<SearchResult>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT p.arxiv_id, sf.title, sf.authors, sf.summary, sf.primary_category, f.rank
                 FROM paper_versions_fts f
                 JOIN papers p ON p.arxiv_id = f.arxiv_id
                 JOIN paper_versions sf ON sf.paper_id = p.id
                 WHERE paper_versions_fts MATCH ?1
                 AND sf.ingested_at = (
                     SELECT MAX(pv2.ingested_at) FROM paper_versions pv2
                     WHERE pv2.paper_id = p.id
                 )
                 ORDER BY f.rank
                 LIMIT ?2",
            )
            .context("failed to prepare search query")?;

        let rows = stmt
            .query_map(rusqlite::params![query, max_results], |row| {
                let authors_json: String = row.get(2)?;
                let authors: Vec<String> = serde_json::from_str(&authors_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                Ok(SearchResult {
                    arxiv_id: row.get(0)?,
                    title: row.get(1)?,
                    authors,
                    summary: row.get(3)?,
                    primary_category: row.get(4)?,
                    rank: row.get(5)?,
                })
            })
            .context("failed to execute search query")?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.context("failed to read search result")?);
        }
        Ok(results)
    }

    /// Persist the supported assertions extracted for a paper version. The
    /// `&[Assertion<Supported>]` parameter is the storage gate — only validated
    /// assertions can be stored. Idempotent per `(paper, version)`: a prior set
    /// for the same paper+version is replaced (support cascades on delete).
    pub fn save_assertions(
        &self,
        arxiv_id: &str,
        version: &str,
        assertions: &[Assertion<Supported>],
    ) -> Result<()> {
        self.conn
            .execute_batch("BEGIN;")
            .context("failed to begin transaction")?;

        let result = (|| -> Result<()> {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO papers (arxiv_id) VALUES (?1)",
                    rusqlite::params![arxiv_id],
                )
                .context("failed to upsert papers row")?;

            let paper_id: i64 = self
                .conn
                .query_row(
                    "SELECT id FROM papers WHERE arxiv_id = ?1",
                    rusqlite::params![arxiv_id],
                    |row| row.get(0),
                )
                .context("failed to retrieve paper_id")?;

            // Idempotent replace: drop the prior set for this paper+version.
            // assertion_support rows cascade via ON DELETE CASCADE.
            self.conn
                .execute(
                    "DELETE FROM assertions WHERE paper_id = ?1 AND version = ?2",
                    rusqlite::params![paper_id, version],
                )
                .context("failed to clear prior assertions")?;

            let created_at = Utc::now().to_rfc3339();
            for assertion in assertions {
                self.conn
                    .execute(
                        "INSERT INTO assertions (paper_id, version, claim, created_at)
                         VALUES (?1, ?2, ?3, ?4)",
                        rusqlite::params![paper_id, version, assertion.claim(), created_at],
                    )
                    .context("failed to insert assertion")?;
                let assertion_id = self.conn.last_insert_rowid();

                for obs in assertion.support() {
                    self.conn
                        .execute(
                            "INSERT INTO assertion_support (assertion_id, quote) VALUES (?1, ?2)",
                            rusqlite::params![assertion_id, obs.text()],
                        )
                        .context("failed to insert assertion support")?;
                }
            }
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn
                    .execute_batch("COMMIT;")
                    .context("failed to commit transaction")?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
    }

    /// Load the stored assertions for a paper (any version), newest first.
    /// Returns an empty vec if the paper is unknown or has no stored assertions.
    pub fn get_assertions(&self, arxiv_id: &str) -> Result<Vec<StoredAssertion>> {
        // Collect the assertion heads first so the statement's borrow is released
        // before we fetch each one's support quotes.
        let heads: Vec<(i64, String, String)> = {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT a.id, a.version, a.claim
                     FROM assertions a
                     JOIN papers p ON p.id = a.paper_id
                     WHERE p.arxiv_id = ?1
                     ORDER BY a.created_at DESC, a.id DESC",
                )
                .context("failed to prepare get_assertions query")?;
            let rows = stmt
                .query_map(rusqlite::params![arxiv_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .context("failed to execute get_assertions query")?;
            rows.collect::<std::result::Result<Vec<_>, _>>()
                .context("failed to read assertion rows")?
        };

        let mut assertions = Vec::with_capacity(heads.len());
        for (id, version, claim) in heads {
            let support = self.support_quotes(id)?;
            assertions.push(StoredAssertion {
                claim,
                version,
                support,
            });
        }
        Ok(assertions)
    }

    fn support_quotes(&self, assertion_id: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT quote FROM assertion_support WHERE assertion_id = ?1 ORDER BY id")
            .context("failed to prepare support query")?;
        let quotes = stmt
            .query_map(rusqlite::params![assertion_id], |row| row.get(0))
            .context("failed to execute support query")?
            .collect::<std::result::Result<Vec<String>, _>>()
            .context("failed to collect support quotes")?;
        Ok(quotes)
    }

    /// Claims asserting about `concept`: the query is resolved to a concept
    /// ([`Store::resolve_concept`]) and that concept's linked claims are returned as
    /// `(arxiv_id, claim)`, ordered by paper then insertion. Empty when the query
    /// resolves to no concept. Seeds `diver dive`. There is no substring matching:
    /// `gan` does not reach a claim that says `organized`.
    pub fn papers_asserting(&self, concept: &str) -> Result<Vec<(String, String)>> {
        match self.resolve_concept(concept)? {
            Some(info) => self.claims_for_concept(&info.id),
            None => Ok(Vec::new()),
        }
    }

    /// Resolve a user query to a persisted concept: the query is normalized with
    /// [`query_key`] (tokenize, drop common words, fold plurals) and looked up by id.
    /// `None` when nothing matches.
    pub fn resolve_concept(&self, query: &str) -> Result<Option<ConceptInfo>> {
        self.ensure_concepts_fresh()?;
        match query_key(query) {
            Some(key) => self.concept_info(&key),
            None => Ok(None),
        }
    }

    /// The claims linked to concept `id`, as `(arxiv_id, claim)`, ordered by paper
    /// then assertion id.
    pub fn claims_for_concept(&self, id: &str) -> Result<Vec<(String, String)>> {
        self.ensure_concepts_fresh()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT p.arxiv_id, a.claim
                 FROM assertion_concepts ac
                 JOIN assertions a ON a.id = ac.assertion_id
                 JOIN papers p ON p.id = a.paper_id
                 WHERE ac.concept_id = ?1
                 ORDER BY p.arxiv_id, a.id",
            )
            .context("failed to prepare claims_for_concept query")?;
        let rows = stmt
            .query_map(rusqlite::params![id], |row| Ok((row.get(0)?, row.get(1)?)))
            .context("failed to execute claims_for_concept query")?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to read concept claim rows")
    }

    /// Concepts linked to concept `id` without being merged into it: for a term, the
    /// phrases containing it (narrower); for a phrase, the terms of its words
    /// (broader). Ordered by paper count descending then id, at most
    /// [`CONCEPT_LIST_CAP`].
    pub fn concepts_related_to(&self, id: &str) -> Result<Vec<ConceptSummary>> {
        self.ensure_concepts_fresh()?;
        let words: Vec<&str> = id.split(' ').collect();
        let mut related: Vec<ConceptSummary> = self
            .concept_summaries()?
            .into_iter()
            .filter(|c| {
                if words.len() == 1 {
                    c.kind == ConceptKind::Phrase && c.id.split(' ').any(|w| w == id)
                } else {
                    c.kind == ConceptKind::Term && words.contains(&c.id.as_str())
                }
            })
            .collect();
        rank_and_cap(&mut related);
        Ok(related)
    }

    /// Concepts to offer when `query` does not resolve: those whose id contains every
    /// folded query token as a whole word, ordered by paper count descending then id;
    /// or, if there are none, those containing any of them, ordered first by how many
    /// query tokens they contain — so `quantum error correction` offers the phrases
    /// `error correction` and `quantum error` before the bare word `error`. At most
    /// [`CONCEPT_LIST_CAP`].
    pub fn concept_suggestions(&self, query: &str) -> Result<Vec<ConceptSummary>> {
        self.ensure_concepts_fresh()?;
        let Some(key) = query_key(query) else {
            return Ok(Vec::new());
        };
        let tokens: Vec<&str> = key.split(' ').collect();
        let all = self.concept_summaries()?;
        let matched = |c: &ConceptSummary| {
            tokens
                .iter()
                .filter(|t| c.id.split(' ').any(|w| w == **t))
                .count()
        };
        let mut out: Vec<ConceptSummary> = all
            .iter()
            .filter(|c| c.id != key && matched(c) == tokens.len())
            .cloned()
            .collect();
        if !out.is_empty() {
            rank_and_cap(&mut out);
            return Ok(out);
        }
        let mut scored: Vec<(usize, ConceptSummary)> = all
            .into_iter()
            .filter(|c| c.id != key)
            .map(|c| (matched(&c), c))
            .filter(|(m, _)| *m > 0)
            .collect();
        scored.sort_by(|(ma, a), (mb, b)| {
            mb.cmp(ma)
                .then(b.paper_count.cmp(&a.paper_count))
                .then(a.id.cmp(&b.id))
        });
        scored.truncate(CONCEPT_LIST_CAP);
        Ok(scored.into_iter().map(|(_, c)| c).collect())
    }

    /// Concepts spelled like `query`, for a "did you mean" when nothing contains it:
    /// concept ids within edit distance 1 of the normalized query (2 when it is
    /// longer than 6 characters), closest first, then by paper count and id. Empty for
    /// queries shorter than 4 characters, where a guess would be noise. At most
    /// [`CONCEPT_LIST_CAP`].
    pub fn similar_concepts(&self, query: &str) -> Result<Vec<ConceptSummary>> {
        self.ensure_concepts_fresh()?;
        let Some(key) = query_key(query) else {
            return Ok(Vec::new());
        };
        let len = key.chars().count();
        if len < 4 {
            return Ok(Vec::new());
        }
        let max_distance = if len > 6 { 2 } else { 1 };
        let mut scored: Vec<(usize, ConceptSummary)> = self
            .concept_summaries()?
            .into_iter()
            .map(|c| (edit_distance(&key, &c.id), c))
            .filter(|(d, _)| *d > 0 && *d <= max_distance)
            .collect();
        scored.sort_by(|(da, a), (db, b)| {
            da.cmp(db)
                .then(b.paper_count.cmp(&a.paper_count))
                .then(a.id.cmp(&b.id))
        });
        scored.truncate(CONCEPT_LIST_CAP);
        Ok(scored.into_iter().map(|(_, c)| c).collect())
    }

    /// How many concepts the stored claims form (after ensuring freshness). `0` for an
    /// empty corpus or one whose claims contain only stopwords.
    pub fn concept_count(&self) -> Result<usize> {
        self.ensure_concepts_fresh()?;
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM concepts", [], |row| row.get(0))
            .context("failed to count concepts")?;
        Ok(n as usize)
    }

    fn concept_info(&self, id: &str) -> Result<Option<ConceptInfo>> {
        let head: Option<(String, String, i64)> = self
            .conn
            .query_row(
                "SELECT label, kind, paper_count FROM concepts WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .context("failed to read concept")?;
        let Some((label, kind, paper_count)) = head else {
            return Ok(None);
        };
        let mut stmt = self
            .conn
            .prepare(
                "SELECT form, count FROM concept_forms WHERE concept_id = ?1
                 ORDER BY count DESC, form",
            )
            .context("failed to prepare concept forms query")?;
        let forms = stmt
            .query_map(rusqlite::params![id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })
            .context("failed to execute concept forms query")?
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to read concept forms")?;
        Ok(Some(ConceptInfo {
            id: id.to_string(),
            label,
            kind: parse_kind(&kind)?,
            paper_count: paper_count as usize,
            forms,
        }))
    }

    fn concept_summaries(&self) -> Result<Vec<ConceptSummary>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, label, kind, paper_count FROM concepts ORDER BY id")
            .context("failed to prepare concept summaries query")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .context("failed to execute concept summaries query")?;
        let mut out = Vec::new();
        for row in rows {
            let (id, label, kind, paper_count) = row.context("failed to read concept row")?;
            out.push(ConceptSummary {
                id,
                label,
                kind: parse_kind(&kind)?,
                paper_count: paper_count as usize,
            });
        }
        Ok(out)
    }

    /// The fingerprint persisted concepts must match: formation version, stoplist
    /// digest, and the assertion count and highest assertion id. Sound because
    /// assertions are only ever inserted or deleted (never updated) by
    /// [`Store::save_assertions`], and `AUTOINCREMENT` never reuses an id: rows with
    /// ids at or below the recorded max can only disappear, never reappear or change.
    /// So an unchanged max means no row was added, and an unchanged count then means
    /// none was removed — the claim set is identical.
    fn concept_fingerprint(&self) -> Result<String> {
        let (count, max_id): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(MAX(id), 0) FROM assertions",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .context("failed to fingerprint assertions")?;
        Ok(format!(
            "v{CONCEPT_FORMATION_VERSION}:{:016x}:{count}:{max_id}",
            rules_digest()
        ))
    }

    fn stored_concept_fingerprint(&self) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT value FROM meta WHERE key = ?1",
                rusqlite::params![CONCEPT_FINGERPRINT_KEY],
                |row| row.get(0),
            )
            .optional()
            .context("failed to read concept fingerprint")
    }

    /// Rebuild persisted concepts in full when the fingerprint no longer matches, so
    /// no concept read can observe stale concepts. Takes the write lock up front and
    /// re-checks under it, so concurrent readers rebuild at most once.
    fn ensure_concepts_fresh(&self) -> Result<()> {
        if self.stored_concept_fingerprint()? == Some(self.concept_fingerprint()?) {
            return Ok(());
        }
        self.conn
            .execute_batch("BEGIN IMMEDIATE;")
            .context("failed to begin concept rebuild")?;
        let result = (|| -> Result<()> {
            let current = self.concept_fingerprint()?;
            if self.stored_concept_fingerprint()?.as_deref() == Some(current.as_str()) {
                return Ok(());
            }
            self.rebuild_concepts()?;
            self.conn
                .execute(
                    "INSERT INTO meta (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    rusqlite::params![CONCEPT_FINGERPRINT_KEY, current],
                )
                .context("failed to store concept fingerprint")?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.conn
                    .execute_batch("COMMIT;")
                    .context("failed to commit concept rebuild")?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
    }

    /// Replace every persisted concept with a fresh [`form_concepts`] over all stored
    /// claims. Runs inside the caller's transaction.
    fn rebuild_concepts(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "DELETE FROM assertion_concepts;
                 DELETE FROM concept_forms;
                 DELETE FROM concepts;",
            )
            .context("failed to clear concepts")?;

        let rows: Vec<(i64, String, String)> = {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT a.id, p.arxiv_id, a.claim
                     FROM assertions a
                     JOIN papers p ON p.id = a.paper_id
                     ORDER BY p.arxiv_id, a.id",
                )
                .context("failed to prepare concept source query")?;
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .context("failed to read concept source claims")?
                .collect::<std::result::Result<Vec<_>, _>>()
                .context("failed to collect concept source claims")?
        };
        let claims: Vec<(String, String)> = rows
            .iter()
            .map(|(_, paper, claim)| (paper.clone(), claim.clone()))
            .collect();
        let set = form_concepts(&claims);

        let mut insert_concept = self
            .conn
            .prepare("INSERT INTO concepts (id, label, kind, paper_count) VALUES (?1, ?2, ?3, ?4)")
            .context("failed to prepare concept insert")?;
        let mut insert_form = self
            .conn
            .prepare("INSERT INTO concept_forms (concept_id, form, count) VALUES (?1, ?2, ?3)")
            .context("failed to prepare concept form insert")?;
        let mut insert_link = self
            .conn
            .prepare("INSERT INTO assertion_concepts (assertion_id, concept_id) VALUES (?1, ?2)")
            .context("failed to prepare concept link insert")?;
        for concept in set.iter() {
            insert_concept
                .execute(rusqlite::params![
                    concept.id,
                    concept.label,
                    concept.kind.as_str(),
                    concept.papers.len() as i64
                ])
                .context("failed to insert concept")?;
            for (form, count) in &concept.forms {
                insert_form
                    .execute(rusqlite::params![concept.id, form, *count as i64])
                    .context("failed to insert concept form")?;
            }
            for &idx in &concept.claims {
                insert_link
                    .execute(rusqlite::params![rows[idx].0, concept.id])
                    .context("failed to insert concept link")?;
            }
        }
        Ok(())
    }

    /// Every persisted assertion as `(arxiv_id, claim)`, ordered by paper then
    /// insertion; empty when none. The corpus of claims co-assertion edges relate.
    pub fn all_claims(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT p.arxiv_id, a.claim
                 FROM assertions a
                 JOIN papers p ON p.id = a.paper_id
                 ORDER BY p.arxiv_id, a.id",
            )
            .context("failed to prepare all_claims query")?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .context("failed to execute all_claims query")?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.context("failed to read claim row")?);
        }
        Ok(result)
    }
}

fn row_to_fact(row: &rusqlite::Row<'_>) -> rusqlite::Result<SourceFact> {
    row_to_fact_raw(row)?.map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })
}

/// Returns `Ok(Ok(SourceFact))` on success, `Ok(Err(anyhow::Error))` on parse failure,
/// `Err(rusqlite::Error)` on SQLite failure.
fn row_to_fact_raw(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<SourceFact>> {
    let arxiv_id: String = row.get(0)?;
    let arxiv_version: String = row.get(1)?;
    let title: String = row.get(2)?;
    let authors_json: String = row.get(3)?;
    let summary: String = row.get(4)?;
    let primary_category_code: String = row.get(5)?;
    let categories_json: String = row.get(6)?;
    let published: String = row.get(7)?;
    let updated: String = row.get(8)?;
    let pdf_url: String = row.get(9)?;
    let source_url: String = row.get(10)?;
    let ingested_at: String = row.get(11)?;

    Ok((|| -> anyhow::Result<SourceFact> {
        let authors: Vec<String> =
            serde_json::from_str(&authors_json).context("failed to deserialize authors")?;
        let category_codes: Vec<String> =
            serde_json::from_str(&categories_json).unwrap_or_default();

        let primary_category = crate::id::ArxivCategory::parse(&primary_category_code)
            .unwrap_or_else(|_| crate::id::ArxivCategory::unknown(&primary_category_code));

        let mut categories: Vec<crate::id::ArxivCategory> = Vec::new();
        for code in &category_codes {
            let cat = crate::id::ArxivCategory::parse(code)
                .unwrap_or_else(|_| crate::id::ArxivCategory::unknown(code));
            if !categories.iter().any(|c| c.code() == cat.code()) {
                categories.push(cat);
            }
        }
        if categories.is_empty() {
            categories.push(primary_category.clone());
        }

        Ok(SourceFact {
            arxiv_id,
            title,
            authors,
            summary,
            primary_category,
            categories,
            published,
            updated,
            pdf_url,
            source_url,
            arxiv_version,
            ingested_at,
        })
    })())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertion::Candidate;
    use crate::id::{ArxivCategory, ArxivId, ArxivVersion};
    use crate::observation::Observation;

    /// Build a validated assertion for storage tests.
    fn supported(claim: &str, quotes: &[&str]) -> Assertion<Supported> {
        let support: Vec<Observation> = quotes
            .iter()
            .map(|q| Observation::new(ArxivId::new("2301.00001"), ArxivVersion(1), *q))
            .collect();
        Assertion::<Candidate>::new(claim, support)
            .validate()
            .expect("non-empty support validates")
    }

    fn count(store: &Store, sql: &str) -> i64 {
        store.conn.query_row(sql, [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn test_resolve_db_path_default() {
        let resolved = resolve_db_path(None, Some(PathBuf::from("/data")));
        assert_eq!(
            resolved,
            PathBuf::from("/data").join("diver").join("diver.db")
        );
    }

    #[test]
    fn test_resolve_db_path_no_data_dir() {
        // The fallback branch, unreachable from a test before the data directory
        // became a parameter (`dirs::data_dir()` cannot be stubbed without env
        // mutation).
        let resolved = resolve_db_path(None, None);
        assert_eq!(resolved, PathBuf::from(".diver").join("diver.db"));
    }

    #[test]
    fn test_default_db_path_matches_legacy() {
        // Pins the *composition* `open()` performs, not just the helper, by calling
        // the same `current_db_path()` that `open()` calls. Nothing else in the
        // suite reaches it: `Store::open()` has no caller outside
        // `diver-cli/src/main.rs`, every other test uses `open_in_memory()` or
        // `open_at()`, and both CLI tests always set DIVER_DB, so none of them ever
        // exercises the default branch. Without this, folding the `join("diver")`
        // into the call site as well as the helper would yield
        // `<data>/diver/diver/diver.db` — silently relocating every existing user's
        // corpus — with the whole suite still green.
        //
        // Unconditional by design: `current_db_path_for` takes the override as a
        // parameter, so this needs no "skip when DIVER_DB is set" guard and cannot
        // quietly become a no-op in the very environment this feature teaches
        // developers to create. `current_db_path()`'s `var_os` read is covered by
        // the CLI tests instead.
        let legacy = dirs::data_dir()
            .map(|d| d.join("diver"))
            .unwrap_or_else(|| PathBuf::from(".diver"))
            .join("diver.db");
        assert_eq!(current_db_path_for(None), legacy);
    }

    #[test]
    fn test_resolve_db_path_override() {
        let target = PathBuf::from("/scratch/nonexistent/corpus.db");
        let resolved = resolve_db_path(Some(OsString::from(&target)), Some(PathBuf::from("/data")));
        // The override wins outright, and the data directory is not consulted.
        assert_eq!(resolved, target);
    }

    #[test]
    fn test_resolve_db_path_empty_override() {
        // `std::env::var_os` yields `Some("")` for `DIVER_DB=`, and SQLite reads an
        // empty filename as a private temporary database discarded on close. Treat
        // it as unset rather than handing the caller a throwaway corpus.
        let resolved = resolve_db_path(Some(OsString::new()), Some(PathBuf::from("/data")));
        assert_eq!(
            resolved,
            PathBuf::from("/data").join("diver").join("diver.db")
        );
    }

    #[test]
    fn test_resolve_db_path_is_side_effect_free() {
        let scratch = tempfile::tempdir().unwrap();
        let absent = scratch.path().join("never-created");

        let resolved = resolve_db_path(None, Some(absent.clone()));

        assert_eq!(resolved, absent.join("diver").join("diver.db"));
        assert!(
            !absent.exists(),
            "resolving a path must not create any directory"
        );
    }

    #[test]
    fn test_open_at_creates_parent_dirs() {
        let scratch = tempfile::tempdir().unwrap();
        let db_path = scratch
            .path()
            .join("nested")
            .join("deeper")
            .join("corpus.db");

        let store = Store::open_at(&db_path).unwrap();

        assert!(db_path.exists(), "open_at creates the database file");
        // A schema-dependent query proves `init_schema` ran.
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn test_open_at_round_trip_persists() {
        let scratch = tempfile::tempdir().unwrap();
        let db_path = scratch.path().join("corpus").join("diver.db");
        let fact = test_fact(
            "2301.00001",
            "v1",
            "Attention Is All You Need",
            "2026-09-02",
        );

        {
            let store = Store::open_at(&db_path).unwrap();
            store.save(&fact).unwrap();
            store
                .save_assertions(
                    "2301.00001",
                    "v1",
                    &[supported(
                        "Attention improves translation accuracy.",
                        &["attention improves translation accuracy"],
                    )],
                )
                .unwrap();
        } // store dropped: the connection closes and the process boundary is crossed.

        let reopened = Store::open_at(&db_path).unwrap();
        let got = reopened
            .get("2301.00001")
            .unwrap()
            .expect("paper persisted");
        assert_eq!(got.title, "Attention Is All You Need");

        let assertions = reopened.get_assertions("2301.00001").unwrap();
        assert_eq!(assertions.len(), 1);
        assert_eq!(
            assertions[0].claim,
            "Attention improves translation accuracy."
        );
        // The support quotes and version must survive the reopen too: the EARS
        // clause promises "the same assertions", and a corpus that came back with
        // its `assertion_support` rows dropped would otherwise still pass.
        assert_eq!(assertions[0].version, "v1");
        assert_eq!(
            assertions[0].support,
            vec!["attention improves translation accuracy".to_string()]
        );
    }

    #[test]
    fn test_assertion_schema_created() {
        let store = Store::open_in_memory().unwrap();
        let n = count(
            &store,
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' \
             AND name IN ('assertions', 'assertion_support')",
        );
        assert_eq!(n, 2);
    }

    #[test]
    fn test_save_assertions_persists() {
        let store = Store::open_in_memory().unwrap();
        let assertions = vec![
            supported(
                "Attention improves accuracy.",
                &["attention improves accuracy"],
            ),
            supported(
                "Transformers scale well.",
                &["transformers scale", "scale well"],
            ),
        ];
        store
            .save_assertions("2301.00001", "v2", &assertions)
            .unwrap();

        assert_eq!(count(&store, "SELECT COUNT(*) FROM assertions"), 2);
        assert_eq!(count(&store, "SELECT COUNT(*) FROM assertion_support"), 3);

        let claim: String = store
            .conn
            .query_row(
                "SELECT claim FROM assertions ORDER BY id LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(claim, "Attention improves accuracy.");
    }

    #[test]
    fn test_save_assertions_idempotent_replace() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_assertions(
                "2301.00001",
                "v1",
                &[
                    supported("First.", &["first quote here"]),
                    supported("Second.", &["second quote here"]),
                ],
            )
            .unwrap();
        // Re-save one different assertion for the same (paper, version).
        store
            .save_assertions(
                "2301.00001",
                "v1",
                &[supported("Only.", &["only quote here"])],
            )
            .unwrap();

        assert_eq!(
            count(&store, "SELECT COUNT(*) FROM assertions"),
            1,
            "prior assertions must be replaced"
        );
        assert_eq!(
            count(&store, "SELECT COUNT(*) FROM assertion_support"),
            1,
            "orphaned support rows must be cascade-deleted"
        );
    }

    #[test]
    fn test_assertion_support_fk_enforced() {
        let store = Store::open_in_memory().unwrap();
        let result = store.conn.execute(
            "INSERT INTO assertion_support (assertion_id, quote) VALUES (99999, 'orphan')",
            [],
        );
        assert!(result.is_err(), "orphan assertion_id must be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("FOREIGN KEY"),
            "expected FK violation, got: {msg}"
        );
    }

    #[test]
    fn test_get_assertions_round_trip() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_assertions(
                "2301.00001",
                "v2",
                &[
                    supported(
                        "Attention improves accuracy.",
                        &["attention improves accuracy"],
                    ),
                    supported("It scales.", &["it scales well", "linear scaling"]),
                ],
            )
            .unwrap();

        let stored = store.get_assertions("2301.00001").unwrap();
        assert_eq!(stored.len(), 2);
        assert!(stored.iter().all(|s| s.version == "v2"));

        let claims: Vec<&str> = stored.iter().map(|s| s.claim.as_str()).collect();
        assert!(claims.contains(&"Attention improves accuracy."));
        assert!(claims.contains(&"It scales."));

        let scales = stored.iter().find(|s| s.claim == "It scales.").unwrap();
        assert_eq!(scales.support.len(), 2);
        assert!(scales.support.contains(&"it scales well".to_string()));
    }

    #[test]
    fn test_get_assertions_unknown_empty() {
        let store = Store::open_in_memory().unwrap();
        // Unknown paper.
        assert!(store.get_assertions("9999.99999").unwrap().is_empty());
        // Known paper with no stored assertions.
        store
            .save(&test_fact("2301.00001", "v1", "T", "2026-08-31T00:00:00Z"))
            .unwrap();
        assert!(store.get_assertions("2301.00001").unwrap().is_empty());
    }

    #[test]
    fn test_papers_asserting_matches() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_assertions(
                "2301.00001",
                "v1",
                &[
                    supported(
                        "Attention improves accuracy.",
                        &["attention improves accuracy"],
                    ),
                    supported("Recurrence limits speed.", &["recurrence limits speed"]),
                ],
            )
            .unwrap();

        // Case-insensitive substring match on the claim.
        let hits = store.papers_asserting("ATTENTION").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "2301.00001");
        assert_eq!(hits[0].1, "Attention improves accuracy.");

        // A concept in no claim yields nothing.
        assert!(store.papers_asserting("teleportation").unwrap().is_empty());
    }

    #[test]
    fn test_papers_asserting_empty() {
        let store = Store::open_in_memory().unwrap();
        assert!(store.papers_asserting("anything").unwrap().is_empty());
    }

    fn save(store: &Store, paper: &str, claims: &[&str]) {
        let supported: Vec<Assertion<Supported>> = claims
            .iter()
            .map(|c| supported(c, &[&c.to_lowercase()]))
            .collect();
        store.save_assertions(paper, "v1", &supported).unwrap();
    }

    #[test]
    fn test_concept_resolution() {
        let store = Store::open_in_memory().unwrap();
        let tables = count(
            &store,
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' \
             AND name IN ('concepts', 'concept_forms', 'assertion_concepts', 'meta')",
        );
        assert_eq!(tables, 4);

        save(
            &store,
            "2301.00001",
            &[
                "Diffusion models generate images.",
                "Recurrence limits speed.",
            ],
        );
        save(&store, "2302.00002", &["A diffusion model denoises."]);

        // The query is folded: `Diffusion Models` resolves to the phrase concept.
        let info = store.resolve_concept("Diffusion Models").unwrap().unwrap();
        assert_eq!(info.id, "diffusion model");
        assert_eq!(info.kind, ConceptKind::Phrase);
        assert_eq!(info.paper_count, 2);
        assert!(info.forms.contains(&("diffusion model".to_string(), 1)));
        assert!(info.forms.contains(&("diffusion models".to_string(), 1)));
        assert!(store.resolve_concept("teleportation").unwrap().is_none());

        // Exactly the linked claims, ordered; the unrelated claim in paper 1 is excluded.
        assert_eq!(
            store.claims_for_concept(&info.id).unwrap(),
            vec![
                (
                    "2301.00001".to_string(),
                    "Diffusion models generate images.".to_string()
                ),
                (
                    "2302.00002".to_string(),
                    "A diffusion model denoises.".to_string()
                ),
            ]
        );

        // No substring matching.
        save(&store, "2303.00003", &["An organized network grows."]);
        assert!(store.papers_asserting("gan").unwrap().is_empty());
        assert!(store.papers_asserting("net").unwrap().is_empty());
        assert_eq!(store.papers_asserting("network").unwrap().len(), 1);

        // Links cascade when their assertions are deleted.
        assert!(count(&store, "SELECT COUNT(*) FROM assertion_concepts") > 0);
        store.conn.execute("DELETE FROM assertions", []).unwrap();
        assert_eq!(count(&store, "SELECT COUNT(*) FROM assertion_concepts"), 0);
    }

    #[test]
    fn test_concepts_freshness() {
        let store = Store::open_in_memory().unwrap();
        save(&store, "A", &["Zeppelin airships float."]);
        assert_eq!(
            store
                .resolve_concept("zeppelin")
                .unwrap()
                .unwrap()
                .paper_count,
            1
        );

        // A later write is visible on the next read, with no rebuild call.
        save(&store, "B", &["Zeppelin variants vary.", "Balloons rise."]);
        assert_eq!(
            store
                .resolve_concept("zeppelin")
                .unwrap()
                .unwrap()
                .paper_count,
            2
        );

        // Replacements that remove the last claims forming a concept remove it.
        save(&store, "B", &["Balloons rise."]);
        save(&store, "A", &[]);
        assert!(store.resolve_concept("zeppelin").unwrap().is_none());

        // A fingerprint differing only in formation version, or only in the rules
        // digest, forces a rebuild: delete a concept row behind the store's back, forge
        // the stored fingerprint, and the next read restores the concept.
        let current = store.concept_fingerprint().unwrap();
        assert_eq!(
            store.stored_concept_fingerprint().unwrap().as_deref(),
            Some(current.as_str())
        );
        let parts: Vec<&str> = current.splitn(3, ':').collect();
        let forged_version = format!(
            "v{}:{}:{}",
            CONCEPT_FORMATION_VERSION + 1,
            parts[1],
            parts[2]
        );
        let forged_digest = format!("{}:{:016x}:{}", parts[0], 0, parts[2]);
        for forged in [forged_version, forged_digest] {
            store
                .conn
                .execute("DELETE FROM concepts WHERE id = 'balloon'", [])
                .unwrap();
            store
                .conn
                .execute(
                    "UPDATE meta SET value = ?1 WHERE key = ?2",
                    rusqlite::params![forged, CONCEPT_FINGERPRINT_KEY],
                )
                .unwrap();
            assert!(
                store.resolve_concept("balloons").unwrap().is_some(),
                "{forged}"
            );
            assert_eq!(
                store.stored_concept_fingerprint().unwrap().as_deref(),
                Some(current.as_str())
            );
        }
    }

    #[test]
    fn test_concept_navigation() {
        let empty = Store::open_in_memory().unwrap();
        assert_eq!(empty.concept_count().unwrap(), 0);
        assert!(empty.concept_suggestions("model").unwrap().is_empty());

        let stopwords = Store::open_in_memory().unwrap();
        save(&stopwords, "A", &["The results show that it is."]);
        assert_eq!(stopwords.concept_count().unwrap(), 0);

        let store = Store::open_in_memory().unwrap();
        save(
            &store,
            "P1",
            &["The attention mechanism aids. Diffusion models generate images."],
        );
        save(
            &store,
            "P2",
            &["An attention mechanism aids. A diffusion model denoises."],
        );
        save(
            &store,
            "P3",
            &["Attention spans vary. A transformer model scales."],
        );
        save(&store, "P4", &["The transformer model scales."]);
        assert!(store.concept_count().unwrap() > 0);

        // Linked, not merged: narrower phrases for a term, broader terms for a phrase.
        let ids = |v: Vec<ConceptSummary>| v.into_iter().map(|c| c.id).collect::<Vec<_>>();
        assert!(
            ids(store.concepts_related_to("attention").unwrap())
                .contains(&"attention mechanism".to_string())
        );
        assert!(
            ids(store.concepts_related_to("attention mechanism").unwrap())
                .contains(&"attention".to_string())
        );

        // A stoplisted word points at the phrases it heads.
        let model = ids(store.concept_suggestions("model").unwrap());
        assert_eq!(&model[..2], &["diffusion model", "transformer model"]);

        // Drive repair: with no concept containing every word, coverage ranks first.
        let partial = ids(store
            .concept_suggestions("quantum attention mechanism")
            .unwrap());
        let phrase = partial.iter().position(|id| id == "attention mechanism");
        let word = partial.iter().position(|id| id == "attention");
        assert!(phrase.unwrap() < word.unwrap(), "{partial:?}");

        // Drive repair: a misspelling finds the concept by edit distance.
        assert!(
            ids(store.similar_concepts("atention").unwrap()).contains(&"attention".to_string())
        );
        assert!(
            store.similar_concepts("atn").unwrap().is_empty(),
            "too short to guess"
        );
        assert!(store.concept_suggestions("zzzz").unwrap().len() <= CONCEPT_LIST_CAP);
    }

    #[test]
    fn test_papers_asserting_wildcards_inert() {
        // Replaces the LIKE-escaping test: resolution is by concept, not substring, so
        // wildcard characters in a query can never widen a match.
        let store = Store::open_in_memory().unwrap();
        store
            .save_assertions(
                "2301.00001",
                "v1",
                &[
                    supported("Uses a 50% train split.", &["fifty percent"]),
                    supported("Attention improves accuracy.", &["attention improves"]),
                ],
            )
            .unwrap();

        for query in ["%", "_", "atten%", "50%", "atten_ion"] {
            assert!(
                store.papers_asserting(query).unwrap().is_empty(),
                "{query:?} must match nothing"
            );
        }
        // Punctuation is ignored, so `%attention%` is just `attention`: the wildcards
        // add nothing, and the match is exactly the concept's.
        let exact = store.papers_asserting("attention").unwrap();
        assert_eq!(exact.len(), 1);
        assert_eq!(store.papers_asserting("%attention%").unwrap(), exact);
    }

    #[test]
    fn test_all_claims() {
        let store = Store::open_in_memory().unwrap();
        store
            .save_assertions("2301.00001", "v1", &[supported("Claim A one.", &["q"])])
            .unwrap();
        store
            .save_assertions(
                "2302.00002",
                "v1",
                &[
                    supported("Claim B one.", &["q"]),
                    supported("Claim B two.", &["q"]),
                ],
            )
            .unwrap();

        let claims = store.all_claims().unwrap();
        assert_eq!(claims.len(), 3);
        assert!(claims.contains(&("2301.00001".to_string(), "Claim A one.".to_string())));
        assert!(claims.contains(&("2302.00002".to_string(), "Claim B two.".to_string())));
    }

    #[test]
    fn test_all_claims_empty() {
        let store = Store::open_in_memory().unwrap();
        assert!(store.all_claims().unwrap().is_empty());
    }

    fn test_fact(id: &str, version: &str, title: &str, ingested_at: &str) -> SourceFact {
        let primary = ArxivCategory::parse("cs.CL").unwrap();
        SourceFact {
            arxiv_id: id.to_string(),
            title: title.to_string(),
            authors: vec!["Alice".to_string(), "Bob".to_string()],
            summary: "A summary.".to_string(),
            primary_category: primary.clone(),
            categories: vec![primary],
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            pdf_url: format!("http://arxiv.org/pdf/{id}"),
            source_url: format!("https://export.arxiv.org/api/query?id_list={id}"),
            arxiv_version: version.to_string(),
            ingested_at: ingested_at.to_string(),
        }
    }

    #[test]
    fn test_store_save_and_get() {
        let store = Store::open_in_memory().unwrap();
        let fact = test_fact("2301.00001", "v1", "Test Paper", "2026-08-28T00:00:00Z");

        store.save(&fact).unwrap();
        let retrieved = store.get("2301.00001").unwrap().unwrap();

        assert_eq!(retrieved.arxiv_id, "2301.00001");
        assert_eq!(retrieved.title, "Test Paper");
        assert_eq!(retrieved.authors, vec!["Alice", "Bob"]);
        assert_eq!(retrieved.primary_category.code(), "cs.CL");
        assert_eq!(
            retrieved.source_url,
            "https://export.arxiv.org/api/query?id_list=2301.00001"
        );
        assert_eq!(retrieved.arxiv_version, "v1");
        assert_eq!(retrieved.ingested_at, "2026-08-28T00:00:00Z");
    }

    #[test]
    fn test_store_multi_version() {
        let store = Store::open_in_memory().unwrap();

        let fact_v1 = test_fact("2301.00001", "v1", "Original Title", "2026-08-28T00:00:00Z");
        let fact_v2 = test_fact("2301.00001", "v2", "Updated Title", "2026-08-28T01:00:00Z");
        store.save(&fact_v1).unwrap();
        store.save(&fact_v2).unwrap();

        let versions = store.get_versions("2301.00001").unwrap();
        assert_eq!(versions, vec!["v1", "v2"]);
    }

    #[test]
    fn test_store_idempotent_save() {
        let store = Store::open_in_memory().unwrap();

        let fact_v1 = test_fact("2301.00001", "v1", "Some Title", "2026-08-28T00:00:00Z");
        store.save(&fact_v1).unwrap();
        store.save(&fact_v1).unwrap();

        let versions = store.get_versions("2301.00001").unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0], "v1");
    }

    #[test]
    fn test_store_metadata_correction_applied() {
        let store = Store::open_in_memory().unwrap();

        let fact = test_fact("2301.00001", "v1", "Original Title", "2026-08-28T00:00:00Z");
        store.save(&fact).unwrap();

        let corrected = test_fact(
            "2301.00001",
            "v1",
            "Corrected Title",
            "2026-08-28T01:00:00Z",
        );
        store.save(&corrected).unwrap();

        let retrieved = store.get("2301.00001").unwrap().unwrap();
        assert_eq!(retrieved.title, "Corrected Title");

        let versions = store.get_versions("2301.00001").unwrap();
        assert_eq!(versions.len(), 1);
    }

    #[test]
    fn test_store_get_returns_latest() {
        let store = Store::open_in_memory().unwrap();

        let fact_v1 = test_fact("2301.00001", "v1", "Original Title", "2026-08-28T00:00:00Z");
        let fact_v2 = test_fact("2301.00001", "v2", "Updated Title", "2026-08-28T01:00:00Z");
        store.save(&fact_v1).unwrap();
        store.save(&fact_v2).unwrap();

        let retrieved = store.get("2301.00001").unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");
        assert_eq!(retrieved.arxiv_version, "v2");
    }

    #[test]
    fn test_store_versions_not_destroyed() {
        let store = Store::open_in_memory().unwrap();

        let fact_v1 = test_fact("2301.00001", "v1", "V1 Title", "2026-08-28T00:00:00Z");
        let fact_v2 = test_fact("2301.00001", "v2", "V2 Title", "2026-08-28T01:00:00Z");
        store.save(&fact_v1).unwrap();
        store.save(&fact_v2).unwrap();

        // Both versions must still exist
        let versions = store.get_versions("2301.00001").unwrap();
        assert!(versions.contains(&"v1".to_string()));
        assert!(versions.contains(&"v2".to_string()));
    }

    #[test]
    fn test_store_get_unknown() {
        let store = Store::open_in_memory().unwrap();
        let result = store.get("9999.99999").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_store_list() {
        let store = Store::open_in_memory().unwrap();

        store
            .save(&test_fact(
                "2301.00001",
                "v1",
                "Paper A",
                "2026-08-28T01:00:00Z",
            ))
            .unwrap();
        store
            .save(&test_fact(
                "2302.00002",
                "v1",
                "Paper B",
                "2026-08-28T02:00:00Z",
            ))
            .unwrap();
        store
            .save(&test_fact(
                "2303.00003",
                "v1",
                "Paper C",
                "2026-08-28T03:00:00Z",
            ))
            .unwrap();

        let facts = store.list().unwrap();
        assert_eq!(facts.len(), 3);
        assert_eq!(facts[0].arxiv_id, "2303.00003");
        assert_eq!(facts[1].arxiv_id, "2302.00002");
        assert_eq!(facts[2].arxiv_id, "2301.00001");
    }

    #[test]
    fn test_store_list_empty() {
        let store = Store::open_in_memory().unwrap();
        let facts = store.list().unwrap();
        assert!(facts.is_empty());
    }

    #[test]
    fn test_fk_constraint_enforced() {
        // Regression (review fix #5): PRAGMA foreign_keys=ON must reject a
        // paper_version whose parent papers row does not exist. Insert directly,
        // bypassing save() — which always creates the papers row first — so the
        // constraint is proven live, not merely declared.
        let store = Store::open_in_memory().unwrap();

        let result = store.conn.execute(
            "INSERT INTO paper_versions
             (paper_id, version, title, authors, summary, primary_category,
              categories, published, updated, pdf_url, source_url, ingested_at)
             VALUES (99999, 'v1', 't', '[]', 's', 'cs.CL', '[]', 'p', 'u', 'pdf', 'src', 'ing')",
            [],
        );

        assert!(
            result.is_err(),
            "insert with an orphan paper_id must be rejected"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("FOREIGN KEY"),
            "expected FK violation, got: {msg}"
        );
    }

    #[test]
    fn test_save_populates_fts() {
        let store = Store::open_in_memory().unwrap();
        let fact = test_fact(
            "2301.00001",
            "v1",
            "Attention Is All You Need",
            "2026-08-28T00:00:00Z",
        );
        store.save(&fact).unwrap();

        let count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM paper_versions_fts WHERE paper_versions_fts MATCH 'attention'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_reingest_older_version_keeps_latest_in_fts() {
        // Regression (review fix #1): the FTS index is refreshed from the latest
        // stored version (max ingested_at), not the incoming fact. Ingesting an
        // older version (lower ingested_at) after a newer one must NOT push the
        // older text into the search index.
        let store = Store::open_in_memory().unwrap();
        let primary = ArxivCategory::parse("cs.CL").unwrap();

        let make = |version: &str, summary: &str, ingested_at: &str| SourceFact {
            arxiv_id: "2301.00001".to_string(),
            title: "Shared Title".to_string(),
            authors: vec!["Alice".to_string()],
            summary: summary.to_string(),
            primary_category: primary.clone(),
            categories: vec![primary.clone()],
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            pdf_url: "http://arxiv.org/pdf/2301.00001".to_string(),
            source_url: "https://export.arxiv.org/api/query?id_list=2301.00001".to_string(),
            arxiv_version: version.to_string(),
            ingested_at: ingested_at.to_string(),
        };

        // v2 carries the newer ingestion timestamp; v1 (older) is ingested after.
        store
            .save(&make(
                "v2",
                "latestquantumfoo results",
                "2026-08-28T02:00:00Z",
            ))
            .unwrap();
        store
            .save(&make(
                "v1",
                "olderclassicbar results",
                "2026-08-28T01:00:00Z",
            ))
            .unwrap();

        // FTS reflects v2 (latest by ingested_at), so the v2 term is searchable...
        let latest = store.search("latestquantumfoo", 10).unwrap();
        assert_eq!(latest.len(), 1, "latest (v2) text must be searchable");

        // ...and the older v1 term never entered the index.
        let older = store.search("olderclassicbar", 10).unwrap();
        assert!(older.is_empty(), "stale v1 text must not be in FTS");
    }

    #[test]
    fn test_upsert_updates_fts() {
        let store = Store::open_in_memory().unwrap();

        let fact1 = test_fact(
            "2301.00001",
            "v1",
            "Original Unique Title",
            "2026-08-28T00:00:00Z",
        );
        store.save(&fact1).unwrap();

        let fact2 = test_fact(
            "2301.00001",
            "v2",
            "Replaced Different Title",
            "2026-08-28T01:00:00Z",
        );
        store.save(&fact2).unwrap();

        let old_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM paper_versions_fts WHERE paper_versions_fts MATCH 'original'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(old_count, 0);

        let new_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM paper_versions_fts WHERE paper_versions_fts MATCH 'replaced'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(new_count, 1);
    }

    fn search_fact(id: &str, title: &str, summary: &str) -> SourceFact {
        let primary = ArxivCategory::parse("cs.CL").unwrap();
        SourceFact {
            arxiv_id: id.to_string(),
            title: title.to_string(),
            authors: vec!["Author".to_string()],
            summary: summary.to_string(),
            primary_category: primary.clone(),
            categories: vec![primary],
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            pdf_url: format!("http://arxiv.org/pdf/{id}"),
            source_url: format!("https://export.arxiv.org/api/query?id_list={id}"),
            arxiv_version: "v1".to_string(),
            ingested_at: "2026-08-28T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_search_ranked_results() {
        let store = Store::open_in_memory().unwrap();

        store
            .save(&search_fact(
                "0001",
                "Attention Mechanisms in Neural Networks",
                "This paper studies convolutional approaches.",
            ))
            .unwrap();
        store
            .save(&search_fact(
                "0002",
                "Recurrent Models for Sequences",
                "We explore attention-based decoding strategies.",
            ))
            .unwrap();
        store
            .save(&search_fact(
                "0003",
                "Attention Is All You Need",
                "We propose attention mechanisms for sequence transduction.",
            ))
            .unwrap();

        let results = store.search("attention", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results.len() <= 3);

        let ids: Vec<&str> = results.iter().map(|r| r.arxiv_id.as_str()).collect();
        assert!(ids.contains(&"0001"));
        assert!(ids.contains(&"0003"));
    }

    #[test]
    fn test_search_no_results() {
        let store = Store::open_in_memory().unwrap();
        store
            .save(&search_fact("0001", "Some Paper", "About something."))
            .unwrap();

        let results = store.search("xyznonexistent", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_max_results() {
        let store = Store::open_in_memory().unwrap();

        for i in 1..=5 {
            store
                .save(&search_fact(
                    &format!("000{i}"),
                    &format!("Test Paper {i}"),
                    "Testing the search functionality.",
                ))
                .unwrap();
        }

        let results = store.search("test", 2).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_phrase() {
        let store = Store::open_in_memory().unwrap();

        store
            .save(&search_fact(
                "0001",
                "Attention Mechanism Design",
                "We study attention mechanism patterns.",
            ))
            .unwrap();
        store
            .save(&search_fact(
                "0002",
                "Mechanism of Action",
                "Attention to detail is important.",
            ))
            .unwrap();

        let results = store.search("\"attention mechanism\"", 10).unwrap();
        assert!(!results.is_empty());
        for r in &results {
            let combined = format!("{} {}", r.title, r.summary).to_lowercase();
            assert!(combined.contains("attention mechanism"));
        }
    }
}
