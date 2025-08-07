/// Git repository operations and detection
use anyhow::{bail, Context, Result};
use colored::*;
use git2::{Repository, RepositoryState, Status, StatusEntry, StatusOptions};
use std::path::{Path, PathBuf};

/// Git repository wrapper with enhanced functionality
pub struct GitRepository {
    repo: Repository,
    root_path: PathBuf,
}

/// Status of files in the repository
#[derive(Debug, Clone)]
pub struct RepoStatus {
    pub staged_files: Vec<FileStatus>,
    pub unstaged_files: Vec<FileStatus>,
    pub untracked_files: Vec<FileStatus>,
    pub conflicted_files: Vec<FileStatus>,
    pub is_clean: bool,
}

/// Individual file status information
#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: PathBuf,
    pub status: FileStatusType,
    pub old_path: Option<PathBuf>, // For renamed files
}

/// Type of file change
#[derive(Debug, Clone, PartialEq)]
pub enum FileStatusType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Conflicted,
}

impl GitRepository {
    /// Detect and open a Git repository from any subdirectory
    pub fn discover<P: AsRef<Path>>(start_path: P) -> Result<Self> {
        let start_path = start_path.as_ref();

        // Try to find the .git directory first, then open the repository
        let mut current_path = start_path.to_path_buf();

        loop {
            let git_dir = current_path.join(".git");
            if git_dir.exists() {
                let repo = Repository::open(&current_path).with_context(|| {
                    format!(
                        "Failed to open Git repository at: {}",
                        current_path.display()
                    )
                })?;

                let root_path = repo
                    .workdir()
                    .ok_or_else(|| {
                        anyhow::anyhow!("Repository has no working directory (bare repository)")
                    })?
                    .to_path_buf();

                return Ok(Self { repo, root_path });
            }

            // Move up one directory
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_path_buf();
            } else {
                bail!(
                    "No Git repository found at or above: {}",
                    start_path.display()
                );
            }
        }
    }

    /// Open a Git repository at a specific path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let repo = Repository::open(path)
            .with_context(|| format!("Failed to open Git repository at: {}", path.display()))?;

        let root_path = repo
            .workdir()
            .ok_or_else(|| {
                anyhow::anyhow!("Repository has no working directory (bare repository)")
            })?
            .to_path_buf();

        Ok(Self { repo, root_path })
    }

    /// Get the repository root path
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Get access to the inner git2::Repository
    pub fn inner(&self) -> &Repository {
        &self.repo
    }

    /// Check if the repository is in a clean state
    pub fn is_clean(&self) -> Result<bool> {
        let statuses = self
            .repo
            .statuses(Some(StatusOptions::new().include_untracked(false)))?;
        Ok(statuses.is_empty())
    }

    /// Get current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;

        if let Some(branch_name) = head.shorthand() {
            Ok(branch_name.to_string())
        } else if head.target().is_some() {
            // Detached HEAD state
            let oid = head.target().unwrap();
            Ok(format!("detached@{}", &oid.to_string()[..8]))
        } else {
            bail!("Unable to determine current branch")
        }
    }

    /// Get repository state (normal, merge, rebase, etc.)
    pub fn state(&self) -> RepositoryState {
        self.repo.state()
    }

    /// Check if repository has uncommitted changes
    pub fn has_changes(&self) -> Result<bool> {
        let statuses = self.repo.statuses(None)?;
        Ok(!statuses.is_empty())
    }

    /// Get detailed repository status
    pub fn status(&self) -> Result<RepoStatus> {
        let mut status_opts = StatusOptions::new();
        status_opts
            .include_untracked(true)
            .include_ignored(false)
            .recurse_untracked_dirs(true);

        let statuses = self.repo.statuses(Some(&mut status_opts))?;

        let mut staged_files = Vec::new();
        let mut unstaged_files = Vec::new();
        let mut untracked_files = Vec::new();
        let mut conflicted_files = Vec::new();

        for entry in statuses.iter() {
            let file_status = self.parse_status_entry(&entry)?;

            // Check for conflicts first
            if entry.status().is_conflicted() {
                conflicted_files.push(file_status.clone());
            }

            // Check staged changes
            if self.is_staged_change(entry.status()) {
                staged_files.push(file_status.clone());
            }

            // Check unstaged changes
            if self.is_unstaged_change(entry.status()) {
                unstaged_files.push(file_status.clone());
            }

            // Check untracked files
            if entry.status().is_wt_new() {
                untracked_files.push(file_status);
            }
        }

        let is_clean = staged_files.is_empty()
            && unstaged_files.is_empty()
            && untracked_files.is_empty()
            && conflicted_files.is_empty();

        Ok(RepoStatus {
            staged_files,
            unstaged_files,
            untracked_files,
            conflicted_files,
            is_clean,
        })
    }

    /// Get staged changes as a diff string
    pub fn staged_diff(&self) -> Result<String> {
        let tree = if let Ok(head) = self.repo.head() {
            Some(head.peel_to_tree()?)
        } else {
            None
        };

        let diff = self.repo.diff_tree_to_index(
            tree.as_ref(),
            None, // Use current index
            None, // Default options
        )?;

        let mut diff_output = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                '+' | '-' | ' ' => diff_output.push(line.origin()),
                _ => {}
            }
            diff_output.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            true
        })?;

        Ok(diff_output)
    }

    /// Get unstaged changes as a diff string
    pub fn unstaged_diff(&self) -> Result<String> {
        let diff = self.repo.diff_index_to_workdir(None, None)?;

        let mut diff_output = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                '+' | '-' | ' ' => diff_output.push(line.origin()),
                _ => {}
            }
            diff_output.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            true
        })?;

        Ok(diff_output)
    }

    /// Check if we're in a merge state
    pub fn is_merging(&self) -> bool {
        matches!(
            self.repo.state(),
            RepositoryState::Merge | RepositoryState::CherryPick | RepositoryState::Revert
        )
    }

    /// Check if we're in a rebase state
    pub fn is_rebasing(&self) -> bool {
        matches!(
            self.repo.state(),
            RepositoryState::Rebase
                | RepositoryState::RebaseInteractive
                | RepositoryState::RebaseMerge
        )
    }

    /// Get Git user configuration
    pub fn user_config(&self) -> Result<UserConfig> {
        let config = self.repo.config()?;

        let name = config
            .get_string("user.name")
            .unwrap_or_else(|_| "Unknown".to_string());
        let email = config
            .get_string("user.email")
            .unwrap_or_else(|_| "unknown@unknown.com".to_string());

        Ok(UserConfig { name, email })
    }

    /// Check if path is ignored by gitignore
    pub fn is_ignored<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        match self.repo.status_file(path.as_ref()) {
            Ok(flags) => Ok(flags.contains(Status::IGNORED)),
            Err(_) => Ok(false), // File not found or other errors
        }
    }

    /// Get list of remotes
    pub fn remotes(&self) -> Result<Vec<String>> {
        let remotes = self.repo.remotes()?;
        Ok(remotes
            .iter()
            .filter_map(|name| name.map(|s| s.to_string()))
            .collect())
    }

    /// Helper methods for status parsing
    fn parse_status_entry(&self, entry: &StatusEntry) -> Result<FileStatus> {
        let path = PathBuf::from(entry.path().unwrap_or(""));
        let status = self.determine_file_status_type(entry.status());
        let old_path = entry
            .head_to_index()
            .and_then(|diff| diff.old_file().path())
            .map(PathBuf::from);

        Ok(FileStatus {
            path,
            status,
            old_path,
        })
    }

    fn determine_file_status_type(&self, status: Status) -> FileStatusType {
        if status.is_conflicted() {
            FileStatusType::Conflicted
        } else if status.is_index_new() || status.is_wt_new() {
            FileStatusType::Added
        } else if status.is_index_modified() || status.is_wt_modified() {
            FileStatusType::Modified
        } else if status.is_index_deleted() || status.is_wt_deleted() {
            FileStatusType::Deleted
        } else if status.is_index_renamed() || status.is_wt_renamed() {
            FileStatusType::Renamed
        } else if status.is_ignored() {
            FileStatusType::Ignored
        } else {
            FileStatusType::Untracked
        }
    }

    fn is_staged_change(&self, status: Status) -> bool {
        status.is_index_new()
            || status.is_index_modified()
            || status.is_index_deleted()
            || status.is_index_renamed()
            || status.is_index_typechange()
    }

    fn is_unstaged_change(&self, status: Status) -> bool {
        status.is_wt_modified()
            || status.is_wt_deleted()
            || status.is_wt_renamed()
            || status.is_wt_typechange()
    }

    /// Get diff between two branches
    pub fn diff_branches(&self, base_branch: &str, target_branch: &str) -> Result<DiffResult> {
        // Get commit objects for both branches
        let base_commit = self.repo.revparse_single(base_branch)?.peel_to_commit()?;
        let target_commit = self.repo.revparse_single(target_branch)?.peel_to_commit()?;

        // Get trees for both commits
        let base_tree = base_commit.tree()?;
        let target_tree = target_commit.tree()?;

        // Create diff between trees
        let diff = self
            .repo
            .diff_tree_to_tree(Some(&base_tree), Some(&target_tree), None)?;

        // Collect changed files
        let mut changed_files = Vec::new();
        let mut insertions = 0;
        let mut deletions = 0;

        diff.foreach(
            &mut |delta, _progress| {
                if let Some(file_path) = delta.new_file().path() {
                    let change_type = match delta.status() {
                        git2::Delta::Added => crate::git::diff::ChangeType::Added,
                        git2::Delta::Modified => crate::git::diff::ChangeType::Modified,
                        git2::Delta::Deleted => crate::git::diff::ChangeType::Deleted,
                        _ => crate::git::diff::ChangeType::Modified,
                    };

                    changed_files.push(crate::git::diff::SimpleFileChange {
                        path: file_path.to_string_lossy().to_string(),
                        status: change_type,
                    });
                }
                true
            },
            None,
            None,
            Some(&mut |_delta, _hunk, line| {
                match line.origin() {
                    '+' => insertions += 1,
                    '-' => deletions += 1,
                    _ => {}
                }
                true
            }),
        )?;

        Ok(DiffResult {
            changed_files,
            insertions,
            deletions,
        })
    }

    /// Get branch commits (commits that are in the target branch but not in base branch)
    pub fn get_branch_commits(
        &self,
        branch_name: &str,
        base_branch: Option<&str>,
    ) -> Result<Vec<GitCommit>> {
        let mut commits = Vec::new();

        // Get target branch commit
        let target_commit = self.repo.revparse_single(branch_name)?.peel_to_commit()?;

        // If we have a base branch, get commits that are only in target
        if let Some(base) = base_branch {
            let base_commit = self
                .repo
                .revparse_single(base)
                .ok()
                .and_then(|obj| obj.peel_to_commit().ok());

            let mut walker = self.repo.revwalk()?;
            walker.push(target_commit.id())?;

            // If base branch exists, hide its commits
            if let Some(base_commit) = base_commit {
                walker.hide(base_commit.id())?;
            }

            for commit_id in walker.take(20) {
                // Limit to 20 commits
                let commit_id = commit_id?;
                let commit = self.repo.find_commit(commit_id)?;

                commits.push(GitCommit {
                    id: commit.id().to_string(),
                    message: commit.message().unwrap_or("").trim().to_string(),
                    author: commit.author().name().unwrap_or("").to_string(),
                    timestamp: commit.time().seconds(),
                });
            }
        }

        Ok(commits)
    }
}

