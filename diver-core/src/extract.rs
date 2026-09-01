//! LLM-backed claim extraction.
//!
//! [`LlmExtractor`] asks Claude to read a paper's abstract and return the factual
//! claims it makes, each with a supporting quote. The non-deterministic network
//! call is thin glue over the pure, unit-tested [`parse_claims`], which turns a
//! Messages API response into grounded [`Assertion<Candidate>`]s. **Grounding** is
//! the epistemic gate at extraction: a claim becomes a candidate only if its quote
//! is actually present in the abstract, so hallucinated claims never enter the
//! pipeline. Candidates then pass through the existing
//! [`Assertion::<Candidate>::validate`] gate like any other.

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::assertion::{Assertion, Candidate};
use crate::fact::SourceFact;
use crate::id::{ArxivId, ArxivVersion};
use crate::observation::Observation;

/// Default model when `DIVER_MODEL` is unset. Per Anthropic guidance, default to
/// the most capable model; the user downgrades via `DIVER_MODEL` for cost.
const DEFAULT_MODEL: &str = "claude-opus-5";

/// Default Anthropic API root. `extract` posts to `{base_url}/v1/messages`. The
/// root is injectable (via [`LlmExtractor::build`] / `ANTHROPIC_BASE_URL`) so tests
/// can point the client at a local mock server.
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Instruction to Claude: extract grounded claims as a JSON array.
const SYSTEM_PROMPT: &str = "You extract the discrete factual claims a research \
paper's abstract asserts. Return ONLY a JSON array. Each element is an object with \
two string fields: \"claim\" (one self-contained factual statement, in your own \
words) and \"quote\" (a span copied verbatim from the abstract that supports the \
claim). Copy each quote exactly as it appears. Do not invent claims the abstract \
does not support. If the abstract makes no factual claims, return [].";

/// Extracts grounded candidate assertions from a paper's abstract by asking Claude.
///
/// The network call is thin glue over the pure [`parse_claims`]; construct with
/// [`LlmExtractor::from_env`] (reads `ANTHROPIC_API_KEY`, optional `DIVER_MODEL`).
pub struct LlmExtractor {
    http: reqwest::Client,
    model: String,
    api_key: String,
    base_url: String,
}

// Manual Debug that redacts the API key — never print the secret.
impl std::fmt::Debug for LlmExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmExtractor")
            .field("model", &self.model)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

impl LlmExtractor {
    /// Build from environment: `ANTHROPIC_API_KEY` (required) and `DIVER_MODEL`
    /// (optional, defaults to [`DEFAULT_MODEL`]).
    pub fn from_env() -> Result<Self> {
        Self::build(
            std::env::var("ANTHROPIC_API_KEY").ok(),
            std::env::var("DIVER_MODEL").ok(),
            std::env::var("ANTHROPIC_BASE_URL").ok(),
        )
    }

    /// Pure construction logic (env-reading split out for testability). A missing
    /// or blank key is an actionable error; a missing/blank model falls back to
    /// [`DEFAULT_MODEL`] and a missing/blank base URL to [`DEFAULT_BASE_URL`].
    fn build(
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
    ) -> Result<Self> {
        let api_key = api_key.filter(|k| !k.trim().is_empty()).context(
            "ANTHROPIC_API_KEY is not set — set it to use LLM extraction, or run \
             `diver extract <id> --deterministic` for the offline extractor",
        )?;
        let model = model
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let base_url = base_url
            .filter(|u| !u.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self {
            http,
            model,
            api_key,
            base_url,
        })
    }

    /// Ask Claude to extract grounded candidate assertions from `fact`'s abstract.
    pub async fn extract(&self, fact: &SourceFact) -> Result<Vec<Assertion<Candidate>>> {
        let user_content = format!("Abstract of \"{}\":\n\n{}", fact.title, fact.summary);
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 2048,
            "system": SYSTEM_PROMPT,
            "messages": [{ "role": "user", "content": user_content }],
        })
        .to_string();

        let response = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .context("failed to reach the Anthropic API")?;

        let status = response.status();
        let text = response
            .text()
            .await
            .context("failed to read Anthropic API response body")?;
        if !status.is_success() {
            anyhow::bail!("Anthropic API returned HTTP {status}: {text}");
        }

        parse_claims(&text, fact)
    }
}

