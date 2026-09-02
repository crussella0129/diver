//! End-to-end validation over a REAL captured arXiv feed, fully offline.
//!
//! Ingest (`parse_feed` → `save`) → deterministic extract (sentence-splitter →
//! `validate` → `save_assertions`) → weighted dive graph. Proves the whole pipeline on
//! real content and guards it against regressions. The fixture
//! `tests/fixtures/real_corpus_feed.xml` holds seven genuine attention/NMT arXiv papers
//! (ingested from the live API, then re-serialized into the Atom feed shape); the test
//! itself makes no network call.

use std::collections::HashSet;

use diver_core::assertion::candidate_assertions;
use diver_core::fact::SourceFact;
use diver_core::graph::{
    ComputedRelation, RelationKind, build_dive, compute_coassertion_relations, compute_relations,
};
use diver_core::observation::extract_observations;
use diver_core::parse;
use diver_core::store::Store;

fn coassertion_triples(rels: &[ComputedRelation]) -> HashSet<(String, String, String)> {
    rels.iter()
        .filter_map(|r| match &r.kind {
            RelationKind::CoAssertion { term, .. } => {
                Some((r.from.clone(), r.to.clone(), term.clone()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn test_real_corpus_dive() {
    let xml = std::fs::read_to_string("tests/fixtures/real_corpus_feed.xml")
        .expect("real_corpus_feed.xml fixture is present");
    let feed = parse::parse_feed(&xml).expect("parse the real arXiv feed");
    assert!(
        feed.papers.len() >= 2,
        "the real corpus has multiple papers (got {})",
        feed.papers.len()
    );

    // Ingest each paper and extract deterministically (offline, no API key).
    let store = Store::open_in_memory().unwrap();
    for paper in feed.papers {
        let fact = SourceFact::from_paper(
            paper,
            "https://export.arxiv.org/api/query?id_list=corpus".to_string(),
        );
        store.save(&fact).unwrap();

        let candidates = candidate_assertions(&extract_observations(&fact));
        let supported: Vec<_> = candidates
            .into_iter()
            .filter_map(|c| c.validate().ok())
            .collect();
        assert!(
            !supported.is_empty(),
            "real paper {} produced at least one grounded assertion",
            fact.arxiv_id
        );
        store
            .save_assertions(&fact.arxiv_id, &fact.arxiv_version, &supported)
            .unwrap();
    }

    let facts = store.list().unwrap();
    let all_claims = store.all_claims().unwrap();
    assert!(
        all_claims.len() >= facts.len(),
        "every paper contributed claims"
    );

    // Structural edges: real same-topic papers share a category (or author).
    let structural = compute_relations(&facts);
    assert!(
        structural.iter().any(|r| matches!(
            r.kind,
            RelationKind::SharedCategory(_) | RelationKind::SharedAuthor(_)
        )),
        "the real corpus has at least one structural (category/author) edge"
    );

    // Co-assertion existence at temperature 1.0 — guaranteed for a same-topic corpus:
    // at least one weighted edge between two distinct real papers, weights in [0.0, 1.0].
    let hot = compute_coassertion_relations(&all_claims, 1.0);
    let mut coassertion_edges = 0;
    for r in &hot {
        if let RelationKind::CoAssertion { weight, .. } = &r.kind {
            coassertion_edges += 1;
            assert_ne!(
                r.from, r.to,
                "co-assertion edges are between distinct papers"
            );
            assert!(
                weight.is_finite() && (0.0..=1.0).contains(weight),
                "weight in range, got {weight}"
            );
        }
    }
    assert!(
        coassertion_edges > 0,
        "the real corpus has at least one weighted co-assertion edge at t=1.0"
    );

    // Weighting is monotonic on real data: the t=0.5 edge set is a subset of the t=1.0 set.
    let warm = compute_coassertion_relations(&all_claims, 0.5);
    assert!(
        coassertion_triples(&warm).is_subset(&coassertion_triples(&hot)),
        "co-assertion edges grow monotonically with temperature on real data"
    );

    // A dive over a real seed lists a related paper (this corpus is about attention).
    let asserting = store.papers_asserting("attention").unwrap();
    if !asserting.is_empty() {
        let mut relations = structural.clone();
        relations.extend(compute_coassertion_relations(&all_claims, 0.5));
        let nodes = build_dive(&facts, &asserting, &relations);
        assert!(
            nodes.iter().any(|n| !n.related.is_empty()),
            "a dive seed node lists at least one related paper"
        );
    }
}
