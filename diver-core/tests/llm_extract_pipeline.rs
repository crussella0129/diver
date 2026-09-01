use diver_core::assertion::{Assertion, Supported};
use diver_core::extract::parse_claims;
use diver_core::fact::SourceFact;
use diver_core::id::{ArxivCategory, ArxivVersion};

fn sample_fact() -> SourceFact {
    let primary = ArxivCategory::parse("cs.CL").unwrap();
    SourceFact {
        arxiv_id: "2301.00001".to_string(),
        title: "Attention Is All You Need".to_string(),
        authors: vec!["Alice".to_string()],
        // Grounds "attention improves accuracy"; does NOT contain "teleports data".
        summary: "In this work, attention improves accuracy on the benchmark.".to_string(),
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

/// A structured (tool-use) response whose model output claims two things: one
/// grounded in the abstract, one hallucinated. The full LLM pipeline must admit only
/// the grounded one and validate it to `Supported`.
#[test]
fn test_llm_extract_pipeline() {
    let fact = sample_fact();
    // Anthropic tool_use envelope: input is the structured { claims: [...] }.
    let body = r#"{
        "content": [
            {"type": "tool_use", "id": "tu_1", "name": "record_claims", "input": {"claims": [
                {"claim": "Attention improves accuracy.", "quote": "attention improves accuracy"},
                {"claim": "The model teleports data.", "quote": "teleports data instantly across the globe"}
            ]}}
        ],
        "stop_reason": "tool_use"
    }"#;

    // parse_claims grounds the claims: only the first survives.
    let candidates = parse_claims(body, &fact).unwrap();
    assert_eq!(candidates.len(), 1, "hallucinated claim must be dropped");

    // The grounded candidate passes the existing validation gate.
    let supported: Vec<Assertion<Supported>> = candidates
        .into_iter()
        .filter_map(|candidate| candidate.validate().ok())
        .collect();
    assert_eq!(supported.len(), 1);

    let assertion = &supported[0];
    assert_eq!(assertion.claim(), "Attention improves accuracy.");
    let obs = &assertion.support()[0];
    assert_eq!(obs.arxiv_id().as_str(), "2301.00001");
    assert_eq!(*obs.version(), ArxivVersion(2));
}
