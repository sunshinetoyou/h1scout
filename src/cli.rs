use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "h1scout", about = "HackerOne bug bounty program selector")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Fetch programs and scopes from HackerOne API
    Fetch {
        /// Force refresh even if cache is fresh
        #[arg(long)]
        force: bool,
        /// Print what would be fetched without actually fetching
        #[arg(long)]
        dry_run: bool,
    },
    /// List and rank programs
    List {
        /// Show only top N results
        #[arg(long)]
        top: Option<usize>,
        /// Filter by type
        #[arg(long, value_enum)]
        filter: Vec<FilterType>,
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Export results to a file
    Export {
        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,
        /// Output file path
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Clone, ValueEnum)]
pub enum FilterType {
    Android,
    Mobility,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}
