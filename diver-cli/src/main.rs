use anyhow::{Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

use diver_core::assertion::candidate_assertions;
use diver_core::client::ArxivClient;
use diver_core::display;
use diver_core::extract::LlmExtractor;
use diver_core::fact::SourceFact;
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
