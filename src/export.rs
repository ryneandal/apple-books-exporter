use std::io::Write;

use anyhow::Result;
use csv::Writer;

use crate::cli::OutputFormat;
use crate::model::BookRecord;
use crate::timeconv::format_datetime;

const CSV_HEADER: [&str; 13] = [
    "title",
    "author",
    "status",
    "reading_progress",
    "high_watermark_progress",
    "finished_at",
    "last_opened_at",
    "last_engaged_at",
    "library_record_created_at",
    "asset_id",
    "asset_guid",
    "store_id",
    "genre",
];

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
        csv.write_record([
            record.title.clone(),
            record.author.clone().unwrap_or_default(),
            serde_variant(&record.status).to_string(),
            opt_f64(record.reading_progress),
            opt_f64(record.high_watermark_progress),
            opt_dt(record.finished_at.as_ref()),
            opt_dt(record.last_opened_at.as_ref()),
            opt_dt(record.last_engaged_at.as_ref()),
            opt_dt(record.library_record_created_at.as_ref()),
            opt_i64(record.asset_id),
            record.asset_guid.clone().unwrap_or_default(),
            opt_i64(record.store_id),
            record.genre.clone().unwrap_or_default(),
        ])?;
    }

    csv.flush()?;
    Ok(())
}

fn serde_variant(status: &crate::model::BookStatus) -> &'static str {
    match status {
        crate::model::BookStatus::Finished => "finished",
        crate::model::BookStatus::InProgress => "in_progress",
        crate::model::BookStatus::NotStartedOrUnknown => "not_started_or_unknown",
    }
}

fn opt_f64(value: Option<f64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn opt_i64(value: Option<i64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn opt_dt(value: Option<&chrono::DateTime<chrono::Utc>>) -> String {
    value.map(format_datetime).unwrap_or_default()
}
