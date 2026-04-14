mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::{TempDir, tempdir_in};

use common::create_valid_library_db;

fn test_tempdir() -> TempDir {
    tempdir_in("/tmp").expect("tempdir")
}

#[test]
fn export_json_from_fixture_db() {
    let tempdir = test_tempdir();
    let db_path = tempdir.path().join("valid_bklibrary.sqlite");
    create_valid_library_db(&db_path);

    Command::cargo_bin("apple-books-data-export")
        .expect("binary")
        .args(["export", "--db"])
        .arg(&db_path)
        .args(["--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\":\"The Great Gatsby\""))
        .stdout(predicate::str::contains("\"status\":\"finished\""));
}

#[test]
fn export_csv_from_fixture_db() {
    let tempdir = test_tempdir();
    let db_path = tempdir.path().join("valid_bklibrary.sqlite");
    create_valid_library_db(&db_path);

    Command::cargo_bin("apple-books-data-export")
        .expect("binary")
        .args(["export", "--db"])
        .arg(&db_path)
        .args(["--format", "csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "title,author,status,reading_progress,high_watermark_progress,finished_at,last_opened_at,last_engaged_at,library_record_created_at,asset_id,asset_guid,store_id,genre",
        ))
        .stdout(predicate::str::contains("The Great Gatsby"));
}

#[test]
fn inspect_fixture_db() {
    let tempdir = test_tempdir();
    let db_path = tempdir.path().join("valid_bklibrary.sqlite");
    create_valid_library_db(&db_path);

    Command::cargo_bin("apple-books-data-export")
        .expect("binary")
        .args(["inspect", "--db"])
        .arg(&db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid: yes"))
        .stdout(predicate::str::contains("rows: 1"));
}

#[test]
fn export_writes_to_output_file() {
    let tempdir = test_tempdir();
    let db_path = tempdir.path().join("valid_bklibrary.sqlite");
    let output_path = tempdir.path().join("books.json");
    create_valid_library_db(&db_path);

    Command::cargo_bin("apple-books-data-export")
        .expect("binary")
        .args(["export", "--db"])
        .arg(&db_path)
        .args(["--output"])
        .arg(&output_path)
        .args(["--pretty"])
        .assert()
        .success();

    let output = std::fs::read_to_string(output_path).expect("output file");
    assert!(output.contains("\"title\": \"The Great Gatsby\""));
}
