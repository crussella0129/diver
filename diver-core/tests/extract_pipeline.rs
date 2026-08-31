use diver_core::assertion::{Assertion, Supported, candidate_assertions};
use diver_core::fact::SourceFact;
use diver_core::id::{ArxivCategory, ArxivVersion};
use diver_core::observation::extract_observations;

fn sample_fact() -> SourceFact {
    let primary = ArxivCategory::parse("cs.CL").unwrap();
    SourceFact {
        arxiv_id: "2301.00001".to_string(),
        title: "Attention Is All You Need".to_string(),
        authors: vec!["Alice".to_string()],
        summary: "Attention improves translation accuracy. Recurrence limits parallelism though. \
                  Transformers scale better overall."
            .to_string(),
        primary_category: primary.clone(),
        categories: vec![primary],
        published: "2023-01-01T00:00:00Z".to_string(),
        updated: "2023-01-01T00:00:00Z".to_string(),
        pdf_url: "http://arxiv.org/pdf/2301.00001".to_string(),
        source_url: "https://export.arxiv.org/api/query?id_list=2301.00001".to_string(),
        arxiv_version: "v2".to_string(),
        ingested_at: "2026-08-30T00:00:00Z".to_string(),
    }
}

#[test]
fn test_extract_pipeline() {
    let fact = sample_fact();

    // SourceFact -> Observations (one per non-trivial sentence).
    let observations = extract_observations(&fact);
    assert_eq!(
        observations.len(),
        3,
        "three sentences -> three observations"
    );

    // Observations -> candidate assertions.
    let candidates = candidate_assertions(&observations);
    assert_eq!(candidates.len(), 3);

    // Candidates -> supported assertions through the validation gate.
    let supported: Vec<Assertion<Supported>> = candidates
        .into_iter()
        .filter_map(|candidate| candidate.validate().ok())
        .collect();
    assert_eq!(
        supported.len(),
        3,
        "each candidate has support and validates"
    );

    // Provenance survives from the source fact through to the supported assertion.
    let first = &supported[0];
    assert_eq!(first.claim(), "Attention improves translation accuracy.");
    let obs = &first.support()[0];
    assert_eq!(obs.arxiv_id().as_str(), "2301.00001");
    assert_eq!(*obs.version(), ArxivVersion(2));
}
