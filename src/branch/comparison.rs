// Branch comparison features for TermAI
//
// This module provides sophisticated comparison capabilities between conversation branches,
// allowing users to analyze different approaches, solutions, and outcomes side-by-side.
#[allow(dead_code)]
use crate::branch::entity::branch_entity::BranchEntity;
use crate::branch::service::BranchService;
use crate::repository::db::SqliteRepository;
use crate::session::model::message::Message;
use anyhow::Result;
use colored::*;
use std::collections::HashMap;

/// Represents a comparison between two or more branches
#[derive(Debug, Clone)]
pub struct BranchComparison {
    pub branches: Vec<BranchEntity>,
    pub message_comparisons: Vec<MessageComparison>,
    pub summary: ComparisonSummary,
}

/// Comparison between messages at the same sequence position
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MessageComparison {
    pub sequence_number: usize,
    pub messages: Vec<Option<Message>>,
    pub similarity_score: f64,
    pub diff_highlights: Vec<DiffHighlight>,
}

/// Highlighted difference between message content
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiffHighlight {
    pub branch_index: usize,
    pub start_pos: usize,
    pub end_pos: usize,
    pub diff_type: DiffType,
    pub content: String,
}

/// Type of difference between messages
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
    Unique,
}

/// Summary of branch comparison results
#[derive(Debug, Clone)]
pub struct ComparisonSummary {
    pub total_messages_compared: usize,
    pub similarity_percentage: f64,
    pub unique_insights: Vec<UniqueInsight>,
    pub recommendations: Vec<String>,
    pub quality_scores: Vec<BranchQualityScore>,
}

/// Unique insight found in a specific branch
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UniqueInsight {
    pub branch_index: usize,
    pub branch_name: String,
    pub insight_type: InsightType,
    pub description: String,
    pub relevance_score: f64,
}

/// Type of unique insight
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum InsightType {
    Solution,
    Approach,
    Explanation,
    Example,
    Warning,
    Alternative,
}

