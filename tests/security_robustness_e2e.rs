//! End-to-end security and robustness tests for the termai binary.
//!
//! Every test runs against an isolated temporary HOME so nothing touches the
//! developer's real `~/.config/termai` secret store. No test performs network
//! I/O: API keys are either absent (commands fail fast before any HTTP
//! request) or obviously-fake values that are only ever printed/masked, never
//! sent.
//!
//! Intentionally skipped case: "ask with only a fake env API key". With
//! CLAUDE_API_KEY set, `termai ask` resolves the key successfully and
//! proceeds to a real HTTPS request to the provider. There is no offline
//! failure point before the network call, so any assertion would depend on
//! network availability/latency and be flaky. That path is deliberately not
//! tested here; the "no key at all" path (which fails deterministically
//! before any request) is covered instead.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

/// Obviously fake key used only to verify masking; must never appear in output.
const FAKE_CLAUDE_KEY: &str = "sk-ant-fake-e2e-test-key-1234567890";

/// Generous upper bound for commands that must fail fast without network.
const FAIL_FAST_TIMEOUT: Duration = Duration::from_secs(60);

/// Build a termai command with an isolated HOME and all provider env keys
/// removed, so results never depend on the host environment.
fn termai(home: &Path) -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("OPENAI_API_KEY")
        .env_remove("CLAUDE_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .current_dir(home);
    cmd
}

fn assert_no_panic(output: &std::process::Output) {
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !combined.contains("panicked at"),
        "binary panicked:\n{combined}"
    );
    assert!(
        !combined.contains("RUST_BACKTRACE"),
        "binary printed a backtrace:\n{combined}"
    );
}

// ---------------------------------------------------------------------------
// 1. Secret store permissions (unix only)
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn secret_store_created_with_restrictive_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let home = temp.path();

    termai(home).arg("--print-config").assert().success();

    let config_dir = home.join(".config").join("termai");
    let db_file = config_dir.join("app.db");
    assert!(config_dir.is_dir(), "config dir should exist");
    assert!(db_file.is_file(), "app.db should exist");

    let dir_mode = fs::metadata(&config_dir).unwrap().permissions().mode() & 0o777;
    let db_mode = fs::metadata(&db_file).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        dir_mode, 0o700,
        "config dir must be 0o700, got {dir_mode:o}"
    );
    assert_eq!(db_mode, 0o600, "app.db must be 0o600, got {db_mode:o}");
}

#[cfg(unix)]
#[test]
fn secret_store_permissions_tightened_on_every_startup() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let home = temp.path();
    let config_dir = home.join(".config").join("termai");
    let db_file = config_dir.join("app.db");

    // Simulate an old install with dangerously loose permissions.
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(&db_file, b"").unwrap();
    fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o777)).unwrap();
    fs::set_permissions(&db_file, fs::Permissions::from_mode(0o666)).unwrap();

    termai(home).arg("--print-config").assert().success();

    let dir_mode = fs::metadata(&config_dir).unwrap().permissions().mode() & 0o777;
    let db_mode = fs::metadata(&db_file).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        dir_mode, 0o700,
        "loose config dir must be tightened to 0o700, got {dir_mode:o}"
    );
    assert_eq!(
        db_mode, 0o600,
        "loose app.db must be tightened to 0o600, got {db_mode:o}"
    );
}

// ---------------------------------------------------------------------------
// 2. No secrets leaked in output
// ---------------------------------------------------------------------------

