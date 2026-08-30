use anyhow::{Context, Result, bail};
use reqwest::Client;

use crate::model::Paper;
use crate::parse::{self, FeedResult};
use crate::query::QueryBuilder;

const ARXIV_API_BASE: &str = "https://export.arxiv.org/api/query";

pub struct ArxivClient {
    http: Client,
}

impl ArxivClient {
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .user_agent("diver/0.1.0 (https://github.com/crussella0129/diver)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self { http })
    }

    pub async fn search(&self, query: &QueryBuilder) -> Result<FeedResult> {
        let url = query.build();

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .context("failed to reach ArXiv API")?;

        let status = response.status();
        if !status.is_success() {
            bail!("ArXiv API returned HTTP {status}");
        }

        let body = response
            .text()
            .await
            .context("failed to read ArXiv response body")?;

        parse::parse_feed(&body).context("failed to parse ArXiv response")
    }

    pub async fn fetch_by_id(&self, arxiv_id: &str) -> Result<(Paper, String)> {
        let url = format!("{ARXIV_API_BASE}?id_list={arxiv_id}&max_results=1");

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .context("failed to reach ArXiv API")?;

        let status = response.status();
        if !status.is_success() {
            bail!("ArXiv API returned HTTP {status}");
        }

        let body = response
            .text()
            .await
            .context("failed to read ArXiv response body")?;

        let feed = parse::parse_feed(&body).context("failed to parse ArXiv response")?;
        let paper = extract_paper(feed)?;
        Ok((paper, url))
    }
}

pub fn extract_paper(feed: FeedResult) -> Result<Paper> {
    if feed.papers.is_empty() {
        bail!("paper not found on ArXiv");
    }

    let paper = feed.papers.into_iter().next().unwrap();

    if paper.title.contains("Error") {
        bail!("paper not found on ArXiv: {}", paper.title);
    }

    Ok(paper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_paper_valid() {
        let xml = std::fs::read_to_string("tests/fixtures/sample_feed.xml").unwrap();
        let feed = parse::parse_feed(&xml).unwrap();
        let paper = extract_paper(feed).unwrap();
        assert_eq!(paper.title, "Attention Is All You Need Revisited");
        assert_eq!(paper.primary_category, "cs.CL");
    }

    #[test]
    fn test_extract_paper_error_entry() {
        let feed = FeedResult {
            papers: vec![Paper {
                title: "Error: arXiv ID does not exist".to_string(),
                authors: vec![],
                summary: String::new(),
                primary_category: String::new(),
                categories: vec![],
                published: String::new(),
                updated: String::new(),
                arxiv_id: String::new(),
                pdf_url: String::new(),
            }],
            total_results: 1,
        };
        let result = extract_paper(feed);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_extract_paper_empty_feed() {
        let feed = FeedResult {
            papers: vec![],
            total_results: 0,
        };
        let result = extract_paper(feed);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
