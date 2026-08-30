use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::fact::SourceFact;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub primary_category: String,
    pub rank: f64,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .map(|d| d.join("diver"))
            .unwrap_or_else(|| std::path::PathBuf::from(".diver"));

        std::fs::create_dir_all(&data_dir)
            .with_context(|| format!("failed to create data directory: {}", data_dir.display()))?;

        let db_path = data_dir.join("diver.db");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("failed to open database: {}", db_path.display()))?;

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
                USING fts5(arxiv_id, title, authors, summary, primary_category);",
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
            .query_map([], |row| row_to_fact(row))
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
    use crate::id::ArxivCategory;

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
