use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "apple-books-data-export")]
#[command(about = "Export Apple Books reading data as JSON or CSV")]
pub struct Cli {
    #[arg(long, global = true)]
    pub db: Option<PathBuf>,

    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Discover,
    Inspect,
    Export {
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,

        #[arg(long)]
        output: Option<PathBuf>,

        #[arg(long)]
        pretty: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Csv,
}
