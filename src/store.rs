use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::fact::SourceFact;

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
            );",
            )
            .context("failed to initialize database schema")?;
        Ok(())
    }

    pub fn save(&self, fact: &SourceFact) -> Result<()> {
        let authors_json =
            serde_json::to_string(&fact.authors).context("failed to serialize authors")?;

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
        Ok(())
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
}
