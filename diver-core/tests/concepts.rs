//! Concept identity end to end through the library: persistence on a real on-disk
//! store, freshness across a reopen, concept-keyed co-assertion, and the checked-in
//! real-corpus fixture (offline).

use std::collections::BTreeSet;

use diver_core::assertion::{Assertion, Candidate, Supported, candidate_assertions};
use diver_core::concept::{ConceptKind, TokenCategory, category, form_concepts};
use diver_core::fact::SourceFact;
use diver_core::graph::{RelationKind, build_dive, compute_coassertion_relations};
use diver_core::id::{ArxivId, ArxivVersion};
use diver_core::observation::{Observation, extract_observations};
use diver_core::parse;
use diver_core::store::Store;

fn supported(paper: &str, claim: &str) -> Assertion<Supported> {
    let obs = Observation::new(ArxivId::new(paper), ArxivVersion(1), claim.to_lowercase());
    Assertion::<Candidate>::new(claim, vec![obs])
        .validate()
        .expect("non-empty support validates")
}

#[test]
fn test_concepts_pipeline() {
    let scratch = tempfile::tempdir().unwrap();
    let db = scratch.path().join("corpus.db");

    {
        let store = Store::open_at(&db).unwrap();
        store
            .save_assertions(
                "2301.00001",
                "v1",
                &[supported("2301.00001", "Neural networks learn.")],
            )
            .unwrap();
        store
            .save_assertions(
                "2302.00002",
                "v1",
                &[supported("2302.00002", "A network generalizes.")],
            )
            .unwrap();

        // Either inflection finds both papers.
        let asserting = store.papers_asserting("networks").unwrap();
        let papers: BTreeSet<&str> = asserting.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(papers, BTreeSet::from(["2301.00001", "2302.00002"]));

        // One concept-keyed edge links them, and the dive lists each as related.
        let rels = compute_coassertion_relations(&store.all_claims().unwrap(), 1.0);
        let concept_edges: Vec<_> = rels
            .iter()
            .filter(|r| matches!(r.kind, RelationKind::CoAssertion { .. }))
            .collect();
        assert_eq!(concept_edges.len(), 1, "{concept_edges:?}");
        let nodes = build_dive(&[], &asserting, &rels);
        assert!(nodes.iter().all(|n| n.related.len() == 1));
    }

    // Reopen the same file: concepts persist, and a later write is visible with no
    // rebuild call.
    let store = Store::open_at(&db).unwrap();
    assert_eq!(store.papers_asserting("network").unwrap().len(), 2);
    store
        .save_assertions(
            "2303.00003",
            "v1",
            &[supported("2303.00003", "Networks converge.")],
        )
        .unwrap();
    assert_eq!(
        store
            .resolve_concept("network")
            .unwrap()
            .unwrap()
            .paper_count,
        3
    );
}

#[test]
fn test_real_corpus_concepts() {
    let xml = std::fs::read_to_string("tests/fixtures/real_corpus_feed.xml")
        .expect("real_corpus_feed.xml fixture is present");
    let feed = parse::parse_feed(&xml).expect("parse the real arXiv feed");
    let store = Store::open_in_memory().unwrap();
    for paper in feed.papers {
        let fact = SourceFact::from_paper(paper, "fixture".to_string());
        store.save(&fact).unwrap();
        let supported: Vec<_> = candidate_assertions(&extract_observations(&fact))
            .into_iter()
            .filter_map(|c| c.validate().ok())
            .collect();
        store
            .save_assertions(&fact.arxiv_id, &fact.arxiv_version, &supported)
            .unwrap();
    }

    let mt = store
        .resolve_concept("machine translation")
        .unwrap()
        .unwrap();
    assert_eq!(mt.kind, ConceptKind::Phrase);
    assert!(mt.paper_count >= 2);

    let network = store.resolve_concept("network").unwrap().unwrap();
    let forms: Vec<&str> = network.forms.iter().map(|(f, _)| f.as_str()).collect();
    assert!(
        forms.contains(&"network") && forms.contains(&"networks"),
        "{forms:?}"
    );

    // A claim substring matching on `networks` would miss: it only says `network`.
    let hits = store.papers_asserting("networks").unwrap();
    assert!(hits.iter().any(|(_, c)| {
        c.starts_with("We propose a new simple network architecture") && !c.contains("networks")
    }));

    // Invariants on real claims: phrase papers ⊆ term papers; order-independent.
    let claims = store.all_claims().unwrap();
    let set = form_concepts(&claims);
    for phrase in set.iter().filter(|c| c.kind == ConceptKind::Phrase) {
        for word in phrase.words() {
            if category(word) == TokenCategory::Content {
                assert!(phrase.papers.is_subset(&set.get(word).unwrap().papers));
            }
        }
    }
    let n = claims.len();
    let reversed: Vec<_> = claims.iter().rev().cloned().collect();
    let mut rset = form_concepts(&reversed);
    for c in rset.concepts.values_mut() {
        c.claims = c.claims.iter().map(|i| n - 1 - i).collect();
    }
    assert_eq!(rset, set);
}
