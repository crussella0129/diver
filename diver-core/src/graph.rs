//! The graph layer: deterministic relations between stored papers, and the
//! `diver dive` neighborhood assembler.
//!
//! [`compute_relations`] derives typed [`ComputedRelation`] edges between papers
//! from their taxonomy categories and authors — no LLM, reproducible. [`build_dive`]
//! turns a concept's seed papers (those whose persisted assertions mention it) plus
//! those edges into a concept-centered neighborhood of [`DiveNode`]s for display.

use std::collections::HashSet;

use crate::fact::SourceFact;

/// Why two papers are related.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationKind {
    /// The papers share this arXiv category code.
    SharedCategory(String),
    /// The papers share this author.
    SharedAuthor(String),
    /// The papers' stored claims share this significant term.
    CoAssertion(String),
}

/// A deterministic edge between two stored papers, by `arxiv_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedRelation {
    pub from: String,
    pub to: String,
    pub kind: RelationKind,
}

/// One node of a `diver dive` neighborhood: a paper that asserts about the
/// concept, its matching claims, and the papers it is related to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiveNode {
    pub arxiv_id: String,
    pub title: String,
    pub claims: Vec<String>,
    pub related: Vec<(String, RelationKind)>,
}

/// Compute the deterministic relations across a set of papers: one
/// [`RelationKind::SharedCategory`] edge per shared category code and one
/// [`RelationKind::SharedAuthor`] edge per shared author, for each unordered pair
/// (`i < j`). No self-edges (pairs with the same `arxiv_id` are skipped).
pub fn compute_relations(facts: &[SourceFact]) -> Vec<ComputedRelation> {
    // Deduplicate each paper's category codes and authors once. arXiv author lists
    // are not deduplicated upstream (unlike categories), so without this a repeated
    // author would yield duplicate edges.
    let categories: Vec<Vec<&str>> = facts
        .iter()
        .map(|f| dedup_preserve_order(f.categories.iter().map(|c| c.code())))
        .collect();
    let authors: Vec<Vec<&str>> = facts
        .iter()
        .map(|f| dedup_preserve_order(f.authors.iter().map(|s| s.as_str())))
        .collect();

    let mut relations = Vec::new();

    for i in 0..facts.len() {
        // Build paper i's lookup sets once, not once per inner-loop iteration.
        let a_categories: HashSet<&str> = categories[i].iter().copied().collect();
        let a_authors: HashSet<&str> = authors[i].iter().copied().collect();

        for j in (i + 1)..facts.len() {
            if facts[i].arxiv_id == facts[j].arxiv_id {
                continue;
            }

            for &code in &categories[j] {
                if a_categories.contains(code) {
                    relations.push(ComputedRelation {
                        from: facts[i].arxiv_id.clone(),
                        to: facts[j].arxiv_id.clone(),
                        kind: RelationKind::SharedCategory(code.to_string()),
                    });
                }
            }

            for &author in &authors[j] {
                if a_authors.contains(author) {
                    relations.push(ComputedRelation {
                        from: facts[i].arxiv_id.clone(),
                        to: facts[j].arxiv_id.clone(),
                        kind: RelationKind::SharedAuthor(author.to_string()),
                    });
                }
            }
        }
    }

    relations
}

/// Collect the items, keeping the first occurrence of each and dropping later
/// duplicates while preserving order.
fn dedup_preserve_order<'a>(items: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut seen = HashSet::new();
    items.filter(|item| seen.insert(*item)).collect()
}

