/// Branch tree visualization and navigation module
use crate::branch::entity::branch_entity::BranchEntity;
use crate::branch::service::BranchService;
use crate::repository::db::SqliteRepository;
use anyhow::Result;
use colored::*;
use std::collections::HashMap;

/// Represents a node in the branch tree visualization
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub branch: BranchEntity,
    pub children: Vec<TreeNode>,
    pub message_count: usize,
    pub is_current: bool,
}

/// Branch tree visualization and navigation
pub struct BranchTree;

impl BranchTree {
    /// Create a visual representation of the branch tree for a session
    pub fn visualize_session_tree(
        repo: &SqliteRepository, 
        session_id: &str,
        current_branch_id: Option<&str>
    ) -> Result<String> {
        // Get all branches for the session
        let branches = BranchService::get_session_branches(repo, session_id)?;
        
        if branches.is_empty() {
            return Ok("No branches found for this session.".dimmed().to_string());
        }

        // Build tree structure
        let tree_nodes = Self::build_tree_structure(&branches, current_branch_id)?;
        
        // Generate visualization
        let mut output = String::new();
        output.push_str(&format!("{}\n", "Branch Tree".bright_green().bold()));
        output.push_str(&format!("{}\n", "═".repeat(12).dimmed()));
        
        for root_node in &tree_nodes {
            Self::render_tree_node(&mut output, root_node, "", true, true)?;
        }
        
        // Add legend
        output.push_str("\n");
        output.push_str(&format!("{}\n", "Legend:".bright_yellow().bold()));
        output.push_str(&format!("  {} Current branch\n", "*".bright_green()));
        output.push_str(&format!("  {} Active branch\n", "●".bright_blue()));
        output.push_str(&format!("  {} Archived branch\n", "○".dimmed()));
        
        Ok(output)
    }

    /// Build tree structure from flat list of branches
    fn build_tree_structure(
        branches: &[BranchEntity], 
        current_branch_id: Option<&str>
    ) -> Result<Vec<TreeNode>> {
        // Create lookup map
        let mut branch_map: HashMap<String, BranchEntity> = HashMap::new();
        let mut children_map: HashMap<String, Vec<BranchEntity>> = HashMap::new();
        
        for branch in branches {
            branch_map.insert(branch.id.clone(), branch.clone());
            
            if let Some(parent_id) = &branch.parent_branch_id {
                children_map.entry(parent_id.clone())
                    .or_insert_with(Vec::new)
                    .push(branch.clone());
            }
        }
        
        // Find root branches (no parent)
        let root_branches: Vec<&BranchEntity> = branches
            .iter()
            .filter(|branch| branch.parent_branch_id.is_none())
            .collect();
        
        // Build tree recursively
        let mut tree_nodes = Vec::new();
        for root_branch in root_branches {
            let tree_node = Self::build_tree_node(
                root_branch,
                &children_map,
                current_branch_id
            )?;
            tree_nodes.push(tree_node);
        }
        
        Ok(tree_nodes)
    }
    
    /// Build a tree node recursively
    fn build_tree_node(
        branch: &BranchEntity,
        children_map: &HashMap<String, Vec<BranchEntity>>,
        current_branch_id: Option<&str>
    ) -> Result<TreeNode> {
        let is_current = current_branch_id == Some(&branch.id);
        
        // Get message count (placeholder for now)
        let message_count = 0; // TODO: Implement message counting
        
        let mut children = Vec::new();
        if let Some(child_branches) = children_map.get(&branch.id) {
            for child_branch in child_branches {
                let child_node = Self::build_tree_node(
                    child_branch, 
                    children_map, 
                    current_branch_id
                )?;
                children.push(child_node);
            }
        }
        
        Ok(TreeNode {
            branch: branch.clone(),
            children,
            message_count,
            is_current,
        })
    }
    
