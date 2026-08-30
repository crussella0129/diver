//! `Observation` — a deterministically-extracted, provenance-carrying unit of
//! what a source paper said. Observations are the raw material the assertion
//! layer ([`crate::assertion`]) builds candidates from. Extraction here is
//! deterministic and LLM-free: it splits a paper's abstract into sentence-level
//! units and tags each with the paper's identity.

use crate::fact::SourceFact;
use crate::id::{ArxivId, ArxivVersion};

/// A single observed statement drawn from a source paper, with provenance back
/// to the exact paper and version it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    arxiv_id: ArxivId,
    version: ArxivVersion,
    text: String,
}

impl Observation {
    /// Build an observation with explicit provenance.
    pub fn new(arxiv_id: ArxivId, version: ArxivVersion, text: impl Into<String>) -> Self {
        Self {
            arxiv_id,
            version,
            text: text.into(),
        }
    }

    /// The paper this observation was drawn from.
    pub fn arxiv_id(&self) -> &ArxivId {
        &self.arxiv_id
    }

    /// The paper version this observation was drawn from.
    pub fn version(&self) -> &ArxivVersion {
        &self.version
    }

    /// The observed text (a sentence from the abstract).
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Sentence fragments shorter than this (after trimming) are dropped so that
/// stray initials, section markers, or trailing scraps do not become
/// observations.
const MIN_OBSERVATION_LEN: usize = 12;

/// Deterministically extract observations from a stored fact's abstract.
///
/// The abstract (`SourceFact::summary`) is split into sentences on `. `, `? `,
/// and `! ` boundaries; each non-trivial sentence becomes one [`Observation`]
/// tagged with the paper's [`ArxivId`] and [`ArxivVersion`]. No network, no LLM.
pub fn extract_observations(fact: &SourceFact) -> Vec<Observation> {
    let arxiv_id = ArxivId::new(fact.arxiv_id.clone());
    let version = ArxivVersion::parse(&fact.arxiv_version);

    split_sentences(&fact.summary)
        .into_iter()
        .map(|text| Observation::new(arxiv_id.clone(), version.clone(), text))
        .collect()
}

/// Split text into trimmed sentences on `. ` / `? ` / `! ` boundaries, dropping
/// fragments shorter than [`MIN_OBSERVATION_LEN`].
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        current.push(c);
        if matches!(c, '.' | '?' | '!') {
            // Only treat as a boundary when followed by whitespace (or end),
            // so "cs.CL", "et al.", and decimals are less likely to split.
            match chars.peek() {
                Some(next) if next.is_whitespace() => {
                    push_if_substantial(&mut sentences, &current);
                    current.clear();
                }
                None => {
                    push_if_substantial(&mut sentences, &current);
                    current.clear();
                }
                _ => {}
            }
        }
    }
    push_if_substantial(&mut sentences, &current);

    sentences
}

fn push_if_substantial(out: &mut Vec<String>, fragment: &str) {
    let trimmed = fragment.trim();
    if trimmed.chars().count() >= MIN_OBSERVATION_LEN {
        out.push(trimmed.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ArxivCategory;

    fn fact_with_summary(summary: &str) -> SourceFact {
        let primary = ArxivCategory::parse("cs.CL").unwrap();
        SourceFact {
            arxiv_id: "2301.00001".to_string(),
            title: "Test Paper".to_string(),
            authors: vec!["Alice".to_string()],
            summary: summary.to_string(),
            primary_category: primary.clone(),
            categories: vec![primary],
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            pdf_url: "http://arxiv.org/pdf/2301.00001".to_string(),
            source_url: "https://export.arxiv.org/api/query?id_list=2301.00001".to_string(),
            arxiv_version: "v3".to_string(),
            ingested_at: "2026-08-30T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_extract_observations_splits_sentences() {
        let fact = fact_with_summary(
            "First finding is stated here. Second finding follows next. Third one too here.",
        );
        let obs = extract_observations(&fact);
        assert_eq!(obs.len(), 3, "got: {obs:?}");
        assert_eq!(obs[0].text(), "First finding is stated here.");
        assert_eq!(obs[1].text(), "Second finding follows next.");
        assert_eq!(obs[2].text(), "Third one too here.");
    }

    #[test]
    fn test_extract_observations_drops_trivial_fragments() {
        // "Fig. 2" is short and should not survive as its own observation.
        let fact = fact_with_summary("We propose a novel attention mechanism. Fig. 2.");
        let obs = extract_observations(&fact);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].text(), "We propose a novel attention mechanism.");
    }

    #[test]
    fn test_observation_provenance() {
        let fact = fact_with_summary("A sufficiently long observation sentence here.");
        let obs = extract_observations(&fact);
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].arxiv_id().as_str(), "2301.00001");
        assert_eq!(*obs[0].version(), ArxivVersion(3));
    }

    #[test]
    fn test_extract_observations_empty_summary() {
        let fact = fact_with_summary("");
        assert!(extract_observations(&fact).is_empty());
    }
}
