//! The graph layer: deterministic relations between stored papers, and the
//! `diver dive` neighborhood assembler.
//!
//! [`compute_relations`] derives typed [`ComputedRelation`] edges between papers
//! from their taxonomy categories and authors — no LLM, reproducible. [`build_dive`]
//! turns a concept's seed papers (those whose persisted assertions mention it) plus
//! those edges into a concept-centered neighborhood of [`DiveNode`]s for display.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::fact::SourceFact;

/// Why two papers are related.
#[derive(Debug, Clone, PartialEq)]
pub enum RelationKind {
    /// The papers share this arXiv category code.
    SharedCategory(String),
    /// The papers share this author.
    SharedAuthor(String),
    /// The papers' stored claims share this significant `term`, whose normalized
    /// IDF `weight` (in `[0.0, 1.0]`; higher = rarer/more distinctive across the
    /// corpus) cleared the `dive` temperature threshold.
    CoAssertion { term: String, weight: f64 },
}

/// A deterministic edge between two stored papers, by `arxiv_id`.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedRelation {
    pub from: String,
    pub to: String,
    pub kind: RelationKind,
}

/// One node of a `diver dive` neighborhood: a paper that asserts about the
/// concept, its matching claims, and the papers it is related to.
#[derive(Debug, Clone, PartialEq)]
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

/// Words excluded from co-assertion terms: common English, generic research filler
/// (`model`, `results`, `method`, `propose`, `existing`, …), near-function words, and
/// web/URL tokens (`https`, `github`). Domain terms (`attention`, `transformer`,
/// `diffusion`, `convolutional`, `translation`, `neural`, …) are intentionally absent, so
/// `dive` links papers by distinctive shared vocabulary, not filler. IDF weights the
/// surviving terms; it cannot do this job alone because a generic-but-corpus-rare word
/// (e.g. `eight`, df 2) still scores a high weight. Built once into a `HashSet` for O(1)
/// membership.
static STOPWORDS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| include_str!("stopwords.txt").split_whitespace().collect());

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
                && !STOPWORDS.contains(tok.as_str())
        })
        .collect()
}

