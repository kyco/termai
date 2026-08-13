//! End-to-end tests for the core (non-AI) CLI surface.
//!
//! Every test runs against an isolated temporary HOME so the user's real
//! `~/.config/termai` is never touched. All tests are deterministic: no
//! network access, no real API keys, and no interactive TTY is required.

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::Connection;
use std::path::Path;
use tempfile::TempDir;

/// Build a `termai` command isolated inside the given temporary HOME.
///
/// - `HOME` and `XDG_CONFIG_HOME` are overridden so config/db files land in
///   the tempdir.
/// - API key environment variables are removed so auth state is always
///   "not configured" unless a test explicitly stores a key.
/// - The working directory is the temp HOME so no project `.termai.toml`
///   from the repository leaks into the test.
fn termai(home: &Path) -> Command {
    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("OPENAI_API_KEY")
        .env_remove("CLAUDE_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .current_dir(home);
    cmd
}

fn temp_home() -> TempDir {
    TempDir::new().expect("failed to create temp HOME")
}

// ---------------------------------------------------------------------------
// 1. Global CLI surface
// ---------------------------------------------------------------------------

#[test]
fn help_lists_all_core_subcommands() {
    let home = temp_home();
    let expected = [
        "setup",
        "config",
        "auth",
        "redact",
        "sessions",
        "completion",
        "ask",
        "chat",
        "commit",
        "review",
        "branch-summary",
        "hooks",
        "stash",
        "tag",
        "rebase",
        "conflicts",
        "preset",
    ];

    let assert = termai(home.path()).arg("--help").assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for subcommand in expected {
        assert!(
            stdout.contains(subcommand),
            "--help output should list the '{}' subcommand",
            subcommand
        );
    }
    assert!(stdout.contains("Usage: termai"));
}

#[test]
fn version_flag_prints_binary_name_and_version() {
    let home = temp_home();
    termai(home.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("termai"));
}

#[test]
fn unknown_flag_shows_error_and_discovery_suggestions() {
    let home = temp_home();
    termai(home.path())
        .arg("--frobnicate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid command line arguments"))
        .stderr(predicate::str::contains("unexpected argument"))
        .stderr(predicate::str::contains("Suggestions"))
        .stderr(predicate::str::contains("termai discovery"));
}

#[test]
fn unknown_nested_subcommand_shows_error_and_suggestions() {
    let home = temp_home();
    termai(home.path())
        .args(["sessions", "frobnicate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"))
        .stderr(predicate::str::contains("termai discovery"));
}

#[test]
fn no_args_prints_usage_banner() {
    let home = temp_home();
    termai(home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("TermAI"))
        .stdout(predicate::str::contains("termai ask"))
        .stdout(predicate::str::contains("termai chat"))
        .stdout(predicate::str::contains("termai setup"))
        .stdout(predicate::str::contains("termai sessions list"));
}

// ---------------------------------------------------------------------------
// 2. Help correctness regression: plural `termai sessions`, never singular
// ---------------------------------------------------------------------------

