use anyhow::{Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

use diver_core::assertion::candidate_assertions;
use diver_core::client::ArxivClient;
use diver_core::display;
use diver_core::extract::LlmExtractor;
use diver_core::fact::SourceFact;
use diver_core::graph::{build_dive, compute_coassertion_relations, compute_relations};
use diver_core::observation::extract_observations;
use diver_core::query::{QueryBuilder, SortBy};
use diver_core::store::Store;

#[derive(Parser)]
#[command(name = "diver", about = "Find knowledge, not just papers, on ArXiv")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search ArXiv for papers
    Search {
        /// Search query
        query: String,

        /// Maximum number of results to return
        #[arg(long, default_value_t = 10)]
        max_results: u32,

        /// Sort results by
        #[arg(long, default_value = "relevance")]
        sort_by: SortOption,
    },

    /// Ingest a paper by ArXiv ID
    Ingest {
        /// ArXiv paper ID (e.g., 2301.00001)
        arxiv_id: String,
    },

    /// Inspect a stored paper's full metadata
    Inspect {
        /// ArXiv paper ID (e.g., 2301.00001)
        arxiv_id: String,
    },

    /// Extract supported assertions from a stored paper's abstract
    Extract {
        /// ArXiv paper ID (e.g., 2301.00001)
        arxiv_id: String,

        /// Use the offline sentence-splitter instead of the Claude API (no key needed)
        #[arg(long)]
        deterministic: bool,
    },

    /// Show the assertions previously extracted and stored for a paper
    Assertions {
        /// ArXiv paper ID (e.g., 2301.00001)
        arxiv_id: String,
    },

    /// Explore a concept: papers that assert about it and how they connect
    Dive {
        /// Concept to explore (matched against stored assertion claims)
        concept: String,

        /// How permissive co-assertion linking is, in [0.0, 1.0]: 0.0 links only
        /// rare/distinctive shared claim terms, 1.0 links every shared term.
        /// Structural (category/author) edges are unaffected.
        #[arg(long, default_value_t = 0.5, value_parser = parse_temperature)]
        temperature: f64,
    },

    /// List all ingested papers
    List,

    /// Search ArXiv and batch-ingest matching papers
    Collect {
        /// Search query
        query: String,

        /// Maximum number of results to collect
        #[arg(long, default_value_t = 10)]
        max_results: u32,

        /// Sort results by
        #[arg(long, default_value = "relevance")]
        sort_by: SortOption,
    },

    /// Search your local corpus
    Find {
        /// Search query
        query: String,

        /// Maximum number of results to return
        #[arg(long, default_value_t = 10)]
        max_results: u32,
    },
}

#[derive(Clone, ValueEnum)]
enum SortOption {
    Relevance,
    Submitted,
    Updated,
}

