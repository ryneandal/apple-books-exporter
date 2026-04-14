use std::path::Path;

use rusqlite::{Connection, params};

pub fn create_valid_library_db(path: &Path) {
    let conn = Connection::open(path).expect("create sqlite db");
    conn.execute_batch(
        r#"
        CREATE TABLE ZBKLIBRARYASSET (
            ZTITLE TEXT,
            ZAUTHOR TEXT,
            ZISFINISHED INTEGER,
            ZDATEFINISHED REAL,
            ZREADINGPROGRESS REAL,
            ZBOOKHIGHWATERMARKPROGRESS REAL,
            ZLASTOPENDATE REAL,
            ZLASTENGAGEDDATE REAL,
            ZCREATIONDATE REAL,
            ZASSETID INTEGER,
            ZASSETGUID TEXT,
            ZSTOREID INTEGER,
            ZGENRE TEXT
        );
        "#,
    )
    .expect("create schema");

    conn.execute(
        r#"
        INSERT INTO ZBKLIBRARYASSET (
            ZTITLE,
            ZAUTHOR,
            ZISFINISHED,
            ZDATEFINISHED,
            ZREADINGPROGRESS,
            ZBOOKHIGHWATERMARKPROGRESS,
            ZLASTOPENDATE,
            ZLASTENGAGEDDATE,
            ZCREATIONDATE,
            ZASSETID,
            ZASSETGUID,
            ZSTOREID,
            ZGENRE
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
        params![
            "The Great Gatsby",
            "F. Scott Fitzgerald",
            1,
            719_876_708.0,
            0.935064911842346_f64,
            0.953216373920441_f64,
            719_844_887.0,
            719_876_708.0,
            748_980_784.0,
            914_355_894_i64,
            "0511A38E-2F6B-4521-8116-D1E4FB0324AC",
            914_355_894_i64,
            "Classics"
        ],
    )
    .expect("insert fixture row");
}
