use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::validate::{self, ValidationResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DatabaseSource {
    Provided,
    Discovered,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SelectedDatabase {
    pub(crate) path: PathBuf,
    pub(crate) source: DatabaseSource,
    pub(crate) validation: ValidationResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProgressHintCandidate {
    path: PathBuf,
    tables: Vec<validate::ProgressTableHint>,
}

pub(crate) fn default_search_roots() -> Vec<PathBuf> {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };

    vec![
        home.join("Library/Containers/com.apple.iBooksX"),
        home.join("Library/Containers/com.apple.BKAgentService"),
        home.join("Library/Group Containers/group.com.apple.iBooks"),
    ]
}

pub(crate) fn resolve_database(
    provided_db: Option<&Path>,
    debug: bool,
    stderr: &mut impl Write,
) -> Result<SelectedDatabase> {
    let search_roots = default_search_roots();
    resolve_database_from_roots(provided_db, &search_roots, debug, stderr)
}

pub(crate) fn resolve_database_from_roots(
    provided_db: Option<&Path>,
    search_roots: &[PathBuf],
    debug: bool,
    stderr: &mut impl Write,
) -> Result<SelectedDatabase> {
    if let Some(path) = provided_db {
        let absolute_path = normalize_path(path)?;
        if debug {
            writeln!(
                stderr,
                "debug: validating provided database {}",
                absolute_path.display()
            )?;
        }

        let validation = validate::validate_database(&absolute_path);
        if validation.is_valid() {
            return Ok(SelectedDatabase {
                path: absolute_path,
                source: DatabaseSource::Provided,
                validation,
            });
        }

        bail!(
            "Provided database is not a supported Apple Books BKLibrary database:\n  {}",
            validation
        );
    }

    if debug {
        for root in search_roots {
            writeln!(stderr, "debug: search root {}", root.display())?;
        }
    }

    let candidates = find_candidate_databases(search_roots)?;
    if debug {
        if candidates.is_empty() {
            writeln!(stderr, "debug: no candidate files found")?;
        } else {
            for candidate in &candidates {
                writeln!(stderr, "debug: candidate file {}", candidate.display())?;
            }
        }
    }

    if let Some(selected) = select_valid_database(&candidates, debug, stderr)? {
        return Ok(selected);
    }

    let fallback_candidates = find_all_sqlite_databases(search_roots)?
        .into_iter()
        .filter(|candidate| !candidates.contains(candidate))
        .collect::<Vec<_>>();

    let mut progress_hint_candidates = Vec::new();

    if !fallback_candidates.is_empty() {
        writeln!(
            stderr,
            "No valid BKLibrary*.sqlite database found; scanning other .sqlite files for Apple Books schema."
        )?;
    }

    for candidate in &fallback_candidates {
        if debug {
            writeln!(
                stderr,
                "debug: fallback sqlite file {}",
                candidate.display()
            )?;
        }
    }

    for candidate in fallback_candidates {
        let validation = validate::validate_database(&candidate);
        if debug {
            writeln!(
                stderr,
                "debug: fallback validation {} -> {}",
                candidate.display(),
                validation
            )?;
        }

        if validation.is_valid() {
            writeln!(
                stderr,
                "Selected valid Apple Books database with nonstandard filename: {}",
                candidate.display()
            )?;
            return Ok(SelectedDatabase {
                path: candidate,
                source: DatabaseSource::Discovered,
                validation,
            });
        }

        let tables = validate::find_progress_table_hints(&candidate);
        if !tables.is_empty() {
            progress_hint_candidates.push(ProgressHintCandidate {
                path: candidate,
                tables,
            });
        }
    }

    bail!(no_valid_database_message(
        search_roots,
        &progress_hint_candidates
    ));
}

pub(crate) fn find_candidate_databases(roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
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
) -> Result<Option<SelectedDatabase>> {
    for candidate in candidates {
        let validation = validate::validate_database(candidate);
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
            return Ok(Some(SelectedDatabase {
                path: candidate.clone(),
                source: DatabaseSource::Discovered,
                validation,
            }));
        }
    }

    Ok(None)
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
    progress_hint_candidates: &[ProgressHintCandidate],
) -> String {
    let mut message = String::from("No valid Apple Books database found.\nChecked search roots:");
    for root in search_roots {
        message.push_str("\n  ");
        message.push_str(&root.display().to_string());
    }

    if !progress_hint_candidates.is_empty() {
        message.push_str("\nFound other .sqlite databases with progress-related columns:");
        for candidate in progress_hint_candidates {
            message.push_str("\n  ");
            message.push_str(&candidate.path.display().to_string());
            for table in &candidate.tables {
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
    use std::fs;
    use std::path::Path;

    use rusqlite::Connection;
    use tempfile::{TempDir, tempdir_in};

    use super::{DatabaseSource, find_candidate_databases, resolve_database_from_roots};
    use crate::validate::{REQUIRED_COLUMNS, REQUIRED_TABLE};

    fn test_tempdir() -> TempDir {
        tempdir_in(Path::new("/tmp")).expect("tempdir")
    }

    #[test]
    fn returns_no_candidates_when_none_exist() {
        let tempdir = test_tempdir();
        let candidates =
            find_candidate_databases(&[tempdir.path().to_path_buf()]).expect("discovery");
        assert!(candidates.is_empty());
    }

    #[test]
    fn finds_single_candidate() {
        let tempdir = test_tempdir();
        let db = tempdir.path().join("BKLibrary-1.sqlite");
        fs::write(&db, []).expect("write fixture");

        let candidates =
            find_candidate_databases(&[tempdir.path().to_path_buf()]).expect("discovery");
        assert_eq!(candidates, vec![db]);
    }

    #[test]
    fn returns_candidates_sorted_lexicographically() {
        let tempdir = test_tempdir();
        let first = tempdir.path().join("BKLibrary-b.sqlite");
        let second = tempdir.path().join("BKLibrary-a.sqlite");
        fs::write(&first, []).expect("write fixture");
        fs::write(&second, []).expect("write fixture");

        let candidates =
            find_candidate_databases(&[tempdir.path().to_path_buf()]).expect("discovery");
        assert_eq!(candidates, vec![second, first]);
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

        assert_eq!(selected.path, valid);
        assert_eq!(selected.source, DatabaseSource::Discovered);
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

        assert_eq!(selected.path, valid);
        let stderr_output = String::from_utf8(stderr).expect("utf8");
        assert!(stderr_output.contains("No valid BKLibrary*.sqlite database found"));
        assert!(stderr_output.contains("nonstandard filename"));
    }

    #[test]
    fn skips_invalid_candidate_and_selects_later_valid_match() {
        let tempdir = test_tempdir();
        let invalid = tempdir.path().join("BKLibrary-a.sqlite");
        let valid = tempdir.path().join("BKLibrary-b.sqlite");

        fs::write(&invalid, "not sqlite").expect("write invalid candidate");
        create_valid_library_db(&valid);

        let mut stderr = Vec::new();
        let selected =
            resolve_database_from_roots(None, &[tempdir.path().to_path_buf()], false, &mut stderr)
                .expect("should resolve");

        assert_eq!(selected.path, valid);
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

        assert_eq!(selected.path, provided);
        assert_eq!(selected.source, DatabaseSource::Provided);
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

    fn create_progress_hint_db(path: &Path) {
        let conn = Connection::open(path).expect("create sqlite db");
        conn.execute(
            "CREATE TABLE reading_state (ZREADINGPROGRESS REAL, ZLASTENGAGEDDATE REAL, OTHER TEXT)",
            [],
        )
        .expect("create schema");
    }
}
