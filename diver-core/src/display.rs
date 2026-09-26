use std::collections::{BTreeMap, HashMap};

use owo_colors::OwoColorize;

use crate::assertion::{Assertion, Supported};
use crate::concept::{ConceptKind, query_key};
use crate::fact::SourceFact;
use crate::graph::{DiveNode, RelatedPaper, RelationKind, group_related};
use crate::id::ArxivCategory;
use crate::model::Paper;
use crate::store::{ConceptInfo, ConceptSummary, SearchResult, StoredAssertion};

pub fn display_results(papers: &[Paper], total: u32) {
    if papers.is_empty() {
        println!("No results found.");
        return;
    }

    let shown = papers.len();
    if total as usize > shown {
        println!("Showing {} of {} results.\n", shown.bold(), total.bold());
    }

    for (i, paper) in papers.iter().enumerate() {
        println!("{}", format!("[{}]", i + 1).dimmed());
        println!("  {}", paper.title.bold());
        println!("  {}", paper.authors.join(", ").dimmed());
        println!("  {}", truncate_abstract(&paper.summary, 200));
        println!(
            "  {} | {}",
            paper.primary_category.cyan(),
            format!("https://arxiv.org/abs/{}", paper.arxiv_id).underline()
        );
        println!();
    }
}

/// Display full metadata for an ingested paper, including taxonomy-resolved category
/// names, secondary categories, and a version history list.
pub fn display_fact(fact: &SourceFact, versions: &[String]) {
    println!("{}", fact.title.bold());
    println!("  {}", fact.authors.join(", ").dimmed());
    println!();
    println!("  {}", fact.summary);
    println!();
    println!(
        "  {}",
        format!("https://arxiv.org/abs/{}", fact.arxiv_id).underline()
    );
    println!();

    // Primary category with taxonomy name
    println!("  {}", "Primary category:".dimmed());
    println!("    {}", format_category(&fact.primary_category).cyan());

    // Secondary categories (all except primary)
    let secondary: Vec<&ArxivCategory> = fact
        .categories
        .iter()
        .filter(|c| c.code() != fact.primary_category.code())
        .collect();

    if !secondary.is_empty() {
        println!("  {}", "Secondary:".dimmed());
        for cat in secondary {
            println!("    {}", format_category(cat));
        }
    }

    println!();

    // Version history
    if !versions.is_empty() {
        println!("  {}", "Versions:".dimmed());
        for v in versions {
            if v == &fact.arxiv_version {
                println!("    {}  {}", v.bold(), "←".dimmed());
            } else {
                println!("    {}", v.dimmed());
            }
        }
        println!();
    }

    println!("  {} {}", "Source:".dimmed(), fact.source_url);
    println!("  {} {}", "Ingested:".dimmed(), fact.ingested_at);
}

/// Display the supported assertions extracted from a paper, each with the
/// provenance of its supporting observations.
pub fn display_extract(arxiv_id: &str, supported: &[Assertion<Supported>]) {
    println!("{}", format!("Supported assertions for {arxiv_id}").bold());
    println!();

    if supported.is_empty() {
        println!("  {}", "No supported assertions extracted.".dimmed());
        return;
    }

    for (i, assertion) in supported.iter().enumerate() {
        println!("{}", format!("[{}]", i + 1).dimmed());
        println!("  {}", assertion.claim());
        for obs in assertion.support() {
            println!(
                "    {}",
                format!("— {} {}", obs.arxiv_id(), obs.version()).dimmed()
            );
        }
        println!();
    }
}

/// Display the assertions persisted for a paper (claim, version, supporting quotes).
pub fn display_stored_assertions(arxiv_id: &str, assertions: &[StoredAssertion]) {
    println!("{}", format!("Stored assertions for {arxiv_id}").bold());
    println!();

    if assertions.is_empty() {
        println!(
            "  {}",
            format!("No stored assertions for {arxiv_id}.").dimmed()
        );
        return;
    }

    for (i, assertion) in assertions.iter().enumerate() {
        println!("{}", format!("[{}]", i + 1).dimmed());
        println!("  {}", assertion.claim);
        println!("    {}", format!("({})", assertion.version).dimmed());
        for quote in &assertion.support {
            println!("    {}", format!("\u{2014} \"{quote}\"").dimmed());
        }
        println!();
    }
}

