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

/// A 4-paper corpus where "attention" is rare (df 2, weight 1.0) and "models" is
/// corpus-ubiquitous (df 4, weight 0.0). Low temperature keeps only the rare edge;
/// high temperature admits the common one too.
#[test]
fn test_coassertion_temperature_pipeline() {
    let store = Store::open_in_memory().unwrap();
    for (id, cat, author) in [
        ("2301.00001", "cs.CL", "Alice"),
        ("2302.00002", "math.NA", "Bob"),
        ("2303.00003", "cs.LG", "Carol"),
        ("2304.00004", "stat.ML", "Dave"),
    ] {
        store.save(&fact(id, "Paper", cat, author)).unwrap();
    }
    for (id, claim) in [
        ("2301.00001", "attention drives models"),
        ("2302.00002", "attention shapes models"),
        ("2303.00003", "recurrence bounds models"),
        ("2304.00004", "convolution stacks models"),
    ] {
        store
            .save_assertions(id, "v1", &[supported(id, claim)])
            .unwrap();
    }
    let corpus = store.all_claims().unwrap();

    // Low temperature: only the distinctive term "attention" (df 2) links, and only
    // the one pair that shares it. The ubiquitous "models" (df 4) is filtered out.
    let cold = compute_coassertion_relations(&corpus, 0.0);
    assert_eq!(cold.len(), 1, "only the rare-term edge survives t=0.0");
    assert_eq!(cold[0].from, "2301.00001");
    assert_eq!(cold[0].to, "2302.00002");
    match &cold[0].kind {
        RelationKind::CoAssertion { term, weight } => {
            assert_eq!(term, "attention");
            assert_eq!(*weight, 1.0);
        }
        other => panic!("expected CoAssertion, got {other:?}"),
    }

    // High temperature admits the common term, so strictly more edges appear,
    // including "models" edges the low-temperature run dropped.
    let hot = compute_coassertion_relations(&corpus, 1.0);
    assert!(
        hot.len() > cold.len(),
        "higher temperature admits common-term edges"
    );
    assert!(
        hot.iter().any(|r| matches!(
            &r.kind,
            RelationKind::CoAssertion { term, .. } if term == "models"
        )),
        "the ubiquitous term links papers only at high temperature"
    );

    // The distinctive edge still surfaces in the dive neighborhood.
    let facts = store.list().unwrap();
    let asserting = store.papers_asserting("attention").unwrap();
    let nodes = build_dive(&facts, &asserting, &hot);
    let node_a = nodes.iter().find(|n| n.arxiv_id == "2301.00001").unwrap();
    assert!(node_a.related.iter().any(|(id, kind)| {
        id == "2302.00002"
            && matches!(kind, RelationKind::CoAssertion { term, .. } if term == "attention")
    }));
}