/// Assemble a concept neighborhood. `asserting` is `(arxiv_id, claim)` for each
/// persisted assertion whose claim matched the concept; `relations` is the full
/// edge set from [`compute_relations`]. Returns one [`DiveNode`] per distinct
/// asserting paper (first-seen order), carrying its title (looked up in `facts`,
/// falling back to the id), its matching claims, and the relations touching it.
pub fn build_dive(
    facts: &[SourceFact],
    asserting: &[(String, String)],
    relations: &[ComputedRelation],
) -> Vec<DiveNode> {
    let mut nodes: Vec<DiveNode> = Vec::new();

    for (arxiv_id, claim) in asserting {
        if let Some(existing) = nodes.iter_mut().find(|n| &n.arxiv_id == arxiv_id) {
            existing.claims.push(claim.clone());
            continue;
        }

        let title = facts
            .iter()
            .find(|f| &f.arxiv_id == arxiv_id)
            .map(|f| f.title.clone())
            .unwrap_or_else(|| arxiv_id.clone());

        let related = relations
            .iter()
            .filter_map(|r| {
                if &r.from == arxiv_id {
                    Some((r.to.clone(), r.kind.clone()))
                } else if &r.to == arxiv_id {
                    Some((r.from.clone(), r.kind.clone()))
                } else {
                    None
                }
            })
            .collect();

        nodes.push(DiveNode {
            arxiv_id: arxiv_id.clone(),
            title,
            claims: vec![claim.clone()],
            related,
        });
    }

    nodes
}

/// Function words excluded from co-assertion terms. Kept intentionally small —
/// domain words are not stopped in v1 (term weighting is future work).
const STOPWORDS: &[&str] = &[
    "the", "and", "for", "are", "was", "were", "has", "have", "had", "this", "that", "with",
    "from", "our", "its", "can", "will", "not", "but", "all", "any", "use", "used", "using",
    "based", "which", "into", "than", "then", "they", "their", "them", "these", "those", "such",
    "also", "over", "via", "per", "onto", "upon", "both", "each", "more", "most", "some", "when",
    "where", "while", "does", "did", "been", "being", "here", "there", "your", "you",
];

/// Extract a claim's significant terms: alphanumeric tokens, lowercased, at least
/// 3 chars long, containing at least one letter, excluding [`STOPWORDS`].
/// Punctuation and case are ignored; pure-number tokens (e.g. "100", "2023") are
/// dropped so shared figures do not spuriously link papers, while mixed tokens
/// ("gpt3", "h100") survive.
fn significant_terms(claim: &str) -> Vec<String> {
    claim
        .split(|c: char| !c.is_alphanumeric())
        .filter(|tok| !tok.is_empty())
        .map(|tok| tok.to_lowercase())
        .filter(|tok| {
            tok.chars().count() >= 3
                && tok.chars().any(|c| c.is_alphabetic())
                && !STOPWORDS.contains(&tok.as_str())
        })
        .collect()
}

