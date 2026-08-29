use anyhow::{Context, Result, bail};
use reqwest::Client;

use crate::parse::{self, FeedResult};
use crate::query::QueryBuilder;

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
}
