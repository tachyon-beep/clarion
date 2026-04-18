//! Schema-apply integration tests.
//!
//! Verifies that migration 0001 produces every table, index, trigger,
//! generated column, and view from detailed-design.md §3, and that
//! applying migrations a second time is a no-op.

use rusqlite::{Connection, params};

use clarion_storage::{pragma, schema};

fn open_fresh(tempdir: &tempfile::TempDir) -> Connection {
    let path = tempdir.path().join("clarion.db");
    let mut conn = Connection::open(&path).expect("open");
    pragma::apply_write_pragmas(&conn).expect("pragmas");
    schema::apply_migrations(&mut conn).expect("apply migrations");
    conn
}

fn table_names(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(std::result::Result::unwrap)
        .collect()
}

fn trigger_names(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='trigger' ORDER BY name")
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(std::result::Result::unwrap)
        .collect()
}

fn view_names(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='view' ORDER BY name")
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(std::result::Result::unwrap)
        .collect()
}

fn index_names(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master \
             WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(std::result::Result::unwrap)
        .collect()
}

#[test]
fn migration_0001_creates_every_expected_table() {
    let tempdir = tempfile::tempdir().unwrap();
    let conn = open_fresh(&tempdir);
    let tables = table_names(&conn);
    for expected in &[
        "edges",
        "entities",
        "entity_tags",
        "findings",
        "runs",
        "schema_migrations",
        "summary_cache",
    ] {
        assert!(
            tables.iter().any(|t| t == expected),
            "missing table {expected} in {tables:?}"
        );
    }
}

#[test]
fn migration_0001_creates_entity_fts_virtual_table() {
    let tempdir = tempfile::tempdir().unwrap();
    let conn = open_fresh(&tempdir);
    let sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE name='entity_fts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(sql.contains("CREATE VIRTUAL TABLE"), "sql was: {sql}");
    conn.execute_batch("SELECT entity_id, name FROM entity_fts LIMIT 0")
        .expect("entity_fts queryable");
}

#[test]
fn migration_0001_creates_all_three_fts_triggers() {
    let tempdir = tempfile::tempdir().unwrap();
    let conn = open_fresh(&tempdir);
    let triggers = trigger_names(&conn);
    for expected in &["entities_ad", "entities_ai", "entities_au"] {
        assert!(
            triggers.iter().any(|t| t == expected),
            "missing trigger {expected} in {triggers:?}"
        );
    }
}

#[test]
fn migration_0001_creates_guidance_sheets_view() {
    let tempdir = tempfile::tempdir().unwrap();
    let conn = open_fresh(&tempdir);
    let views = view_names(&conn);
    assert!(
        views.iter().any(|v| v == "guidance_sheets"),
        "views: {views:?}"
    );
    conn.execute_batch("SELECT id, name, priority FROM guidance_sheets LIMIT 0")
        .expect("guidance_sheets queryable");
}

#[test]
fn migration_0001_creates_partial_indexes() {
    let tempdir = tempfile::tempdir().unwrap();
    let conn = open_fresh(&tempdir);
    let indexes = index_names(&conn);
    for expected in &["ix_entities_churn", "ix_entities_priority"] {
        assert!(
            indexes.iter().any(|i| i == expected),
            "missing index {expected} in {indexes:?}"
        );
    }
}

#[test]
fn entity_generated_columns_extract_from_properties_json() {
    let tempdir = tempfile::tempdir().unwrap();
    let conn = open_fresh(&tempdir);
    let props = r#"{"priority": "P1", "git_churn_count": 42}"#;
    conn.execute(
        "INSERT INTO entities (id, plugin_id, kind, name, short_name, properties, \
         created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, \
         strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        params![
            "python:function:demo.f",
            "python",
            "function",
            "demo.f",
            "f",
            props
        ],
    )
    .unwrap();
    let (priority, churn): (Option<String>, Option<i64>) = conn
        .query_row(
            "SELECT priority, git_churn_count FROM entities WHERE id = ?1",
            params!["python:function:demo.f"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(priority.as_deref(), Some("P1"));
    assert_eq!(churn, Some(42));
}

#[test]
fn migrations_are_idempotent() {
    let tempdir = tempfile::tempdir().unwrap();
    let mut conn = open_fresh(&tempdir);
    schema::apply_migrations(&mut conn).expect("second apply should be a no-op");
    assert_eq!(schema::applied_count(&conn).unwrap(), 1);
    let tables_after = table_names(&conn);
    assert!(tables_after.contains(&"entities".to_owned()));
}

#[test]
fn schema_migrations_records_one_row() {
    let tempdir = tempfile::tempdir().unwrap();
    let conn = open_fresh(&tempdir);
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
    let name: String = conn
        .query_row(
            "SELECT name FROM schema_migrations WHERE version = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(name, "0001_initial_schema");
}