#[derive(Debug, Deserialize)]
struct MessagesResponse {
    #[serde(default)]
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaimJson {
    claim: String,
    quote: String,
}

/// Parse a Claude Messages API response body into grounded candidate assertions.
///
/// Only claims whose `quote` is grounded in `fact.summary` become candidates;
/// ungrounded (hallucinated) quotes are dropped. Returns `Err` — never panics —
/// on an unparseable envelope, a missing text block, or non-JSON model output.
pub fn parse_claims(response_body: &str, fact: &SourceFact) -> Result<Vec<Assertion<Candidate>>> {
    let response: MessagesResponse = serde_json::from_str(response_body)
        .context("failed to parse Messages API response envelope")?;

    let text = response
        .content
        .iter()
        .find(|b| b.kind == "text")
        .map(|b| b.text.as_str())
        .context("Messages API response had no text content block")?;

    let claims = parse_claim_array(text).context("failed to parse claims from model output")?;

    let arxiv_id = ArxivId::new(fact.arxiv_id.clone());
    let version = ArxivVersion::parse(&fact.arxiv_version);

    let candidates = claims
        .into_iter()
        .filter(|c| is_grounded(&c.quote, &fact.summary))
        .map(|c| {
            let obs = Observation::new(arxiv_id.clone(), version.clone(), c.quote);
            Assertion::<Candidate>::new(c.claim, vec![obs])
        })
        .collect();

    Ok(candidates)
}

/// Parse the model's text into a claim array, tolerating markdown fences and
/// surrounding prose: try direct JSON first, then a fence-stripped parse, then a
/// last-resort slice from the first `[` to the last `]`.
fn parse_claim_array(text: &str) -> Result<Vec<ClaimJson>> {
    if let Ok(claims) = serde_json::from_str::<Vec<ClaimJson>>(text.trim()) {
        return Ok(claims);
    }

    let stripped = strip_fences(text);
    if let Ok(claims) = serde_json::from_str::<Vec<ClaimJson>>(stripped.trim()) {
        return Ok(claims);
    }

    // Last resort: slice the outer array out of surrounding prose.
    if let (Some(start), Some(end)) = (text.find('['), text.rfind(']'))
        && end > start
    {
        return serde_json::from_str(&text[start..=end])
            .context("model output was not a JSON array of claims");
    }

    anyhow::bail!("model output contained no JSON claim array");
}

/// Strip a leading ```lang fence and trailing ``` fence, returning the inner body.
fn strip_fences(text: &str) -> String {
    let t = text.trim();
    match t.strip_prefix("```") {
        Some(rest) => {
            // Drop the optional language tag on the fence's first line.
            let after_lang = rest.split_once('\n').map(|(_, b)| b).unwrap_or(rest);
            after_lang
                .strip_suffix("```")
                .unwrap_or(after_lang)
                .trim()
                .to_string()
        }
        None => t.to_string(),
    }
}

/// Is `quote` grounded in `summary`? Case-insensitive, whitespace-normalized
/// substring — honoring the verbatim-copy contract with tolerance for whitespace.
fn is_grounded(quote: &str, summary: &str) -> bool {
    let q = normalize(quote);
    !q.is_empty() && normalize(summary).contains(&q)
}

fn normalize(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ArxivCategory;

    fn fact_with_summary(summary: &str) -> SourceFact {
        let primary = ArxivCategory::parse("cs.CL").unwrap();
        SourceFact {
            arxiv_id: "2301.00001".to_string(),
            title: "Test Paper".to_string(),
            authors: vec!["Alice".to_string()],
            summary: summary.to_string(),
            primary_category: primary.clone(),
            categories: vec![primary],
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            pdf_url: "http://arxiv.org/pdf/2301.00001".to_string(),
            source_url: "https://export.arxiv.org/api/query?id_list=2301.00001".to_string(),
            arxiv_version: "v2".to_string(),
            ingested_at: "2026-08-31T00:00:00Z".to_string(),
        }
    }

    /// Wrap model output text in a Messages API response envelope.
    fn envelope(text: &str) -> String {
        serde_json::json!({
            "content": [{ "type": "text", "text": text }],
            "stop_reason": "end_turn"
        })
        .to_string()
    }

    #[test]
    fn test_parse_claims_grounded() {
        let fact = fact_with_summary(
            "We show that attention improves translation accuracy by five points.",
        );
        let body = envelope(
            r#"[{"claim": "Attention improves translation accuracy.", "quote": "attention improves translation accuracy by five points"}]"#,
        );
        let candidates = parse_claims(&body, &fact).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].claim(),
            "Attention improves translation accuracy."
        );
        let obs = &candidates[0].support()[0];
        assert_eq!(obs.arxiv_id().as_str(), "2301.00001");
        assert_eq!(*obs.version(), ArxivVersion(2));
    }

    #[test]
    fn test_parse_claims_drops_hallucinated() {
        let fact = fact_with_summary("We study attention mechanisms for sequence models.");
        let body = envelope(
            r#"[
                {"claim": "Attention is studied.", "quote": "We study attention mechanisms"},
                {"claim": "The model cures cancer.", "quote": "our model cures cancer completely"}
            ]"#,
        );
        let candidates = parse_claims(&body, &fact).unwrap();
        // Only the grounded claim survives; the fabricated quote is dropped.
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].claim(), "Attention is studied.");
    }

    #[test]
    fn test_parse_claims_tolerates_fences() {
        let fact = fact_with_summary("Transformers scale better than recurrence.");
        let fenced = "```json\n[{\"claim\": \"Transformers scale well.\", \"quote\": \"Transformers scale better than recurrence\"}]\n```";
        let body = envelope(fenced);
        let candidates = parse_claims(&body, &fact).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].claim(), "Transformers scale well.");
    }

    #[test]
    fn test_parse_claims_tolerates_prose() {
        let fact = fact_with_summary("Recurrence limits parallelism during training.");
        let prose = "Here are the claims:\n[{\"claim\": \"Recurrence limits parallelism.\", \"quote\": \"Recurrence limits parallelism during training\"}]\nThat's all.";
        let body = envelope(prose);
        let candidates = parse_claims(&body, &fact).unwrap();
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn test_parse_claims_empty_array() {
        let fact = fact_with_summary("An abstract with no extractable claims.");
        let candidates = parse_claims(&envelope("[]"), &fact).unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_parse_claims_malformed_errors() {
        let fact = fact_with_summary("Whatever.");
        // Not a JSON envelope at all.
        assert!(parse_claims("not json", &fact).is_err());
        // Envelope with no text block.
        let no_text = r#"{"content": [], "stop_reason": "end_turn"}"#;
        assert!(parse_claims(no_text, &fact).is_err());
        // Text block that is not a claim array.
        assert!(parse_claims(&envelope("the model rambled with no json"), &fact).is_err());
    }

    #[test]
    fn test_is_grounded_whitespace_and_case() {
        assert!(is_grounded(
            "Attention   Improves",
            "we show attention improves things"
        ));
        assert!(!is_grounded(
            "teleportation works",
            "a paper about attention"
        ));
        assert!(!is_grounded("", "any summary"));
    }

    #[test]
    fn test_build_missing_key_errors() {
        let err = LlmExtractor::build(None, None, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("ANTHROPIC_API_KEY"), "got: {err}");
        // A blank/whitespace key is treated as absent.
        let err_blank = LlmExtractor::build(Some("   ".to_string()), None, None)
            .unwrap_err()
            .to_string();
        assert!(err_blank.contains("ANTHROPIC_API_KEY"), "got: {err_blank}");
    }

    #[test]
    fn test_build_model_default_and_override() {
        let default = LlmExtractor::build(Some("sk-test".to_string()), None, None).unwrap();
        assert_eq!(default.model, "claude-opus-5");

        let overridden = LlmExtractor::build(
            Some("sk-test".to_string()),
            Some("claude-haiku-4-5".to_string()),
            None,
        )
        .unwrap();
        assert_eq!(overridden.model, "claude-haiku-4-5");

        // A blank model falls back to the default.
        let blank =
            LlmExtractor::build(Some("sk-test".to_string()), Some("  ".to_string()), None).unwrap();
        assert_eq!(blank.model, "claude-opus-5");
    }

    #[test]
    fn test_build_base_url_default_and_override() {
        // Unset/blank base URL falls back to the default endpoint root.
        let default = LlmExtractor::build(Some("sk-test".to_string()), None, None).unwrap();
        assert_eq!(default.base_url, "https://api.anthropic.com");
        let blank = LlmExtractor::build(Some("sk-test".to_string()), None, Some("   ".to_string()))
            .unwrap();
        assert_eq!(blank.base_url, "https://api.anthropic.com");

        // An explicit base URL (e.g. a mock server) is used verbatim.
        let overridden = LlmExtractor::build(
            Some("sk-test".to_string()),
            None,
            Some("http://127.0.0.1:9".to_string()),
        )
        .unwrap();
        assert_eq!(overridden.base_url, "http://127.0.0.1:9");
    }
}
