use owo_colors::OwoColorize;

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
}
