// Conversation branching module for TermAI
//
// This module provides tree-like conversation management, allowing users to:
// - Branch conversations at any point to explore alternative paths
// - Compare different approaches and solutions
// - Merge successful branches back into main conversations
// - Navigate complex problem-solving workflows efficiently
#[allow(dead_code)]
#[allow(unused_imports)]
pub mod entity;
pub mod repository;
pub mod service;
pub mod manager;
pub mod tree;
pub mod comparison;
pub mod merge;

// Public API for command integrations
pub use service::BranchService;
pub use tree::{BranchTree, BranchNavigator};
pub use comparison::{BranchComparator, QuickCompare};
pub use merge::{BranchMerger, MergeStrategy, ExportFormat, CleanupStrategy};