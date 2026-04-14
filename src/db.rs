use std::cmp::Ordering;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use rusqlite::{Connection, Row, types::ValueRef};

use crate::model::{BookRecord, BookStatus};
use crate::timeconv::apple_timestamp_to_datetime;
use crate::validate;

/// As of Version 8.5 (6570), this is the progress schema.
const EXTRACTION_QUERY: &str = r#"
SELECT
    ZTITLE,
    ZAUTHOR,
    ZISFINISHED,
    ZREADINGPROGRESS,
    ZBOOKHIGHWATERMARKPROGRESS,
    ZDATEFINISHED,
    ZLASTOPENDATE,
    ZLASTENGAGEDDATE,
    ZCREATIONDATE,
    ZASSETID,
    ZASSETGUID,
    ZSTOREID,
    ZGENRE
FROM ZBKLIBRARYASSET
"#;

pub(crate) fn count_books(path: &Path) -> Result<i64> {
    let conn = validate::open_read_only(path)?;
    let row_count = conn.query_row("SELECT COUNT(*) FROM ZBKLIBRARYASSET", [], |row| row.get(0))?;
    Ok(row_count)
}

pub(crate) fn extract_books(
    path: &Path,
    debug: bool,
    stderr: &mut impl Write,
) -> Result<Vec<BookRecord>> {
    let conn = validate::open_read_only(path)?;
    extract_books_from_connection(&conn, debug, stderr)
}

pub(crate) fn extract_books_from_connection(
    conn: &Connection,
    debug: bool,
    stderr: &mut impl Write,
) -> Result<Vec<BookRecord>> {
    let mut statement = conn.prepare(EXTRACTION_QUERY)?;
    let mut rows = statement.query([])?;
    let mut records = Vec::new();
    let mut row_index = 0usize;

    while let Some(row) = rows.next()? {
        row_index += 1;
        let (record, warnings) = map_row(row, row_index);
        if debug {
            for warning in warnings {
                writeln!(stderr, "debug: {warning}")?;
            }
        }
        records.push(record);
    }

    sort_records(&mut records);

    if debug {
        writeln!(stderr, "debug: row count exported {}", records.len())?;
    }

    Ok(records)
}

fn map_row(row: &Row<'_>, row_index: usize) -> (BookRecord, Vec<String>) {
    let mut warnings = Vec::new();

    let title = read_required_text(row, 0, "ZTITLE", row_index, &mut warnings);
    let author = read_optional_text(row, 1, "ZAUTHOR", row_index, &mut warnings);
    let is_finished = read_optional_i64(row, 2, "ZISFINISHED", row_index, &mut warnings);
    let reading_progress = read_optional_f64(row, 3, "ZREADINGPROGRESS", row_index, &mut warnings);
    let high_watermark_progress = read_optional_f64(
        row,
        4,
        "ZBOOKHIGHWATERMARKPROGRESS",
        row_index,
        &mut warnings,
    );

    let finished_date_raw = read_optional_f64(row, 5, "ZDATEFINISHED", row_index, &mut warnings);
    let last_opened_raw = read_optional_f64(row, 6, "ZLASTOPENDATE", row_index, &mut warnings);
    let last_engaged_raw = read_optional_f64(row, 7, "ZLASTENGAGEDDATE", row_index, &mut warnings);
    let created_raw = read_optional_f64(row, 8, "ZCREATIONDATE", row_index, &mut warnings);

    let finished_date_present = !matches!(safe_value_ref(row, 5), ValueRef::Null);

    let status = BookStatus::derive(is_finished, finished_date_present, reading_progress);
    let (reading_progress, high_watermark_progress) = normalized_progress(
        status == BookStatus::Finished,
        reading_progress,
        high_watermark_progress,
    );

    let record = BookRecord {
        title,
        author,
        status,
        reading_progress,
        high_watermark_progress,
        finished_at: apple_timestamp_to_datetime(finished_date_raw),
        last_opened_at: apple_timestamp_to_datetime(last_opened_raw),
        last_engaged_at: apple_timestamp_to_datetime(last_engaged_raw),
        library_record_created_at: apple_timestamp_to_datetime(created_raw),
        asset_id: read_optional_i64(row, 9, "ZASSETID", row_index, &mut warnings),
        asset_guid: read_optional_text(row, 10, "ZASSETGUID", row_index, &mut warnings),
        store_id: read_optional_i64(row, 11, "ZSTOREID", row_index, &mut warnings),
        genre: read_optional_text(row, 12, "ZGENRE", row_index, &mut warnings),
    };

    (record, warnings)
}

fn read_required_text(
    row: &Row<'_>,
    index: usize,
    field: &str,
    row_index: usize,
    warnings: &mut Vec<String>,
) -> String {
    read_optional_text(row, index, field, row_index, warnings).unwrap_or_else(|| {
        warnings.push(format!(
            "row {row_index}: {field} was null or unreadable; using empty string"
        ));
        String::new()
    })
}

