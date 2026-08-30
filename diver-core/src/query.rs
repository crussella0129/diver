const BASE_URL: &str = "https://export.arxiv.org/api/query";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Relevance,
    SubmittedDate,
    LastUpdatedDate,
}

impl SortBy {
    fn as_param(self) -> &'static str {
        match self {
            SortBy::Relevance => "relevance",
            SortBy::SubmittedDate => "submittedDate",
            SortBy::LastUpdatedDate => "lastUpdatedDate",
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryBuilder {
    terms: Vec<String>,
    max_results: u32,
    sort_by: SortBy,
    start: u32,
}

impl QueryBuilder {
    pub fn new(query: &str) -> Self {
        let terms: Vec<String> = query
            .split_whitespace()
            .map(|w| format!("all:{w}"))
            .collect();

        Self {
            terms,
            max_results: 10,
            sort_by: SortBy::Relevance,
            start: 0,
        }
    }

    pub fn max_results(mut self, n: u32) -> Self {
        self.max_results = n;
        self
    }

    pub fn sort_by(mut self, sort: SortBy) -> Self {
        self.sort_by = sort;
        self
    }

    pub fn start(mut self, offset: u32) -> Self {
        self.start = offset;
        self
    }

    pub fn build(&self) -> String {
        let search_query = self.terms.join("+AND+");
        format!(
            "{}?search_query={}&start={}&max_results={}&sortBy={}&sortOrder=descending",
            BASE_URL,
            search_query,
            self.start,
            self.max_results,
            self.sort_by.as_param(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_default() {
        let url = QueryBuilder::new("attention").build();
        assert!(url.contains("max_results=10"));
        assert!(url.contains("sortBy=relevance"));
    }

    #[test]
    fn test_query_max_results() {
        let url = QueryBuilder::new("attention").max_results(5).build();
        assert!(url.contains("max_results=5"));
    }

    #[test]
    fn test_query_sort_by_submitted() {
        let url = QueryBuilder::new("attention")
            .sort_by(SortBy::SubmittedDate)
            .build();
        assert!(url.contains("sortBy=submittedDate"));
    }

    #[test]
    fn test_query_sort_by_updated() {
        let url = QueryBuilder::new("attention")
            .sort_by(SortBy::LastUpdatedDate)
            .build();
        assert!(url.contains("sortBy=lastUpdatedDate"));
    }

    #[test]
    fn test_query_multiword() {
        let url = QueryBuilder::new("transformer attention").build();
        assert!(url.contains("search_query=all:transformer+AND+all:attention"));
    }
}