#[test]
fn env_api_key_is_masked_in_config_env_output() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    let assert = termai(home)
        .env("CLAUDE_API_KEY", FAKE_CLAUDE_KEY)
        .args(["config", "env"])
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.contains(FAKE_CLAUDE_KEY),
        "full API key leaked to stdout:\n{stdout}"
    );
    assert!(
        !stderr.contains(FAKE_CLAUDE_KEY),
        "full API key leaked to stderr:\n{stderr}"
    );
    // Masking format per redact_env_value: first 4 chars ... last 4 chars.
    assert!(
        stdout.contains("sk-a...7890"),
        "expected masked key 'sk-a...7890' in stdout:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// 3. ask without any key: fast graceful failure, no network hang
// ---------------------------------------------------------------------------

#[test]
fn ask_without_any_key_fails_fast_with_guidance() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    termai(home)
        .timeout(FAIL_FAST_TIMEOUT)
        .args(["ask", "hello"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("API key").and(predicate::str::contains("panicked at").not()),
        );
}

// ---------------------------------------------------------------------------
// 5. Redaction storage roundtrip
// ---------------------------------------------------------------------------

#[test]
fn redact_add_list_remove_roundtrip_with_special_regex_chars() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    // Contains regex metacharacters; must be stored and listed literally.
    let special = "p@$$w{o}rd.*";

    termai(home)
        .args(["redact", "add", "secret123"])
        .assert()
        .success();

    termai(home)
        .args(["redact", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret123"));

    termai(home)
        .args(["redact", "add", special])
        .assert()
        .success();

    termai(home)
        .args(["redact", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(special).and(predicate::str::contains("secret123")));

    termai(home)
        .args(["redact", "remove", "secret123"])
        .assert()
        .success();

    termai(home)
        .args(["redact", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret123").not())
        .stdout(predicate::str::contains(special));

    // Re-add after removal must work.
    termai(home)
        .args(["redact", "add", "secret123"])
        .assert()
        .success();

    termai(home)
        .args(["redact", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret123").and(predicate::str::contains(special)));
}

// ---------------------------------------------------------------------------
// 6. .termai.toml project configuration
// ---------------------------------------------------------------------------

#[test]
fn valid_project_termai_toml_is_picked_up_by_config_show() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    let project = home.join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join(".termai.toml"),
        "[providers]\ndefault = \"claude\"\n",
    )
    .unwrap();

    termai(home)
        .current_dir(&project)
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".termai.toml"));
}

#[test]
fn malformed_project_termai_toml_fails_gracefully() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    let project = home.join("project");
    fs::create_dir_all(&project).unwrap();
    // Deliberate TOML syntax errors: unclosed table header, double equals.
    fs::write(
        project.join(".termai.toml"),
        "[providers\ndefault = = \"claude\"\n",
    )
    .unwrap();

    let assert = termai(home)
        .current_dir(&project)
        .args(["config", "show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Configuration"));

    assert_no_panic(assert.get_output());
}

// ---------------------------------------------------------------------------
// 7. Robustness against hostile inputs
// ---------------------------------------------------------------------------

#[test]
fn extremely_long_ask_argument_does_not_panic() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    let long_arg = "A".repeat(10_000);

    let assert = termai(home)
        .timeout(FAIL_FAST_TIMEOUT)
        .args(["ask", &long_arg])
        .assert()
        .failure(); // no API key configured, so a graceful fast error

    assert_no_panic(assert.get_output());
}

#[test]
fn weird_session_names_are_handled_gracefully() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    let hostile_names = [
        "../../etc/passwd",
        "name\twith\ncontrol-\u{3a9}\u{2713}",
        "\u{1}\u{2}\u{7f}",
        "'; DROP TABLE sessions; --",
    ];
    for name in hostile_names {
        let assert = termai(home)
            .args(["sessions", "show", name])
            .assert()
            .failure();
        assert_no_panic(assert.get_output());
    }
}

#[cfg(unix)]
#[test]
fn invalid_utf8_session_name_is_rejected_gracefully() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let temp = TempDir::new().unwrap();
    let home = temp.path();

    // 0xFF 0xFE is not valid UTF-8; clap must reject it without panicking.
    let bad = OsStr::from_bytes(b"weird\xff\xfename");
    let assert = termai(home)
        .args(["sessions", "show"])
        .arg(bad)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid UTF-8"));
    assert_no_panic(assert.get_output());
}

#[test]
fn every_visible_subcommand_help_exits_zero() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    let subcommands = [
        "setup",
        "config",
        "auth",
        "redact",
        "sessions",
        "ask",
        "chat",
        "preset",
        "commit",
        "review",
        "branch-summary",
        "hooks",
        "stash",
        "tag",
        "rebase",
        "conflicts",
        "completion",
    ];

    for sub in subcommands {
        let assert = termai(home)
            .args([sub, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
        assert_no_panic(assert.get_output());
    }
}

// ---------------------------------------------------------------------------
// 8. Database corruption
// ---------------------------------------------------------------------------

#[test]
fn corrupted_database_yields_graceful_error_with_guidance() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    let config_dir = home.join(".config").join("termai");
    fs::create_dir_all(&config_dir).unwrap();

    // Garbage bytes that are definitely not a SQLite header.
    let garbage: Vec<u8> = (0u16..512)
        .map(|i| (i.wrapping_mul(31) % 251) as u8)
        .collect();
    fs::write(config_dir.join("app.db"), &garbage).unwrap();

    let assert = termai(home)
        .args(["sessions", "list"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Failed to initialize database").and(
                predicate::str::contains("Database Initialization Troubleshooting"),
            ),
        );
    assert_no_panic(assert.get_output());
}

// ---------------------------------------------------------------------------
// 9. Concurrent invocations against the same store
// ---------------------------------------------------------------------------

#[test]
fn concurrent_invocations_do_not_panic() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    // Warm up: initialize the database once.
    termai(home).args(["sessions", "list"]).assert().success();

    // Spawn three simultaneous invocations sharing the same HOME/database.
    let bin = env!("CARGO_BIN_EXE_termai");
    let children: Vec<_> = (0..3)
        .map(|_| {
            std::process::Command::new(bin)
                .env("HOME", home)
                .env("XDG_CONFIG_HOME", home.join(".config"))
                .env_remove("OPENAI_API_KEY")
                .env_remove("CLAUDE_API_KEY")
                .env_remove("ANTHROPIC_API_KEY")
                .current_dir(home)
                .args(["sessions", "list"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("failed to spawn termai")
        })
        .collect();

    for child in children {
        let output = child.wait_with_output().expect("failed to wait on termai");
        // Each invocation must exit normally (success or a graceful SQLite
        // locking error) — never a panic, never killed by a signal.
        assert!(
            output.status.code().is_some(),
            "process was killed by a signal: {:?}",
            output.status
        );
        assert_no_panic(&output);
    }
}
