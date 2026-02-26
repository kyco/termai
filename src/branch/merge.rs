/// Branch merging and integration module for TermAI
/// 
/// This module provides sophisticated merging capabilities for conversation branches,
/// allowing users to consolidate insights, resolve conflicts, and maintain conversation flow.
use crate::branch::entity::branch_entity::BranchEntity;
use crate::branch::service::BranchService;
use crate::branch::comparison::{BranchComparator, BranchComparison};
use crate::repository::db::SqliteRepository;
use crate::session::model::message::Message;
use anyhow::Result;
use colored::*;

/// Represents a merge operation between branches
#[derive(Debug, Clone)]
pub struct BranchMerge {
    pub source_branches: Vec<BranchEntity>,
    pub target_branch: BranchEntity,
    pub merge_strategy: MergeStrategy,
    pub conflicts: Vec<MergeConflict>,
    pub resolution_plan: MergeResolutionPlan,
}

/// Strategy for merging branches
#[derive(Debug, Clone, PartialEq)]
pub enum MergeStrategy {
    /// Merge all messages from all branches sequentially
    Sequential,
    /// Merge messages intelligently, removing duplicates and conflicts
    Intelligent,
    /// Allow user to manually select which messages to include
    Selective,
    /// Create a consolidated summary of all branches
    Summary,
    /// Keep only the best responses based on quality scores
    BestOf,
}

/// Conflict between branches during merge
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MergeConflict {
    pub conflict_type: ConflictType,
    pub message_sequence: usize,
    pub conflicting_messages: Vec<ConflictingMessage>,
    pub suggested_resolution: ConflictResolution,
    pub severity: ConflictSeverity,
}

/// Type of merge conflict
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ConflictType {
    DuplicateContent,
    ContradictoryResponses,
    SequenceGap,
    RoleConflict,
    QualityDisparities,
}

/// Conflicting message in a merge
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConflictingMessage {
    pub branch_id: String,
    pub branch_name: String,
    pub message: Message,
    pub confidence_score: f64,
}

/// Resolution for a merge conflict
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConflictResolution {
    pub resolution_type: ResolutionType,
    pub selected_messages: Vec<String>, // Message IDs
    pub merged_content: Option<String>,
    pub rationale: String,
}

/// Type of conflict resolution
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ResolutionType {
    KeepFirst,
    KeepBest,
    KeepAll,
    Merge,
    Skip,
    UserDecision,
}

/// Severity of merge conflict
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ConflictSeverity {
    Low,    // Minor differences, auto-resolvable
    Medium, // Significant differences, suggestion provided
    High,   // Major conflicts, requires user decision
}

/// Plan for resolving merge conflicts
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MergeResolutionPlan {
    pub auto_resolvable_conflicts: Vec<usize>,
    pub user_decision_required: Vec<usize>,
    pub suggested_actions: Vec<String>,
    pub estimated_result_quality: f64,
}

/// Branch merge engine
pub struct BranchMerger;

impl BranchMerger {
    /// Merge multiple branches into a target branch
    pub fn merge_branches(
        repo: &mut SqliteRepository,
        source_branch_ids: &[String],
        target_branch_id: &str,
        strategy: MergeStrategy,
    ) -> Result<BranchMerge> {
        // Fetch source branches
        let mut source_branches = Vec::new();
        for branch_id in source_branch_ids {
            if let Some(branch) = BranchService::get_branch(repo, branch_id)? {
                source_branches.push(branch);
            } else {
                return Err(anyhow::anyhow!("Source branch {} not found", branch_id));
            }
        }

        // Fetch target branch
        let target_branch = BranchService::get_branch(repo, target_branch_id)?
            .ok_or_else(|| anyhow::anyhow!("Target branch {} not found", target_branch_id))?;

        // Analyze potential conflicts
        let comparison = BranchComparator::compare_branches(repo, source_branch_ids)?;
        let conflicts = Self::detect_conflicts(&comparison, &source_branches)?;
        
        // Create resolution plan
        let resolution_plan = Self::create_resolution_plan(&conflicts, &strategy)?;

        Ok(BranchMerge {
            source_branches,
            target_branch,
            merge_strategy: strategy,
            conflicts,
            resolution_plan,
        })
    }

