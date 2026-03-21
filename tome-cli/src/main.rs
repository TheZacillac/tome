use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
enum Format {
    Human,
    Json,
    Yaml,
    Markdown,
}

#[derive(Parser)]
#[command(
    name = "tome",
    about = "A reference database for internet TLDs, DNS record types, and domain name terminology",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(short, long, value_enum, default_value = "human", global = true)]
    format: Format,
}

#[derive(Subcommand)]
enum Commands {
    /// Look up information about a top-level domain
    Tld {
        /// The TLD to look up (e.g., "com", ".uk", "app")
        query: String,
    },

    /// Look up information about a DNS record type
    Record {
        /// The record type to look up (e.g., "A", "MX", "CNAME") or numeric type code
        query: String,
    },

    /// Look up a glossary term
    Glossary {
        /// The term to look up (e.g., "registrar", "DNSSEC", "EPP")
        query: String,
    },

    /// Search across all databases
    Search {
        /// The search query
        query: String,
    },

    /// List entries in a database
    List {
        #[command(subcommand)]
        target: ListTarget,
    },
}

#[derive(Subcommand)]
enum ListTarget {
    /// List all TLDs, optionally filtered by type
    Tlds {
        /// Filter by TLD type
        #[arg(short, long)]
        r#type: Option<String>,
    },

    /// List all DNS record types
    Records {
        /// Show only commonly used record types
        #[arg(short, long)]
        common: bool,
    },

    /// List all glossary terms, optionally filtered by category
    Glossary {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Tld { query } => {
            println!("Looking up TLD: {query}");
            // TODO: Wire up to tome-core TldDatabase
        }
        Commands::Record { query } => {
            println!("Looking up record type: {query}");
            // TODO: Wire up to tome-core RecordTypeDatabase
        }
        Commands::Glossary { query } => {
            println!("Looking up glossary term: {query}");
            // TODO: Wire up to tome-core GlossaryDatabase
        }
        Commands::Search { query } => {
            println!("Searching: {query}");
            // TODO: Wire up cross-database search
        }
        Commands::List { target } => match target {
            ListTarget::Tlds { r#type } => {
                println!("Listing TLDs (filter: {type:?})");
            }
            ListTarget::Records { common } => {
                println!("Listing record types (common only: {common})");
            }
            ListTarget::Glossary { category } => {
                println!("Listing glossary terms (category: {category:?})");
            }
        },
    }

    Ok(())
}
