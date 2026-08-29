use anyhow::{bail, Result};
use std::fmt;
use std::sync::OnceLock;

/// Embedded arXiv category taxonomy snapshot.
/// Sourced from <https://arxiv.org/category_taxonomy> on 2026-08-29.
const TAXONOMY_JSON: &str = include_str!("../taxonomy/arxiv_categories.json");

/// A normalised arXiv paper identifier (bare, without version suffix).
/// Example: `ArxivId::new("2301.00001")`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArxivId(String);

impl ArxivId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArxivId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A version tag for an arXiv paper, e.g. v1, v2.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArxivVersion(pub u32);

impl fmt::Display for ArxivVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// An arXiv category code validated against the bundled taxonomy snapshot.
/// Example: `ArxivCategory::parse("cs.CV")`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArxivCategory {
    code: String,
    name: String,
    group: String,
}

impl ArxivCategory {
    /// Parse and validate an arXiv category code against the embedded taxonomy.
    /// Returns `Err` if the code is not in the taxonomy.
    pub fn parse(code: &str) -> Result<Self> {
        if code.starts_with('_') {
            bail!("'{}' is a metadata key, not an arXiv category", code);
        }

        let taxonomy = Self::taxonomy();

        let entry = match taxonomy.get(code) {
            Some(e) => e,
            None => bail!("arXiv category '{}' is not in the taxonomy snapshot", code),
        };

        let name = entry
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(code)
            .to_string();

        let group = entry
            .get("group")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            code: code.to_string(),
            name,
            group,
        })
    }

    /// Build a placeholder for a category code not in the taxonomy snapshot.
    pub fn unknown(code: &str) -> Self {
        Self {
            code: code.to_string(),
            name: format!("Unknown ({code})"),
            group: String::new(),
        }
    }

    fn taxonomy() -> &'static serde_json::Value {
        static TAXONOMY: OnceLock<serde_json::Value> = OnceLock::new();
        TAXONOMY.get_or_init(|| {
            serde_json::from_str(TAXONOMY_JSON)
                .expect("bundled taxonomy JSON is malformed — this is a compile-time bug")
        })
    }

    /// The raw category code, e.g. `"cs.CV"`.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The human-readable name from the taxonomy, e.g. `"Computer Vision and Pattern Recognition"`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The top-level group, e.g. `"cs"`.
    pub fn group(&self) -> &str {
        &self.group
    }
}

impl fmt::Display for ArxivCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} — {}", self.code, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taxonomy_valid_code() {
        let cat = ArxivCategory::parse("cs.CV").unwrap();
        assert_eq!(cat.code(), "cs.CV");
        assert_eq!(cat.name(), "Computer Vision and Pattern Recognition");
        assert_eq!(cat.group(), "cs");
    }

    #[test]
    fn test_taxonomy_invalid_code() {
        let result = ArxivCategory::parse("invalid.XX");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not in the taxonomy"), "got: {msg}");
    }

    #[test]
    fn test_taxonomy_rejects_meta_key() {
        let result = ArxivCategory::parse("_meta");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("metadata key"), "got: {msg}");
    }

    #[test]
    fn test_unknown_preserves_code() {
        let cat = ArxivCategory::unknown("astro-ph.IM");
        assert_eq!(cat.code(), "astro-ph.IM");
        assert!(cat.name().contains("Unknown"), "got: {}", cat.name());
        assert!(cat.name().contains("astro-ph.IM"), "got: {}", cat.name());
    }

    #[test]
    fn test_taxonomy_math_na() {
        let cat = ArxivCategory::parse("math.NA").unwrap();
        assert_eq!(cat.code(), "math.NA");
        assert_eq!(cat.name(), "Numerical Analysis");
        assert_eq!(cat.group(), "math");
    }

    #[test]
    fn test_taxonomy_stat_ml() {
        let cat = ArxivCategory::parse("stat.ML").unwrap();
        assert_eq!(cat.code(), "stat.ML");
        assert_eq!(cat.name(), "Machine Learning");
    }

    #[test]
    fn test_arxiv_version_display() {
        assert_eq!(ArxivVersion(1).to_string(), "v1");
        assert_eq!(ArxivVersion(2).to_string(), "v2");
        assert_eq!(ArxivVersion(10).to_string(), "v10");
    }

    #[test]
    fn test_arxiv_id_construction() {
        let id = ArxivId::new("2301.00001");
        assert_eq!(id.as_str(), "2301.00001");
        assert_eq!(id.to_string(), "2301.00001");
    }

    #[test]
    fn test_category_display() {
        let cat = ArxivCategory::parse("cs.LG").unwrap();
        let s = cat.to_string();
        assert!(s.contains("cs.LG"), "got: {s}");
        assert!(s.contains("Machine Learning"), "got: {s}");
    }
}