/// Quality score for a branch
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BranchQualityScore {
    pub branch_index: usize,
    pub branch_name: String,
    pub overall_score: f64,
    pub criteria_scores: HashMap<String, f64>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

/// Branch comparison engine
pub struct BranchComparator;

impl BranchComparator {
    /// Compare two branches side-by-side
    pub fn compare_branches(
        repo: &SqliteRepository,
        branch_ids: &[String],
    ) -> Result<BranchComparison> {
        if branch_ids.len() < 2 {
            return Err(anyhow::anyhow!("Need at least 2 branches to compare"));
        }

        // Fetch branch entities
        let mut branches = Vec::new();
        for branch_id in branch_ids {
            if let Some(branch) = BranchService::get_branch(repo, branch_id)? {
                branches.push(branch);
            } else {
                return Err(anyhow::anyhow!("Branch {} not found", branch_id));
            }
        }

        // Get messages for each branch
        let mut branch_messages = Vec::new();
        for branch in &branches {
            let messages = BranchService::get_branch_messages(repo, &branch.id)?;
            branch_messages.push(messages);
        }

        // Perform message-level comparisons
        let message_comparisons = Self::compare_messages(&branch_messages)?;
        
        // Generate summary and insights
        let summary = Self::generate_comparison_summary(&branches, &message_comparisons)?;

        Ok(BranchComparison {
            branches,
            message_comparisons,
            summary,
        })
    }

    /// Compare branches by name instead of ID
    pub fn compare_branches_by_name(
        repo: &SqliteRepository,
        session_id: &str,
        branch_names: &[String],
    ) -> Result<BranchComparison> {
        let all_branches = BranchService::get_session_branches(repo, session_id)?;
        
        let mut branch_ids = Vec::new();
        for name in branch_names {
            if let Some(branch) = all_branches.iter().find(|b| {
                b.branch_name.as_deref() == Some(name) || b.id == *name
            }) {
                branch_ids.push(branch.id.clone());
            } else {
                return Err(anyhow::anyhow!("Branch '{}' not found in session", name));
            }
        }

        Self::compare_branches(repo, &branch_ids)
    }

    /// Generate side-by-side comparison display
    pub fn format_side_by_side_comparison(comparison: &BranchComparison) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!("{}\n", "📊 Side-by-Side Branch Comparison".bright_green().bold()));
        output.push_str(&format!("{}\n", "═".repeat(40).dimmed()));
        output.push('\n');

        // Branch headers
        let branch_names: Vec<String> = comparison.branches.iter()
            .map(|b| b.branch_name.as_deref().unwrap_or("unnamed").to_string())
            .collect();

        let col_width = 40;
        let separator = " │ ";
        
        // Create column headers
        let mut header_line = String::new();
        for (i, name) in branch_names.iter().enumerate() {
            if i > 0 {
                header_line.push_str(separator);
            }
            header_line.push_str(&format!("{:width$}", 
                format!("🌿 {}", name).bright_blue().bold(), 
                width = col_width
            ));
        }
        output.push_str(&format!("{}\n", header_line));
        
        // Separator line
        let mut sep_line = String::new();
        for i in 0..branch_names.len() {
            if i > 0 {
                sep_line.push_str(&"─┼─".dimmed().to_string());
            }
            sep_line.push_str(&"─".repeat(col_width).dimmed().to_string());
        }
        output.push_str(&format!("{}\n", sep_line));

        // Message comparisons
        for (msg_idx, msg_comparison) in comparison.message_comparisons.iter().enumerate() {
            if msg_comparison.messages.iter().any(|m| m.is_some()) {
                output.push_str(&format!("\n{} {}\n", 
                    "💬 Message".bright_yellow(), 
                    (msg_idx + 1).to_string().bright_white().bold()
                ));
                
                let mut content_lines = Vec::new();
                let max_lines = msg_comparison.messages.iter()
                    .map(|m| match m {
                        Some(msg) => Self::wrap_text(&msg.content, col_width - 2).len(),
                        None => 1,
                    })
                    .max()
                    .unwrap_or(1);

                // Prepare wrapped content for each branch
                for msg_opt in &msg_comparison.messages {
                    match msg_opt {
                        Some(msg) => {
                            let wrapped = Self::wrap_text(&msg.content, col_width - 2);
                            content_lines.push(wrapped);
                        }
                        None => {
                            content_lines.push(vec!["(no message)".dimmed().to_string()]);
                        }
                    }
                }

                // Display content line by line
                for line_idx in 0..max_lines {
                    let mut line = String::new();
                    for (branch_idx, branch_lines) in content_lines.iter().enumerate() {
                        if branch_idx > 0 {
                            line.push_str(separator);
                        }
                        
                        let content = if line_idx < branch_lines.len() {
                            &branch_lines[line_idx]
                        } else {
                            ""
                        };
                        
                        line.push_str(&format!("{:width$}", content, width = col_width));
                    }
                    output.push_str(&format!("{}\n", line));
                }

                // Show similarity score if available
                if msg_comparison.similarity_score < 1.0 {
                    output.push_str(&format!("   {} {:.1}%\n", 
                        "Similarity:".dimmed(), 
                        msg_comparison.similarity_score * 100.0
                    ));
                }
            }
        }

        output
    }

    /// Generate comparison summary with insights
    pub fn format_comparison_summary(comparison: &BranchComparison) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}\n", "📋 Comparison Summary".bright_yellow().bold()));
        output.push_str(&format!("{}\n", "═".repeat(20).dimmed()));
        output.push('\n');

        let summary = &comparison.summary;
        
        // Overall statistics
        output.push_str(&format!("{} {}\n", 
            "Messages compared:".bright_cyan(), 
            summary.total_messages_compared.to_string().bright_white()
        ));
        output.push_str(&format!("{} {:.1}%\n", 
            "Overall similarity:".bright_cyan(), 
            summary.similarity_percentage
        ));
        output.push('\n');

        // Quality scores
        if !summary.quality_scores.is_empty() {
            output.push_str(&format!("{}\n", "🏆 Quality Scores:".bright_yellow().bold()));
            for score in &summary.quality_scores {
                let score_color = if score.overall_score >= 80.0 {
                    "bright_green"
                } else if score.overall_score >= 60.0 {
                    "bright_yellow"
                } else {
                    "bright_red"
                };
                
                output.push_str(&format!("   {} {:.1}/100\n", 
                    score.branch_name.bright_white(), 
                    match score_color {
                        "bright_green" => score.overall_score.to_string().bright_green(),
                        "bright_yellow" => score.overall_score.to_string().bright_yellow(),
                        _ => score.overall_score.to_string().bright_red(),
                    }
                ));
            }
            output.push('\n');
        }

        // Unique insights
        if !summary.unique_insights.is_empty() {
            output.push_str(&format!("{}\n", "💡 Unique Insights:".bright_yellow().bold()));
            for insight in &summary.unique_insights {
                let insight_icon = match insight.insight_type {
                    InsightType::Solution => "✅",
                    InsightType::Approach => "🔄",
                    InsightType::Explanation => "📚",
                    InsightType::Example => "💡",
                    InsightType::Warning => "⚠️",
                    InsightType::Alternative => "🔀",
                };
                
                output.push_str(&format!("   {} {} {}\n", 
                    insight_icon, 
                    insight.branch_name.bright_blue(), 
                    insight.description
                ));
            }
            output.push('\n');
        }

        // Recommendations
        if !summary.recommendations.is_empty() {
            output.push_str(&format!("{}\n", "🎯 Recommendations:".bright_yellow().bold()));
            for rec in &summary.recommendations {
                output.push_str(&format!("   • {}\n", rec));
            }
        }

        output
    }

    /// Compare messages across branches
    fn compare_messages(branch_messages: &[Vec<Message>]) -> Result<Vec<MessageComparison>> {
        let max_messages = branch_messages.iter().map(|msgs| msgs.len()).max().unwrap_or(0);
        let mut comparisons = Vec::new();

        for seq_num in 0..max_messages {
            let mut messages = Vec::new();
            for branch_msgs in branch_messages {
                if seq_num < branch_msgs.len() {
                    messages.push(Some(branch_msgs[seq_num].clone()));
                } else {
                    messages.push(None);
                }
            }

            let similarity_score = Self::calculate_message_similarity(&messages);
            let diff_highlights = Self::generate_diff_highlights(&messages);

            comparisons.push(MessageComparison {
                sequence_number: seq_num,
                messages,
                similarity_score,
                diff_highlights,
            });
        }

        Ok(comparisons)
    }

    /// Calculate similarity between messages
    fn calculate_message_similarity(messages: &[Option<Message>]) -> f64 {
        let valid_messages: Vec<&Message> = messages.iter().filter_map(|m| m.as_ref()).collect();
        
        if valid_messages.len() < 2 {
            return 1.0; // Single message or no messages
        }

        // Simple similarity based on common words
        let mut total_similarity = 0.0;
        let mut comparisons = 0;

        for i in 0..valid_messages.len() {
            for j in (i + 1)..valid_messages.len() {
                let sim = Self::text_similarity(&valid_messages[i].content, &valid_messages[j].content);
                total_similarity += sim;
                comparisons += 1;
            }
        }

        if comparisons > 0 {
            total_similarity / comparisons as f64
        } else {
            1.0
        }
    }

    /// Simple text similarity calculation
    fn text_similarity(text1: &str, text2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Generate diff highlights (placeholder for now)
    fn generate_diff_highlights(_messages: &[Option<Message>]) -> Vec<DiffHighlight> {
        // TODO: Implement sophisticated diff highlighting
        Vec::new()
    }

    /// Generate comparison summary and insights
    fn generate_comparison_summary(
        branches: &[BranchEntity],
        message_comparisons: &[MessageComparison],
    ) -> Result<ComparisonSummary> {
        let total_messages = message_comparisons.len();
        
        let avg_similarity = if total_messages > 0 {
            message_comparisons.iter()
                .map(|mc| mc.similarity_score)
                .sum::<f64>() / total_messages as f64 * 100.0
        } else {
            100.0
        };

        // Generate quality scores
        let quality_scores = Self::calculate_quality_scores(branches, message_comparisons);
        
        // Generate insights
        let unique_insights = Self::extract_unique_insights(branches, message_comparisons);
        
        // Generate recommendations
        let recommendations = Self::generate_recommendations(branches, &quality_scores, avg_similarity);

        Ok(ComparisonSummary {
            total_messages_compared: total_messages,
            similarity_percentage: avg_similarity,
            unique_insights,
            recommendations,
            quality_scores,
        })
    }

    /// Calculate quality scores for branches
    fn calculate_quality_scores(
        branches: &[BranchEntity],
        message_comparisons: &[MessageComparison],
    ) -> Vec<BranchQualityScore> {
        let mut scores = Vec::new();

        for (i, branch) in branches.iter().enumerate() {
            let branch_name = branch.branch_name.as_deref().unwrap_or("unnamed").to_string();
            
            // Simple scoring based on message presence and length
            let mut total_score = 0.0;
            let mut message_count = 0;
            
            for comparison in message_comparisons {
                if i < comparison.messages.len() {
                    if let Some(msg) = &comparison.messages[i] {
                        // Score based on content length and role
                        let length_score = (msg.content.len() as f64 / 10.0).min(100.0);
                        let role_bonus = match msg.role {
                            crate::llm::common::model::role::Role::Assistant => 10.0,
                            crate::llm::common::model::role::Role::User => 5.0,
                            crate::llm::common::model::role::Role::System => 2.0,
                        };
                        total_score += length_score + role_bonus;
                        message_count += 1;
                    }
                }
            }

            let overall_score = if message_count > 0 {
                (total_score / message_count as f64).min(100.0)
            } else {
                0.0
            };

            let mut criteria_scores = HashMap::new();
            criteria_scores.insert("completeness".to_string(), overall_score * 0.8);
            criteria_scores.insert("clarity".to_string(), overall_score * 0.9);
            criteria_scores.insert("depth".to_string(), overall_score * 0.7);

            let strengths = if overall_score >= 70.0 {
                vec!["Comprehensive responses".to_string(), "Good detail level".to_string()]
            } else {
                vec![]
            };

            let weaknesses = if overall_score < 50.0 {
                vec!["Limited responses".to_string(), "Could be more detailed".to_string()]
            } else {
                vec![]
            };

            scores.push(BranchQualityScore {
                branch_index: i,
                branch_name,
                overall_score,
                criteria_scores,
                strengths,
                weaknesses,
            });
        }

        scores
    }

    /// Extract unique insights from branches
    fn extract_unique_insights(
        branches: &[BranchEntity],
        _message_comparisons: &[MessageComparison],
    ) -> Vec<UniqueInsight> {
        let mut insights = Vec::new();

        for (i, branch) in branches.iter().enumerate() {
            let branch_name = branch.branch_name.as_deref().unwrap_or("unnamed").to_string();
            
            // Generate sample insights based on branch characteristics
            if let Some(description) = &branch.description {
                if description.to_lowercase().contains("error") || description.to_lowercase().contains("debug") {
                    insights.push(UniqueInsight {
                        branch_index: i,
                        branch_name: branch_name.clone(),
                        insight_type: InsightType::Solution,
                        description: "Focuses on error handling and debugging approaches".to_string(),
                        relevance_score: 0.8,
                    });
                }
                
                if description.to_lowercase().contains("alternative") || description.to_lowercase().contains("different") {
                    insights.push(UniqueInsight {
                        branch_index: i,
                        branch_name,
                        insight_type: InsightType::Alternative,
                        description: "Explores alternative implementation approaches".to_string(),
                        relevance_score: 0.75,
                    });
                }
            }
        }

        insights
    }

    /// Generate recommendations based on comparison
    fn generate_recommendations(
        branches: &[BranchEntity],
        quality_scores: &[BranchQualityScore],
        avg_similarity: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Find best performing branch
        if let Some(best_branch) = quality_scores.iter().max_by(|a, b| a.overall_score.partial_cmp(&b.overall_score).unwrap()) {
            if best_branch.overall_score > 70.0 {
                recommendations.push(format!(
                    "Consider using '{}' as the primary approach (score: {:.1})",
                    best_branch.branch_name, best_branch.overall_score
                ));
            }
        }

        // Similarity-based recommendations
        if avg_similarity < 30.0 {
            recommendations.push("Branches show significantly different approaches - consider merging unique insights".to_string());
        } else if avg_similarity > 80.0 {
            recommendations.push("Branches are very similar - consider consolidating to avoid redundancy".to_string());
        }

        // Branch count recommendations
        if branches.len() > 4 {
            recommendations.push("Multiple branches detected - consider organizing or archiving completed explorations".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("All branches provide valuable perspectives - continue exploring as needed".to_string());
        }

        recommendations
    }

    /// Wrap text to specified width
    fn wrap_text(text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > width {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                
                // Handle very long words
                if word.len() > width {
                    lines.push(word.to_string());
                } else {
                    current_line = word.to_string();
                }
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        if lines.is_empty() {
            lines.push(String::new());
        }
        
        lines
    }
}

/// Quick comparison utilities
pub struct QuickCompare;

impl QuickCompare {
    /// Compare just the outcomes/conclusions of branches
    pub fn compare_outcomes(
        repo: &SqliteRepository,
        branch_ids: &[String],
    ) -> Result<String> {
        let comparison = BranchComparator::compare_branches(repo, branch_ids)?;
        
        let mut output = String::new();
        output.push_str(&format!("{}\n", "🎯 Branch Outcomes Comparison".bright_green().bold()));
        output.push_str(&format!("{}\n", "═".repeat(30).dimmed()));
        output.push('\n');

        for (i, branch) in comparison.branches.iter().enumerate() {
            let branch_name = branch.branch_name.as_deref().unwrap_or("unnamed");
            let quality_score = comparison.summary.quality_scores.get(i);
            
            output.push_str(&format!("🌿 {}\n", branch_name.bright_blue().bold()));
            
            if let Some(score) = quality_score {
                // Show strengths and weaknesses
                if !score.strengths.is_empty() {
                    output.push_str(&format!("   {} {}\n", "✅".bright_green(), "Strengths:"));
                    for strength in &score.strengths {
                        output.push_str(&format!("     • {}\n", strength.bright_green()));
                    }
                }
                
                if !score.weaknesses.is_empty() {
                    output.push_str(&format!("   {} {}\n", "⚠️".bright_yellow(), "Areas for improvement:"));
                    for weakness in &score.weaknesses {
                        output.push_str(&format!("     • {}\n", weakness.dimmed()));
                    }
                }
                
                output.push_str(&format!("   {} {:.1}/100\n", "Score:".bright_cyan(), score.overall_score));
            }
            
            if let Some(description) = &branch.description {
                output.push_str(&format!("   {} {}\n", "Description:".dimmed(), description.dimmed()));
            }
            
            output.push('\n');
        }

        // Add recommendations
        if !comparison.summary.recommendations.is_empty() {
            output.push_str(&format!("{}\n", "💡 Recommendations:".bright_yellow().bold()));
            for rec in &comparison.summary.recommendations {
                output.push_str(&format!("   • {}\n", rec));
            }
        }

        Ok(output)
    }
}