use std::cmp::Ordering;
use std::env;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OpenFlags, Row, params, types::FromSql};
use walkdir::WalkDir;

use crate::model::{BookRecord, BookStatus, apple_timestamp_to_datetime};

pub(crate) const REQUIRED_TABLE: &str = "ZBKLIBRARYASSET";
pub(crate) const REQUIRED_COLUMNS: &[&str] = &[
    "ZTITLE",
    "ZAUTHOR",
    "ZISFINISHED",
    "ZDATEFINISHED",
    "ZREADINGPROGRESS",
    "ZBOOKHIGHWATERMARKPROGRESS",
    "ZLASTOPENDATE",
    "ZLASTENGAGEDDATE",
    "ZCREATIONDATE",
    "ZASSETID",
    "ZASSETGUID",
    "ZSTOREID",
    "ZGENRE",
];
pub(crate) const PROGRESS_RELATED_COLUMNS: &[&str] = &[
    "ZREADINGPROGRESS",
    "ZBOOKHIGHWATERMARKPROGRESS",
    "ZLASTOPENDATE",
    "ZLASTENGAGEDDATE",
    "ZDATEFINISHED",
];

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

const COL_TITLE: usize = 0;
const COL_AUTHOR: usize = 1;
const COL_IS_FINISHED: usize = 2;
const COL_READING_PROGRESS: usize = 3;
const COL_HIGH_WATERMARK_PROGRESS: usize = 4;
const COL_DATE_FINISHED: usize = 5;
const COL_LAST_OPEN_DATE: usize = 6;
const COL_LAST_ENGAGED_DATE: usize = 7;
const COL_CREATION_DATE: usize = 8;
const COL_ASSET_ID: usize = 9;
const COL_ASSET_GUID: usize = 10;
const COL_STORE_ID: usize = 11;
const COL_GENRE: usize = 12;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgressTableHint {
    pub(crate) table_name: String,
    pub(crate) matching_columns: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidationResult {
    table_present: bool,
    missing_columns: Vec<String>,
    open_error: Option<String>,
}

impl ValidationResult {
    fn missing_table() -> Self {
        Self {
            table_present: false,
            missing_columns: Vec::new(),
            open_error: None,
        }
    }

    fn open_error(message: impl Into<String>) -> Self {
        Self {
            table_present: false,
            missing_columns: Vec::new(),
            open_error: Some(message.into()),
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.table_present && self.missing_columns.is_empty() && self.open_error.is_none()
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(error) = &self.open_error {
            return f.write_str(error);
        }

        if !self.table_present {
            return write!(f, "missing required table {REQUIRED_TABLE}");
        }

        if !self.missing_columns.is_empty() {
            return write!(
                f,
                "missing required columns {}",
                self.missing_columns.join(", ")
            );
        }

        f.write_str("valid")
    }
}

pub(crate) fn open_read_only(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
}

pub(crate) fn resolve_database(
    provided_db: Option<&Path>,
    debug: bool,
    stderr: &mut impl Write,
) -> Result<PathBuf> {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return resolve_database_from_roots(provided_db, &[], debug, stderr);
    };
    let search_roots = vec![
        home.join("Library/Containers/com.apple.iBooksX"),
        home.join("Library/Containers/com.apple.BKAgentService"),
        home.join("Library/Group Containers/group.com.apple.iBooks"),
    ];
    resolve_database_from_roots(provided_db, &search_roots, debug, stderr)
}

pub(crate) fn resolve_database_from_roots(
    provided_db: Option<&Path>,
    search_roots: &[PathBuf],
    debug: bool,
    stderr: &mut impl Write,
) -> Result<PathBuf> {
    if let Some(path) = provided_db {
        return validate_provided_database(path, debug, stderr);
    }

    for root in search_roots {
        let message = format!("search root {}", root.display());
        debugln(debug, stderr, &message)?;
    }

    let candidates = find_candidate_databases(search_roots)?;
    if candidates.is_empty() {
        debugln(debug, stderr, "no candidate files found")?;
    } else {
        for candidate in &candidates {
            let message = format!("candidate file {}", candidate.display());
            debugln(debug, stderr, &message)?;
        }
    }

    if let Some(path) = select_valid_database(&candidates, debug, stderr)? {
        return Ok(path);
    }

    let fallback_candidates = find_all_sqlite_databases(search_roots)?
        .into_iter()
        .filter(|candidate| !candidates.contains(candidate))
        .collect::<Vec<_>>();

    let mut progress_hint_candidates = Vec::new();
    if !fallback_candidates.is_empty() {
        writeln!(
            stderr,
            "No valid database matching pattern BKLibrary*.sqlite found. Scanning other sqlite files for Apple Books schema."
        )?;
    }

    for candidate in fallback_candidates {
        let debug_message = format!("fallback sqlite file {}", candidate.display());
        debugln(debug, stderr, &debug_message)?;
        let validation = validate_database(&candidate);
        let debug_message = format!(
            "fallback validation {} -> {}",
            candidate.display(),
            validation
        );
        debugln(debug, stderr, &debug_message)?;

        if validation.is_valid() {
            writeln!(
                stderr,
                "Selected valid Apple Books database with nonstandard filename: {}",
                candidate.display()
            )?;
            return Ok(candidate);
        }

        let tables = match open_read_only(&candidate) {
            Ok(conn) => identify_book_progress_table(&conn).unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        if !tables.is_empty() {
            progress_hint_candidates.push((candidate, tables));
        }
    }

    bail!(no_valid_database_message(
        search_roots,
        &progress_hint_candidates
    ));
}

pub(crate) fn count_books(path: &Path) -> Result<i64> {
    let conn = open_read_only(path)?;
    let row_count = conn.query_row(
        &format!("SELECT COUNT(*) FROM {REQUIRED_TABLE}"),
        [],
        |row| row.get(0),
    )?;
    Ok(row_count)
}

pub(crate) fn extract_books(
    path: &Path,
    debug: bool,
    stderr: &mut impl Write,
) -> Result<Vec<BookRecord>> {
    let conn = open_read_only(path)?;
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

    while let Some(row) = rows.next()? {
        records.push(map_row(row));
    }

    sort_records(&mut records);

    if debug {
        writeln!(stderr, "debug: row count exported {}", records.len())?;
    }

    Ok(records)
}

fn validate_database(path: &Path) -> ValidationResult {
    match open_read_only(path) {
        Ok(conn) => match validate_connection(&conn) {
            Ok(result) => result,
            Err(err) => ValidationResult::open_error(format!("failed to inspect schema: {err}")),
        },
        Err(err) => ValidationResult::open_error(format!("failed to open sqlite database: {err}")),
    }
}

fn validate_connection(conn: &Connection) -> rusqlite::Result<ValidationResult> {
    let table_present = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        params![REQUIRED_TABLE],
        |row| row.get::<_, i64>(0),
    )? == 1;

    if !table_present {
        return Ok(ValidationResult::missing_table());
    }

    let mut statement = conn.prepare(&format!("PRAGMA table_info({REQUIRED_TABLE})"))?;
    let column_names = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let missing_columns = REQUIRED_COLUMNS
        .iter()
        .filter(|required| !column_names.iter().any(|column| column == **required))
        .map(|column| (*column).to_string())
        .collect::<Vec<_>>();

    Ok(ValidationResult {
        table_present: true,
        missing_columns,
        open_error: None,
    })
}

fn identify_book_progress_table(conn: &Connection) -> rusqlite::Result<Vec<ProgressTableHint>> {
    let mut statement = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )?;
    let table_names = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut hints = Vec::new();
    for table_name in table_names {
        let pragma = format!("PRAGMA table_info(\"{}\")", table_name.replace('"', "\"\""));
        let mut table_info = conn.prepare(&pragma)?;
        let columns = table_info
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let matching_columns = PROGRESS_RELATED_COLUMNS
            .iter()
            .filter(|required| columns.iter().any(|column| column == **required))
            .map(|column| (*column).to_string())
            .collect::<Vec<_>>();

        if !matching_columns.is_empty() {
            hints.push(ProgressTableHint {
                table_name,
                matching_columns,
            });
        }
    }
    Ok(hints)
}