#[test]
fn help_never_mentions_singular_session_command() {
    let home = temp_home();

    // `--help` must reference the plural `sessions` subcommand and must not
    // contain the old singular form "termai session " anywhere.
    termai(home.path())
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("sessions"))
        .stdout(predicate::str::contains("termai session ").not());

    // The no-args usage banner must also use the plural form.
    termai(home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("termai session ").not());

    // The sessions subcommand's own help must not use the singular form.
    termai(home.path())
        .args(["sessions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("termai session ").not());
}

// ---------------------------------------------------------------------------
// 3. Shell completion and man page generation
// ---------------------------------------------------------------------------

#[test]
fn completion_bash_emits_bash_script() {
    let home = temp_home();
    termai(home.path())
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_termai()"))
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn completion_zsh_emits_zsh_script() {
    let home = temp_home();
    termai(home.path())
        .args(["completion", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef termai"))
        .stdout(predicate::str::contains("_termai"));
}

#[test]
fn completion_fish_emits_fish_script() {
    let home = temp_home();
    termai(home.path())
        .args(["completion", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c termai"));
}

#[test]
fn hidden_man_command_emits_roff_output() {
    let home = temp_home();
    termai(home.path())
        .arg("man")
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH termai"))
        .stdout(predicate::str::contains(".SH"));
}

// ---------------------------------------------------------------------------
// 4. Config management (non-interactive actions only)
// ---------------------------------------------------------------------------

#[test]
fn config_show_works_on_fresh_home() {
    let home = temp_home();
    termai(home.path())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Effective Configuration"))
        .stdout(predicate::str::contains("Default provider:"))
        .stdout(predicate::str::contains("Default model:"));
}

#[test]
fn config_set_provider_roundtrips_through_show() {
    let home = temp_home();
    termai(home.path())
        .args(["config", "set-provider", "openai"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default provider set to openai"));

    termai(home.path())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default provider: openai"));
}

#[test]
fn config_set_model_with_explicit_name_roundtrips_through_show() {
    let home = temp_home();
    // Passing an explicit model name skips the interactive selector.
    termai(home.path())
        .args(["config", "set-model", "claude-sonnet-4-20250514"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("Default model set to"));

    termai(home.path())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Default model: claude-sonnet-4-20250514",
        ));
}

#[test]
fn config_set_model_gpt_5_6_sol_roundtrips_through_show() {
    let home = temp_home();
    // gpt-5.6-sol is part of the built-in catalog: no network required.
    termai(home.path())
        .args(["config", "set-model", "gpt-5.6-sol"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("Default model set to"));

    termai(home.path())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Default provider: codex"))
        .stdout(predicate::str::contains("Default model: gpt-5.6-sol"));
}

#[test]
fn config_list_models_codex_shows_gpt_5_6_family() {
    let home = temp_home();
    // Filtering to codex without an API key uses the built-in catalog: no
    // network required.
    termai(home.path())
        .args(["config", "list-models", "--provider", "codex"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Models"))
        .stdout(predicate::str::contains("OpenAI Codex (ChatGPT Plus/Pro)"))
        .stdout(predicate::str::contains("gpt-5.6-sol"))
        .stdout(predicate::str::contains("gpt-5.6-terra"))
        .stdout(predicate::str::contains("gpt-5.6-luna"))
        .stdout(predicate::str::contains("gpt-5.6"));
}

#[test]
fn config_set_model_rejects_invalid_model_gracefully() {
    let home = temp_home();
    termai(home.path())
        .args(["config", "set-model", "definitely-not-a-real-model"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("Invalid model name"));
}

#[test]
fn config_list_models_shows_static_claude_catalog() {
    let home = temp_home();
    // Filtering to claude uses the built-in catalog: no network required.
    termai(home.path())
        .args(["config", "list-models", "--provider", "claude"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Models"))
        .stdout(predicate::str::contains("Claude (Anthropic)"))
        .stdout(predicate::str::contains("claude-sonnet-4-20250514"));
}

#[test]
fn config_env_reports_auth_variables_unset() {
    let home = temp_home();
    termai(home.path())
        .args(["config", "env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OPENAI_API_KEY"))
        .stdout(predicate::str::contains("CLAUDE_API_KEY"))
        .stdout(predicate::str::contains("not set"));
}

// ---------------------------------------------------------------------------
// 5. Auth status / logout / login
// ---------------------------------------------------------------------------

#[test]
fn auth_status_claude_fresh_home_reports_not_configured() {
    let home = temp_home();
    termai(home.path())
        .args(["auth", "status", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Claude Status"))
        .stdout(predicate::str::contains("Status: Not configured"));
}

#[test]
fn auth_status_openai_fresh_home_reports_not_configured() {
    let home = temp_home();
    termai(home.path())
        .args(["auth", "status", "openai"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OpenAI Status"))
        .stdout(predicate::str::contains("Status: Not configured"));
}

#[test]
fn auth_status_codex_fresh_home_reports_not_authenticated() {
    let home = temp_home();
    termai(home.path())
        .args(["auth", "status", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OpenAI Codex Status"))
        .stdout(predicate::str::contains("Not authenticated"));
}

#[test]
fn auth_logout_on_fresh_home_is_graceful() {
    let home = temp_home();
    termai(home.path())
        .args(["auth", "logout", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Claude API key cleared"));

    termai(home.path())
        .args(["auth", "logout", "openai"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OpenAI API key cleared"));

    termai(home.path())
        .args(["auth", "logout", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "not currently authenticated with Codex",
        ));
}

#[test]
fn auth_login_without_tty_fails_gracefully() {
    // `auth login claude|openai` uses an interactive prompt. Without a TTY it
    // must fail with a helpful error rather than hanging or panicking.
    let home = temp_home();
    for provider in ["claude", "openai"] {
        let assert = termai(home.path())
            .args(["auth", "login", provider])
            .write_stdin("sk-test-not-a-real-key\n")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Authentication command failed"))
            .stderr(predicate::str::contains("panicked").not());
        drop(assert);
    }
}

// ---------------------------------------------------------------------------
// 6. Redaction pattern lifecycle
// ---------------------------------------------------------------------------

#[test]
fn redact_add_list_remove_lifecycle() {
    let home = temp_home();

    // Add a pattern.
    termai(home.path())
        .args(["redact", "add", "secret123"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret123"));

    // List shows the pattern as active.
    termai(home.path())
        .args(["redact", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret123"))
        .stdout(predicate::str::contains("pattern(s) active"));

    // Remove the pattern.
    termai(home.path())
        .args(["redact", "remove", "secret123"])
        .assert()
        .success();

    // List is empty again.
    termai(home.path())
        .args(["redact", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No redaction patterns configured"));
}

#[test]
fn redact_duplicate_add_is_graceful() {
    let home = temp_home();
    termai(home.path())
        .args(["redact", "add", "dup-pattern"])
        .assert()
        .success();
    // Adding the same pattern again must not crash.
    termai(home.path())
        .args(["redact", "add", "dup-pattern"])
        .assert()
        .success();
    termai(home.path())
        .args(["redact", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dup-pattern"));
}

#[test]
fn redact_remove_nonexistent_is_graceful() {
    let home = temp_home();
    termai(home.path())
        .args(["redact", "remove", "never-added"])
        .assert()
        .success()
        .stderr(predicate::str::contains("panicked").not());
}

// ---------------------------------------------------------------------------
// 7. Legacy flag compatibility (deprecation shims)
// ---------------------------------------------------------------------------

#[test]
fn legacy_chat_gpt_api_key_flag_still_stores_key() {
    let home = temp_home();
    termai(home.path())
        .args(["--chat-gpt-api-key", "test-key-123"])
        .assert()
        .success()
        .stderr(predicate::str::contains("deprecated"));

    // The key must actually land in the sqlite config store.
    let db_path = home.path().join(".config").join("termai").join("app.db");
    assert!(db_path.exists(), "app.db should exist after legacy command");
    let conn = Connection::open(&db_path).unwrap();
    let value: String = conn
        .query_row(
            "SELECT value FROM config WHERE key = 'chat_gpt_api_key'",
            [],
            |row| row.get(0),
        )
        .expect("chat_gpt_api_key should be stored in config table");
    assert_eq!(value, "test-key-123");

    // And the legacy --print-config shim must show the key as configured,
    // but MASKED - never the cleartext secret.
    termai(home.path())
        .arg("--print-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("chat_gpt_api_key"))
        .stdout(predicate::str::contains("test...-123"))
        .stdout(predicate::str::contains("test-key-123").not())
        .stderr(predicate::str::contains("deprecated"));
}

#[test]
fn legacy_claude_api_key_flag_still_stores_key() {
    let home = temp_home();
    termai(home.path())
        .args(["--claude-api-key", "ck-test-456"])
        .assert()
        .success()
        .stderr(predicate::str::contains("deprecated"));

    let db_path = home.path().join(".config").join("termai").join("app.db");
    let conn = Connection::open(&db_path).unwrap();
    let value: String = conn
        .query_row(
            "SELECT value FROM config WHERE key = 'claude_api_key'",
            [],
            |row| row.get(0),
        )
        .expect("claude_api_key should be stored in config table");
    assert_eq!(value, "ck-test-456");
}

#[test]
fn legacy_redact_add_flag_still_adds_pattern() {
    let home = temp_home();
    termai(home.path())
        .args(["--redact-add", "legacy-pattern"])
        .assert()
        .success()
        .stderr(predicate::str::contains("deprecated"));

    termai(home.path())
        .args(["redact", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("legacy-pattern"));
}

#[test]
fn legacy_print_config_flag_shows_configuration() {
    let home = temp_home();
    termai(home.path())
        .arg("--print-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("Current Configuration"))
        .stderr(predicate::str::contains("deprecated"));
}

// ---------------------------------------------------------------------------
// 8. Sessions and database bootstrap
// ---------------------------------------------------------------------------

#[test]
fn sessions_list_on_fresh_home_succeeds() {
    let home = temp_home();
    termai(home.path())
        .args(["sessions", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("panicked").not());
}

#[test]
fn startup_never_dumps_database_schema() {
    // Bootstrapping the database on a fresh HOME must not pollute stdout with
    // a schema dump ("Table: messages" etc.) - it breaks scripting output.
    let home = temp_home();
    termai(home.path())
        .args(["sessions", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Table:").not());

    termai(home.path())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Table:").not());
}

#[test]
fn db_bootstrap_creates_expected_schema() {
    let home = temp_home();
    termai(home.path())
        .args(["config", "show"])
        .assert()
        .success();

    let db_path = home.path().join(".config").join("termai").join("app.db");
    assert!(db_path.exists(), "app.db should be created under temp HOME");

    let conn = Connection::open(&db_path).unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master
             WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        )
        .unwrap();
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(|t| t.unwrap())
        .collect();

    for expected in ["messages", "config", "sessions", "conversation_branches"] {
        assert!(
            tables.contains(&expected.to_string()),
            "expected table '{}' in app.db, got: {:?}",
            expected,
            tables
        );
    }
}

// ---------------------------------------------------------------------------
// 9. Interactive chat: non-TTY fallback mode
// ---------------------------------------------------------------------------

/// With piped stdin/stdout, `termai chat` must run in the plain line-based
/// fallback: no raw mode, no bottom-anchored UI, and a clean exit on /exit.
#[test]
fn chat_piped_stdin_exits_cleanly_in_fallback_mode() {
    let home = temp_home();
    let assert = termai(home.path())
        .arg("chat")
        .write_stdin("/exit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("TermAI Interactive Chat Mode"))
        .stdout(predicate::str::contains("Goodbye"));

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Raw-mode/anchored-UI escape sequences must never leak into piped output
    for (seq, label) in [
        ("\u{1b}[?2004h", "bracketed paste enable"),
        ("\u{1b}[?2004l", "bracketed paste disable"),
        ("\u{1b}[2K", "clear-line (anchor erase)"),
        ("\u{1b}[1A", "cursor-up (anchor repaint)"),
    ] {
        assert!(
            !stdout.contains(seq),
            "piped chat output should not contain {} escape sequence",
            label
        );
    }
}

/// EOF on stdin (no input at all) must also exit the fallback loop cleanly.
#[test]
fn chat_piped_stdin_eof_exits_cleanly() {
    let home = temp_home();
    termai(home.path())
        .arg("chat")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye"));
}
