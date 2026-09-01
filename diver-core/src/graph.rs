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
}