    /// Perform selective merge with user-specified messages
    pub fn selective_merge(
        repo: &mut SqliteRepository,
        source_branch_id: &str,
        target_branch_id: &str,
        selected_message_indices: &[usize],
    ) -> Result<BranchMerge> {
        let source_branch = BranchService::get_branch(repo, source_branch_id)?
            .ok_or_else(|| anyhow::anyhow!("Source branch {} not found", source_branch_id))?;
        
        let target_branch = BranchService::get_branch(repo, target_branch_id)?
            .ok_or_else(|| anyhow::anyhow!("Target branch {} not found", target_branch_id))?;

        // Get source messages
        let source_messages = BranchService::get_branch_messages(repo, source_branch_id)?;
        
        // Validate indices
        for &index in selected_message_indices {
            if index >= source_messages.len() {
                return Err(anyhow::anyhow!("Message index {} out of range", index));
            }
        }

        // Add selected messages to target branch
        for &index in selected_message_indices {
            let message = &source_messages[index];
            BranchService::add_message_to_branch(repo, target_branch_id, message)?;
        }

        Ok(BranchMerge {
            source_branches: vec![source_branch],
            target_branch,
            merge_strategy: MergeStrategy::Selective,
            conflicts: Vec::new(),
            resolution_plan: MergeResolutionPlan {
                auto_resolvable_conflicts: Vec::new(),
                user_decision_required: Vec::new(),
                suggested_actions: vec![format!("Merged {} selected messages", selected_message_indices.len())],
                estimated_result_quality: 85.0,
            },
        })
    }

    /// Archive branches after successful merge
    pub fn archive_merged_branches(
        repo: &mut SqliteRepository,
        branch_ids: &[String],
    ) -> Result<Vec<String>> {
        let mut archived_branches = Vec::new();

        for branch_id in branch_ids {
            // Update branch status to archived
            repo.conn.execute(
                "UPDATE conversation_branches SET status = 'archived' WHERE id = ?1",
                [branch_id],
            )?;
            
            // Add archived metadata
            repo.conn.execute(
                "INSERT OR REPLACE INTO branch_metadata (branch_id, key, value) VALUES (?1, ?2, ?3)",
                [branch_id, "archived_reason", "merged"],
            )?;
            
            archived_branches.push(branch_id.clone());
        }

        Ok(archived_branches)
    }

    /// Clean up branches marked for deletion
    pub fn cleanup_branches(
        repo: &mut SqliteRepository,
        session_id: &str,
        cleanup_strategy: CleanupStrategy,
    ) -> Result<CleanupResult> {
        let branches = BranchService::get_session_branches(repo, session_id)?;
        let mut cleaned_branches = Vec::new();
        let mut preserved_branches = Vec::new();

        for branch in &branches {
            let should_cleanup = match cleanup_strategy {
                CleanupStrategy::ArchiveOld => {
                    branch.status == "archived" && 
                    Self::is_branch_old(branch, 30) // 30 days
                }
                CleanupStrategy::RemoveEmpty => {
                    let messages = BranchService::get_branch_messages(repo, &branch.id)?;
                    messages.is_empty()
                }
                CleanupStrategy::ConsolidateSimilar => {
                    // This would require more complex analysis
                    false // Placeholder
                }
                CleanupStrategy::RemoveDuplicates => {
                    Self::is_duplicate_branch(repo, branch, &branches)?
                }
            };

            if should_cleanup {
                Self::delete_branch(repo, &branch.id)?;
                cleaned_branches.push(branch.clone());
            } else {
                preserved_branches.push(branch.clone());
            }
        }

        Ok(CleanupResult {
            cleaned_branches,
            preserved_branches,
            cleanup_strategy,
        })
    }

    /// Export branches to external formats
    pub fn export_branches(
        repo: &SqliteRepository,
        branch_ids: &[String],
        export_format: ExportFormat,
    ) -> Result<ExportResult> {
        let mut exported_data = String::new();
        let mut exported_branches = Vec::new();

        for branch_id in branch_ids {
            if let Some(branch) = BranchService::get_branch(repo, branch_id)? {
                let messages = BranchService::get_branch_messages(repo, branch_id)?;
                
                let branch_data = match export_format {
                    ExportFormat::Json => Self::export_branch_json(&branch, &messages)?,
                    ExportFormat::Markdown => Self::export_branch_markdown(&branch, &messages)?,
                    ExportFormat::PlainText => Self::export_branch_text(&branch, &messages)?,
                    ExportFormat::Csv => Self::export_branch_csv(&branch, &messages)?,
                };
                
                exported_data.push_str(&branch_data);
                exported_branches.push(branch);
            }
        }

        Ok(ExportResult {
            exported_branches,
            export_format,
            exported_data,
        })
    }

