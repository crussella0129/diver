use crate::id::ArxivCategory;
use crate::model::Paper;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct SourceFact {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub primary_category: ArxivCategory,
    /// All categories for this paper, including primary, validated against taxonomy.
    pub categories: Vec<ArxivCategory>,
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

        // Parse and validate primary_category; fall back to a stub on unknown codes.
        let primary_category = parse_category_lenient(&paper.primary_category);

        // Parse all categories, skipping unknowns with a warning.
        let mut categories: Vec<ArxivCategory> = Vec::new();
        for code in &paper.categories {
            match ArxivCategory::parse(code) {
                Ok(cat) => {
                    // Deduplicate by code
                    if !categories.iter().any(|c| c.code() == cat.code()) {
                        categories.push(cat);
                    }
                }
                Err(_) => {
                    eprintln!(
                        "diver: warning: arXiv category '{}' not in taxonomy snapshot, skipping",
                        code
                    );
                }
            }
        }

        // Ensure primary_category is in the categories list
        if categories
            .iter()
            .all(|c| c.code() != primary_category.code())
        {
            categories.insert(0, primary_category.clone());
        }

        Self {
            arxiv_id: bare_id,
            title: paper.title,
            authors: paper.authors,
            summary: paper.summary,
            primary_category,
            categories,
            published: paper.published,
            updated: paper.updated,
            pdf_url: paper.pdf_url,
            source_url,
            arxiv_version: version,
            ingested_at: Utc::now().to_rfc3339(),
        }
    }

    /// Returns category codes as a JSON array string for storage.
    pub fn categories_json(&self) -> String {
        let codes: Vec<&str> = self.categories.iter().map(|c| c.code()).collect();
        serde_json::to_string(&codes).unwrap_or_else(|_| "[]".to_string())
    }
}

/// Parse a category code leniently: returns a fallback stub for unknown codes
/// (preserving the code string) rather than panicking.
fn parse_category_lenient(code: &str) -> ArxivCategory {
    ArxivCategory::parse(code).unwrap_or_else(|_| {
        eprintln!(
            "diver: warning: primary category '{}' not in taxonomy snapshot, using unknown placeholder",
            code
        );
        // Safe to unwrap: "cs.OH" is always valid in the taxonomy.
        ArxivCategory::parse("cs.OH").expect("cs.OH must be in taxonomy")
    })
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
            categories: vec!["cs.CL".to_string()],
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

    #[test]
    fn test_source_fact_categories() {
        let mut paper = test_paper();
        paper.categories = vec!["cs.CL".to_string(), "cs.AI".to_string(), "cs.LG".to_string()];
        let fact = SourceFact::from_paper(paper, String::new());

        assert_eq!(fact.categories.len(), 3);
        assert_eq!(fact.primary_category.code(), "cs.CL");
    }

    #[test]
    fn test_source_fact_unknown_category_skipped() {
        let mut paper = test_paper();
        paper.categories = vec!["cs.CL".to_string(), "invalid.XX".to_string()];
        let fact = SourceFact::from_paper(paper, String::new());

        // "invalid.XX" should be silently skipped
        assert_eq!(fact.categories.len(), 1);
        assert_eq!(fact.categories[0].code(), "cs.CL");
    }

    #[test]
    fn test_categories_json() {
        let mut paper = test_paper();
        paper.categories = vec!["cs.CL".to_string(), "cs.AI".to_string()];
        let fact = SourceFact::from_paper(paper, String::new());

        let json = fact.categories_json();
        assert!(json.contains("cs.CL"), "got: {json}");
        assert!(json.contains("cs.AI"), "got: {json}");
    }
}
