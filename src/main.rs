use anyhow::{Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

use diver::client::ArxivClient;
use diver::display;
use diver::fact::SourceFact;
use diver::query::{QueryBuilder, SortBy};
use diver::store::Store;

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

    /// List all ingested papers
    List,
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
                Some(fact) => display::display_fact(&fact),
                None => bail!("Paper not found: {arxiv_id}"),
            }
        }

        Commands::List => {
            let store = Store::open()?;
            let facts = store.list()?;
            display::display_fact_list(&facts);
        }
    }

    Ok(())
}