fn map_row(row: &Row<'_>) -> BookRecord {
    let title = read_required_text(row, COL_TITLE);
    let author = read_opt::<String>(row, COL_AUTHOR);
    let is_finished = read_opt::<i64>(row, COL_IS_FINISHED);
    let reading_progress = read_opt::<f64>(row, COL_READING_PROGRESS);
    let high_watermark_progress = read_opt::<f64>(row, COL_HIGH_WATERMARK_PROGRESS);

    let finished_date_raw = read_opt::<f64>(row, COL_DATE_FINISHED);
    let last_opened_raw = read_opt::<f64>(row, COL_LAST_OPEN_DATE);
    let last_engaged_raw = read_opt::<f64>(row, COL_LAST_ENGAGED_DATE);
    let created_raw = read_opt::<f64>(row, COL_CREATION_DATE);

    let finished_date_present = finished_date_raw.is_some();
    let status = BookStatus::derive(is_finished, finished_date_present, reading_progress);
    let (reading_progress, high_watermark_progress) = normalized_progress(
        status == BookStatus::Finished,
        reading_progress,
        high_watermark_progress,
    );

    BookRecord {
        title,
        author,
        status,
        reading_progress,
        high_watermark_progress,
        finished_at: apple_timestamp_to_datetime(finished_date_raw),
        last_opened_at: apple_timestamp_to_datetime(last_opened_raw),
        last_engaged_at: apple_timestamp_to_datetime(last_engaged_raw),
        library_record_created_at: apple_timestamp_to_datetime(created_raw),
        asset_id: read_opt::<i64>(row, COL_ASSET_ID),
        asset_guid: read_opt::<String>(row, COL_ASSET_GUID),
        store_id: read_opt::<i64>(row, COL_STORE_ID),
        genre: read_opt::<String>(row, COL_GENRE),
    }
}