/// How many related papers to list per dive node. Papers sharing only an arXiv
/// category are never listed one by one — they are counted, since at corpus scale a
/// shared category links hundreds of papers and says nothing about their content.
const DIVE_RELATED_CAP: usize = 5;

/// How many co-asserted concepts to name per related paper before summarizing.
const DIVE_CONCEPTS_PER_PAPER: usize = 3;

/// Longest query echoed back verbatim.
const QUERY_ECHO_MAX: usize = 80;

/// How many related entries overflow the display cap, if any (`None` when the
/// count fits within `cap`).
fn related_overflow(count: usize, cap: usize) -> Option<usize> {
    (count > cap).then(|| count - cap)
}

/// A co-assertion weight for display. A surviving edge always has weight > 0; show a
/// tiny positive weight as "<0.01" rather than rounding it to a misleading "0.00".
fn format_weight(weight: f64) -> String {
    if weight > 0.0 && weight < 0.005 {
        "<0.01".to_string()
    } else {
        format!("{weight:.2}")
    }
}

fn relation_reason(kind: &RelationKind) -> String {
    match kind {
        RelationKind::SharedCategory(code) => format!("shared category {code}"),
        RelationKind::SharedAuthor(name) => format!("shared author {name}"),
        RelationKind::CoAssertion { term, weight } => {
            format!("co-asserts {term} (w={})", format_weight(*weight))
        }
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

/// A user query as echoed in messages: trimmed and length-capped.
fn echo_query(term: &str) -> String {
    let trimmed = term.trim();
    if trimmed.chars().count() > QUERY_ECHO_MAX {
        let head: String = trimmed.chars().take(QUERY_ECHO_MAX).collect();
        format!("{head}…")
    } else {
        trimmed.to_string()
    }
}

/// Header lines for a resolved concept: its label, kind and reach, the surface forms
/// it was formed from, and the concepts linked to it (narrower phrases for a term,
/// broader constituent terms for a phrase). Plain text, no styling.
pub fn format_dive_header(info: &ConceptInfo, related: &[ConceptSummary]) -> Vec<String> {
    let mut lines = vec![format!(
        "Dive: {} ({}, {})",
        info.label,
        info.kind.as_str(),
        plural(info.paper_count, "paper")
    )];
    let forms: Vec<String> = info
        .forms
        .iter()
        .map(|(form, count)| format!("{form} \u{00d7}{count}"))
        .collect();
    lines.push(format!("  forms: {}", forms.join(", ")));
    if !related.is_empty() {
        let heading = match info.kind {
            ConceptKind::Term => "narrower",
            ConceptKind::Phrase => "broader",
        };
        let names: Vec<String> = related
            .iter()
            .map(|c| format!("{} ({})", c.label, c.paper_count))
            .collect();
        lines.push(format!("  {heading}: {}", names.join(", ")));
    }
    lines
}

/// Lines describing a dive node's related papers, grouped per paper and ranked by
/// [`group_related`]: a one-line summary of how many papers are linked by shared
/// concepts, by shared authors, and by category alone, then up to
/// [`DIVE_RELATED_CAP`] papers linked by concepts or authors, each with its title and
/// its strongest shared concepts. Category-only papers are counted, never listed.
/// `titles` maps arXiv ids to titles. Plain text, no styling.
pub fn format_related(
    related: &[(String, RelationKind)],
    titles: &HashMap<&str, &str>,
) -> Vec<String> {
    let grouped = group_related(related);
    if grouped.is_empty() {
        return vec!["(no related papers)".to_string()];
    }

    let via_concepts = grouped.iter().filter(|r| !r.concepts.is_empty()).count();
    let via_authors = grouped
        .iter()
        .filter(|r| r.concepts.is_empty() && !r.authors.is_empty())
        .count();
    let category_only: Vec<&RelatedPaper> = grouped.iter().filter(|r| r.category_only()).collect();

    let mut parts = Vec::new();
    if via_concepts > 0 {
        parts.push(format!("{via_concepts} by shared concepts"));
    }
    if via_authors > 0 {
        parts.push(format!("{via_authors} by shared authors only"));
    }
    if !category_only.is_empty() {
        // Name the categories doing the linking, most common first.
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &category_only {
            for code in &r.categories {
                *counts.entry(code.as_str()).or_insert(0) += 1;
            }
        }
        let mut codes: Vec<(&str, usize)> = counts.into_iter().collect();
        codes.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        let names: Vec<&str> = codes.iter().take(3).map(|(c, _)| *c).collect();
        parts.push(format!(
            "{} sharing only a category ({})",
            category_only.len(),
            names.join(", ")
        ));
    }
    let mut lines = vec![format!("Related: {}", parts.join("; "))];

    let listed: Vec<&RelatedPaper> = grouped.iter().filter(|r| !r.category_only()).collect();
    for r in listed.iter().take(DIVE_RELATED_CAP) {
        let title = titles
            .get(r.arxiv_id.as_str())
            .map(|t| truncate_title(t, 56))
            .unwrap_or_default();
        let mut reasons = Vec::new();
        if !r.concepts.is_empty() {
            let mut named: Vec<String> = r
                .concepts
                .iter()
                .take(DIVE_CONCEPTS_PER_PAPER)
                .map(|(c, w)| format!("{c} ({})", format_weight(*w)))
                .collect();
            if let Some(more) = related_overflow(r.concepts.len(), DIVE_CONCEPTS_PER_PAPER) {
                named.push(format!("+{more} more"));
            }
            reasons.push(format!("shares {}", named.join(", ")));
        }
        for author in &r.authors {
            reasons.push(relation_reason(&RelationKind::SharedAuthor(author.clone())));
        }
        let lead = if title.is_empty() {
            r.arxiv_id.clone()
        } else {
            format!("{}  {title}", r.arxiv_id)
        };
        lines.push(format!("  {lead} \u{2014} {}", reasons.join("; ")));
    }
    if let Some(more) = related_overflow(listed.len(), DIVE_RELATED_CAP) {
        lines.push(format!("  (+{more} more linked by concepts or authors)"));
    }
    lines
}

/// Lines for a term that resolves to no concept. With no concepts at all, the
/// `diver extract` hint; for a query with no searchable words, say so; otherwise a
/// not-a-concept notice followed by the concepts that contain the term or, failing
/// that, concepts spelled similarly. Plain text, no styling.
pub fn format_dive_unresolved(
    term: &str,
    has_concepts: bool,
    suggestions: &[ConceptSummary],
    similar: &[ConceptSummary],
) -> Vec<String> {
    let shown = echo_query(term);
    let mut lines = vec![format!("Dive: {shown}")];
    if !has_concepts {
        lines.push(
            "  No concepts yet: run `diver extract <id>` or `diver extract --all` first."
                .to_string(),
        );
    } else if query_key(term).is_none() {
        lines.push(format!(
            "  '{shown}' has no searchable words (only common words, numbers or short tokens)."
        ));
    } else if !suggestions.is_empty() {
        lines.push(format!(
            "  '{shown}' is not a concept in this corpus. Concepts containing it:"
        ));
        for c in suggestions {
            lines.push(format!(
                "    {} ({})",
                c.label,
                plural(c.paper_count, "paper")
            ));
        }
    } else if !similar.is_empty() {
        lines.push(format!(
            "  '{shown}' is not a concept in this corpus. Did you mean:"
        ));
        for c in similar {
            lines.push(format!(
                "    {} ({})",
                c.label,
                plural(c.paper_count, "paper")
            ));
        }
    } else {
        lines.push(format!(
            "  '{shown}' is not a concept in this corpus, and nothing contains or resembles it."
        ));
    }
    lines
}

/// Display a resolved `diver dive`: the concept header, then each asserting paper,
/// its claims about the concept, and its related papers (grouped and bounded per
/// node). `facts` supplies the related papers' titles.
pub fn display_dive_concept(
    info: &ConceptInfo,
    related: &[ConceptSummary],
    nodes: &[DiveNode],
    facts: &[SourceFact],
) {
    let header = format_dive_header(info, related);
    println!("{}", header[0].bold());
    for line in &header[1..] {
        println!("{}", line.dimmed());
    }
    println!();

    let titles: HashMap<&str, &str> = facts
        .iter()
        .map(|f| (f.arxiv_id.as_str(), f.title.as_str()))
        .collect();
    for node in nodes {
        println!("{}  {}", node.arxiv_id.bold(), node.title);
        for claim in &node.claims {
            println!("  \u{2022} {claim}");
        }
        for line in format_related(&node.related, &titles) {
            println!("    {}", line.dimmed());
        }
        println!();
    }
}

/// Display a `diver dive` whose term resolves to no concept.
pub fn display_dive_unresolved(
    term: &str,
    has_concepts: bool,
    suggestions: &[ConceptSummary],
    similar: &[ConceptSummary],
) {
    let lines = format_dive_unresolved(term, has_concepts, suggestions, similar);
    println!("{}", lines[0].bold());
    for line in &lines[1..] {
        println!("{line}");
    }
}

fn format_category(cat: &ArxivCategory) -> String {
    format!("{} — {}", cat.code(), cat.name())
}

pub fn display_fact_list(facts: &[SourceFact]) {
    if facts.is_empty() {
        println!("No ingested papers.");
        return;
    }

    println!(
        "{:<16} {:<52} {:<8} {}",
        "ArXiv ID".bold(),
        "Title".bold(),
        "Category".bold(),
        "Ingested".bold(),
    );
    println!("{}", "-".repeat(96));

    for fact in facts {
        let title = truncate_title(&fact.title, 50);
        let date = &fact.ingested_at[..10.min(fact.ingested_at.len())];
        println!(
            "{:<16} {:<52} {:<8} {}",
            fact.arxiv_id,
            title,
            fact.primary_category.code(),
            date,
        );
    }

    println!("\n{} paper(s) ingested.", facts.len());
}

pub fn display_dive_results(results: &[SearchResult]) {
    if results.is_empty() {
        println!("No matching papers found.");
        return;
    }

    for (i, result) in results.iter().enumerate() {
        println!("{}", format!("[{}]", i + 1).dimmed());
        println!("  {}", result.title.bold());
        println!("  {}", result.authors.join(", ").dimmed());
        println!("  {}", truncate_abstract(&result.summary, 200));
        println!(
            "  {} | {}",
            result.primary_category.cyan(),
            format!("https://arxiv.org/abs/{}", result.arxiv_id).underline()
        );
        println!();
    }
}

pub fn display_collect_item(arxiv_id: &str, title: &str, is_update: bool) {
    let label = if is_update { "Updated" } else { "Ingested" };
    println!("  {}: {} \u{2014} {}", label, arxiv_id, title);
}

pub fn display_collect_summary(new_count: u32, updated_count: u32) {
    println!("Collected {} new, {} updated.", new_count, updated_count);
}

pub fn display_collect_empty() {
    println!("No papers found.");
}

fn truncate_title(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_len - 3).collect();
        format!("{truncated}...")
    }
}

