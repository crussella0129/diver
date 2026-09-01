//! Agent-agnostic LLM-backed claim extraction.
//!
//! [`LlmExtractor`] asks a model to read a paper's abstract and return the factual
//! claims it makes, each with a supporting quote, as **structured output**. It is
//! provider-agnostic: a [`ProviderConfig`] selects one of two compiled request/response
//! [`ProviderShape`]s — Anthropic (Messages API tool-use) or OpenAI-compatible (Chat
//! Completions `response_format` json_schema, which also covers Grok and local
//! llama.cpp/Ollama/vLLM servers such as Animus_Ferric). Both shapes yield the same
//! `{ claim, quote }` objects, which are then **grounded** (a claim becomes a candidate
//! only if its quote is present in the abstract, so hallucinated claims never enter the
//! pipeline) and passed through the existing [`Assertion::<Candidate>::validate`] gate.

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::assertion::{Assertion, Candidate};
use crate::fact::SourceFact;
use crate::id::{ArxivId, ArxivVersion};
use crate::observation::Observation;

/// Default model when `DIVER_MODEL` is unset (Anthropic env fallback).
const DEFAULT_MODEL: &str = "claude-opus-5";

/// Default Anthropic API root for the env fallback.
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Name of the structured-output tool / schema the model fills in.
const CLAIMS_TOOL: &str = "record_claims";

/// Short instruction; the JSON schema carries the structure.
const SYSTEM_PROMPT: &str = "Extract the discrete factual claims a research paper's \
abstract asserts. For each, give a self-contained claim in your own words and a quote \
copied verbatim from the abstract that supports it. Do not invent claims the abstract \
does not support.";

/// A compiled provider request/response contract. Runtime config picks one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderShape {
    /// Anthropic Messages API: forced `record_claims` tool → `tool_use` block.
    Anthropic,
    /// OpenAI-compatible Chat Completions: `response_format` json_schema →
    /// `choices[0].message.content`. Covers OpenAI, Grok, and local servers
    /// (llama.cpp/Ollama/vLLM, e.g. Animus_Ferric via `ferric server`).
    OpenAiCompatible,
}

/// A fully-resolved provider. Built by a front-end (hot-loadable) via
/// [`LlmExtractor::from_config`], or resolved from env/config by
/// [`LlmExtractor::from_env`].
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub shape: ProviderShape,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

/// Extracts grounded candidate assertions from a paper's abstract by asking a model,
/// over a configurable provider.
pub struct LlmExtractor {
    http: reqwest::Client,
    config: ProviderConfig,
}

// Manual Debug that redacts the API key — never print the secret.
impl std::fmt::Debug for LlmExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmExtractor")
            .field("shape", &self.config.shape)
            .field("base_url", &self.config.base_url)
            .field("model", &self.config.model)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

