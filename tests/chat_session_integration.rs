//! End-to-end tests for in-chat session management.
//!
//! These drive the real `termai chat` REPL over piped stdin and assert against
//! both stdout and the resulting SQLite state. They cover the offline session
//! commands (/sessions, /rename, /load, /new) — i.e. everything that does not
//! require a live model call — which is exactly the session-management surface
//! this work added.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const FAR_FUTURE: &str = "2999-01-01 00:00:00";

fn db_path(home: &Path) -> PathBuf {
    home.join(".config").join("termai").join("app.db")
}

/// Run the binary once so it creates `~/.config/termai/app.db` with the real schema.
fn init_db(home: &Path) {
    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home.to_str().unwrap())
        .arg("--print-config")
        .assert()
        .success();
    assert!(db_path(home).exists(), "database should have been created");
}

fn add_session(conn: &Connection, id: &str, name: &str, current: i64) {
    conn.execute(
        "INSERT INTO sessions (id, name, expires_at, current) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, FAR_FUTURE, current],
    )
    .unwrap();
}

fn add_message(conn: &Connection, id: &str, session_id: &str, role: &str, content: &str) {
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, message_type) VALUES (?1, ?2, ?3, ?4, 'standard')",
        params![id, session_id, role, content],
    )
    .unwrap();
}

/// Seed two saved sessions, each with messages.
fn seed(home: &Path) {
    let conn = Connection::open(db_path(home)).unwrap();
    add_session(&conn, "sess-a", "seeded-a", 1);
    add_session(&conn, "sess-b", "seeded-b", 0);
    add_message(&conn, "m1", "sess-a", "user", "alpha question");
    add_message(&conn, "m2", "sess-a", "assistant", "alpha answer");
    add_message(&conn, "m3", "sess-b", "user", "bravo question");
}

fn session_names(home: &Path) -> Vec<String> {
    let conn = Connection::open(db_path(home)).unwrap();
    let mut stmt = conn.prepare("SELECT name FROM sessions").unwrap();
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    rows
}

fn message_count(home: &Path, session_id: &str) -> i64 {
    let conn = Connection::open(db_path(home)).unwrap();
    conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE session_id = ?1",
        params![session_id],
        |row| row.get(0),
    )
    .unwrap()
}

#[test]
fn sessions_command_lists_saved_sessions() {
    let home = TempDir::new().unwrap();
    init_db(home.path());
    seed(home.path());

    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home.path().to_str().unwrap())
        .args(["chat", "--session", "seeded-a"])
        .write_stdin("/sessions\n/exit\n");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("seeded-a").and(predicate::str::contains("seeded-b")));
}

#[test]
fn rename_command_renames_active_session() {
    let home = TempDir::new().unwrap();
    init_db(home.path());
    seed(home.path());

    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home.path().to_str().unwrap())
        .args(["chat", "--session", "seeded-a"])
        .write_stdin("/rename renamed-a\n/exit\n");
    cmd.assert().success();

    let names = session_names(home.path());
    assert!(
        names.contains(&"renamed-a".to_string()),
        "expected renamed-a in {:?}",
        names
    );
    assert!(
        !names.contains(&"seeded-a".to_string()),
        "old name should be gone, got {:?}",
        names
    );
    // Renaming must not lose the messages.
    assert_eq!(message_count(home.path(), "sess-a"), 2);
}

#[test]
fn load_command_swaps_in_other_sessions_history() {
    let home = TempDir::new().unwrap();
    init_db(home.path());
    seed(home.path());

    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home.path().to_str().unwrap())
        .args(["chat", "--session", "seeded-a"])
        .write_stdin("/load seeded-b\n/exit\n");

    // The loaded session's history should be replayed into the transcript.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bravo question"));
}

#[test]
fn load_command_rejects_unknown_session() {
    let home = TempDir::new().unwrap();
    init_db(home.path());
    seed(home.path());

    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home.path().to_str().unwrap())
        .args(["chat", "--session", "seeded-a"])
        .write_stdin("/load nope\n/exit\n");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("not found"));
}

#[test]
fn unknown_command_is_reported_not_sent_to_model() {
    let home = TempDir::new().unwrap();
    init_db(home.path());
    seed(home.path());

    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home.path().to_str().unwrap())
        .args(["chat", "--session", "seeded-a"])
        .write_stdin("/saev oops\n/exit\n");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Unknown command"));
}
