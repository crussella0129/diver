use diver_core::assertion::{Assertion, Candidate, Supported};
use diver_core::fact::SourceFact;
use diver_core::graph::{RelationKind, build_dive, compute_coassertion_relations};
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

fn supported(id: &str, claim: &str) -> Assertion<Supported> {
    let obs = Observation::new(ArxivId::new(id), ArxivVersion(1), "quote");
    Assertion::<Candidate>::new(claim, vec![obs])
        .validate()
        .unwrap()
}

/// Two papers with distinct categories AND authors (so the only possible link is
/// epistemic) whose claims share the term "attention" must be linked by a
/// CoAssertion edge, and that edge must surface in the dive neighborhood.
#[test]
fn test_coassertion_pipeline() {
    let store = Store::open_in_memory().unwrap();
    store
        .save(&fact("2301.00001", "Paper A", "cs.CL", "Alice"))
        .unwrap();
    store
        .save(&fact("2302.00002", "Paper B", "math.NA", "Bob"))
        .unwrap();

    store
        .save_assertions(
            "2301.00001",
            "v1",
            &[supported("2301.00001", "Attention improves accuracy.")],
        )
        .unwrap();
    store
        .save_assertions(
            "2302.00002",
            "v1",
            &[supported("2302.00002", "Attention reduces cost.")],
        )
        .unwrap();

    // The only shared significant term is "attention". N == 2 → small-corpus
    // guard → weight 1.0 regardless of temperature.
    let relations = compute_coassertion_relations(&store.all_claims().unwrap(), 1.0);
    assert_eq!(relations.len(), 1);
    assert_eq!(
        relations[0].kind,
        RelationKind::CoAssertion {
            term: "attention".to_string(),
            weight: 1.0,
        }
    );

    // In a dive for "attention", Paper A's neighborhood lists Paper B via the edge.
    let facts = store.list().unwrap();
    let asserting = store.papers_asserting("attention").unwrap();
    let nodes = build_dive(&facts, &asserting, &relations);
    let node_a = nodes.iter().find(|n| n.arxiv_id == "2301.00001").unwrap();
    assert!(node_a.related.iter().any(|(id, kind)| {
        id == "2302.00002"
            && *kind
                == RelationKind::CoAssertion {
                    term: "attention".to_string(),
                    weight: 1.0,
                }
    }));
}