impl LlmExtractor {
    /// Build from an explicit, fully-resolved [`ProviderConfig`] — the hot-loadable
    /// seam a front-end uses to switch providers without a rebuild.
    pub fn from_config(config: ProviderConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self { http, config })
    }

    /// Resolve a provider from the environment. For now this is the Anthropic env
    /// fallback (`ANTHROPIC_API_KEY` / `DIVER_MODEL` / `ANTHROPIC_BASE_URL`); the
    /// hot-loadable providers-config resolution layers in front of this (INT-0016 T-1502).
    pub fn from_env() -> Result<Self> {
        let config = anthropic_config_from_env(
            std::env::var("ANTHROPIC_API_KEY").ok(),
            std::env::var("DIVER_MODEL").ok(),
            std::env::var("ANTHROPIC_BASE_URL").ok(),
        )?;
        Self::from_config(config)
    }

    /// Ask the configured model to extract grounded candidate assertions from `fact`.
    pub async fn extract(&self, fact: &SourceFact) -> Result<Vec<Assertion<Candidate>>> {
        let user_content = format!("Abstract of \"{}\":\n\n{}", fact.title, fact.summary);

        let (url, body) = match self.config.shape {
            ProviderShape::Anthropic => (
                format!("{}/v1/messages", self.config.base_url),
                self.anthropic_body(&user_content),
            ),
            ProviderShape::OpenAiCompatible => (
                format!("{}/v1/chat/completions", self.config.base_url),
                self.openai_body(&user_content),
            ),
        };

        let request = self
            .http
            .post(url)
            .header("content-type", "application/json")
            .body(body);
        let request = match self.config.shape {
            ProviderShape::Anthropic => request
                .header("x-api-key", &self.config.api_key)
                .header("anthropic-version", "2023-06-01"),
            ProviderShape::OpenAiCompatible => {
                request.header("authorization", format!("Bearer {}", self.config.api_key))
            }
        };

        let response = request
            .send()
            .await
            .context("failed to reach the provider API")?;

        let status = response.status();
        let text = response
            .text()
            .await
            .context("failed to read provider API response body")?;
        if !status.is_success() {
            anyhow::bail!("provider API returned HTTP {status}: {text}");
        }

        let claims = match self.config.shape {
            ProviderShape::Anthropic => parse_anthropic_claims(&text)?,
            ProviderShape::OpenAiCompatible => parse_openai_claims(&text)?,
        };
        Ok(ground_claims(claims, fact))
    }

    /// Anthropic Messages request body: a forced `record_claims` tool.
    fn anthropic_body(&self, user_content: &str) -> String {
        serde_json::json!({
            "model": self.config.model,
            "max_tokens": 2048,
            "system": SYSTEM_PROMPT,
            "messages": [{ "role": "user", "content": user_content }],
            "tools": [{
                "name": CLAIMS_TOOL,
                "description": "Record the discrete factual claims the abstract asserts.",
                "input_schema": claims_schema(),
            }],
            "tool_choice": { "type": "tool", "name": CLAIMS_TOOL },
        })
        .to_string()
    }

    /// OpenAI-compatible Chat Completions body: a strict json_schema `response_format`.
    fn openai_body(&self, user_content: &str) -> String {
        serde_json::json!({
            "model": self.config.model,
            "max_tokens": 2048,
            "messages": [
                { "role": "system", "content": SYSTEM_PROMPT },
                { "role": "user", "content": user_content },
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": { "name": "claims", "strict": true, "schema": claims_schema() },
            },
        })
        .to_string()
    }
}

/// Validate + default the Anthropic env inputs into a [`ProviderConfig`]. A missing or
/// blank key is an actionable error; blank model/base URL fall back to defaults.
fn anthropic_config_from_env(
    api_key: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
) -> Result<ProviderConfig> {
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
    Ok(ProviderConfig {
        shape: ProviderShape::Anthropic,
        base_url,
        model,
        api_key,
    })
}

/// The shared JSON schema for the structured claims payload (used as the Anthropic
/// tool `input_schema` and the OpenAI `response_format` schema). `strict`-compatible:
/// closed objects with every property required.
fn claims_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["claims"],
        "properties": {
            "claims": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["claim", "quote"],
                    "properties": {
                        "claim": { "type": "string", "description": "one self-contained factual statement, in your own words" },
                        "quote": { "type": "string", "description": "a span copied verbatim from the abstract that supports the claim" }
                    }
                }
            }
        }
    })
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    content: Vec<AnthropicBlock>,
}

#[derive(Debug, Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    input: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    #[serde(default)]
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    #[serde(default)]
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaimsInput {
    claims: Vec<ClaimJson>,
}

#[derive(Debug, Deserialize)]
struct ClaimJson {
    claim: String,
    quote: String,
}

/// Parse an Anthropic Messages response into claims: read the `tool_use` block's
/// `input` (the structured `{ claims: [...] }`). `Err` — never panics — on an
/// unparseable envelope, a missing `tool_use` block, or input that fails the schema.
///
/// Kept `pub` (its historical name) so callers/tests can parse an Anthropic envelope
/// without a live call; [`LlmExtractor::extract`] uses it plus [`ground_claims`].
pub fn parse_claims(response_body: &str, fact: &SourceFact) -> Result<Vec<Assertion<Candidate>>> {
    let claims = parse_anthropic_claims(response_body)?;
    Ok(ground_claims(claims, fact))
}

fn parse_anthropic_claims(response_body: &str) -> Result<Vec<ClaimJson>> {
    let response: AnthropicResponse = serde_json::from_str(response_body)
        .context("failed to parse Messages API response envelope")?;
    let input = response
        .content
        .into_iter()
        .find(|b| b.kind == "tool_use")
        .and_then(|b| b.input)
        .context("Messages API response had no tool_use block")?;
    let parsed: ClaimsInput =
        serde_json::from_value(input).context("tool_use input did not match the claims schema")?;
    Ok(parsed.claims)
}