fn read_optional_text(
    row: &Row<'_>,
    index: usize,
    field: &str,
    row_index: usize,
    warnings: &mut Vec<String>,
) -> Option<String> {
    match safe_value_ref(row, index) {
        ValueRef::Null => None,
        ValueRef::Text(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
        ValueRef::Integer(value) => {
            warnings.push(format!(
                "row {row_index}: {field} was integer; coerced to text"
            ));
            Some(value.to_string())
        }
        ValueRef::Real(value) => {
            warnings.push(format!(
                "row {row_index}: {field} was real; coerced to text"
            ));
            Some(value.to_string())
        }
        ValueRef::Blob(_) => {
            warnings.push(format!(
                "row {row_index}: {field} was blob; treating as null"
            ));
            None
        }
    }
}

fn read_optional_i64(
    row: &Row<'_>,
    index: usize,
    field: &str,
    row_index: usize,
    warnings: &mut Vec<String>,
) -> Option<i64> {
    match safe_value_ref(row, index) {
        ValueRef::Null => None,
        ValueRef::Integer(value) => Some(value),
        ValueRef::Real(value) if value.is_finite() && value.fract() == 0.0 => Some(value as i64),
        ValueRef::Real(_) => {
            warnings.push(format!(
                "row {row_index}: {field} had non-integral real value; treating as null"
            ));
            None
        }
        ValueRef::Text(bytes) => match String::from_utf8_lossy(bytes).parse::<i64>() {
            Ok(value) => {
                warnings.push(format!(
                    "row {row_index}: {field} was text; parsed as integer"
                ));
                Some(value)
            }
            Err(_) => {
                warnings.push(format!(
                    "row {row_index}: {field} had unparseable text; treating as null"
                ));
                None
            }
        },
        ValueRef::Blob(_) => {
            warnings.push(format!(
                "row {row_index}: {field} was blob; treating as null"
            ));
            None
        }
    }
}

fn read_optional_f64(
    row: &Row<'_>,
    index: usize,
    field: &str,
    row_index: usize,
    warnings: &mut Vec<String>,
) -> Option<f64> {
    match safe_value_ref(row, index) {
        ValueRef::Null => None,
        ValueRef::Integer(value) => Some(value as f64),
        ValueRef::Real(value) if value.is_finite() => Some(value),
        ValueRef::Real(_) => {
            warnings.push(format!(
                "row {row_index}: {field} had non-finite real value; treating as null"
            ));
            None
        }
        ValueRef::Text(bytes) => match String::from_utf8_lossy(bytes).parse::<f64>() {
            Ok(value) if value.is_finite() => {
                warnings.push(format!(
                    "row {row_index}: {field} was text; parsed as float"
                ));
                Some(value)
            }
            _ => {
                warnings.push(format!(
                    "row {row_index}: {field} had unparseable text; treating as null"
                ));
                None
            }
        },
        ValueRef::Blob(_) => {
            warnings.push(format!(
                "row {row_index}: {field} was blob; treating as null"
            ));
            None
        }
    }
}

fn safe_value_ref<'a>(row: &'a Row<'_>, index: usize) -> ValueRef<'a> {
    row.get_ref(index).unwrap_or(ValueRef::Null)
}

fn sort_records(records: &mut [BookRecord]) {
    records.sort_by(|left, right| {
        compare_desc_option_datetime(left.finished_at.as_ref(), right.finished_at.as_ref())
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.author.cmp(&right.author))
            .then_with(|| left.asset_id.cmp(&right.asset_id))
            .then_with(|| left.asset_guid.cmp(&right.asset_guid))
    });
}

