use std::fmt;

#[derive(Debug, Clone)]
pub struct Paper {
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub primary_category: String,
    pub published: String,
    pub updated: String,
    pub arxiv_id: String,
    pub pdf_url: String,
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n  Authors: {}\n  https://arxiv.org/abs/{}",
            self.title,
            self.authors.join(", "),
            self.arxiv_id,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_display() {
        let paper = Paper {
            title: "Attention Is All You Need".to_string(),
            authors: vec!["Alice".to_string(), "Bob".to_string()],
            summary: "A summary.".to_string(),
            primary_category: "cs.CL".to_string(),
            published: "2023-01-01".to_string(),
            updated: "2023-01-01".to_string(),
            arxiv_id: "2301.00001".to_string(),
            pdf_url: "http://arxiv.org/pdf/2301.00001".to_string(),
        };
        let output = paper.to_string();
        assert!(output.contains("Attention Is All You Need"));
        assert!(output.contains("Alice, Bob"));
        assert!(output.contains("https://arxiv.org/abs/2301.00001"));
    }
}
