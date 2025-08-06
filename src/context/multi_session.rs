use crate::context::analyzer::FileScore;
use crate::context::chunker::{ChunkingStrategy, ContextChunk, ContextChunker};
use crate::session::model::session::Session;
use anyhow::Result;
use std::collections::HashMap;

/// Manages multiple focused conversation sessions for large projects
pub struct MultiSessionManager {
    pub base_session_name: String,
    pub chunks: Vec<ContextChunk>,
    pub sessions: HashMap<String, Session>,
    pub current_chunk_index: usize,
    pub global_context: String, // Accumulated insights across chunks
}

impl MultiSessionManager {
    pub fn new(base_session_name: String) -> Self {
        Self {
            base_session_name,
            chunks: Vec::new(),
            sessions: HashMap::new(),
            current_chunk_index: 0,
            global_context: String::new(),
        }
    }

    /// Initialize chunked analysis for a large project
    pub async fn initialize_chunked_analysis(
        &mut self,
        files: &[FileScore],
        max_tokens_per_chunk: usize,
        strategy: ChunkingStrategy,
    ) -> Result<()> {
        let chunker = ContextChunker::new(max_tokens_per_chunk, strategy);
        self.chunks = chunker.create_chunks(files).await?;

        println!("🔄 Project Analysis Plan");
        println!("═══════════════════════");
        println!("📦 Created {} context chunks:", self.chunks.len());

        for (i, chunk) in self.chunks.iter().enumerate() {
            let chunk_icon = match chunk.chunk_type {
                crate::context::chunker::ChunkType::Overview => "📋",
                crate::context::chunker::ChunkType::Core => "🎯",
                crate::context::chunker::ChunkType::Utils => "🔧",
                crate::context::chunker::ChunkType::Tests => "🧪",
                crate::context::chunker::ChunkType::Config => "⚙️",
                crate::context::chunker::ChunkType::Docs => "📚",
            };

            println!(
                "  {}: {} {} - {} files (~{} tokens)",
                i + 1,
                chunk_icon,
                chunk.name,
                chunk.files.len(),
                chunk.estimated_tokens
            );
        }

        println!("\n💡 Recommended approach:");
        println!("   1. Start with Overview chunk for project understanding");
        println!("   2. Analyze Core chunks for main functionality");
        println!("   3. Review supporting chunks as needed");
        println!("   4. Use /switch <chunk_number> to change focus");
        println!();

        Ok(())
    }

    /// Get the current chunk being analyzed
    pub fn get_current_chunk(&self) -> Option<&ContextChunk> {
        self.chunks.get(self.current_chunk_index)
    }

    /// Switch to a different chunk
    pub fn switch_to_chunk(&mut self, chunk_index: usize) -> Result<&ContextChunk> {
        if chunk_index >= self.chunks.len() {
            return Err(anyhow::anyhow!(
                "Chunk index {} out of bounds (max: {})",
                chunk_index,
                self.chunks.len() - 1
            ));
        }

        self.current_chunk_index = chunk_index;
        Ok(&self.chunks[chunk_index])
    }