    /// Render a single tree node and its children
    fn render_tree_node(
        output: &mut String,
        node: &TreeNode,
        prefix: &str,
        is_last: bool,
        is_root: bool,
    ) -> Result<()> {
        // Choose connector
        let connector = if is_root {
            ""
        } else if is_last {
            "└── "
        } else {
            "├── "
        };
        
        // Choose status indicator
        let status_indicator = if node.is_current {
            "*".bright_green()
        } else if node.branch.status == "active" {
            "●".bright_blue()
        } else {
            "○".dimmed()
        };
        
        // Format branch name
        let branch_name = if let Some(name) = &node.branch.branch_name {
            name.clone()
        } else {
            format!("branch-{}", &node.branch.id[..8])
        };
        
        let branch_display = if node.is_current {
            branch_name.bright_green().bold()
        } else if node.branch.status == "active" {
            branch_name.bright_white()
        } else {
            branch_name.dimmed()
        };
        
        // Format message count
        let message_info = if node.message_count > 0 {
            format!(" ({} msgs)", node.message_count).dimmed().to_string()
        } else {
            "".to_string()
        };
        
        // Format timestamp
        let time_info = format!(" [{}]", 
            node.branch.created_at.format("%Y-%m-%d").to_string()
        ).dimmed();
        
        // Add line to output
        output.push_str(&format!(
            "{}{} {}{}{}{}\n", 
            prefix, 
            connector, 
            status_indicator, 
            branch_display, 
            message_info,
            time_info
        ));
        
        // Render children
        let child_prefix = if is_root {
            prefix.to_string()
        } else if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };
        
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            Self::render_tree_node(output, child, &child_prefix, is_last_child, false)?;
        }
        
        Ok(())
    }

    /// Generate a compact tree view for session listings
    #[allow(dead_code)]
    pub fn compact_tree_view(
        repo: &SqliteRepository,
        session_id: &str,
        max_lines: usize,
    ) -> Result<String> {
        let branches = BranchService::get_session_branches(repo, session_id)?;
        
        if branches.is_empty() {
            return Ok("No branches".dimmed().to_string());
        }
        
        let mut output = String::new();
        let branch_count = branches.len();
        
        if branch_count == 1 {
            // Single branch - show inline
            let branch = &branches[0];
            let name = branch.branch_name
                .as_deref()
                .unwrap_or("main");
            output.push_str(&format!("{}", name.bright_blue()));
        } else if branch_count <= max_lines {
            // Show all branches
            for (i, branch) in branches.iter().enumerate() {
                let name = branch.branch_name
                    .as_deref()
                    .unwrap_or("main");
                
                let connector = if i == branches.len() - 1 { "└─" } else { "├─" };
                output.push_str(&format!("{} {}\n", connector.dimmed(), name.bright_blue()));
            }
            output.pop(); // Remove last newline
        } else {
            // Show summary
            output.push_str(&format!(
                "{} branches ({}...)", 
                branch_count.to_string().bright_blue().bold(),
                branches.first().unwrap().branch_name
                    .as_deref()
                    .unwrap_or("main")
                    .bright_blue()
            ));
        }
        
        Ok(output)
    }

    /// Generate an interactive tree navigation view
    #[allow(dead_code)]
    pub fn interactive_tree_view(
        repo: &SqliteRepository,
        session_id: &str,
        selected_branch_id: Option<&str>,
    ) -> Result<String> {
        let branches = BranchService::get_session_branches(repo, session_id)?;
        
        if branches.is_empty() {
            return Ok("No branches to navigate.".dimmed().to_string());
        }

        let tree_nodes = Self::build_tree_structure(&branches, selected_branch_id)?;
        
        let mut output = String::new();
        output.push_str(&format!("{}\n", "🌳 Interactive Branch Navigator".bright_green().bold()));
        output.push_str(&format!("{}\n", "═".repeat(32).dimmed()));
        output.push_str(&format!("{}\n", "[↑↓] Navigate  [Enter] Switch  [q] Quit".bright_yellow()));
        output.push_str("\n");
        
        for root_node in &tree_nodes {
            Self::render_interactive_node(&mut output, root_node, "", true, selected_branch_id)?;
        }
        
        Ok(output)
    }
    
    /// Render interactive tree node with selection highlighting
    #[allow(dead_code)]
    fn render_interactive_node(
        output: &mut String,
        node: &TreeNode,
        prefix: &str,
        is_last: bool,
        selected_branch_id: Option<&str>,
    ) -> Result<()> {
        let connector = if is_last { "└── " } else { "├── " };
        
        let branch_name = node.branch.branch_name
            .as_deref()
            .unwrap_or("main");
        
        let is_selected = selected_branch_id == Some(&node.branch.id);
        
        let line = if is_selected {
            format!("{}{}→ {} {}", 
                prefix, 
                connector, 
                branch_name.bright_green().bold().on_blue(), 
                "[SELECTED]".bright_white().bold()
            )
        } else if node.is_current {
            format!("{}{}{} {} {}", 
                prefix, 
                connector, 
                "*".bright_green(), 
                branch_name.bright_green(), 
                "[current]".dimmed()
            )
        } else {
            format!("{}{} {}", 
                prefix, 
                connector, 
                branch_name.bright_white()
            )
        };
        
        output.push_str(&format!("{}\n", line));
        
        // Render children
        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };
        
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            Self::render_interactive_node(
                output, 
                child, 
                &child_prefix, 
                is_last_child, 
                selected_branch_id
            )?;
        }
        
        Ok(())
    }
}