/// Git user configuration
#[derive(Debug, Clone)]
pub struct UserConfig {
    pub name: String,
    pub email: String,
}

impl std::fmt::Display for FileStatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileStatusType::Added => write!(f, "{}", "Added".green()),
            FileStatusType::Modified => write!(f, "{}", "Modified".yellow()),
            FileStatusType::Deleted => write!(f, "{}", "Deleted".red()),
            FileStatusType::Renamed => write!(f, "{}", "Renamed".blue()),
            FileStatusType::Copied => write!(f, "{}", "Copied".cyan()),
            FileStatusType::Untracked => write!(f, "{}", "Untracked".bright_black()),
            FileStatusType::Ignored => write!(f, "{}", "Ignored".dimmed()),
            FileStatusType::Conflicted => write!(f, "{}", "Conflicted".bright_red()),
        }
    }
}

impl RepoStatus {
    /// Get total count of changed files
    pub fn total_changes(&self) -> usize {
        self.staged_files.len() + self.unstaged_files.len() + self.untracked_files.len()
    }

    /// Check if there are staged changes
    pub fn has_staged_changes(&self) -> bool {
        !self.staged_files.is_empty()
    }

    /// Check if there are unstaged changes
    pub fn has_unstaged_changes(&self) -> bool {
        !self.unstaged_files.is_empty()
    }

