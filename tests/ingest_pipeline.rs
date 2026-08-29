use diver::client::extract_paper;
use diver::fact::SourceFact;
use diver::parse;
use diver::store::Store;

#[test]
fn test_ingest_pipeline() {
    let xml = std::fs::read_to_string("tests/fixtures/sample_feed.xml").unwrap();
    let feed = parse::parse_feed(&xml).unwrap();

    let paper = extract_paper(feed).unwrap();
    assert_eq!(paper.title, "Attention Is All You Need Revisited");

    let fact = SourceFact::from_paper(
        paper,
        "https://export.arxiv.org/api/query?id_list=2301.00001".to_string(),
    );
    assert_eq!(fact.arxiv_id, "2301.00001");
    assert_eq!(fact.arxiv_version, "v1");

    let store = Store::open_in_memory().unwrap();
    store.save(&fact).unwrap();

    let retrieved = store.get("2301.00001").unwrap().unwrap();
    assert_eq!(retrieved.title, "Attention Is All You Need Revisited");
    assert_eq!(retrieved.authors, vec!["Alice Smith", "Bob Jones"]);
    assert_eq!(retrieved.primary_category.code(), "cs.CL");
    assert_eq!(
        retrieved.source_url,
        "https://export.arxiv.org/api/query?id_list=2301.00001"
    );
    assert_eq!(retrieved.arxiv_version, "v1");
    assert!(!retrieved.ingested_at.is_empty());
}

#[test]
fn test_ingest_pipeline_multi_category() {
    // The sample_feed.xml first entry has cs.CL + cs.AI categories
    let xml = std::fs::read_to_string("tests/fixtures/sample_feed.xml").unwrap();
    let feed = parse::parse_feed(&xml).unwrap();

    let paper = extract_paper(feed).unwrap();
    // Parser should have collected both cs.CL and cs.AI
    assert!(paper.categories.contains(&"cs.CL".to_string()));
    assert!(paper.categories.contains(&"cs.AI".to_string()));
    assert_eq!(paper.categories.len(), 2);
}