/// Compute co-assertion edges: papers whose persisted claims share a significant
/// term, weighted by inverse document frequency and gated by `temperature`.
///
/// `claims` is `(arxiv_id, claim)` for every persisted assertion. Each paper is a
/// document; a term's document frequency `df` is the number of papers whose
/// (deduplicated) significant terms contain it. A *shared* term has `df >= 2`, so
/// its normalized weight `w = ln(N/df) / ln(N/2)` lies in `[0.0, 1.0]` — `1.0` when
/// shared by exactly two papers (rarest, most distinctive), `0.0` when shared by
/// all `N` (ubiquitous). An edge is emitted for a shared term iff
/// `w >= 1.0 - temperature`:
/// - `temperature == 1.0` keeps every shared term (threshold `0.0`);
/// - `temperature == 0.0` keeps only `df == 2` terms (threshold `1.0`);
/// - the kept set is monotonic non-decreasing in `temperature`.
///
/// When `N <= 2` there is no discriminating power (and `ln(N/2)` would be `0`), so
/// every shared term is kept with `weight = 1.0`, at any temperature. `temperature`
/// is clamped to `[0.0, 1.0]`; a non-finite value (NaN/inf) is treated as `1.0`
/// (fully permissive). One edge per shared term per unordered pair of
/// distinct papers; shared terms are emitted in sorted order for stable,
/// deterministic output. No self-edges.
pub fn compute_coassertion_relations(
    claims: &[(String, String)],
    temperature: f64,
) -> Vec<ComputedRelation> {
    // A non-finite temperature (NaN/inf) has no meaningful clamp: `f64::clamp`
    // passes NaN straight through, which would make `threshold` NaN and silently
    // drop every edge (`w >= NaN` is always false). Treat any non-finite value as
    // the fully-permissive 1.0 rather than erasing the graph.
    let t = if temperature.is_finite() {
        temperature.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let threshold = 1.0 - t;

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

    // Document frequency: how many papers' deduped terms contain each term.
    let n = papers.len();
    let mut df: HashMap<&str, usize> = HashMap::new();
    for terms in &terms_by_paper {
        for term in terms {
            *df.entry(term.as_str()).or_insert(0) += 1;
        }
    }

    // Normalized IDF weight per term, computed once. With N <= 2 every shared term
    // has df == N (no discriminating power) and ln(N/2) == 0, so weight 1.0.
    let ln_max = if n > 2 { (n as f64 / 2.0).ln() } else { 0.0 };
    let weight_by_term: HashMap<&str, f64> = df
        .iter()
        .map(|(&term, &dft)| {
            let w = if ln_max <= 0.0 {
                1.0
            } else {
                ((n as f64 / dft as f64).ln() / ln_max).clamp(0.0, 1.0)
            };
            (term, w)
        })
        .collect();

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
                let w = weight_by_term[term];
                if w >= threshold {
                    relations.push(ComputedRelation {
                        from: papers[i].clone(),
                        to: papers[j].clone(),
                        kind: RelationKind::CoAssertion {
                            term: term.to_string(),
                            weight: w,
                        },
                    });
                }
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
        // 'the' and generic filler ('improves') stopped; domain terms + acronym kept.
        assert_eq!(
            significant_terms("Attention improves the RNN accuracy!"),
            vec!["attention", "rnn", "accuracy"],
            "'the'/'improves' stopped; punctuation/case ignored; 3-char acronym kept"
        );
        // Two-char tokens and stopwords are dropped.
        assert!(significant_terms("AI is ML").is_empty());
        // Pure-number tokens are dropped (no letter); 'data'/'with' stopped; 'gpt3' kept.
        assert_eq!(
            significant_terms("Trained for 100 epochs on 2023 data with GPT3."),
            vec!["epochs", "gpt3"],
            "'100'/'2023' dropped as numbers; 'trained'/'data'/'with' stopped; 'gpt3' kept"
        );
    }

    #[test]
    fn test_significant_terms_stoplist() {
        // Generic English, research filler, and web tokens are all dropped.
        let noise = significant_terms(
            "The model shows existing results between multiple https github.com repos",
        );
        for w in [
            "the", "model", "shows", "existing", "results", "between", "multiple", "https",
            "github", "com",
        ] {
            assert!(
                !noise.contains(&w.to_string()),
                "'{w}' should be stopped: {noise:?}"
            );
        }
        // Distinctive domain terms survive.
        assert_eq!(
            significant_terms("attention convolutional diffusion transformer translation bleu"),
            vec![
                "attention",
                "convolutional",
                "diffusion",
                "transformer",
                "translation",
                "bleu"
            ],
        );
    }

    /// Flatten co-assertion edges to `(from, to, term)` triples (dropping weight),
    /// for set/subset assertions.
    fn coassertion_terms(rels: &[ComputedRelation]) -> Vec<(String, String, String)> {
        rels.iter()
            .filter_map(|r| match &r.kind {
                RelationKind::CoAssertion { term, .. } => {
                    Some((r.from.clone(), r.to.clone(), term.clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// A 4-paper corpus where document frequency separates three terms:
    /// `rare` (df 2 → weight 1.0), `mid` (df 3 → ~0.415), `common` (df 4 → 0.0).
    fn tfidf_corpus() -> Vec<(String, String)> {
        vec![
            claim("2301.00001", "rare mid common"),
            claim("2302.00002", "rare mid common"),
            claim("2303.00003", "mid common"),
            claim("2304.00004", "common"),
        ]
    }

    #[test]
    fn test_coassertion_shared_term() {
        let claims = vec![
            claim("2301.00001", "Attention improves accuracy."),
            claim("2302.00002", "Attention reduces cost."),
        ];
        // N == 2 → small-corpus guard → weight 1.0 regardless of temperature.
        let rels = compute_coassertion_relations(&claims, 1.0);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].from, "2301.00001");
        assert_eq!(rels[0].to, "2302.00002");
        assert_eq!(
            rels[0].kind,
            RelationKind::CoAssertion {
                term: "attention".to_string(),
                weight: 1.0,
            }
        );
    }

    #[test]
    fn test_coassertion_no_self_edges() {
        assert!(
            compute_coassertion_relations(&[claim("2301.00001", "Attention here.")], 1.0)
                .is_empty()
        );
        // Multiple claims for the same paper group into one node → no self edge.
        let dup = vec![
            claim("2301.00001", "Attention here."),
            claim("2301.00001", "Attention there."),
        ];
        assert!(compute_coassertion_relations(&dup, 1.0).is_empty());
    }

    #[test]
    fn test_coassertion_dedups_repeated_term() {
        let claims = vec![
            claim("2301.00001", "Attention. Attention again."),
            claim("2302.00002", "Attention elsewhere."),
        ];
        let rels = compute_coassertion_relations(&claims, 1.0);
        assert_eq!(rels.len(), 1, "a repeated term yields one edge");
        assert_eq!(
            rels[0].kind,
            RelationKind::CoAssertion {
                term: "attention".to_string(),
                weight: 1.0,
            }
        );
    }

    #[test]
    fn test_coassertion_disjoint_none() {
        let claims = vec![
            claim("2301.00001", "Recurrence limits speed."),
            claim("2302.00002", "Convolution reduces cost."),
        ];
        assert!(compute_coassertion_relations(&claims, 1.0).is_empty());
    }

    #[test]
    fn test_coassertion_sorted_deterministic() {
        let claims = vec![
            claim("2301.00001", "Zebra apple mango."),
            claim("2302.00002", "Mango zebra apple."),
        ];
        let terms: Vec<String> = compute_coassertion_relations(&claims, 1.0)
            .into_iter()
            .map(|r| match r.kind {
                RelationKind::CoAssertion { term, .. } => term,
                other => panic!("expected CoAssertion, got {other:?}"),
            })
            .collect();
        assert_eq!(terms, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn test_coassertion_weighted_threshold() {
        let corpus = tfidf_corpus();

        // At temperature 0.5 (threshold 0.5) only the distinctive `rare` term
        // (df 2, weight 1.0) links A-B; `mid` (~0.415) and `common` (0.0) drop.
        let warm = compute_coassertion_relations(&corpus, 0.5);
        let ab: Vec<&ComputedRelation> = warm
            .iter()
            .filter(|r| r.from == "2301.00001" && r.to == "2302.00002")
            .collect();
        let terms: Vec<&str> = ab
            .iter()
            .filter_map(|r| match &r.kind {
                RelationKind::CoAssertion { term, .. } => Some(term.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            terms,
            vec!["rare"],
            "only the distinctive term clears t=0.5"
        );
        match &ab[0].kind {
            RelationKind::CoAssertion { weight, .. } => {
                assert!((weight - 1.0).abs() < 1e-9, "df==2 term weighs 1.0")
            }
            other => panic!("expected CoAssertion, got {other:?}"),
        }

        // At temperature 1.0 every shared term is emitted; `mid` (df 3 of N 4)
        // carries the fractional IDF weight ln(4/3)/ln(4/2).
        let hot = compute_coassertion_relations(&corpus, 1.0);
        let mid_w = hot
            .iter()
            .find_map(|r| match &r.kind {
                RelationKind::CoAssertion { term, weight }
                    if r.from == "2301.00001" && r.to == "2302.00002" && term == "mid" =>
                {
                    Some(*weight)
                }
                _ => None,
            })
            .expect("mid edge present at t=1.0");
        let expected = (4.0_f64 / 3.0).ln() / (4.0_f64 / 2.0).ln();
        assert!(
            (mid_w - expected).abs() < 1e-9,
            "mid weight {mid_w} != {expected}"
        );
    }

    #[test]
    fn test_coassertion_temperature_endpoints() {
        let corpus = tfidf_corpus();

        // t = 1.0 → threshold 0.0 → every shared term; A-B shares all three.
        let hot = coassertion_terms(&compute_coassertion_relations(&corpus, 1.0));
        let mut ab: Vec<&str> = hot
            .iter()
            .filter(|(f, t, _)| f == "2301.00001" && t == "2302.00002")
            .map(|(_, _, term)| term.as_str())
            .collect();
        ab.sort_unstable();
        assert_eq!(ab, vec!["common", "mid", "rare"]);

        // t = 0.0 → threshold 1.0 → only df==2 terms; `rare` links exactly A-B.
        let cold = coassertion_terms(&compute_coassertion_relations(&corpus, 0.0));
        assert!(
            cold.iter().all(|(_, _, term)| term == "rare"),
            "only df==2 terms survive t=0.0: {cold:?}"
        );
        assert_eq!(cold.len(), 1, "rare links exactly the one pair A-B");
    }

    #[test]
    fn test_coassertion_temperature_monotonic() {
        let corpus = tfidf_corpus();
        let at = |t: f64| -> HashSet<(String, String, String)> {
            coassertion_terms(&compute_coassertion_relations(&corpus, t))
                .into_iter()
                .collect()
        };
        let cold = at(0.0);
        let warm = at(0.5);
        let hot = at(1.0);
        assert!(cold.is_subset(&warm), "edge set grows with temperature");
        assert!(warm.is_subset(&hot), "edge set grows with temperature");
        assert!(
            cold.len() < hot.len(),
            "higher temperature yields strictly more edges here"
        );
    }

    #[test]
    fn test_coassertion_small_corpus_guard() {
        // N == 2: no discriminating power; the shared term is kept at any
        // temperature with a finite weight 1.0 (no NaN from ln(N/2) == 0).
        let corpus = vec![
            claim("2301.00001", "attention model"),
            claim("2302.00002", "attention method"),
        ];
        let rels = compute_coassertion_relations(&corpus, 0.0);
        assert_eq!(rels.len(), 1);
        match &rels[0].kind {
            RelationKind::CoAssertion { term, weight } => {
                assert_eq!(term, "attention");
                assert!(weight.is_finite(), "weight must be finite, got {weight}");
                assert_eq!(*weight, 1.0);
            }
            other => panic!("expected CoAssertion, got {other:?}"),
        }
    }

    #[test]
    fn test_coassertion_temperature_sanitized() {
        let corpus = tfidf_corpus();
        let permissive = coassertion_terms(&compute_coassertion_relations(&corpus, 1.0));
        let selective = coassertion_terms(&compute_coassertion_relations(&corpus, 0.0));

        // A non-finite temperature must NOT silently drop every edge (the NaN would
        // otherwise poison the threshold); it is treated as the permissive 1.0.
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                coassertion_terms(&compute_coassertion_relations(&corpus, bad)),
                permissive,
                "non-finite temperature {bad} should behave like t=1.0, not empty"
            );
        }

        // Out-of-range finite values clamp into [0.0, 1.0].
        assert_eq!(
            coassertion_terms(&compute_coassertion_relations(&corpus, 5.0)),
            permissive,
            "t > 1 clamps to 1.0"
        );
        assert_eq!(
            coassertion_terms(&compute_coassertion_relations(&corpus, -5.0)),
            selective,
            "t < 0 clamps to 0.0"
        );
    }
}
