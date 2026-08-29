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
                "CREATE TABLE IF NOT EXISTS source_facts (
                arxiv_id         TEXT PRIMARY KEY,
                title            TEXT NOT NULL,
                authors          TEXT NOT NULL,
                summary          TEXT NOT NULL,
                primary_category TEXT NOT NULL,
                published        TEXT NOT NULL,
                updated          TEXT NOT NULL,
                pdf_url          TEXT NOT NULL,
                source_url       TEXT NOT NULL,
                arxiv_version    TEXT NOT NULL,
                ingested_at      TEXT NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS source_facts_fts
            USING fts5(arxiv_id, title, authors, summary, primary_category);",
            )
            .context("failed to initialize database schema")?;

        self.backfill_fts()?;
        Ok(())
    }

    fn backfill_fts(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM source_facts_fts", [], |row| {
                row.get(0)
            })
            .context("failed to count FTS rows")?;

        if count > 0 {
            return Ok(());
        }

        let source_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM source_facts", [], |row| row.get(0))
            .context("failed to count source_facts rows")?;

        if source_count == 0 {
            return Ok(());
        }

        self.conn
            .execute_batch(
                "INSERT INTO source_facts_fts (arxiv_id, title, authors, summary, primary_category)
                 SELECT arxiv_id, title, authors, summary, primary_category
                 FROM source_facts;",
            )
            .context("failed to backfill FTS index")?;

        Ok(())
    }

    pub fn save(&self, fact: &SourceFact) -> Result<()> {
        let authors_json =
            serde_json::to_string(&fact.authors).context("failed to serialize authors")?;
        let authors_text = fact.authors.join(", ");

        self.conn
            .execute_batch("BEGIN;")
            .context("failed to begin transaction")?;

        let result = (|| -> Result<()> {
            self.conn
                .execute(
                    "INSERT OR REPLACE INTO source_facts
                 (arxiv_id, title, authors, summary, primary_category,
                  published, updated, pdf_url, source_url, arxiv_version, ingested_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    rusqlite::params![
                        fact.arxiv_id,
                        fact.title,
                        authors_json,
                        fact.summary,
                        fact.primary_category,
                        fact.published,
                        fact.updated,
                        fact.pdf_url,
                        fact.source_url,
                        fact.arxiv_version,
                        fact.ingested_at,
                    ],
                )
                .context("failed to save source fact")?;

            self.conn
                .execute(
                    "DELETE FROM source_facts_fts WHERE arxiv_id = ?1",
                    rusqlite::params![fact.arxiv_id],
                )
                .context("failed to delete old FTS entry")?;

            self.conn
                .execute(
                    "INSERT INTO source_facts_fts (arxiv_id, title, authors, summary, primary_category)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        fact.arxiv_id,
                        fact.title,
                        authors_text,
                        fact.summary,
                        fact.primary_category,
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

    pub fn get(&self, arxiv_id: &str) -> Result<Option<SourceFact>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT arxiv_id, title, authors, summary, primary_category,
                    published, updated, pdf_url, source_url, arxiv_version, ingested_at
             FROM source_facts WHERE arxiv_id = ?1",
            )
            .context("failed to prepare get query")?;

        let mut rows = stmt
            .query(rusqlite::params![arxiv_id])
            .context("failed to execute get query")?;

        match rows.next().context("failed to read row")? {
            Some(row) => {
                let authors_json: String = row.get(2)?;
                let authors: Vec<String> =
                    serde_json::from_str(&authors_json).context("failed to deserialize authors")?;

                Ok(Some(SourceFact {
                    arxiv_id: row.get(0)?,
                    title: row.get(1)?,
                    authors,
                    summary: row.get(3)?,
                    primary_category: row.get(4)?,
                    published: row.get(5)?,
                    updated: row.get(6)?,
                    pdf_url: row.get(7)?,
                    source_url: row.get(8)?,
                    arxiv_version: row.get(9)?,
                    ingested_at: row.get(10)?,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn list(&self) -> Result<Vec<SourceFact>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT arxiv_id, title, authors, summary, primary_category,
                    published, updated, pdf_url, source_url, arxiv_version, ingested_at
             FROM source_facts ORDER BY ingested_at DESC",
            )
            .context("failed to prepare list query")?;

        let facts = stmt
            .query_map([], |row| {
                let authors_json: String = row.get(2)?;
                let authors: Vec<String> = serde_json::from_str(&authors_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                Ok(SourceFact {
                    arxiv_id: row.get(0)?,
                    title: row.get(1)?,
                    authors,
                    summary: row.get(3)?,
                    primary_category: row.get(4)?,
                    published: row.get(5)?,
                    updated: row.get(6)?,
                    pdf_url: row.get(7)?,
                    source_url: row.get(8)?,
                    arxiv_version: row.get(9)?,
                    ingested_at: row.get(10)?,
                })
            })
            .context("failed to execute list query")?;

        let mut result = Vec::new();
        for fact in facts {
            result.push(fact.context("failed to read source fact row")?);
        }
        Ok(result)
    }

    pub fn exists(&self, arxiv_id: &str) -> Result<bool> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM source_facts WHERE arxiv_id = ?1",
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
                "SELECT f.arxiv_id, f.title, sf.authors, f.summary, f.primary_category, f.rank
                 FROM source_facts_fts f
                 JOIN source_facts sf ON sf.arxiv_id = f.arxiv_id
                 WHERE source_facts_fts MATCH ?1
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fact(id: &str, title: &str, ingested_at: &str) -> SourceFact {
        SourceFact {
            arxiv_id: id.to_string(),
            title: title.to_string(),
            authors: vec!["Alice".to_string(), "Bob".to_string()],
            summary: "A summary.".to_string(),
            primary_category: "cs.CL".to_string(),
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            pdf_url: format!("http://arxiv.org/pdf/{id}"),
            source_url: format!("https://export.arxiv.org/api/query?id_list={id}"),
            arxiv_version: "v1".to_string(),
            ingested_at: ingested_at.to_string(),
        }
    }

    #[test]
    fn test_store_save_and_get() {
        let store = Store::open_in_memory().unwrap();
        let fact = test_fact("2301.00001", "Test Paper", "2026-08-28T00:00:00Z");

        store.save(&fact).unwrap();
        let retrieved = store.get("2301.00001").unwrap().unwrap();

        assert_eq!(retrieved.arxiv_id, "2301.00001");
        assert_eq!(retrieved.title, "Test Paper");
        assert_eq!(retrieved.authors, vec!["Alice", "Bob"]);
        assert_eq!(retrieved.primary_category, "cs.CL");
        assert_eq!(
            retrieved.source_url,
            "https://export.arxiv.org/api/query?id_list=2301.00001"
        );
        assert_eq!(retrieved.arxiv_version, "v1");
        assert_eq!(retrieved.ingested_at, "2026-08-28T00:00:00Z");
    }

    #[test]
    fn test_store_upsert() {
        let store = Store::open_in_memory().unwrap();

        let fact1 = test_fact("2301.00001", "Original Title", "2026-08-28T00:00:00Z");
        store.save(&fact1).unwrap();

        let fact2 = test_fact("2301.00001", "Updated Title", "2026-08-28T01:00:00Z");
        store.save(&fact2).unwrap();

        let retrieved = store.get("2301.00001").unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");

        let all = store.list().unwrap();
        assert_eq!(all.len(), 1);
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
            .save(&test_fact("2301.00001", "Paper A", "2026-08-28T01:00:00Z"))
            .unwrap();
        store
            .save(&test_fact("2302.00002", "Paper B", "2026-08-28T02:00:00Z"))
            .unwrap();
        store
            .save(&test_fact("2303.00003", "Paper C", "2026-08-28T03:00:00Z"))
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
    fn test_save_populates_fts() {
        let store = Store::open_in_memory().unwrap();
        let fact = test_fact(
            "2301.00001",
            "Attention Is All You Need",
            "2026-08-28T00:00:00Z",
        );
        store.save(&fact).unwrap();

        let count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM source_facts_fts WHERE source_facts_fts MATCH 'attention'",
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
            "Original Unique Title",
            "2026-08-28T00:00:00Z",
        );
        store.save(&fact1).unwrap();

        let mut fact2 = test_fact(
            "2301.00001",
            "Replaced Different Title",
            "2026-08-28T01:00:00Z",
        );
        fact2.summary = "New summary content.".to_string();
        store.save(&fact2).unwrap();

        let old_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM source_facts_fts WHERE source_facts_fts MATCH 'original'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(old_count, 0);

        let new_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM source_facts_fts WHERE source_facts_fts MATCH 'replaced'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(new_count, 1);
    }

    #[test]
    fn test_fts_indexes_multiple_fields() {
        let store = Store::open_in_memory().unwrap();
        let mut fact = test_fact("2301.00001", "Some Paper", "2026-08-28T00:00:00Z");
        fact.authors = vec!["Vaswani".to_string(), "Shazeer".to_string()];
        fact.primary_category = "cs.LG".to_string();
        store.save(&fact).unwrap();

        let by_author: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM source_facts_fts WHERE source_facts_fts MATCH 'vaswani'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(by_author, 1);

        let by_category: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM source_facts_fts WHERE source_facts_fts MATCH 'cs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(by_category, 1);
    }

    #[test]
    fn test_init_schema_backfills_existing_facts() {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS source_facts (
                arxiv_id         TEXT PRIMARY KEY,
                title            TEXT NOT NULL,
                authors          TEXT NOT NULL,
                summary          TEXT NOT NULL,
                primary_category TEXT NOT NULL,
                published        TEXT NOT NULL,
                updated          TEXT NOT NULL,
                pdf_url          TEXT NOT NULL,
                source_url       TEXT NOT NULL,
                arxiv_version    TEXT NOT NULL,
                ingested_at      TEXT NOT NULL
            );",
        )
        .unwrap();

        conn.execute(
            "INSERT INTO source_facts VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            rusqlite::params![
                "2301.00001",
                "Pre-existing Paper About Transformers",
                "[\"Alice\"]",
                "A pre-existing summary.",
                "cs.CL",
                "2023-01-01T00:00:00Z",
                "2023-01-01T00:00:00Z",
                "http://arxiv.org/pdf/2301.00001",
                "https://export.arxiv.org/api/query?id_list=2301.00001",
                "v1",
                "2026-08-28T00:00:00Z",
            ],
        )
        .unwrap();

        let store = Store { conn };
        store.init_schema().unwrap();

        let count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM source_facts_fts WHERE source_facts_fts MATCH 'transformers'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    fn search_fact(id: &str, title: &str, summary: &str) -> SourceFact {
        SourceFact {
            arxiv_id: id.to_string(),
            title: title.to_string(),
            authors: vec!["Author".to_string()],
            summary: summary.to_string(),
            primary_category: "cs.CL".to_string(),
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
