use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionState {
    /// The summary text replacing older messages
    pub summary: String,
    /// Messages at indices [0..=compacted_through_ix] are replaced by summary
    pub compacted_through_ix: usize,
    /// Metadata about how this compaction was produced
    pub metadata: CompactionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionMetadata {
    pub mode: SummaryMode,
    pub source: CompactionSource,
    pub num_messages_summarized: usize,
    pub token_usage_before: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SummaryMode {
    Simple,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompactionSource {
    Background,
    Foreground,
    Manual,
}