fn read_required_text(row: &Row<'_>, index: usize) -> String {
    read_opt::<String>(row, index).unwrap_or_default()
}

fn read_opt<T: FromSql>(row: &Row<'_>, index: usize) -> Option<T> {
    row.get::<_, Option<T>>(index).ok().flatten()
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

fn normalized_progress(
    finished: bool,
    reading_progress: Option<f64>,
    high_watermark_progress: Option<f64>,
) -> (Option<f64>, Option<f64>) {
    if finished {
        (Some(1.0), Some(1.0))
    } else {
        let normalize = |value: Option<f64>| {
            value.and_then(|value| {
                value
                    .is_finite()
                    .then_some((value * 100_000.0).round() / 100_000.0)
            })
        };
        (
            normalize(reading_progress),
            normalize(high_watermark_progress),
        )
    }
}

fn find_candidate_databases(roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
    find_sqlite_databases_matching(roots, |file_name| {
        file_name.starts_with("BKLibrary") && file_name.ends_with(".sqlite")
    })
}

fn find_all_sqlite_databases(roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
    find_sqlite_databases_matching(roots, |file_name| file_name.ends_with(".sqlite"))
}

fn find_sqlite_databases_matching(
    roots: &[PathBuf],
    predicate: impl Fn(&str) -> bool,
) -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }

        for entry in WalkDir::new(root).follow_links(false) {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy();
            if predicate(&file_name) {
                candidates.push(normalize_path(entry.path())?);
            }
        }
    }

    candidates.sort();
    candidates.dedup();
    Ok(candidates)
}

fn select_valid_database(
    candidates: &[PathBuf],
    debug: bool,
    stderr: &mut impl Write,
) -> Result<Option<PathBuf>> {
    for candidate in candidates {
        let validation = validate_database(candidate);
        if debug {
            writeln!(
                stderr,
                "debug: validation {} -> {}",
                candidate.display(),
                validation
            )?;
        }
        if validation.is_valid() {
            if debug {
                writeln!(stderr, "debug: selected db {}", candidate.display())?;
            }
            return Ok(Some(candidate.clone()));
        }
    }

    Ok(None)
}

fn validate_provided_database(
    path: &Path,
    debug: bool,
    stderr: &mut impl Write,
) -> Result<PathBuf> {
    let absolute_path = normalize_path(path)?;
    let debug_message = format!("validating provided database {}", absolute_path.display());
    debugln(debug, stderr, &debug_message)?;

    let validation = validate_database(&absolute_path);
    if validation.is_valid() {
        return Ok(absolute_path);
    }

    bail!(
        "Provided database is not a supported Apple Books BKLibrary database:\n  {}",
        validation
    );
}

fn debugln(debug: bool, stderr: &mut impl Write, message: &str) -> Result<()> {
    if debug {
        writeln!(stderr, "debug: {message}")?;
    }
    Ok(())
}

