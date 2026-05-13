use std::io::Write;

use anyhow::Result;
use clap::ValueEnum;
use csv::Writer;

use crate::model::BookRecord;
use crate::model::format_datetime;

const CSV_HEADER: [&str; 11] = [
    "title",
    "author",
    "status",
    "reading_progress",
    "high_watermark_progress",
    "finished_at",
    "last_opened_at",
    "last_engaged_at",
    "library_record_created_at",
    "asset_guid",
    "genre",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum OutputFormat {
    Json,
    Csv,
}

pub(crate) fn write_records(
    writer: impl Write,
    records: &[BookRecord],
    format: OutputFormat,
    pretty: bool,
) -> Result<()> {
    match format {
        OutputFormat::Json => write_json(writer, records, pretty),
        OutputFormat::Csv => write_csv(writer, records),
    }
}

fn write_json(mut writer: impl Write, records: &[BookRecord], pretty: bool) -> Result<()> {
    if pretty {
        serde_json::to_writer_pretty(&mut writer, records)?;
    } else {
        serde_json::to_writer(&mut writer, records)?;
    }
    writer.write_all(b"\n")?;
    Ok(())
}

fn write_csv(writer: impl Write, records: &[BookRecord]) -> Result<()> {
    let mut csv = Writer::from_writer(writer);
    csv.write_record(CSV_HEADER)?;

    for record in records {
        let format_opt_datetime = |value: Option<&chrono::DateTime<chrono::Utc>>| {
            value.map(format_datetime).unwrap_or_default()
        };

        csv.write_record([
            record.title.clone(),
            record.author.clone().unwrap_or_default(),
            record.status.as_str().to_string(),
            opt_to_string(record.reading_progress),
            opt_to_string(record.high_watermark_progress),
            format_opt_datetime(record.finished_at.as_ref()),
            format_opt_datetime(record.last_opened_at.as_ref()),
            format_opt_datetime(record.last_engaged_at.as_ref()),
            format_opt_datetime(record.library_record_created_at.as_ref()),
            record.asset_guid.clone().unwrap_or_default(),
            record.genre.clone().unwrap_or_default(),
        ])?;
    }

    csv.flush()?;
    Ok(())
}

fn opt_to_string<T: ToString>(value: Option<T>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}