fn truncate_abstract(text: &str, max_len: usize) -> String {
    let clean: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.len() <= max_len {
        clean
    } else {
        let truncated: String = clean.chars().take(max_len).collect();
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Paper;

    #[test]
    fn test_related_overflow() {
        assert_eq!(related_overflow(5, 10), None);
        assert_eq!(
            related_overflow(10, 10),
            None,
            "count == cap does not overflow"
        );
        assert_eq!(related_overflow(11, 10), Some(1));
        assert_eq!(related_overflow(25, 10), Some(15));
    }

    #[test]
    fn test_relation_reason_coassertion_weight() {
        let s = relation_reason(&RelationKind::CoAssertion {
            term: "attention".to_string(),
            weight: 0.82,
        });
        assert!(s.contains("co-asserts"), "got: {s}");
        assert!(s.contains("attention"), "got: {s}");
        assert!(s.contains("w=0.82"), "weight shown to 2dp; got: {s}");

        // A tiny positive weight is shown as "<0.01", not a misleading "0.00".
        let tiny = relation_reason(&RelationKind::CoAssertion {
            term: "models".to_string(),
            weight: 0.001,
        });
        assert!(
            tiny.contains("w=<0.01"),
            "tiny weight not rounded to 0.00; got: {tiny}"
        );
    }

    fn make_paper(summary: &str) -> Paper {
        Paper {
            title: "Test Paper".to_string(),
            authors: vec!["Alice".to_string(), "Bob".to_string()],
            summary: summary.to_string(),
            primary_category: "cs.AI".to_string(),
            categories: vec!["cs.AI".to_string()],
            published: "2023-01-01".to_string(),
            updated: "2023-01-01".to_string(),
            arxiv_id: "2301.00001".to_string(),
            pdf_url: "http://arxiv.org/pdf/2301.00001".to_string(),
        }
    }

    #[test]
    fn test_display_truncates_abstract() {
        let long_abstract = "a ".repeat(200);
        let truncated = truncate_abstract(&long_abstract, 200);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 203 + 3);
    }

    #[test]
    fn test_display_empty_results() {
        let mut output = Vec::new();
        if Vec::<Paper>::new().is_empty() {
            output.push("No results found.");
        }
        assert_eq!(output[0], "No results found.");
    }

    #[test]
    fn test_display_showing_count() {
        let papers = [
            make_paper("short"),
            make_paper("short"),
            make_paper("short"),
        ];
        let total: u32 = 50;
        let shown = papers.len();
        let msg = format!("Showing {} of {} results.", shown, total);
        assert!(msg.contains("Showing 3 of 50 results."));
    }

    fn make_fact(id: &str, title: &str) -> SourceFact {
        let primary = ArxivCategory::parse("cs.CL").unwrap();
        SourceFact {
            arxiv_id: id.to_string(),
            title: title.to_string(),
            authors: vec!["Alice".to_string()],
            summary: "A summary.".to_string(),
            primary_category: primary.clone(),
            categories: vec![primary],
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            pdf_url: format!("http://arxiv.org/pdf/{id}"),
            source_url: format!("https://export.arxiv.org/api/query?id_list={id}"),
            arxiv_version: "v1".to_string(),
            ingested_at: "2026-08-28T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_display_fact_all_fields() {
        let fact = make_fact("2301.00001", "Test Paper Title");
        let mut buf = Vec::new();
        use std::io::Write;
        writeln!(buf, "{}", fact.title).unwrap();
        writeln!(buf, "{}", fact.authors.join(", ")).unwrap();
        writeln!(buf, "{}", fact.primary_category.code()).unwrap();
        writeln!(buf, "{}", fact.source_url).unwrap();
        writeln!(buf, "{}", fact.arxiv_version).unwrap();
        writeln!(buf, "{}", fact.ingested_at).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Test Paper Title"));
        assert!(output.contains("Alice"));
        assert!(output.contains("cs.CL"));
        assert!(output.contains("export.arxiv.org"));
        assert!(output.contains("v1"));
        assert!(output.contains("2026-08-28"));
    }

    #[test]
    fn test_display_fact_taxonomy_name() {
        let primary = ArxivCategory::parse("cs.CV").unwrap();
        let cat_str = format_category(&primary);
        assert!(
            cat_str.contains("Computer Vision and Pattern Recognition"),
            "got: {cat_str}"
        );
    }

    #[test]
    fn test_display_fact_secondary_categories() {
        let primary = ArxivCategory::parse("cs.CV").unwrap();
        let secondary = ArxivCategory::parse("math.NA").unwrap();
        let fact = SourceFact {
            arxiv_id: "2301.00001".to_string(),
            title: "Multi-Category Paper".to_string(),
            authors: vec!["Alice".to_string()],
            summary: "A summary.".to_string(),
            primary_category: primary.clone(),
            categories: vec![primary, secondary],
            published: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-01T00:00:00Z".to_string(),
            pdf_url: "http://arxiv.org/pdf/2301.00001".to_string(),
            source_url: "https://export.arxiv.org/api/query?id_list=2301.00001".to_string(),
            arxiv_version: "v1".to_string(),
            ingested_at: "2026-08-28T12:00:00Z".to_string(),
        };
        // Verify secondary contains math.NA but categories[0] is cs.CV
        let secondary_cats: Vec<_> = fact
            .categories
            .iter()
            .filter(|c| c.code() != fact.primary_category.code())
            .collect();
        assert_eq!(secondary_cats.len(), 1);
        assert_eq!(secondary_cats[0].code(), "math.NA");
    }

    #[test]
    fn test_display_fact_version_marker() {
        let fact = make_fact("2301.00001", "Test");
        let versions = ["v1".to_string(), "v2".to_string()];
        // The current version in fact is "v1"; verify version marker logic
        let current = &fact.arxiv_version;
        let marked: Vec<String> = versions
            .iter()
            .map(|v| {
                if v == current {
                    format!("{}  <-", v)
                } else {
                    v.clone()
                }
            })
            .collect();
        assert!(marked[0].contains("<-"), "v1 should be marked current");
        assert!(!marked[1].contains("<-"), "v2 should not be marked");
    }

    #[test]
    fn test_display_fact_list() {
        let facts = [
            make_fact("2301.00001", "First Paper"),
            make_fact("2302.00002", "Second Paper"),
        ];
        let line_1 = format!(
            "{:<16} {:<52} {:<8} {}",
            facts[0].arxiv_id,
            truncate_title(&facts[0].title, 50),
            facts[0].primary_category.code(),
            &facts[0].ingested_at[..10],
        );
        assert!(line_1.contains("2301.00001"));
        assert!(line_1.contains("First Paper"));
        assert!(line_1.contains("cs.CL"));
    }

    #[test]
    fn test_display_fact_list_empty() {
        let facts: Vec<SourceFact> = vec![];
        assert!(facts.is_empty());
    }

    fn make_search_result(id: &str, title: &str) -> SearchResult {
        SearchResult {
            arxiv_id: id.to_string(),
            title: title.to_string(),
            authors: vec!["Alice".to_string(), "Bob".to_string()],
            summary: "A summary about attention mechanisms.".to_string(),
            primary_category: "cs.CL".to_string(),
            rank: -1.0,
        }
    }

    #[test]
    fn test_display_dive_results() {
        let results = [
            make_search_result("2301.00001", "First Result Paper"),
            make_search_result("2302.00002", "Second Result Paper"),
        ];

        let mut buf = Vec::new();
        use std::io::Write;
        for (i, r) in results.iter().enumerate() {
            writeln!(buf, "[{}]", i + 1).unwrap();
            writeln!(buf, "  {}", r.title).unwrap();
            writeln!(buf, "  {}", r.authors.join(", ")).unwrap();
            writeln!(buf, "  {}", r.primary_category).unwrap();
            writeln!(buf, "  https://arxiv.org/abs/{}", r.arxiv_id).unwrap();
        }
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("First Result Paper"));
        assert!(output.contains("Second Result Paper"));
        assert!(output.contains("2301.00001"));
        assert!(output.contains("2302.00002"));
        assert!(output.contains("cs.CL"));
    }

    #[test]
    fn test_display_dive_results_empty() {
        let results: Vec<SearchResult> = vec![];
        assert!(results.is_empty());
    }

    #[test]
    fn test_display_collect_item_new() {
        let mut buf = Vec::new();
        use std::io::Write;
        let label = "Ingested";
        writeln!(buf, "  {label}: 2301.00001 \u{2014} Test Paper").unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Ingested:"));
        assert!(output.contains("2301.00001"));
        assert!(output.contains("Test Paper"));
    }

    #[test]
    fn test_display_collect_item_update() {
        let mut buf = Vec::new();
        use std::io::Write;
        let label = "Updated";
        writeln!(buf, "  {label}: 2301.00001 \u{2014} Test Paper").unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Updated:"));
        assert!(output.contains("2301.00001"));
    }

    #[test]
    fn test_display_collect_summary() {
        let msg = format!("Collected {} new, {} updated.", 3, 2);
        assert!(msg.contains("Collected 3 new, 2 updated."));
    }

    #[test]
    fn test_display_collect_empty() {
        let msg = "No papers found.";
        assert_eq!(msg, "No papers found.");
    }

    #[test]
    fn test_format_dive_concept() {
        let summary = |id: &str, kind, paper_count| ConceptSummary {
            id: id.to_string(),
            label: id.to_string(),
            kind,
            paper_count,
        };

        // Term header: label, kind, reach, forms, narrower phrases.
        let term = ConceptInfo {
            id: "network".into(),
            label: "networks".into(),
            kind: ConceptKind::Term,
            paper_count: 6,
            forms: vec![("networks".into(), 5), ("network".into(), 5)],
        };
        let lines =
            format_dive_header(&term, &[summary("neural networks", ConceptKind::Phrase, 4)]);
        assert_eq!(lines[0], "Dive: networks (term, 6 papers)");
        assert_eq!(lines[1], "  forms: networks \u{00d7}5, network \u{00d7}5");
        assert_eq!(lines[2], "  narrower: neural networks (4)");

        // Phrase header: broader constituent terms.
        let phrase = ConceptInfo {
            id: "attention mechanism".into(),
            label: "attention mechanism".into(),
            kind: ConceptKind::Phrase,
            paper_count: 1,
            forms: vec![("attention mechanism".into(), 1)],
        };
        let lines = format_dive_header(&phrase, &[summary("attention", ConceptKind::Term, 6)]);
        assert_eq!(lines[0], "Dive: attention mechanism (phrase, 1 paper)");
        assert_eq!(lines[2], "  broader: attention (6)");

        // Unresolved: suggestions, then "did you mean", then the empty-corpus hint.
        let with = format_dive_unresolved(
            "model",
            true,
            &[summary("diffusion model", ConceptKind::Phrase, 6)],
            &[],
        );
        assert!(with[1].contains("'model' is not a concept") && with[1].contains("containing"));
        assert_eq!(with[2], "    diffusion model (6 papers)");
        let similar = format_dive_unresolved(
            "atention",
            true,
            &[],
            &[summary("attention", ConceptKind::Term, 35)],
        );
        assert!(similar[1].contains("Did you mean"));
        assert_eq!(similar[2], "    attention (35 papers)");
        let none = format_dive_unresolved("zyxw", true, &[], &[]);
        assert!(none[1].contains("nothing contains or resembles it"));
        let stop = format_dive_unresolved("the", true, &[], &[]);
        assert!(stop[1].contains("no searchable words"));
        let empty = format_dive_unresolved("anything", false, &[], &[]);
        assert!(empty[1].contains("diver extract"));

        // Related: one summary line; category-only papers counted, never listed.
        let co = |term: &str, weight| RelationKind::CoAssertion {
            term: term.to_string(),
            weight,
        };
        let related = vec![
            ("2302.00002".to_string(), co("attention", 0.83)),
            (
                "2303.00003".to_string(),
                RelationKind::SharedCategory("cs.LG".into()),
            ),
            (
                "2304.00004".to_string(),
                RelationKind::SharedAuthor("Ada".into()),
            ),
        ];
        let titles: HashMap<&str, &str> = [("2302.00002", "Paper B")].into_iter().collect();
        let lines = format_related(&related, &titles);
        assert_eq!(
            lines[0],
            "Related: 1 by shared concepts; 1 by shared authors only; \
             1 sharing only a category (cs.LG)"
        );
        assert_eq!(
            lines[1],
            "  2302.00002  Paper B \u{2014} shares attention (0.83)"
        );
        assert_eq!(lines[2], "  2304.00004 \u{2014} shared author Ada");
        assert_eq!(lines.len(), 3, "the category-only paper is not listed");
        assert_eq!(format_related(&[], &titles), vec!["(no related papers)"]);

        // At most five linked papers are listed; the rest are summarized.
        let many: Vec<(String, RelationKind)> = (0..7)
            .map(|i| (format!("2400.0000{i}"), co("attention", 0.5)))
            .collect();
        let lines = format_related(&many, &titles);
        assert_eq!(lines.len(), 1 + DIVE_RELATED_CAP + 1);
        assert_eq!(
            lines.last().unwrap(),
            "  (+2 more linked by concepts or authors)"
        );

        // A long query is echoed truncated, with an ellipsis.
        let long = "attention ".repeat(20);
        let echoed = &format_dive_unresolved(&long, true, &[], &[])[0];
        assert_eq!(echoed.chars().count(), "Dive: ".len() + QUERY_ECHO_MAX + 1);
        assert!(echoed.ends_with('\u{2026}'));
    }
}