    /// Check if there are untracked files
    pub fn has_untracked_files(&self) -> bool {
        !self.untracked_files.is_empty()
    }

    /// Check if there are merge conflicts
    pub fn has_conflicts(&self) -> bool {
        !self.conflicted_files.is_empty()
    }

    /// Display formatted status summary
    pub fn display_summary(&self) {
        if self.is_clean {
            println!("{}", "✅ Working tree is clean".green());
            return;
        }

        if !self.staged_files.is_empty() {
            println!(
                "\n{} ({} files)",
                "📝 Changes to be committed:".green().bold(),
                self.staged_files.len()
            );
            for file in &self.staged_files {
                println!("   {} {}", file.status, file.path.display());
            }
        }

        if !self.unstaged_files.is_empty() {
            println!(
                "\n{} ({} files)",
                "📋 Changes not staged for commit:".yellow().bold(),
                self.unstaged_files.len()
            );
            for file in &self.unstaged_files {
                println!("   {} {}", file.status, file.path.display());
            }
        }

        if !self.untracked_files.is_empty() {
            println!(
                "\n{} ({} files)",
                "❓ Untracked files:".bright_black().bold(),
                self.untracked_files.len()
            );
            for file in &self.untracked_files {
                println!("   {} {}", file.status, file.path.display());
            }
        }

        if !self.conflicted_files.is_empty() {
            println!(
                "\n{} ({} files)",
                "⚠️  Conflicted files:".red().bold(),
                self.conflicted_files.len()
            );
            for file in &self.conflicted_files {
                println!("   {} {}", file.status, file.path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, GitRepository) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Initialize repository
        let repo = Repository::init(temp_dir.path()).expect("Failed to init repo");

        // Set up user config
        let mut config = repo.config().expect("Failed to get config");
        config
            .set_str("user.name", "Test User")
            .expect("Failed to set user name");
        config
            .set_str("user.email", "test@example.com")
            .expect("Failed to set user email");

        let git_repo = GitRepository::open(temp_dir.path()).expect("Failed to open test repo");

        (temp_dir, git_repo)
    }

    #[test]
    fn test_repo_creation_and_detection() {
        let (temp_dir, git_repo) = create_test_repo();

        // Test repository detection from subdirectory
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).expect("Failed to create subdirectory");

        let discovered = GitRepository::discover(&subdir).expect("Failed to discover repo");
        assert_eq!(discovered.root_path(), git_repo.root_path());
    }

    #[test]
    fn test_initial_status() {
        let (_temp_dir, git_repo) = create_test_repo();

        let status = git_repo.status().expect("Failed to get status");
        assert!(status.is_clean);
        assert_eq!(status.total_changes(), 0);
    }

    #[test]
    fn test_user_config() {
        let (_temp_dir, git_repo) = create_test_repo();

        let user_config = git_repo.user_config().expect("Failed to get user config");
        assert_eq!(user_config.name, "Test User");
        assert_eq!(user_config.email, "test@example.com");
    }

    #[test]
    fn test_file_status_with_changes() {
        let (temp_dir, git_repo) = create_test_repo();

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, world!").expect("Failed to write test file");

        let status = git_repo.status().expect("Failed to get status");
        assert!(!status.is_clean);
        assert_eq!(status.untracked_files.len(), 1);
        assert_eq!(status.untracked_files[0].path, PathBuf::from("test.txt"));
    }

    #[test]
    fn test_non_git_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        let result = GitRepository::open(temp_dir.path());
        assert!(result.is_err());

        let result = GitRepository::discover(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_branch_detection() {
        let (_temp_dir, git_repo) = create_test_repo();

        // Initially, there's no branch until we make a commit
        // But we should be able to handle the initial state
        let branch_result = git_repo.current_branch();

        // This might fail for empty repos, which is expected behavior
        // The important thing is that it doesn't panic
        if let Ok(branch) = branch_result {
            assert!(!branch.is_empty());
        }
    }

    #[test]
    fn test_repository_state() {
        let (_temp_dir, git_repo) = create_test_repo();

        let state = git_repo.state();
        assert_eq!(state, RepositoryState::Clean);

        assert!(!git_repo.is_merging());
        assert!(!git_repo.is_rebasing());
    }
}

/// Result of a diff operation between branches
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub changed_files: Vec<crate::git::diff::SimpleFileChange>,
    pub insertions: usize,
    pub deletions: usize,
}

/// Represents a Git commit
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub id: String,
    pub message: String,
    pub author: String,
    pub timestamp: i64,
}
