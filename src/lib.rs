//! Apple Books export CLI implementation.
//!
//! This crate is a CLI tool for exporting Apple Books data from local macOS SQLite databases as JSON or CSV.

mod database;
mod export;
mod model;

use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use export::OutputFormat;

const CLI_BANNER_LINES: &[&str] = &[
    "Apple Books Exporter v0.1.0",
    "Tested/verified as working with Apple Books v8.5 (6570)",
    "Have issues/comments/improvements? Let me know at https://github.com/ryne/apple-books-exporter",
];

#[derive(Debug, Parser)]
#[command(name = "apple-books-data-export")]
#[command(about = "Export Apple Books reading data as JSON or CSV")]
struct Cli {
    #[arg(long, global = true)]
    db: Option<PathBuf>,

    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
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

/// Parse CLI arguments and execute the selected command.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    run_cli(cli, &mut stdout, &mut stderr)
}

fn run_cli(cli: Cli, stdout: &mut impl Write, stderr: &mut impl Write) -> Result<()> {
    print_banner(stderr)?;
    let selected_path = database::resolve_database(cli.db.as_deref(), cli.debug, stderr)?;

    match cli.command {
        Commands::Discover => {
            writeln!(stdout, "{}", selected_path.display())?;
            Ok(())
        }
        Commands::Inspect => print_db_info(selected_path, stdout),
        Commands::Export {
            format,
            output,
            pretty,
        } => export_data(
            selected_path,
            cli.debug,
            format,
            output,
            pretty,
            stdout,
            stderr,
        ),
    }?;

    Ok(())
}

fn print_banner(stderr: &mut impl Write) -> Result<()> {
    for line in CLI_BANNER_LINES {
        writeln!(stderr, "{line}")?;
    }
    writeln!(stderr)?;
    Ok(())
}

fn print_db_info(selected_path: PathBuf, stdout: &mut impl Write) -> Result<()> {
    let row_count = database::count_books(&selected_path)?;
    writeln!(stdout, "database: {}", selected_path.display())?;
    writeln!(stdout, "valid: yes")?;
    writeln!(stdout, "table: {}", database::REQUIRED_TABLE)?;
    writeln!(stdout, "required_columns: all present")?;
    writeln!(stdout, "rows: {row_count}")?;
    Ok(())
}

fn export_data(
    selected_path: PathBuf,
    debug: bool,
    format: OutputFormat,
    output: Option<std::path::PathBuf>,
    pretty: bool,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Result<()> {
    let records = database::extract_books(&selected_path, debug, stderr)?;
    if let Some(path) = output {
        let file = File::create(&path)
            .with_context(|| format!("failed to create output file {}", path.display()))?;
        export::write_records(file, &records, format, pretty)?;
    } else {
        export::write_records(stdout, &records, format, pretty)?;
    }
    Ok(())
}