impl From<SortOption> for SortBy {
    fn from(opt: SortOption) -> Self {
        match opt {
            SortOption::Relevance => SortBy::Relevance,
            SortOption::Submitted => SortBy::SubmittedDate,
            SortOption::Updated => SortBy::LastUpdatedDate,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search {
            query,
            max_results,
            sort_by,
        } => {
            let qb = QueryBuilder::new(&query)
                .max_results(max_results)
                .sort_by(sort_by.into());

            let client = ArxivClient::new()?;
            let result = client.search(&qb).await?;

            display::display_results(&result.papers, result.total_results);
        }

        Commands::Ingest { arxiv_id } => {
            let client = ArxivClient::new()?;
            let store = Store::open()?;

            let updating = store.exists(&arxiv_id)?;
            let (paper, source_url) = client.fetch_by_id(&arxiv_id).await?;
            let fact = SourceFact::from_paper(paper, source_url);
            store.save(&fact)?;

            if updating {
                println!("Updated: {} — {}", fact.arxiv_id, fact.title);
            } else {
                println!("Ingested: {} — {}", fact.arxiv_id, fact.title);
            }
        }

        Commands::Inspect { arxiv_id } => {
            let store = Store::open()?;

            match store.get(&arxiv_id)? {
                Some(fact) => {
                    let versions = store.get_versions(&arxiv_id)?;
                    display::display_fact(&fact, &versions);
                }
                None => bail!("Paper not found: {arxiv_id}"),
            }
        }

        Commands::Extract {
            arxiv_id,
            deterministic,
        } => {
            let store = Store::open()?;

            match store.get(&arxiv_id)? {
                Some(fact) => {
                    let candidates = if deterministic {
                        candidate_assertions(&extract_observations(&fact))
                    } else {
                        let extractor = LlmExtractor::from_env()?;
                        extractor.extract(&fact).await?
                    };
                    let supported = candidates
                        .into_iter()
                        .filter_map(|candidate| candidate.validate().ok())
                        .collect::<Vec<_>>();
                    store.save_assertions(&fact.arxiv_id, &fact.arxiv_version, &supported)?;
                    display::display_extract(&fact.arxiv_id, &supported);
                }
                None => bail!("Paper not found: {arxiv_id}"),
            }
        }

        Commands::Assertions { arxiv_id } => {
            let store = Store::open()?;
            let stored = store.get_assertions(&arxiv_id)?;
            display::display_stored_assertions(&arxiv_id, &stored);
        }

        Commands::Dive {
            concept,
            temperature,
        } => {
            let store = Store::open()?;
            let asserting = store.papers_asserting(&concept)?;
            if asserting.is_empty() {
                display::display_dive(&concept, &[]);
            } else {
                let facts = store.list()?;
                let mut relations = compute_relations(&facts);
                relations.extend(compute_coassertion_relations(
                    &store.all_claims()?,
                    temperature,
                ));
                let nodes = build_dive(&facts, &asserting, &relations);
                display::display_dive(&concept, &nodes);
            }
        }

        Commands::List => {
            let store = Store::open()?;
            let facts = store.list()?;
            display::display_fact_list(&facts);
        }

        Commands::Collect {
            query,
            max_results,
            sort_by,
        } => {
            let qb = QueryBuilder::new(&query)
                .max_results(max_results)
                .sort_by(sort_by.into());

            let source_url = qb.build();
            let client = ArxivClient::new()?;
            let result = client.search(&qb).await?;

            if result.papers.is_empty() {
                display::display_collect_empty();
                return Ok(());
            }

            let store = Store::open()?;
            let mut new_count: u32 = 0;
            let mut updated_count: u32 = 0;

            for paper in result.papers {
                let fact = SourceFact::from_paper(paper, source_url.clone());
                let is_update = store.exists(&fact.arxiv_id)?;
                display::display_collect_item(&fact.arxiv_id, &fact.title, is_update);
                store.save(&fact)?;

                if is_update {
                    updated_count += 1;
                } else {
                    new_count += 1;
                }
            }

            display::display_collect_summary(new_count, updated_count);
        }

        Commands::Find { query, max_results } => {
            let store = Store::open()?;
            let results = store.search(&query, max_results)?;
            display::display_dive_results(&results);
        }
    }

    Ok(())
}

/// Parse and validate the `diver dive --temperature` value: a finite `f64` in the
/// closed range `[0.0, 1.0]`. Rejects NaN, infinities, and out-of-range values.
fn parse_temperature(s: &str) -> Result<f64, String> {
    let t: f64 = s.parse().map_err(|_| format!("`{s}` is not a number"))?;
    if !t.is_finite() {
        return Err(format!("temperature must be a finite number, got `{s}`"));
    }
    if !(0.0..=1.0).contains(&t) {
        return Err(format!("temperature must be in [0.0, 1.0], got `{t}`"));
    }
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_temperature() {
        assert_eq!(parse_temperature("0.0").unwrap(), 0.0);
        assert_eq!(parse_temperature("0.5").unwrap(), 0.5);
        assert_eq!(parse_temperature("1.0").unwrap(), 1.0);

        assert!(parse_temperature("-0.1").is_err(), "below range");
        assert!(parse_temperature("1.1").is_err(), "above range");
        assert!(parse_temperature("NaN").is_err(), "NaN rejected");
        assert!(parse_temperature("inf").is_err(), "infinity rejected");
        assert!(parse_temperature("warm").is_err(), "non-numeric rejected");
    }
}
