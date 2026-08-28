use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use diver::client::ArxivClient;
use diver::display;
use diver::query::{QueryBuilder, SortBy};

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
    }

    Ok(())
}
