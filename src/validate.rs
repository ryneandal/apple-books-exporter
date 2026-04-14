use std::fmt;
use std::path::Path;

use rusqlite::{Connection, OpenFlags, params};

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

pub(crate) fn validate_database(path: &Path) -> ValidationResult {
    match open_read_only(path) {
        Ok(conn) => match validate_connection(&conn) {
            Ok(result) => result,
            Err(err) => ValidationResult::open_error(format!("failed to inspect schema: {err}")),
        },
        Err(err) => ValidationResult::open_error(format!("failed to open sqlite database: {err}")),
    }
}

pub(crate) fn validate_connection(conn: &Connection) -> rusqlite::Result<ValidationResult> {
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

pub(crate) fn find_progress_table_hints(path: &Path) -> Vec<ProgressTableHint> {
    match open_read_only(path) {
        Ok(conn) => find_progress_table_hints_in_connection(&conn).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn find_progress_table_hints_in_connection(
    conn: &Connection,
) -> rusqlite::Result<Vec<ProgressTableHint>> {
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

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{
        REQUIRED_COLUMNS, REQUIRED_TABLE, find_progress_table_hints_in_connection,
        validate_connection,
    };

    #[test]
    fn accepts_valid_schema() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute(
            &format!("CREATE TABLE {REQUIRED_TABLE} ({})", required_columns_sql()),
            [],
        )
        .expect("create table");

        let result = validate_connection(&conn).expect("validation should succeed");
        assert!(result.is_valid());
    }

    #[test]
    fn rejects_missing_table() {
        let conn = Connection::open_in_memory().expect("in-memory db");

        let result = validate_connection(&conn).expect("validation should succeed");
        assert!(!result.table_present);
    }

    #[test]
    fn rejects_missing_columns() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute(
            &format!("CREATE TABLE {REQUIRED_TABLE} (ZTITLE TEXT, ZAUTHOR TEXT)"),
            [],
        )
        .expect("create table");

        let result = validate_connection(&conn).expect("validation should succeed");
        assert!(!result.is_valid());
        assert_eq!(
            result.to_string(),
            "missing required columns ZISFINISHED, ZDATEFINISHED, ZREADINGPROGRESS, ZBOOKHIGHWATERMARKPROGRESS, ZLASTOPENDATE, ZLASTENGAGEDDATE, ZCREATIONDATE, ZASSETID, ZASSETGUID, ZSTOREID, ZGENRE"
        );
    }

    #[test]
    fn finds_progress_related_columns_in_other_tables() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute(
            "CREATE TABLE reading_state (ZREADINGPROGRESS REAL, ZLASTENGAGEDDATE REAL, OTHER TEXT)",
            [],
        )
        .expect("create table");

        let hints =
            find_progress_table_hints_in_connection(&conn).expect("schema hint lookup should work");

        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].table_name, "reading_state");
        assert_eq!(
            hints[0].matching_columns,
            vec![
                "ZREADINGPROGRESS".to_string(),
                "ZLASTENGAGEDDATE".to_string()
            ]
        );
    }

    fn required_columns_sql() -> String {
        REQUIRED_COLUMNS
            .iter()
            .map(|column| format!("{column} TEXT"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
