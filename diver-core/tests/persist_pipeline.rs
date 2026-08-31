use diver_core::extract::parse_claims;
use diver_core::fact::SourceFact;
use diver_core::id::ArxivCategory;
use diver_core::store::Store;

fn sample_fact() -> SourceFact {
    let primary = ArxivCategory::parse("cs.CL").unwrap();
    SourceFact {
        arxiv_id: "2301.00001".to_string(),
        title: "Attention Is All You Need".to_string(),
        authors: vec!["Alice".to_string()],
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

/// Full persist loop: ingest a paper, extract a grounded claim from a fixture
/// Messages-API body, validate it, persist it, and read it back.
#[test]
fn test_persist_pipeline() {
    let store = Store::open_in_memory().unwrap();
    let fact = sample_fact();
    store.save(&fact).unwrap();

    let body = r#"{"content": [{"type": "text", "text": "[{\"claim\": \"Attention improves accuracy.\", \"quote\": \"attention improves accuracy\"}]"}]}"#;
    let candidates = parse_claims(body, &fact).unwrap();
    let supported: Vec<_> = candidates
        .into_iter()
        .filter_map(|candidate| candidate.validate().ok())
        .collect();
    assert_eq!(supported.len(), 1);

    store
        .save_assertions(&fact.arxiv_id, &fact.arxiv_version, &supported)
        .unwrap();

    let stored = store.get_assertions(&fact.arxiv_id).unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].claim, "Attention improves accuracy.");
    assert_eq!(stored[0].version, "v2");
    assert_eq!(
        stored[0].support,
        vec!["attention improves accuracy".to_string()]
    );
}
