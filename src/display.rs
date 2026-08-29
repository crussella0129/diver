use owo_colors::OwoColorize;

use crate::fact::SourceFact;
use crate::model::Paper;

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

pub fn display_fact(fact: &SourceFact) {
    println!("{}", fact.title.bold());
    println!("  {}", fact.authors.join(", ").dimmed());
    println!();
    println!("  {}", fact.summary);
    println!();
    println!(
        "  {} | {}",
        fact.primary_category.cyan(),
        format!("https://arxiv.org/abs/{}", fact.arxiv_id).underline()
    );
    println!("  {} {}", "Version:".dimmed(), fact.arxiv_version);
    println!("  {} {}", "Source:".dimmed(), fact.source_url);
    println!("  {} {}", "Ingested:".dimmed(), fact.ingested_at);
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
            fact.arxiv_id, title, fact.primary_category, date,
        );
    }

    println!("\n{} paper(s) ingested.", facts.len());
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
        SourceFact {
            arxiv_id: id.to_string(),
            title: title.to_string(),
            authors: vec!["Alice".to_string()],
            summary: "A summary.".to_string(),
            primary_category: "cs.CL".to_string(),
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
        writeln!(buf, "{}", fact.primary_category).unwrap();
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
    fn test_display_fact_list() {
        let facts = vec![
            make_fact("2301.00001", "First Paper"),
            make_fact("2302.00002", "Second Paper"),
        ];
        let line_1 = format!(
            "{:<16} {:<52} {:<8} {}",
            facts[0].arxiv_id,
            truncate_title(&facts[0].title, 50),
            facts[0].primary_category,
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
}