fn normalize_path(path: &Path) -> Result<PathBuf> {
    if path.starts_with("~") {
        let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
            bail!("unable to expand ~ without HOME set");
        };
        let without_tilde = path
            .strip_prefix("~")
            .context("failed to strip leading ~")?;
        return Ok(home.join(without_tilde));
    }

    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn no_valid_database_message(
    search_roots: &[PathBuf],
    progress_hint_candidates: &[(PathBuf, Vec<ProgressTableHint>)],
) -> String {
    let mut message = String::from("No valid Apple Books database found.\nChecked search roots:");
    for root in search_roots {
        message.push_str("\n  ");
        message.push_str(&root.display().to_string());
    }

    if !progress_hint_candidates.is_empty() {
        message.push_str("\nFound other .sqlite databases with progress-related columns:");
        for (path, tables) in progress_hint_candidates {
            message.push_str("\n  ");
            message.push_str(&path.display().to_string());
            for table in tables {
                message.push_str("\n    table ");
                message.push_str(&table.table_name);
                message.push_str(": ");
                message.push_str(&table.matching_columns.join(", "));
            }
        }
    }

    message
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::TimeZone;
    use rusqlite::Connection;
    use tempfile::{TempDir, tempdir_in};

    use super::{
        REQUIRED_COLUMNS, REQUIRED_TABLE, extract_books_from_connection,
        resolve_database_from_roots,
    };

    fn test_tempdir() -> TempDir {
        tempdir_in(Path::new("/tmp")).expect("tempdir")
    }

    #[test]
    fn selects_first_valid_candidate() {
        let tempdir = test_tempdir();
        let valid = tempdir.path().join("BKLibrary-a.sqlite");
        create_valid_library_db(&valid);

        let mut stderr = Vec::new();
        let selected =
            resolve_database_from_roots(None, &[tempdir.path().to_path_buf()], false, &mut stderr)
                .expect("should resolve");

        assert_eq!(selected, valid);
    }

    #[test]
    fn selects_valid_non_bklibrary_sqlite_as_fallback() {
        let tempdir = test_tempdir();
        let valid = tempdir.path().join("library-cache.sqlite");
        create_valid_library_db(&valid);

        let mut stderr = Vec::new();
        let selected =
            resolve_database_from_roots(None, &[tempdir.path().to_path_buf()], false, &mut stderr)
                .expect("should resolve");

        assert_eq!(selected, valid);
        let stderr_output = String::from_utf8(stderr).expect("utf8");
        assert!(
            stderr_output.contains("No valid database matching pattern BKLibrary*.sqlite found")
        );
        assert!(stderr_output.contains("nonstandard filename"));
    }

    #[test]
    fn skips_invalid_candidate_and_selects_later_valid_match() {
        let tempdir = test_tempdir();
        let invalid = tempdir.path().join("BKLibrary-a.sqlite");
        let valid = tempdir.path().join("BKLibrary-b.sqlite");

        std::fs::write(&invalid, "not sqlite").expect("write invalid candidate");
        create_valid_library_db(&valid);

        let mut stderr = Vec::new();
        let selected =
            resolve_database_from_roots(None, &[tempdir.path().to_path_buf()], false, &mut stderr)
                .expect("should resolve");

        assert_eq!(selected, valid);
    }

    #[test]
    fn provided_db_override_bypasses_discovery() {
        let tempdir = test_tempdir();
        let provided = tempdir.path().join("custom.sqlite");
        create_valid_library_db(&provided);

        let roots = vec![tempdir.path().join("does-not-exist")];
        let mut stderr = Vec::new();
        let selected = resolve_database_from_roots(Some(&provided), &roots, false, &mut stderr)
            .expect("should resolve");

        assert_eq!(selected, provided);
    }

    #[test]
    fn reports_progress_related_sqlite_hints_when_no_valid_database_is_found() {
        let tempdir = test_tempdir();
        let hinted = tempdir.path().join("reading-history.sqlite");
        create_progress_hint_db(&hinted);

        let mut stderr = Vec::new();
        let err =
            resolve_database_from_roots(None, &[tempdir.path().to_path_buf()], false, &mut stderr)
                .expect_err("should fail");

        let message = err.to_string();
        assert!(message.contains("progress-related columns"));
        assert!(message.contains("reading-history.sqlite"));
        assert!(message.contains("ZREADINGPROGRESS"));
        assert!(message.contains("ZLASTENGAGEDDATE"));
    }

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
    }

    #[test]
    fn treats_type_mismatches_as_missing_optional_values() {
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
        assert!(debug_output.contains("row count exported 1"));
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

    fn create_valid_library_db(path: &Path) {
        let conn = Connection::open(path).expect("create sqlite db");
        let columns = REQUIRED_COLUMNS
            .iter()
            .map(|column| {
                let ty = match *column {
                    "ZISFINISHED" | "ZASSETID" | "ZSTOREID" => "INTEGER",
                    "ZREADINGPROGRESS"
                    | "ZBOOKHIGHWATERMARKPROGRESS"
                    | "ZDATEFINISHED"
                    | "ZLASTOPENDATE"
                    | "ZLASTENGAGEDDATE"
                    | "ZCREATIONDATE" => "REAL",
                    _ => "TEXT",
                };
                format!("{column} {ty}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        conn.execute(&format!("CREATE TABLE {REQUIRED_TABLE} ({columns})"), [])
            .expect("create schema");
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

    fn create_progress_hint_db(path: &Path) {
        let conn = Connection::open(path).expect("create sqlite db");
        conn.execute(
            "CREATE TABLE reading_state (ZREADINGPROGRESS REAL, ZLASTENGAGEDDATE REAL, OTHER TEXT)",
            [],
        )
        .expect("create schema");
    }

    fn null<T>() -> Option<T> {
        None
    }
}
