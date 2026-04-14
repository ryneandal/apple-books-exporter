//! Apple Books export CLI implementation.
//! 
//! This crate is a CLI tool for exporting Apple Books data from local macOS SQLite databases as JSON or CSV.

mod cli;
mod db;
mod discover;
mod export;
mod model;
mod timeconv;
mod validate;

use std::fs::File;
use std::io::{self, Write};

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};

/// Parse CLI arguments and execute the selected command.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    run_cli(cli, &mut stdout, &mut stderr)
}

fn run_cli(cli: Cli, stdout: &mut impl Write, stderr: &mut impl Write) -> Result<()> {
    writeln!(stderr, "Apple Books Exporter v0.1.0")?;
    writeln!(
        stderr,
        "Tested/verified as working with Apple Books v8.5 (6570)"
    )?;
    writeln!(
        stderr,
        "Have issues/comments/improvements? Let me know at https://github.com/ryne/apple-books-exporter"
    )?;
    writeln!(stderr, "")?;

    match cli.command {
        Commands::Discover => {
            let selected = discover::resolve_database(cli.db.as_deref(), cli.debug, stderr)?;
            writeln!(stdout, "{}", selected.path.display())?;
        }
        Commands::Inspect => {
            let selected = discover::resolve_database(cli.db.as_deref(), cli.debug, stderr)?;
            let row_count = db::count_books(&selected.path)?;

            writeln!(stdout, "database: {}", selected.path.display())?;
            writeln!(stdout, "valid: yes")?;
            writeln!(stdout, "table: {}", validate::REQUIRED_TABLE)?;
            writeln!(stdout, "required_columns: all present")?;
            writeln!(stdout, "rows: {row_count}")?;
        }
        Commands::Export {
            format,
            output,
            pretty,
        } => {
            let selected = discover::resolve_database(cli.db.as_deref(), cli.debug, stderr)?;
            let records = db::extract_books(&selected.path, cli.debug, stderr)?;

            match output {
                Some(path) => {
                    let file = File::create(&path).with_context(|| {
                        format!("failed to create output file {}", path.display())
                    })?;
                    export::write_records(file, &records, format, pretty)?;
                }
                None => export::write_records(stdout, &records, format, pretty)?,
            }
        }
    }

    Ok(())
}