/// Branch navigation utilities
pub struct BranchNavigator;

impl BranchNavigator {
    /// Get navigation suggestions for a session
    pub fn get_navigation_suggestions(
        repo: &SqliteRepository,
        session_id: &str,
        current_branch_id: Option<&str>,
    ) -> Result<Vec<NavigationSuggestion>> {
        let branches = BranchService::get_session_branches(repo, session_id)?;
        let mut suggestions = Vec::new();
        
        if let Some(current_id) = current_branch_id {
            // Find current branch
            if let Some(current_branch) = branches.iter().find(|b| b.id == current_id) {
                // Suggest parent branch
                if let Some(parent_id) = &current_branch.parent_branch_id {
                    if let Some(parent) = branches.iter().find(|b| b.id == *parent_id) {
                        suggestions.push(NavigationSuggestion {
                            branch_id: parent.id.clone(),
                            branch_name: parent.branch_name.clone(),
                            suggestion_type: SuggestionType::Parent,
                            reason: "Navigate back to parent branch".to_string(),
                        });
                    }
                }
                
                // Suggest sibling branches
                for branch in &branches {
                    if branch.id != current_id && 
                       branch.parent_branch_id == current_branch.parent_branch_id {
                        suggestions.push(NavigationSuggestion {
                            branch_id: branch.id.clone(),
                            branch_name: branch.branch_name.clone(),
                            suggestion_type: SuggestionType::Sibling,
                            reason: "Explore alternative approach".to_string(),
                        });
                    }
                }
                
                // Suggest child branches
                for branch in &branches {
                    if branch.parent_branch_id == Some(current_id.to_string()) {
                        suggestions.push(NavigationSuggestion {
                            branch_id: branch.id.clone(),
                            branch_name: branch.branch_name.clone(),
                            suggestion_type: SuggestionType::Child,
                            reason: "Continue deeper exploration".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(suggestions)
    }
}

/// Navigation suggestion for branch switching
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NavigationSuggestion {
    pub branch_id: String,
    pub branch_name: Option<String>,
    pub suggestion_type: SuggestionType,
    pub reason: String,
}

/// Type of navigation suggestion
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SuggestionType {
    Parent,
    Sibling,
    Child,
    Recent,
    Popular,
}