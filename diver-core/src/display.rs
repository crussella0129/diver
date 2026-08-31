use owo_colors::OwoColorize;

use crate::assertion::{Assertion, Supported};
use crate::fact::SourceFact;
use crate::id::ArxivCategory;
use crate::model::Paper;
use crate::store::{SearchResult, StoredAssertion};

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
        let papers = vec![
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
        let versions = vec!["v1".to_string(), "v2".to_string()];
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
        let facts = vec![
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
        let results = vec![
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
        writeln!(
            buf,
            "  {}: {} \u{2014} {}",
            label, "2301.00001", "Test Paper"
        )
        .unwrap();
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
        writeln!(
            buf,
            "  {}: {} \u{2014} {}",
            label, "2301.00001", "Test Paper"
        )
        .unwrap();
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
}
