#[allow(dead_code)]
pub mod commit;
#[allow(dead_code)]
pub mod diff;
#[allow(dead_code)]
pub mod hooks;
/// Git integration module for TermAI
///
/// This module provides comprehensive Git integration capabilities including:
/// - Repository detection and operations
/// - Diff analysis and parsing
/// - Commit message generation
/// - Code review functionality
/// - Hook management
/// - Interactive workflows
#[allow(dead_code)]
pub mod repository;
#[allow(dead_code)]
pub mod review;

// pub use repository::GitRepository;
// pub use diff::{DiffAnalyzer, ChangeType, FileChange};
// pub use commit::{CommitMessageGenerator, CommitMessage};
// pub use review::{CodeReviewer, ReviewResult, ReviewIssue};
// pub use hooks::{HookManager, HookType};