/// Compute co-assertion edges: papers whose persisted claims share a significant
/// term. `claims` is `(arxiv_id, claim)` for every persisted assertion. One
/// [`RelationKind::CoAssertion`] edge is emitted per shared term for each unordered
/// pair of distinct papers; shared terms are emitted in sorted order for stable,
/// deterministic output. No self-edges.
pub fn compute_coassertion_relations(claims: &[(String, String)]) -> Vec<ComputedRelation> {
    // Group each paper's significant terms (deduplicated, order-preserving),
    // keeping papers in first-seen order.
    let mut papers: Vec<String> = Vec::new();
    let mut terms_by_paper: Vec<Vec<String>> = Vec::new();
    for (arxiv_id, claim) in claims {
        let idx = match papers.iter().position(|p| p == arxiv_id) {
            Some(idx) => idx,
            None => {
                papers.push(arxiv_id.clone());
                terms_by_paper.push(Vec::new());
                papers.len() - 1
            }
        };
        terms_by_paper[idx].extend(significant_terms(claim));
    }
    for terms in &mut terms_by_paper {
        *terms = dedup_preserve_order(terms.iter().map(|s| s.as_str()))
            .into_iter()
            .map(str::to_string)
            .collect();
    }

    let mut relations = Vec::new();
    for i in 0..papers.len() {
        let a_terms: HashSet<&str> = terms_by_paper[i].iter().map(|s| s.as_str()).collect();
        // `papers` is distinct by construction (grouping above), so every (i, j)
        // with i < j is a pair of different papers — no self-edge guard needed.
        for j in (i + 1)..papers.len() {
            // `terms_by_paper[j]` is already deduplicated, so the filtered
            // intersection is unique; sorting alone gives stable output.
            let mut shared: Vec<&str> = terms_by_paper[j]
                .iter()
                .map(|s| s.as_str())
                .filter(|t| a_terms.contains(t))
                .collect();
            shared.sort_unstable();
            for term in shared {
                relations.push(ComputedRelation {
                    from: papers[i].clone(),
                    to: papers[j].clone(),
                    kind: RelationKind::CoAssertion(term.to_string()),
                });
            }
        }
    }

    relations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ArxivCategory;

    fn fact(id: &str, title: &str, categories: &[&str], authors: &[&str]) -> SourceFact {
        let categories: Vec<ArxivCategory> = categories
            .iter()
            .map(|c| ArxivCategory::parse(c).unwrap())
            .collect();
        let primary = categories[0].clone();
        SourceFact {
            arxiv_id: id.to_string(),
            title: title.to_string(),
            authors: authors.iter().map(|s| s.to_string()).collect(),
            summary: "A summary.".to_string(),
            primary_category: primary,
            categories,
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            pdf_url: format!("http://arxiv.org/pdf/{id}"),
            source_url: format!("https://export.arxiv.org/api/query?id_list={id}"),
            arxiv_version: "v1".to_string(),
            ingested_at: "2026-09-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_compute_relations_shared_category() {
        let facts = vec![
            fact("2301.00001", "A", &["cs.CL"], &["Alice"]),
            fact("2302.00002", "B", &["cs.CL"], &["Bob"]),
        ];
        let rels = compute_relations(&facts);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].from, "2301.00001");
        assert_eq!(rels[0].to, "2302.00002");
        assert_eq!(
            rels[0].kind,
            RelationKind::SharedCategory("cs.CL".to_string())
        );
    }

    #[test]
    fn test_compute_relations_shared_author() {
        let facts = vec![
            fact("2301.00001", "A", &["cs.CL"], &["Alice", "Bob"]),
            fact("2302.00002", "B", &["math.NA"], &["Bob"]),
        ];
        let rels = compute_relations(&facts);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].kind, RelationKind::SharedAuthor("Bob".to_string()));
    }

    #[test]
    fn test_compute_relations_dedups_repeated_author() {
        // A paper whose author list repeats a name must not yield duplicate edges.
        let facts = vec![
            fact("2301.00001", "A", &["cs.CL"], &["Bob"]),
            fact("2302.00002", "B", &["math.NA"], &["Bob", "Bob"]),
        ];
        let rels = compute_relations(&facts);
        assert_eq!(rels.len(), 1, "repeated author must not duplicate the edge");
        assert_eq!(rels[0].kind, RelationKind::SharedAuthor("Bob".to_string()));
    }

    #[test]
    fn test_compute_relations_no_edges_when_disjoint() {
        let facts = vec![
            fact("2301.00001", "A", &["cs.CL"], &["Alice"]),
            fact("2302.00002", "B", &["math.NA"], &["Bob"]),
        ];
        assert!(compute_relations(&facts).is_empty());
    }

    #[test]
    fn test_compute_relations_no_self_edges() {
        // A single paper yields no pairs.
        let one = vec![fact("2301.00001", "A", &["cs.CL"], &["Alice"])];
        assert!(compute_relations(&one).is_empty());
        // Even a duplicated id does not produce a self-edge.
        let dup = vec![
            fact("2301.00001", "A", &["cs.CL"], &["Alice"]),
            fact("2301.00001", "A", &["cs.CL"], &["Alice"]),
        ];
        assert!(compute_relations(&dup).is_empty());
    }

    #[test]
    fn test_build_dive_assembles_neighborhood() {
        let facts = vec![
            fact("2301.00001", "Paper A", &["cs.CL"], &["Alice"]),
            fact("2302.00002", "Paper B", &["cs.CL"], &["Bob"]),
        ];
        let relations = compute_relations(&facts);
        // Paper A asserts about the concept.
        let asserting = vec![(
            "2301.00001".to_string(),
            "Attention improves accuracy.".to_string(),
        )];

        let nodes = build_dive(&facts, &asserting, &relations);
        assert_eq!(nodes.len(), 1);
        let node = &nodes[0];
        assert_eq!(node.arxiv_id, "2301.00001");
        assert_eq!(node.title, "Paper A");
        assert_eq!(
            node.claims,
            vec!["Attention improves accuracy.".to_string()]
        );
        // Related to Paper B by the shared cs.CL category.
        assert_eq!(node.related.len(), 1);
        assert_eq!(node.related[0].0, "2302.00002");
        assert_eq!(
            node.related[0].1,
            RelationKind::SharedCategory("cs.CL".to_string())
        );
    }

    #[test]
    fn test_build_dive_groups_claims_by_paper() {
        let facts = vec![fact("2301.00001", "Paper A", &["cs.CL"], &["Alice"])];
        let asserting = vec![
            ("2301.00001".to_string(), "First claim.".to_string()),
            ("2301.00001".to_string(), "Second claim.".to_string()),
        ];
        let nodes = build_dive(&facts, &asserting, &[]);
        assert_eq!(nodes.len(), 1, "same paper groups into one node");
        assert_eq!(nodes[0].claims.len(), 2);
    }

    fn claim(id: &str, text: &str) -> (String, String) {
        (id.to_string(), text.to_string())
    }

    #[test]
    fn test_significant_terms() {
        assert_eq!(
            significant_terms("Attention improves the RNN accuracy!"),
            vec!["attention", "improves", "rnn", "accuracy"],
            "'the' stopped; punctuation/case ignored; 3-char acronym kept"
        );
        // Two-char tokens and stopwords are dropped.
        assert!(significant_terms("AI is ML").is_empty());
        // Pure-number tokens are dropped (no letter); mixed alphanumerics survive.
        assert_eq!(
            significant_terms("Trained for 100 epochs on 2023 data with GPT3."),
            vec!["trained", "epochs", "data", "gpt3"],
            "'100'/'2023' dropped as pure numbers; 'gpt3' kept (has a letter)"
        );
    }

    #[test]
    fn test_coassertion_shared_term() {
        let claims = vec![
            claim("2301.00001", "Attention improves accuracy."),
            claim("2302.00002", "Attention reduces cost."),
        ];
        let rels = compute_coassertion_relations(&claims);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].from, "2301.00001");
        assert_eq!(rels[0].to, "2302.00002");
        assert_eq!(
            rels[0].kind,
            RelationKind::CoAssertion("attention".to_string())
        );
    }

    #[test]
    fn test_coassertion_no_self_edges() {
        assert!(
            compute_coassertion_relations(&[claim("2301.00001", "Attention here.")]).is_empty()
        );
        // Multiple claims for the same paper group into one node → no self edge.
        let dup = vec![
            claim("2301.00001", "Attention here."),
            claim("2301.00001", "Attention there."),
        ];
        assert!(compute_coassertion_relations(&dup).is_empty());
    }

    #[test]
    fn test_coassertion_dedups_repeated_term() {
        let claims = vec![
            claim("2301.00001", "Attention. Attention again."),
            claim("2302.00002", "Attention elsewhere."),
        ];
        let rels = compute_coassertion_relations(&claims);
        assert_eq!(rels.len(), 1, "a repeated term yields one edge");
        assert_eq!(
            rels[0].kind,
            RelationKind::CoAssertion("attention".to_string())
        );
    }

    #[test]
    fn test_coassertion_disjoint_none() {
        let claims = vec![
            claim("2301.00001", "Recurrence limits speed."),
            claim("2302.00002", "Convolution reduces cost."),
        ];
        assert!(compute_coassertion_relations(&claims).is_empty());
    }

    #[test]
    fn test_coassertion_sorted_deterministic() {
        let claims = vec![
            claim("2301.00001", "Zebra apple mango."),
            claim("2302.00002", "Mango zebra apple."),
        ];
        let terms: Vec<String> = compute_coassertion_relations(&claims)
            .into_iter()
            .map(|r| match r.kind {
                RelationKind::CoAssertion(t) => t,
                other => panic!("expected CoAssertion, got {other:?}"),
            })
            .collect();
        assert_eq!(terms, vec!["apple", "mango", "zebra"]);
    }
}