fn parse_openai_claims(response_body: &str) -> Result<Vec<ClaimJson>> {
    let response: OpenAiResponse = serde_json::from_str(response_body)
        .context("failed to parse chat completions response envelope")?;
    let content = response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .context("chat completions response had no choices")?;
    let parsed: ClaimsInput = serde_json::from_str(content.trim())
        .context("message content was not the structured claims JSON")?;
    Ok(parsed.claims)
}

/// Ground each claim against the abstract and lift the survivors to candidates.
fn ground_claims(claims: Vec<ClaimJson>, fact: &SourceFact) -> Vec<Assertion<Candidate>> {
    let arxiv_id = ArxivId::new(fact.arxiv_id.clone());
    let version = ArxivVersion::parse(&fact.arxiv_version);
    claims
        .into_iter()
        .filter(|c| is_grounded(&c.quote, &fact.summary))
        .map(|c| {
            let obs = Observation::new(arxiv_id.clone(), version.clone(), c.quote);
            Assertion::<Candidate>::new(c.claim, vec![obs])
        })
        .collect()
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
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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

    /// Wrap a claims array (JSON `Value`) in an Anthropic `tool_use` response envelope.
    fn tool_use_envelope(claims: serde_json::Value) -> String {
        serde_json::json!({
            "content": [{
                "type": "tool_use",
                "id": "tu_1",
                "name": "record_claims",
                "input": { "claims": claims }
            }],
            "stop_reason": "tool_use"
        })
        .to_string()
    }

    /// Wrap a claims array in an OpenAI Chat Completions structured-output envelope
    /// (`message.content` is the JSON text of `{ "claims": [...] }`).
    fn openai_envelope(claims: serde_json::Value) -> String {
        let content = serde_json::json!({ "claims": claims }).to_string();
        serde_json::json!({
            "choices": [{ "message": { "role": "assistant", "content": content } }]
        })
        .to_string()
    }

    #[test]
    fn test_parse_claims_grounded() {
        let fact = fact_with_summary(
            "We show that attention improves translation accuracy by five points.",
        );
        let body = tool_use_envelope(serde_json::json!([
            { "claim": "Attention improves translation accuracy.", "quote": "attention improves translation accuracy by five points" }
        ]));
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
        let body = tool_use_envelope(serde_json::json!([
            { "claim": "Attention is studied.", "quote": "We study attention mechanisms" },
            { "claim": "The model cures cancer.", "quote": "our model cures cancer completely" }
        ]));
        let candidates = parse_claims(&body, &fact).unwrap();
        // Only the grounded claim survives; the fabricated quote is dropped.
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].claim(), "Attention is studied.");
    }

    #[test]
    fn test_parse_claims_empty() {
        let fact = fact_with_summary("An abstract with no extractable claims.");
        let candidates = parse_claims(&tool_use_envelope(serde_json::json!([])), &fact).unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_parse_claims_requires_structured() {
        let fact = fact_with_summary("Whatever.");
        // Not a JSON envelope at all.
        assert!(parse_claims("not json", &fact).is_err());
        // Envelope with only a text block (no tool_use) — a contract violation now.
        let text_only = r#"{"content":[{"type":"text","text":"[{\"claim\":\"x\",\"quote\":\"x\"}]"}],"stop_reason":"end_turn"}"#;
        assert!(parse_claims(text_only, &fact).is_err());
        // tool_use block whose input does not match the claims schema.
        let bad_input = r#"{"content":[{"type":"tool_use","id":"tu_1","name":"record_claims","input":{"nope":1}}]}"#;
        assert!(parse_claims(bad_input, &fact).is_err());
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
    fn test_anthropic_config_from_env() {
        // Missing/blank key errors actionably.
        let err = anthropic_config_from_env(None, None, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("ANTHROPIC_API_KEY"), "got: {err}");
        assert!(
            anthropic_config_from_env(Some("   ".to_string()), None, None).is_err(),
            "blank key treated as absent"
        );

        // Defaults + overrides.
        let default = anthropic_config_from_env(Some("sk-test".to_string()), None, None).unwrap();
        assert_eq!(default.shape, ProviderShape::Anthropic);
        assert_eq!(default.model, "claude-opus-5");
        assert_eq!(default.base_url, "https://api.anthropic.com");

        let overridden = anthropic_config_from_env(
            Some("sk-test".to_string()),
            Some("claude-haiku-4-5".to_string()),
            Some("http://127.0.0.1:9".to_string()),
        )
        .unwrap();
        assert_eq!(overridden.model, "claude-haiku-4-5");
        assert_eq!(overridden.base_url, "http://127.0.0.1:9");

        // Blank model/base URL fall back to defaults.
        let blank = anthropic_config_from_env(
            Some("sk-test".to_string()),
            Some("  ".to_string()),
            Some("  ".to_string()),
        )
        .unwrap();
        assert_eq!(blank.model, "claude-opus-5");
        assert_eq!(blank.base_url, "https://api.anthropic.com");
    }

    fn config(shape: ProviderShape, base_url: String) -> ProviderConfig {
        ProviderConfig {
            shape,
            base_url,
            model: "test-model".to_string(),
            api_key: "sk-test".to_string(),
        }
    }

    /// Anthropic shape: real round-trip against a mock returning a tool_use envelope;
    /// the request declares the forced `record_claims` tool with the Anthropic headers.
    #[tokio::test]
    async fn test_extract_anthropic_tool_use() {
        let server = MockServer::start().await;
        let body = tool_use_envelope(serde_json::json!([
            { "claim": "Attention improves accuracy.", "quote": "attention improves accuracy" },
            { "claim": "It teleports data.", "quote": "teleports data across the globe" }
        ]));
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("x-api-key", "sk-test"))
            .and(header("anthropic-version", "2023-06-01"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .expect(1)
            .mount(&server)
            .await;

        let fact = fact_with_summary("In this work, attention improves accuracy on the benchmark.");
        let extractor =
            LlmExtractor::from_config(config(ProviderShape::Anthropic, server.uri())).unwrap();
        let candidates = extractor.extract(&fact).await.unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].claim(), "Attention improves accuracy.");

        let sent = server.received_requests().await.unwrap();
        let body = String::from_utf8_lossy(&sent[0].body);
        assert!(body.contains("record_claims"), "tool in body: {body}");
        assert!(body.contains("tool_choice"), "forced tool: {body}");
        assert!(body.contains("test-model"), "model in body: {body}");
    }

    /// OpenAI-compatible shape (also the Animus_Ferric / Grok / local-server path):
    /// mock returns structured `message.content`; request carries Bearer + response_format.
    #[tokio::test]
    async fn test_extract_openai_structured() {
        let server = MockServer::start().await;
        let body = openai_envelope(serde_json::json!([
            { "claim": "Attention improves accuracy.", "quote": "attention improves accuracy" },
            { "claim": "It teleports data.", "quote": "teleports data across the globe" }
        ]));
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("authorization", "Bearer sk-test"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .expect(1)
            .mount(&server)
            .await;

        let fact = fact_with_summary("In this work, attention improves accuracy on the benchmark.");
        let extractor =
            LlmExtractor::from_config(config(ProviderShape::OpenAiCompatible, server.uri()))
                .unwrap();
        let candidates = extractor.extract(&fact).await.unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].claim(), "Attention improves accuracy.");

        let sent = server.received_requests().await.unwrap();
        let body = String::from_utf8_lossy(&sent[0].body);
        assert!(body.contains("response_format"), "response_format: {body}");
        assert!(body.contains("json_schema"), "json_schema: {body}");
        assert!(body.contains("test-model"), "model in body: {body}");
    }

    #[tokio::test]
    async fn test_extract_anthropic_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(500).set_body_string("upstream boom"))
            .mount(&server)
            .await;
        let extractor =
            LlmExtractor::from_config(config(ProviderShape::Anthropic, server.uri())).unwrap();
        let err = extractor
            .extract(&fact_with_summary("x"))
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("500"), "status: {err}");
        assert!(err.contains("upstream boom"), "body: {err}");
    }

    #[tokio::test]
    async fn test_extract_openai_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("bad key"))
            .mount(&server)
            .await;
        let extractor =
            LlmExtractor::from_config(config(ProviderShape::OpenAiCompatible, server.uri()))
                .unwrap();
        let err = extractor
            .extract(&fact_with_summary("x"))
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("401"), "status: {err}");
        assert!(err.contains("bad key"), "body: {err}");
    }
}
