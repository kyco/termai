/// Extended end-to-end tests for Git functionality: stash, tag, rebase,
/// conflicts, commit --add-all, hooks, and no-API-key behavior.
///
/// Every test runs against a real temporary Git repository and overrides HOME
/// to a temporary directory so the user's ~/.config/termai is never touched.
/// No network access or API keys are required: where a command would call an
/// LLM, the tests assert the graceful no-key behavior instead.
///
/// Note on interactivity: several destructive actions (stash drop/clear, tag
/// create/delete) are gated behind a dialoguer confirmation prompt that
/// requires a TTY. Under a test harness stdin is not a TTY, so those actions
/// fail gracefully without changing repository state. The tests below encode
/// that real, observed behavior. Passing `--yes` / `-y` skips the prompt and
/// proceeds as if confirmed, enabling non-interactive (scripting/CI) use.
use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Run a git command in the fixture repo (HOME overridden) and require success.
fn git(repo_path: &Path, home: &Path, args: &[&str]) {
    Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .env("HOME", home)
        .assert()
        .success();
}

/// Run a git command that is expected to fail (e.g. a conflicting merge).
fn git_expect_failure(repo_path: &Path, home: &Path, args: &[&str]) {
    Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .env("HOME", home)
        .assert()
        .failure();
}

/// Run a git command and capture its stdout as a String.
fn git_stdout(repo_path: &Path, home: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .env("HOME", home)
        .output()
        .expect("failed to run git");
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Build a termai command bound to the fixture repo with HOME overridden and
/// all AI provider environment variables removed (deterministic no-key runs).
fn termai(repo_path: &Path, home: &Path) -> Command {
    let mut cmd = cargo_bin_cmd!("termai");
    cmd.current_dir(repo_path)
        .env("HOME", home)
        .env_remove("CLAUDE_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("OPENAI_API_KEY");
    cmd
}

/// Create a temp git repo (branch `main`, one commit) and a temp HOME dir.
fn setup_repo() -> (TempDir, TempDir) {
    let repo = TempDir::new().expect("create repo tempdir");
    let home = TempDir::new().expect("create home tempdir");
    let path = repo.path();

    git(path, home.path(), &["init"]);
    // Deterministic branch name regardless of git version/system config
    git(
        path,
        home.path(),
        &["symbolic-ref", "HEAD", "refs/heads/main"],
    );
    git(
        path,
        home.path(),
        &["config", "user.email", "test@example.com"],
    );
    git(path, home.path(), &["config", "user.name", "Test User"]);

    fs::write(path.join("README.md"), "# Extended e2e fixture\n").expect("write README");
    git(path, home.path(), &["add", "README.md"]);
    git(path, home.path(), &["commit", "-m", "Initial commit"]);

    (repo, home)
}

/// Create `main` and `feature` branches that edit the same line of shared.txt,
/// leaving the repo checked out on `main`. Merging or rebasing across the two
/// branches produces a real conflict.
fn setup_conflicting_branches(repo_path: &Path, home: &Path) {
    fs::write(repo_path.join("shared.txt"), "line1\nline2\nline3\n").expect("write shared.txt");
    git(repo_path, home, &["add", "shared.txt"]);
    git(repo_path, home, &["commit", "-m", "add shared file"]);

    git(repo_path, home, &["checkout", "-b", "feature"]);
    fs::write(
        repo_path.join("shared.txt"),
        "line1\nFEATURE CHANGE\nline3\n",
    )
    .expect("write feature edit");
    git(repo_path, home, &["commit", "-am", "feat: feature edit"]);

    git(repo_path, home, &["checkout", "main"]);
    fs::write(repo_path.join("shared.txt"), "line1\nMAIN CHANGE\nline3\n")
        .expect("write main edit");
    git(repo_path, home, &["commit", "-am", "fix: main edit"]);
}

// ---------------------------------------------------------------------------
// Stash
// ---------------------------------------------------------------------------

#[test]
fn stash_apply_keeps_stash_and_restores_working_tree() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    fs::write(path.join("README.md"), "# stashed edit\n").expect("modify README");

    termai(path, home.path())
        .args(["stash", "push", "--message", "apply e2e stash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stash created"));

    // Working tree restored by push
    assert_eq!(
        fs::read_to_string(path.join("README.md")).unwrap(),
        "# Extended e2e fixture\n"
    );

    termai(path, home.path())
        .args(["stash", "apply", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes applied successfully"))
        .stdout(predicate::str::contains("remains in stash list"));

    // Changes are back in the working tree
    assert_eq!(
        fs::read_to_string(path.join("README.md")).unwrap(),
        "# stashed edit\n"
    );

    // Unlike pop, the stash must still exist afterwards
    termai(path, home.path())
        .args(["stash", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("stash@{0}"))
        .stdout(predicate::str::contains("apply e2e stash"));
}

#[test]
fn stash_drop_without_tty_fails_gracefully_and_keeps_stash() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    fs::write(path.join("README.md"), "# to be dropped\n").expect("modify README");
    termai(path, home.path())
        .args(["stash", "push", "--message", "keep me"])
        .assert()
        .success();

    // Drop is gated behind a TTY confirmation prompt; under the test harness
    // (piped stdin, even with "y\n" written) it fails gracefully.
    termai(path, home.path())
        .args(["stash", "drop", "0"])
        .write_stdin("y\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Stash command failed"));

    // The stash must be untouched
    let stashes = git_stdout(path, home.path(), &["stash", "list"]);
    assert!(
        stashes.contains("keep me"),
        "stash must survive a failed drop, got: {}",
        stashes
    );
}

#[test]
fn stash_clear_reports_empty_and_is_confirm_gated_with_stashes() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    // With no stashes, clear succeeds and reports there is nothing to do
    termai(path, home.path())
        .args(["stash", "clear"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No stashes to clear"));

    // Create two real stashes
    fs::write(path.join("README.md"), "# first\n").expect("modify README");
    termai(path, home.path())
        .args(["stash", "push", "--message", "first stash"])
        .assert()
        .success();
    fs::write(path.join("README.md"), "# second\n").expect("modify README");
    termai(path, home.path())
        .args(["stash", "push", "--message", "second stash"])
        .assert()
        .success();

    // Clear with stashes present requires TTY confirmation; without one it
    // fails gracefully and deletes nothing.
    termai(path, home.path())
        .args(["stash", "clear"])
        .write_stdin("y\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Stash command failed"));

    let stashes = git_stdout(path, home.path(), &["stash", "list"]);
    assert_eq!(
        stashes.lines().count(),
        2,
        "both stashes must survive a failed clear, got: {}",
        stashes
    );
}

#[test]
fn stash_drop_with_yes_flag_drops_stash_noninteractively() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    fs::write(path.join("README.md"), "# drop me\n").expect("modify README");
    termai(path, home.path())
        .args(["stash", "push", "--message", "drop me"])
        .assert()
        .success();
    assert_eq!(
        git_stdout(path, home.path(), &["stash", "list"])
            .lines()
            .count(),
        1
    );

    // --yes skips the confirmation prompt entirely (no TTY needed) and the
    // drop really happens: stash@{0} is gone afterwards.
    termai(path, home.path())
        .args(["stash", "drop", "0", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("has been dropped"));

    let stashes = git_stdout(path, home.path(), &["stash", "list"]);
    assert!(
        stashes.trim().is_empty(),
        "stash must be dropped with --yes, got: {}",
        stashes
    );
}

#[test]
fn stash_clear_with_yes_flag_empties_stash_list_noninteractively() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    // Create two real stashes
    fs::write(path.join("README.md"), "# first\n").expect("modify README");
    termai(path, home.path())
        .args(["stash", "push", "--message", "first stash"])
        .assert()
        .success();
    fs::write(path.join("README.md"), "# second\n").expect("modify README");
    termai(path, home.path())
        .args(["stash", "push", "--message", "second stash"])
        .assert()
        .success();
    assert_eq!(
        git_stdout(path, home.path(), &["stash", "list"])
            .lines()
            .count(),
        2
    );

    // Short form -y works too; both stashes are really deleted.
    termai(path, home.path())
        .args(["stash", "clear", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("All stashes have been cleared"));

    let stashes = git_stdout(path, home.path(), &["stash", "list"]);
    assert!(
        stashes.trim().is_empty(),
        "all stashes must be cleared with -y, got: {}",
        stashes
    );
}

#[test]
fn stash_show_lists_real_changed_files() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    fs::write(path.join("README.md"), "# show me\n").expect("modify README");
    termai(path, home.path())
        .args(["stash", "push", "--message", "show e2e stash"])
        .assert()
        .success();

    termai(path, home.path())
        .args(["stash", "show", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stash Details: stash@{0}"))
        .stdout(predicate::str::contains("show e2e stash"))
        .stdout(predicate::str::contains("Files Changed"))
        // The real changed file with its real status
        .stdout(predicate::str::contains("M README.md"));

    // A nonexistent stash index fails gracefully (no panic)
    termai(path, home.path())
        .args(["stash", "show", "5"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Stash command failed"));
}

#[test]
fn stash_push_include_untracked_flag() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    fs::write(path.join("README.md"), "# tracked change\n").expect("modify README");
    fs::write(path.join("brand_new.txt"), "untracked content\n").expect("write untracked");

    termai(path, home.path())
        .args(["stash", "push", "-u", "--message", "untracked e2e stash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stash created"))
        .stdout(predicate::str::contains("with untracked files"));

    // The untracked file must actually have been stashed away
    assert!(
        !path.join("brand_new.txt").exists(),
        "untracked file must be removed from the working tree by stash -u"
    );
    let stashes = git_stdout(path, home.path(), &["stash", "list"]);
    assert!(stashes.contains("untracked e2e stash"));

    // `stash show` must list BOTH the modified tracked file and the untracked
    // file (stored on the stash commit's third parent).
    termai(path, home.path())
        .args(["stash", "show", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("README.md"))
        .stdout(predicate::str::contains("brand_new.txt"));
}

#[test]
fn stash_push_on_clean_tree_is_graceful() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    termai(path, home.path())
        .args(["stash", "push"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Working directory is clean - nothing to stash",
        ));

    let stashes = git_stdout(path, home.path(), &["stash", "list"]);
    assert!(stashes.trim().is_empty(), "no stash must be created");
}

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------

#[test]
fn tag_create_without_tty_is_confirm_gated_and_creates_nothing() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    // Annotated create: even with name and message supplied, creation asks
    // for confirmation, which requires a TTY. Without one it fails gracefully
    // and must not create the tag.
    termai(path, home.path())
        .args(["tag", "create", "v1.0.0", "-m", "First release"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Tag command failed"));

    // Lightweight create hits the same confirmation gate
    termai(path, home.path())
        .args(["tag", "create", "v1.0.1", "--lightweight"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Tag command failed"));

    let tags = git_stdout(path, home.path(), &["tag", "-l"]);
    assert!(
        tags.trim().is_empty(),
        "no tags must be created on a failed confirm, got: {}",
        tags
    );
}

#[test]
fn tag_create_and_delete_with_yes_flag_work_noninteractively() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    // Annotated create with --yes skips the confirmation prompt and really
    // creates the tag, even without a TTY.
    termai(path, home.path())
        .args(["tag", "create", "v1.0.0", "-m", "First release", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created successfully"));

    let tags = git_stdout(path, home.path(), &["tag", "-l"]);
    assert!(
        tags.contains("v1.0.0"),
        "tag must be created with --yes, got: {}",
        tags
    );

    // The real annotated tag is visible through termai itself
    termai(path, home.path())
        .args(["tag", "show", "v1.0.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Type: Annotated"))
        .stdout(predicate::str::contains("First release"));

    // Delete with the short form -y really deletes it
    termai(path, home.path())
        .args(["tag", "delete", "v1.0.0", "-y"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted successfully"));

    let tags = git_stdout(path, home.path(), &["tag", "-l"]);
    assert!(
        tags.trim().is_empty(),
        "tag must be deleted with -y, got: {}",
        tags
    );
}

#[test]
fn tag_create_invalid_name_fails_gracefully() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    termai(path, home.path())
        .args(["tag", "create", "not-a-version", "-m", "bad name"])
        .assert()
        .failure()
        // The generic banner AND the specific honest reason must both surface.
        .stderr(predicate::str::contains("Tag command failed"))
        .stderr(predicate::str::contains("Invalid tag name"));

    let tags = git_stdout(path, home.path(), &["tag", "-l"]);
    assert!(tags.trim().is_empty());
}

#[test]
fn tag_create_duplicate_fails_and_preserves_original() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    git(
        path,
        home.path(),
        &["tag", "-a", "v1.0.0", "-m", "Original message"],
    );

    // Duplicate without --force prompts to overwrite (TTY required) and fails
    termai(path, home.path())
        .args(["tag", "create", "v1.0.0", "-m", "Overwritten message"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Tag command failed"));

    // Duplicate with --force still hits the final creation confirmation
    termai(path, home.path())
        .args([
            "tag",
            "create",
            "v1.0.0",
            "-m",
            "Overwritten message",
            "--force",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Tag command failed"));

    // The original annotated tag message must be preserved
    termai(path, home.path())
        .args(["tag", "show", "v1.0.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Original message"))
        .stdout(predicate::str::contains("Overwritten message").not());
}

#[test]
fn tag_show_annotated_shows_real_metadata() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    git(
        path,
        home.path(),
        &["tag", "-a", "v1.2.3", "-m", "Annotated release message"],
    );
    let head = git_stdout(path, home.path(), &["rev-parse", "HEAD"]);
    let head = head.trim();

    termai(path, home.path())
        .args(["tag", "show", "v1.2.3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tag Details: v1.2.3"))
        .stdout(predicate::str::contains("Type: Annotated"))
        // Real target commit and real tag object metadata
        .stdout(predicate::str::contains(head))
        .stdout(predicate::str::contains("Test User"))
        .stdout(predicate::str::contains("Annotated release message"))
        .stdout(predicate::str::contains("Initial commit"));
}

#[test]
fn tag_show_lightweight_shows_type_and_commit() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    git(path, home.path(), &["tag", "v0.5.0"]);
    let head = git_stdout(path, home.path(), &["rev-parse", "HEAD"]);
    let head = head.trim();

    termai(path, home.path())
        .args(["tag", "show", "v0.5.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Type: Lightweight"))
        .stdout(predicate::str::contains(head))
        .stdout(predicate::str::contains("Initial commit"));
}

#[test]
fn tag_delete_behaviors() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    // Deleting a nonexistent tag fails gracefully (bails before any prompt)
    // and the honest reason reaches stderr.
    termai(path, home.path())
        .args(["tag", "delete", "v9.9.9"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Tag command failed"))
        .stderr(predicate::str::contains("does not exist"));

    // Deleting an existing tag is confirmation-gated (TTY required); without
    // one it fails gracefully and the tag survives.
    git(path, home.path(), &["tag", "v0.5.0"]);
    termai(path, home.path())
        .args(["tag", "delete", "v0.5.0"])
        .write_stdin("y\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Tag command failed"));

    let tags = git_stdout(path, home.path(), &["tag", "-l"]);
    assert!(
        tags.contains("v0.5.0"),
        "tag must survive a failed delete, got: {}",
        tags
    );
}

#[test]
fn tag_commands_in_zero_commit_repo_are_graceful() {
    let repo = TempDir::new().expect("create repo tempdir");
    let home = TempDir::new().expect("create home tempdir");
    let path = repo.path();

    git(path, home.path(), &["init"]);
    git(
        path,
        home.path(),
        &["symbolic-ref", "HEAD", "refs/heads/main"],
    );
    git(
        path,
        home.path(),
        &["config", "user.email", "test@example.com"],
    );
    git(path, home.path(), &["config", "user.name", "Test User"]);

    // Listing tags in an empty repo is fine
    termai(path, home.path())
        .args(["tag", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No tags found"));

    // Creating a tag with no commits fails gracefully (no HEAD to tag)
    termai(path, home.path())
        .args(["tag", "create", "v0.1.0", "-m", "no commits yet"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Tag command failed"));

    // Suggesting a tag reports the honest empty state
    termai(path, home.path())
        .args(["tag", "suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no tags yet"));
}

// ---------------------------------------------------------------------------
// Rebase
// ---------------------------------------------------------------------------

#[test]
fn rebase_start_shows_real_plan_then_fails_honestly() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    fs::write(path.join("helper.txt"), "helper\n").expect("write helper");
    git(path, home.path(), &["add", "helper.txt"]);
    git(path, home.path(), &["commit", "-m", "feat: add helper"]);

    // start analyzes real commits, prints a plan, then errors because rebase
    // execution is intentionally not implemented (honest failure). The
    // specific "not supported yet" reason must reach stderr, not just the
    // generic banner.
    termai(path, home.path())
        .args(["rebase", "start"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Suggested Rebase Plan"))
        .stdout(predicate::str::contains("feat: add helper"))
        .stderr(predicate::str::contains("Rebase command failed"))
        .stderr(predicate::str::contains("not supported yet"));
}

#[test]
fn rebase_continue_abort_skip_are_noops_without_a_rebase() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    termai(path, home.path())
        .args(["rebase", "continue"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No rebase in progress"));

    termai(path, home.path())
        .args(["rebase", "abort"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to abort"));

    termai(path, home.path())
        .args(["rebase", "skip"])
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to skip"));
}

#[test]
fn rebase_detects_real_in_progress_conflicted_rebase() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    setup_conflicting_branches(path, home.path());

    // Start a real conflicting rebase with native git: rebasing feature onto
    // main stops on the conflicting commit.
    git(path, home.path(), &["checkout", "feature"]);
    git_expect_failure(path, home.path(), &["rebase", "main"]);

    // termai must detect the real in-progress rebase and its conflicts
    termai(path, home.path())
        .args(["rebase", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active Rebase"))
        .stdout(predicate::str::contains("Progress:"))
        .stdout(predicate::str::contains("shared.txt"));

    // continue/abort/skip mid-rebase are honest unsupported errors that leave
    // the rebase state intact for native git.
    for action in ["continue", "abort", "skip"] {
        termai(path, home.path())
            .args(["rebase", action])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Rebase command failed"));
    }

    // The rebase must still be abortable with native git...
    git(path, home.path(), &["rebase", "--abort"]);

    // ...after which termai reports the clean state
    termai(path, home.path())
        .args(["rebase", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No rebase in progress"));
}

// ---------------------------------------------------------------------------
// Conflicts
// ---------------------------------------------------------------------------

#[test]
fn conflicts_detect_parses_real_conflict_markers() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    setup_conflicting_branches(path, home.path());
    git_expect_failure(path, home.path(), &["merge", "feature"]);

    // The working tree really contains conflict markers now
    let content = fs::read_to_string(path.join("shared.txt")).unwrap();
    assert!(content.contains("<<<<<<<"), "merge must leave real markers");

    termai(path, home.path())
        .args(["conflicts", "detect"])
        .assert()
        .success()
        .stdout(predicate::str::contains("conflicts detected"))
        .stdout(predicate::str::contains("shared.txt"))
        // Parsed from the real markers: line number plus ours/theirs labels
        .stdout(predicate::str::contains("Line 2:"))
        .stdout(predicate::str::contains("HEAD"))
        .stdout(predicate::str::contains("feature"));
}

#[test]
fn conflicts_status_tracks_merge_lifecycle() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    setup_conflicting_branches(path, home.path());
    git_expect_failure(path, home.path(), &["merge", "feature"]);

    // During the failed merge: operation and conflicted file are reported
    termai(path, home.path())
        .args(["conflicts", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Merge in progress"))
        .stdout(predicate::str::contains("Conflicted Files"))
        .stdout(predicate::str::contains("shared.txt"));

    // After aborting the merge, the clean state is reported honestly
    git(path, home.path(), &["merge", "--abort"]);
    termai(path, home.path())
        .args(["conflicts", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No merge operation in progress"))
        .stdout(predicate::str::contains("No conflicted files"));
}

// ---------------------------------------------------------------------------
// Commit
// ---------------------------------------------------------------------------

#[test]
fn commit_add_all_stages_changes_before_llm_step() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    fs::write(path.join("dirty.txt"), "uncommitted change\n").expect("write dirty file");

    // Without API keys the AI step fails and falls back to a heuristic
    // message; without a TTY the interactive approval then fails. The overall
    // command exits non-zero, but --add-all must have staged the changes
    // BEFORE the LLM call.
    termai(path, home.path())
        .args(["commit", "--add-all"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Staged all changes"))
        .stdout(predicate::str::contains("AI generation failed"))
        .stdout(predicate::str::contains(
            "Falling back to heuristic generation",
        ))
        // No leftover debug tracing may pollute stderr.
        .stderr(predicate::str::contains("DEBUG:").not());

    // The file really is staged in the index
    let staged = git_stdout(path, home.path(), &["diff", "--cached", "--name-only"]);
    assert!(
        staged.contains("dirty.txt"),
        "commit --add-all must stage files before the LLM step, staged: {}",
        staged
    );

    // But no commit was created (approval never happened)
    let count = git_stdout(path, home.path(), &["rev-list", "--count", "HEAD"]);
    assert_eq!(
        count.trim(),
        "1",
        "no commit must be created without approval"
    );
}

// ---------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------

#[test]
fn hooks_install_creates_real_hook_files() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    termai(path, home.path())
        .args(["hooks", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Git Hooks Status"))
        .stdout(predicate::str::contains("pre-commit"));

    // Install a single hook non-interactively by naming it
    termai(path, home.path())
        .args(["hooks", "install", "pre-commit"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "pre-commit hook installed successfully",
        ));

    let pre_commit = path.join(".git/hooks/pre-commit");
    assert!(pre_commit.exists(), "pre-commit hook file must be created");
    let content = fs::read_to_string(&pre_commit).unwrap();
    assert!(
        content.contains("TermAI"),
        "installed hook must be TermAI-managed, got: {}",
        content
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&pre_commit).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "hook must be executable");
    }

    // install-all installs the remaining recommended hooks for real
    termai(path, home.path())
        .args(["hooks", "install-all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hook installation completed"));
    assert!(
        path.join(".git/hooks/commit-msg").exists(),
        "install-all must create the commit-msg hook"
    );

    // Status now reflects the really installed hooks
    termai(path, home.path())
        .args(["hooks", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TermAI managed: 2"));
}

// ---------------------------------------------------------------------------
// AI-dependent commands without API keys
// ---------------------------------------------------------------------------

#[test]
fn review_and_branch_summary_run_without_api_keys() {
    let (repo, home) = setup_repo();
    let path = repo.path();

    // review is rule-based and completes offline against staged changes
    fs::write(path.join("review_me.txt"), "content to review\n").expect("write file");
    git(path, home.path(), &["add", "review_me.txt"]);

    termai(path, home.path())
        .arg("review")
        .assert()
        .success()
        .stdout(predicate::str::contains("Code Review Results"));

    // branch-summary is likewise rule-based and needs no key
    termai(path, home.path())
        .arg("branch-summary")
        .assert()
        .success()
        .stdout(predicate::str::contains("Branch Analysis"));
}

// ---------------------------------------------------------------------------
// Outside any git repository
// ---------------------------------------------------------------------------

#[test]
fn stash_commit_and_hooks_outside_git_repo_fail_gracefully() {
    // tag/rebase/conflicts/branch-summary outside a repo are already covered
    // by tests/git_integration_tests.rs; this covers the remaining commands.
    let non_repo = TempDir::new().expect("create non-repo tempdir");
    let home = TempDir::new().expect("create home tempdir");
    let path = non_repo.path();

    termai(path, home.path())
        .args(["stash", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Stash command failed"));

    termai(path, home.path())
        .arg("commit")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Commit command failed"));

    termai(path, home.path())
        .args(["hooks", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Hooks command failed"));
}