    /// Get a session for the current chunk (creates if doesn't exist)
    pub fn get_or_create_session(&mut self, chunk: &ContextChunk) -> &mut Session {
        let session_name = format!("{}_{}", self.base_session_name, chunk.id);

        if !self.sessions.contains_key(&session_name) {
            let mut session = Session::new_temporary();
            session.name = session_name.clone();

            // Add chunk context as initial system message if this is a new session
            if !self.global_context.is_empty() {
                let context_prompt = format!(
                    "You are analyzing a specific part of a larger project. Here's what we've learned so far:\n\n{}\n\nNow focusing on: {}\n{}\n\nFiles in this chunk: {}",
                    self.global_context,
                    chunk.name,
                    chunk.description,
                    chunk.files.iter()
                        .map(|f| format!("- {}", f.path))
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                session.add_raw_message(
                    context_prompt,
                    crate::llm::common::model::role::Role::System,
                );
            }

            self.sessions.insert(session_name.clone(), session);
        }

        self.sessions.get_mut(&session_name).unwrap()
    }

    /// Add insights to global context from current analysis
    pub fn add_global_insight(&mut self, insight: String) {
        if !self.global_context.is_empty() {
            self.global_context.push_str("\n\n");
        }
        self.global_context.push_str(&format!(
            "From {}: {}",
            self.get_current_chunk()
                .map_or("unknown".to_string(), |c| c.name.clone()),
            insight
        ));
    }

    /// Generate a comprehensive project summary
    pub fn generate_project_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str("🏗️ **Project Analysis Summary**\n");
        summary.push_str("═══════════════════════════════\n\n");

        // Overview statistics
        let total_files: usize = self.chunks.iter().map(|c| c.files.len()).sum();
        let total_tokens: usize = self.chunks.iter().map(|c| c.estimated_tokens).sum();

        summary.push_str(&format!("📊 **Statistics**\n"));
        summary.push_str(&format!(
            "   • Total chunks analyzed: {}\n",
            self.chunks.len()
        ));
        summary.push_str(&format!("   • Total files: {}\n", total_files));
        summary.push_str(&format!("   • Estimated tokens: ~{}\n\n", total_tokens));

        // Chunk breakdown
        summary.push_str("📁 **Chunks Overview**\n");
        for (i, chunk) in self.chunks.iter().enumerate() {
            let status = if self
                .sessions
                .contains_key(&format!("{}_{}", self.base_session_name, chunk.id))
            {
                "✅ Analyzed"
            } else {
                "⏳ Pending"
            };

            summary.push_str(&format!(
                "   {}. {} - {} ({})\n",
                i + 1,
                chunk.name,
                chunk.description,
                status
            ));
        }

        // Global insights
        if !self.global_context.is_empty() {
            summary.push_str("\n🧠 **Key Insights**\n");
            summary.push_str(&self.global_context);
        }

        summary.push_str("\n\n💡 Use /switch <number> to analyze different chunks or /summary for this overview.");
        summary
    }

    /// Get navigation help
    pub fn get_navigation_help(&self) -> String {
        let mut help = String::new();
        help.push_str("🧭 **Multi-Chunk Navigation**\n");
        help.push_str("══════════════════════════\n\n");

        help.push_str("**Available Commands:**\n");
        help.push_str("   /switch <number>  - Switch to chunk <number>\n");
        help.push_str("   /list            - Show all chunks\n");
        help.push_str("   /summary         - Project analysis summary\n");
        help.push_str("   /current         - Show current chunk info\n");
        help.push_str("   /insight <text>  - Add insight to global context\n\n");

        help.push_str("**Current Status:**\n");
        if let Some(chunk) = self.get_current_chunk() {
            help.push_str(&format!(
                "   📍 Currently analyzing: {} (chunk {})\n",
                chunk.name,
                self.current_chunk_index + 1
            ));
            help.push_str(&format!(
                "   📄 Files: {}, Tokens: ~{}\n",
                chunk.files.len(),
                chunk.estimated_tokens
            ));
        }

        help.push_str(&format!(
            "   🎯 Progress: {}/{} chunks have been analyzed\n",
            self.sessions.len(),
            self.chunks.len()
        ));

        help
    }

    /// Get the list of all chunks
    pub fn list_chunks(&self) -> String {
        let mut list = String::new();
        list.push_str("📋 **All Project Chunks**\n");
        list.push_str("═══════════════════════════\n\n");

        for (i, chunk) in self.chunks.iter().enumerate() {
            let current_marker = if i == self.current_chunk_index {
                " 👈 CURRENT"
            } else {
                ""
            };
            let analyzed_marker = if self
                .sessions
                .contains_key(&format!("{}_{}", self.base_session_name, chunk.id))
            {
                " ✅"
            } else {
                " ⏳"
            };

            let chunk_icon = match chunk.chunk_type {
                crate::context::chunker::ChunkType::Overview => "📋",
                crate::context::chunker::ChunkType::Core => "🎯",
                crate::context::chunker::ChunkType::Utils => "🔧",
                crate::context::chunker::ChunkType::Tests => "🧪",
                crate::context::chunker::ChunkType::Config => "⚙️",
                crate::context::chunker::ChunkType::Docs => "📚",
            };

            list.push_str(&format!(
                "{}. {} {} - {}{}{}\n",
                i + 1,
                chunk_icon,
                chunk.name,
                chunk.description,
                analyzed_marker,
                current_marker
            ));
            list.push_str(&format!(
                "   📄 {} files, ~{} tokens (priority: {:.1})\n\n",
                chunk.files.len(),
                chunk.estimated_tokens,
                chunk.priority
            ));
        }

        list
    }
}