fn compare_desc_option_datetime(
    left: Option<&chrono::DateTime<chrono::Utc>>,
    right: Option<&chrono::DateTime<chrono::Utc>>,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.cmp(left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn round_progress(value: Option<f64>) -> Option<f64> {
    let value = value?;
    if !value.is_finite() {
        return Some(value);
    }

    Some((value * 100_000.0).round() / 100_000.0)
}

fn normalized_progress(
    finished: bool,
    reading_progress: Option<f64>,
    high_watermark_progress: Option<f64>,
) -> (Option<f64>, Option<f64>) {
    if finished {
        (Some(1.0), Some(1.0))
    } else {
        (
            round_progress(reading_progress),
            round_progress(high_watermark_progress),
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use rusqlite::Connection;

    use super::extract_books_from_connection;
    use crate::validate::REQUIRED_TABLE;

    #[test]
    fn maps_full_valid_row() {
        let conn = fixture_connection();
        let finished_at = chrono::Utc
            .with_ymd_and_hms(2023, 10, 11, 22, 18, 28)
            .single()
            .expect("valid date")
            .timestamp() as f64
            - 978_307_200.0;
        conn.execute(
            &format!(
                "INSERT INTO {REQUIRED_TABLE} VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            ),
            rusqlite::params![
                "The Great Gatsby",
                "F. Scott Fitzgerald",
                1,
                0.93,
                0.95,
                finished_at,
                719_845_287.0,
                finished_at,
                748_980_784.0,
                914_355_894_i64,
                "0511A38E-2F6B-4521-8116-D1E4FB0324AC",
                914_355_894_i64,
                "Classics"
            ],
        )
        .expect("insert row");

        let mut stderr = Vec::new();
        let books = extract_books_from_connection(&conn, true, &mut stderr).expect("extract");
        let book = books.first().expect("record");

        assert_eq!(book.title, "The Great Gatsby");
        assert_eq!(book.author.as_deref(), Some("F. Scott Fitzgerald"));
        assert_eq!(book.status, crate::model::BookStatus::Finished);
        assert_eq!(book.reading_progress, Some(1.0));
        assert_eq!(book.high_watermark_progress, Some(1.0));
        assert_eq!(book.asset_id, Some(914_355_894));
        assert_eq!(
            crate::timeconv::format_datetime(
                book.finished_at.as_ref().expect("finished timestamp")
            ),
            "2023-10-11T22:18:28Z"
        );
        let debug_output = String::from_utf8(stderr).expect("utf8");
        assert!(debug_output.contains("row count exported 1"));
    }

    #[test]
    fn degrades_safely_for_malformed_optional_values() {
        let conn = fixture_connection();
        conn.execute(
            &format!(
                "INSERT INTO {REQUIRED_TABLE} VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            ),
            rusqlite::params![
                "Book",
                "Author",
                0,
                "not-a-number",
                0.4,
                null::<i64>(),
                "also-bad",
                null::<i64>(),
                null::<i64>(),
                12_i64,
                "GUID",
                12_i64,
                "Genre"
            ],
        )
        .expect("insert row");

        let mut stderr = Vec::new();
        let books = extract_books_from_connection(&conn, true, &mut stderr).expect("extract");
        let book = books.first().expect("record");

        assert_eq!(book.reading_progress, None);
        assert_eq!(book.finished_at, None);
        assert_eq!(book.status, crate::model::BookStatus::NotStartedOrUnknown);
        let debug_output = String::from_utf8(stderr).expect("utf8");
        assert!(debug_output.contains("ZREADINGPROGRESS"));
        assert!(debug_output.contains("ZLASTOPENDATE"));
    }

    #[test]
    fn rounds_progress_fields_to_five_significant_figures() {
        let conn = fixture_connection();
        conn.execute(
            &format!(
                "INSERT INTO {REQUIRED_TABLE} VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            ),
            rusqlite::params![
                "Book",
                "Author",
                0,
                0.9060149788856506_f64,
                0.9060149788856506_f64,
                null::<i64>(),
                null::<i64>(),
                null::<i64>(),
                null::<i64>(),
                12_i64,
                "GUID",
                12_i64,
                "Genre"
            ],
        )
        .expect("insert row");

        let mut stderr = Vec::new();
        let books = extract_books_from_connection(&conn, false, &mut stderr).expect("extract");
        let book = books.first().expect("record");

        assert_eq!(book.reading_progress, Some(0.90601));
        assert_eq!(book.high_watermark_progress, Some(0.90601));
    }

    #[test]
    fn finished_books_force_progress_fields_to_one_hundred_percent() {
        let conn = fixture_connection();
        conn.execute(
            &format!(
                "INSERT INTO {REQUIRED_TABLE} VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            ),
            rusqlite::params![
                "Finished Book",
                "Author",
                1,
                0.9060149788856506_f64,
                0.8123456789_f64,
                null::<i64>(),
                null::<i64>(),
                null::<i64>(),
                null::<i64>(),
                12_i64,
                "GUID",
                12_i64,
                "Genre"
            ],
        )
        .expect("insert row");

        let mut stderr = Vec::new();
        let books = extract_books_from_connection(&conn, false, &mut stderr).expect("extract");
        let book = books.first().expect("record");

        assert_eq!(book.status, crate::model::BookStatus::Finished);
        assert_eq!(book.reading_progress, Some(1.0));
        assert_eq!(book.high_watermark_progress, Some(1.0));
    }

    fn fixture_connection() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute(
            &format!(
                "CREATE TABLE {REQUIRED_TABLE} (
                    ZTITLE TEXT,
                    ZAUTHOR TEXT,
                    ZISFINISHED INTEGER,
                    ZREADINGPROGRESS REAL,
                    ZBOOKHIGHWATERMARKPROGRESS REAL,
                    ZDATEFINISHED REAL,
                    ZLASTOPENDATE REAL,
                    ZLASTENGAGEDDATE REAL,
                    ZCREATIONDATE REAL,
                    ZASSETID INTEGER,
                    ZASSETGUID TEXT,
                    ZSTOREID INTEGER,
                    ZGENRE TEXT
                )"
            ),
            [],
        )
        .expect("create table");
        conn
    }

    fn null<T>() -> Option<T> {
        None
    }
}
