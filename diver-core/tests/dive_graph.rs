use diver_core::assertion::{Assertion, Candidate};
use diver_core::fact::SourceFact;
use diver_core::graph::{RelationKind, build_dive, compute_relations};
use diver_core::id::{ArxivCategory, ArxivId, ArxivVersion};
use diver_core::observation::Observation;
use diver_core::store::Store;

fn fact(id: &str, title: &str, category: &str, author: &str) -> SourceFact {
    let cat = ArxivCategory::parse(category).unwrap();
    SourceFact {
        arxiv_id: id.to_string(),
        title: title.to_string(),
        authors: vec![author.to_string()],
        summary: "A summary.".to_string(),
        primary_category: cat.clone(),
        categories: vec![cat],
        published: "2023-01-01T00:00:00Z".to_string(),
        updated: "2023-01-01T00:00:00Z".to_string(),
        pdf_url: format!("http://arxiv.org/pdf/{id}"),
        source_url: format!("https://export.arxiv.org/api/query?id_list={id}"),
        arxiv_version: "v1".to_string(),
        ingested_at: "2026-09-01T00:00:00Z".to_string(),
    }
}

/// Full dive loop: two papers sharing a category, one with a persisted assertion
/// about the concept → a one-node neighborhood linked to the other paper.
#[test]
fn test_dive_pipeline() {
    let store = Store::open_in_memory().unwrap();
    store
        .save(&fact("2301.00001", "Paper A", "cs.CL", "Alice"))
        .unwrap();
    store
        .save(&fact("2302.00002", "Paper B", "cs.CL", "Bob"))
        .unwrap();

    // Persist a supported assertion for Paper A that mentions the concept.
    let obs = Observation::new(
        ArxivId::new("2301.00001"),
        ArxivVersion(1),
        "attention improves accuracy",
    );
    let supported = Assertion::<Candidate>::new("Attention improves accuracy.", vec![obs])
        .validate()
        .unwrap();
    store
        .save_assertions("2301.00001", "v1", &[supported])
        .unwrap();

    // Seed → relations → neighborhood.
    let asserting = store.papers_asserting("attention").unwrap();
    assert_eq!(asserting.len(), 1);

    let facts = store.list().unwrap();
    let relations = compute_relations(&facts);
    let nodes = build_dive(&facts, &asserting, &relations);

    assert_eq!(nodes.len(), 1);
    let node = &nodes[0];
    assert_eq!(node.arxiv_id, "2301.00001");
    assert_eq!(node.title, "Paper A");
    assert_eq!(
        node.claims,
        vec!["Attention improves accuracy.".to_string()]
    );
    // Linked to Paper B by the shared cs.CL category (distinct authors → no author edge).
    assert_eq!(node.related.len(), 1);
    assert_eq!(node.related[0].0, "2302.00002");
    assert_eq!(
        node.related[0].1,
        RelationKind::SharedCategory("cs.CL".to_string())
    );
}