    /// Format merge preview for user review
    pub fn format_merge_preview(merge: &BranchMerge) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}\n", "📋 Merge Preview".bright_green().bold()));
        output.push_str(&format!("{}\n", "═".repeat(15).dimmed()));
        output.push('\n');

        // Source branches
        output.push_str(&format!("{}\n", "Source branches:".bright_cyan()));
        for branch in &merge.source_branches {
            let name = branch.branch_name.as_deref().unwrap_or("unnamed");
            output.push_str(&format!("   🌿 {}\n", name.bright_white()));
        }

        // Target branch
        let target_name = merge.target_branch.branch_name.as_deref().unwrap_or("unnamed");
        output.push_str(&format!("{} {}\n", "Target branch:".bright_cyan(), target_name.bright_white()));
        
        // Strategy
        let strategy_desc = match merge.merge_strategy {
            MergeStrategy::Sequential => "Sequential merge (all messages in order)",
            MergeStrategy::Intelligent => "Intelligent merge (remove duplicates, resolve conflicts)",
            MergeStrategy::Selective => "Selective merge (user-chosen messages only)",
            MergeStrategy::Summary => "Summary merge (consolidated overview)",
            MergeStrategy::BestOf => "Best-of merge (highest quality responses only)",
        };
        output.push_str(&format!("{} {}\n", "Strategy:".bright_cyan(), strategy_desc.bright_white()));
        
        output.push('\n');

        // Conflicts
        if !merge.conflicts.is_empty() {
            output.push_str(&format!("{}\n", "⚠️  Conflicts Detected:".bright_yellow().bold()));
            output.push_str(&format!("   {} conflicts require attention\n", merge.conflicts.len()));
            
            let high_severity = merge.conflicts.iter().filter(|c| c.severity == ConflictSeverity::High).count();
            let medium_severity = merge.conflicts.iter().filter(|c| c.severity == ConflictSeverity::Medium).count();
            let low_severity = merge.conflicts.iter().filter(|c| c.severity == ConflictSeverity::Low).count();
            
            if high_severity > 0 {
                output.push_str(&format!("   {} {} high-priority conflicts\n", "❗".bright_red(), high_severity));
            }
            if medium_severity > 0 {
                output.push_str(&format!("   {} {} medium-priority conflicts\n", "⚠️".bright_yellow(), medium_severity));
            }
            if low_severity > 0 {
                output.push_str(&format!("   {} {} low-priority conflicts (auto-resolvable)\n", "ℹ️".bright_blue(), low_severity));
            }
        } else {
            output.push_str(&format!("{}\n", "✅ No conflicts detected".bright_green()));
        }

        // Resolution plan
        output.push('\n');
        output.push_str(&format!("{}\n", "🎯 Resolution Plan:".bright_yellow().bold()));
        for action in &merge.resolution_plan.suggested_actions {
            output.push_str(&format!("   • {}\n", action));
        }
        
        if merge.resolution_plan.user_decision_required.is_empty() {
            output.push_str(&format!("   {} All conflicts can be auto-resolved\n", "✅".bright_green()));
        } else {
            output.push_str(&format!("   {} {} conflicts require your decision\n", 
                "👤".bright_blue(), 
                merge.resolution_plan.user_decision_required.len()
            ));
        }

        output.push_str(&format!("\n{} {:.1}%\n", 
            "Estimated result quality:".bright_cyan(),
            merge.resolution_plan.estimated_result_quality
        ));

        output
    }

    /// Detect conflicts between branches
    fn detect_conflicts(
        comparison: &BranchComparison,
        _source_branches: &[BranchEntity],
    ) -> Result<Vec<MergeConflict>> {
        let mut conflicts = Vec::new();

        // Analyze message comparisons for conflicts
        for (seq_num, msg_comparison) in comparison.message_comparisons.iter().enumerate() {
            // Check for low similarity (potential conflict)
            if msg_comparison.similarity_score < 0.3 && msg_comparison.messages.iter().filter(|m| m.is_some()).count() > 1 {
                // This would be a more sophisticated conflict detection
                // For now, we'll create a placeholder conflict
                conflicts.push(MergeConflict {
                    conflict_type: ConflictType::ContradictoryResponses,
                    message_sequence: seq_num,
                    conflicting_messages: Vec::new(), // Would be populated with actual conflicts
                    suggested_resolution: ConflictResolution {
                        resolution_type: ResolutionType::KeepBest,
                        selected_messages: Vec::new(),
                        merged_content: None,
                        rationale: "Messages have low similarity, suggest keeping highest quality response".to_string(),
                    },
                    severity: if msg_comparison.similarity_score < 0.1 {
                        ConflictSeverity::High
                    } else {
                        ConflictSeverity::Medium
                    },
                });
            }
        }

        Ok(conflicts)
    }

    /// Create resolution plan for conflicts
    fn create_resolution_plan(
        conflicts: &[MergeConflict],
        strategy: &MergeStrategy,
    ) -> Result<MergeResolutionPlan> {
        let auto_resolvable: Vec<usize> = conflicts.iter()
            .enumerate()
            .filter(|(_, conflict)| {
                conflict.severity == ConflictSeverity::Low || 
                *strategy == MergeStrategy::BestOf
            })
            .map(|(i, _)| i)
            .collect();

        let user_decision_required: Vec<usize> = conflicts.iter()
            .enumerate()
            .filter(|(_, conflict)| conflict.severity == ConflictSeverity::High)
            .map(|(i, _)| i)
            .collect();

        let suggested_actions = match strategy {
            MergeStrategy::Sequential => vec!["Merge all messages in chronological order".to_string()],
            MergeStrategy::Intelligent => vec![
                "Auto-resolve low-priority conflicts".to_string(),
                "Present high-priority conflicts for user decision".to_string(),
            ],
            MergeStrategy::Selective => vec!["Present message selection interface".to_string()],
            MergeStrategy::Summary => vec!["Generate consolidated summary of all branches".to_string()],
            MergeStrategy::BestOf => vec!["Keep only highest-scoring responses".to_string()],
        };

        let estimated_quality = match strategy {
            MergeStrategy::Sequential => 70.0,
            MergeStrategy::Intelligent => 85.0,
            MergeStrategy::Selective => 90.0,
            MergeStrategy::Summary => 80.0,
            MergeStrategy::BestOf => 95.0,
        };

        Ok(MergeResolutionPlan {
            auto_resolvable_conflicts: auto_resolvable,
            user_decision_required,
            suggested_actions,
            estimated_result_quality: estimated_quality,
        })
    }

    /// Check if branch is old based on last activity
    fn is_branch_old(branch: &BranchEntity, days_threshold: i64) -> bool {
        use chrono::{Duration, Utc};
        let threshold = Utc::now().naive_utc() - Duration::days(days_threshold);
        branch.last_activity < threshold
    }

    /// Check if branch is duplicate
    fn is_duplicate_branch(
        _repo: &SqliteRepository,
        _branch: &BranchEntity,
        _all_branches: &[BranchEntity],
    ) -> Result<bool> {
        // This would implement sophisticated duplicate detection
        // For now, return false as placeholder
        Ok(false)
    }

    /// Delete a branch and all its data
    fn delete_branch(repo: &mut SqliteRepository, branch_id: &str) -> Result<()> {
        // Delete branch messages
        repo.conn.execute(
            "DELETE FROM branch_messages WHERE branch_id = ?1",
            [branch_id],
        )?;

        // Delete branch metadata
        repo.conn.execute(
            "DELETE FROM branch_metadata WHERE branch_id = ?1",
            [branch_id],
        )?;

        // Delete the branch itself
        repo.conn.execute(
            "DELETE FROM conversation_branches WHERE id = ?1",
            [branch_id],
        )?;

        Ok(())
    }

    /// Export branch to JSON format
    fn export_branch_json(branch: &BranchEntity, messages: &[Message]) -> Result<String> {
        use serde_json::json;
        
        let branch_json = json!({
            "branch": {
                "id": branch.id,
                "name": branch.branch_name,
                "description": branch.description,
                "created_at": branch.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                "status": branch.status
            },
            "messages": messages.iter().map(|msg| json!({
                "id": msg.id,
                "role": format!("{:?}", msg.role),
                "content": msg.content
            })).collect::<Vec<_>>()
        });

        Ok(format!("{}\n", serde_json::to_string_pretty(&branch_json)?))
    }

    /// Export branch to Markdown format
    fn export_branch_markdown(branch: &BranchEntity, messages: &[Message]) -> Result<String> {
        let mut output = String::new();
        
        let branch_name = branch.branch_name.as_deref().unwrap_or("Unnamed Branch");
        output.push_str(&format!("# {}\n\n", branch_name));
        
        if let Some(desc) = &branch.description {
            output.push_str(&format!("**Description:** {}\n\n", desc));
        }
        
        output.push_str(&format!("**Created:** {}\n", 
            branch.created_at.format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!("**Status:** {}\n\n", branch.status));
        
        output.push_str("## Messages\n\n");
        
        for (i, message) in messages.iter().enumerate() {
            let role_icon = match message.role {
                crate::llm::common::model::role::Role::User => "👤",
                crate::llm::common::model::role::Role::Assistant => "🤖",
                crate::llm::common::model::role::Role::System => "⚙️",
            };
            
            output.push_str(&format!("### {} Message {} ({:?})\n\n", role_icon, i + 1, message.role));
            output.push_str(&format!("{}\n\n", message.content));
        }
        
        Ok(output)
    }

    /// Export branch to plain text format
    fn export_branch_text(branch: &BranchEntity, messages: &[Message]) -> Result<String> {
        let mut output = String::new();
        
        let branch_name = branch.branch_name.as_deref().unwrap_or("Unnamed Branch");
        output.push_str(&format!("Branch: {}\n", branch_name));
        
        if let Some(desc) = &branch.description {
            output.push_str(&format!("Description: {}\n", desc));
        }
        
        output.push_str(&format!("Created: {}\n", 
            branch.created_at.format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!("Status: {}\n", branch.status));
        output.push('\n');
        
        for (i, message) in messages.iter().enumerate() {
            output.push_str(&format!("Message {} ({:?}):\n", i + 1, message.role));
            output.push_str(&format!("{}\n\n", message.content));
        }
        
        Ok(output)
    }

    /// Export branch to CSV format
    fn export_branch_csv(branch: &BranchEntity, messages: &[Message]) -> Result<String> {
        let mut output = String::new();
        
        // CSV header
        output.push_str("branch_id,branch_name,branch_description,branch_created,branch_status,message_id,message_role,message_content\n");
        
        for message in messages {
            let branch_name = branch.branch_name.as_deref().unwrap_or("");
            let branch_desc = branch.description.as_deref().unwrap_or("");
            
            // Escape CSV content
            let escaped_content = message.content.replace("\"", "\"\"");
            let escaped_desc = branch_desc.replace("\"", "\"\"");
            let escaped_name = branch_name.replace("\"", "\"\"");
            
            output.push_str(&format!("\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{:?}\",\"{}\"\n",
                branch.id,
                escaped_name,
                escaped_desc,
                branch.created_at.format("%Y-%m-%d %H:%M:%S"),
                branch.status,
                message.id,
                message.role,
                escaped_content
            ));
        }
        
        Ok(output)
    }
}

/// Cleanup strategy for branch maintenance
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum CleanupStrategy {
    /// Archive branches older than threshold
    ArchiveOld,
    /// Remove branches with no messages
    RemoveEmpty,
    /// Consolidate very similar branches
    ConsolidateSimilar,
    /// Remove duplicate branches
    RemoveDuplicates,
}

/// Result of cleanup operation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CleanupResult {
    pub cleaned_branches: Vec<BranchEntity>,
    pub preserved_branches: Vec<BranchEntity>,
    pub cleanup_strategy: CleanupStrategy,
}

/// Export format for branches
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ExportFormat {
    Json,
    Markdown,
    PlainText,
    Csv,
}

/// Result of export operation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExportResult {
    pub exported_branches: Vec<BranchEntity>,
    pub export_format: ExportFormat,
    pub exported_data: String,
}