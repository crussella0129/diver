use crate::model::Paper;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct SourceFact {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub primary_category: String,
    pub published: String,
    pub updated: String,
    pub pdf_url: String,
    pub source_url: String,
    pub arxiv_version: String,
    pub ingested_at: String,
}

impl SourceFact {
    pub fn from_paper(paper: Paper, source_url: String) -> Self {
        let (bare_id, version) = parse_arxiv_id(&paper.arxiv_id);

        Self {
            arxiv_id: bare_id,
            title: paper.title,
            authors: paper.authors,
            summary: paper.summary,
            primary_category: paper.primary_category,
            published: paper.published,
            updated: paper.updated,
            pdf_url: paper.pdf_url,
            source_url,
            arxiv_version: version,
            ingested_at: Utc::now().to_rfc3339(),
        }
    }
}

fn parse_arxiv_id(raw: &str) -> (String, String) {
    if let Some(pos) = raw.rfind('v') {
        let after = &raw[pos + 1..];
        if !after.is_empty() && after.chars().all(|c| c.is_ascii_digit()) {
            let bare = raw[..pos].to_string();
            let version = format!("v{after}");
            return (bare, version);
        }
    }
    (raw.to_string(), "v1".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_paper() -> Paper {
        Paper {
            title: "Test Paper".to_string(),
            authors: vec!["Alice".to_string(), "Bob".to_string()],
            summary: "A summary.".to_string(),
            primary_category: "cs.CL".to_string(),
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            arxiv_id: "2301.00001v2".to_string(),
            pdf_url: "http://arxiv.org/pdf/2301.00001v2".to_string(),
        }
    }

    #[test]
    fn test_source_fact_from_paper() {
        let paper = test_paper();
        let fact = SourceFact::from_paper(
            paper,
            "https://export.arxiv.org/api/query?id_list=2301.00001".to_string(),
        );

        assert_eq!(fact.title, "Test Paper");
        assert_eq!(fact.authors, vec!["Alice", "Bob"]);
        assert_eq!(
            fact.source_url,
            "https://export.arxiv.org/api/query?id_list=2301.00001"
        );
        assert!(!fact.ingested_at.is_empty());
    }

    #[test]
    fn test_source_fact_version_extraction() {
        let paper = test_paper();
        let fact = SourceFact::from_paper(paper, String::new());

        assert_eq!(fact.arxiv_id, "2301.00001");
        assert_eq!(fact.arxiv_version, "v2");
    }

    #[test]
    fn test_source_fact_default_version() {
        let mut paper = test_paper();
        paper.arxiv_id = "2301.00001".to_string();
        let fact = SourceFact::from_paper(paper, String::new());

        assert_eq!(fact.arxiv_id, "2301.00001");
        assert_eq!(fact.arxiv_version, "v1");
    }
}
