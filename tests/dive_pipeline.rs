use diver::fact::SourceFact;
use diver::store::Store;

fn make_fact(id: &str, title: &str, summary: &str) -> SourceFact {
    SourceFact {
        arxiv_id: id.to_string(),
        title: title.to_string(),
        authors: vec!["Alice".to_string(), "Bob".to_string()],
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
fn test_dive_pipeline() {
    let store = Store::open_in_memory().unwrap();

    store
        .save(&make_fact(
            "2301.00001",
            "Attention Is All You Need",
            "We propose a new architecture based on attention mechanisms for sequence transduction.",
        ))
        .unwrap();
    store
        .save(&make_fact(
            "2302.00002",
            "BERT: Pre-training of Deep Bidirectional Transformers",
            "We introduce a new language representation model called BERT.",
        ))
        .unwrap();
    store
        .save(&make_fact(
            "2303.00003",
            "Recurrent Neural Networks for Sequence Modeling",
            "We study recurrent attention-based approaches for sequence modeling.",
        ))
        .unwrap();

    let results = store.search("attention", 10).unwrap();
    assert!(!results.is_empty());

    let ids: Vec<&str> = results.iter().map(|r| r.arxiv_id.as_str()).collect();
    assert!(ids.contains(&"2301.00001"));
    assert!(ids.contains(&"2303.00003"));

    let limited = store.search("attention", 1).unwrap();
    assert_eq!(limited.len(), 1);

    let empty = store.search("xyznonexistent", 10).unwrap();
    assert!(empty.is_empty());
}
