//! Agent-to-Agent Communication Protocol
//!
//! The manifest is the standard message format passed between agents.
//! It accumulates context as it moves through the consensus process.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Processing flags for the manifest
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestFlags {
    /// Emergency stop requested
    pub emergency_stop: bool,

    /// Privacy redaction enabled
    pub privacy_enabled: bool,

    /// Additional flags
    #[serde(default)]
    pub additional: HashMap<String, bool>,
}

/// A single turn in the conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// The user's query
    pub query: String,

    /// The agent's response
    pub response: String,

    /// Timestamp of this turn
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Agent-to-Agent Manifest
///
/// This is the core data structure that flows through the consensus system.
/// Each agent reads from and writes to the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AManifest {
    /// Unique identifier for this conversation turn
    pub id: String,

    /// Session ID for multi-turn conversations
    pub session_id: String,

    /// The original user query
    pub query: String,

    /// Query after privacy redaction
    pub redacted_query: Option<String>,

    /// Conversation history (previous turns)
    #[serde(default)]
    pub history: Vec<ConversationTurn>,

    /// Intent framing from first agent
    pub pathos_framing: Option<String>,

    /// First agent confidence score
    pub pathos_confidence: Option<f32>,

    /// Response from second agent
    pub logos_response: Option<String>,

    /// Second agent confidence score
    pub logos_confidence: Option<f32>,

    /// Verification notes from third agent
    pub ethos_verification: Option<String>,

    /// Third agent confidence score
    pub ethos_confidence: Option<f32>,

    /// Current consensus round
    pub round: u8,

    /// Feedback from previous rounds
    #[serde(default)]
    pub feedback: Vec<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Timestamps
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Processing flags
    #[serde(default)]
    pub flags: ManifestFlags,
}

impl A2AManifest {
    /// Create a new manifest for a query
    pub fn new(query: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            session_id: Uuid::new_v4().to_string(),
            query,
            redacted_query: None,
            history: vec![],
            pathos_framing: None,
            pathos_confidence: None,
            logos_response: None,
            logos_confidence: None,
            ethos_verification: None,
            ethos_confidence: None,
            round: 0,
            feedback: vec![],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            flags: ManifestFlags::default(),
        }
    }

    /// Create a manifest with session context
    pub fn with_session(query: String, session_id: String, history: Vec<ConversationTurn>) -> Self {
        let mut manifest = Self::new(query);
        manifest.session_id = session_id;
        manifest.history = history;
        manifest
    }

    /// Add feedback to the manifest
    pub fn add_feedback(&mut self, feedback: String) {
        self.feedback.push(feedback);
        self.updated_at = chrono::Utc::now();
    }

    /// Advance to the next round
    pub fn next_round(&mut self) {
        self.round += 1;
        self.updated_at = chrono::Utc::now();
    }

    /// Update the redacted query
    pub fn set_redacted(&mut self, redacted: String) {
        self.redacted_query = Some(redacted);
        self.updated_at = chrono::Utc::now();
    }

    /// Check if privacy is enabled
    pub fn is_privacy_enabled(&self) -> bool {
        self.flags.privacy_enabled
    }

    /// Enable privacy mode
    pub fn enable_privacy(&mut self) {
        self.flags.privacy_enabled = true;
        self.updated_at = chrono::Utc::now();
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.updated_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = A2AManifest::new("Test query".to_string());
        assert_eq!(manifest.query, "Test query");
        assert_eq!(manifest.round, 0);
        assert!(manifest.feedback.is_empty());
    }

    #[test]
    fn test_manifest_with_session() {
        let history = vec![ConversationTurn {
            query: "Previous query".to_string(),
            response: "Previous response".to_string(),
            timestamp: chrono::Utc::now(),
        }];
        let manifest =
            A2AManifest::with_session("New query".to_string(), "session-123".to_string(), history);
        assert_eq!(manifest.session_id, "session-123");
        assert_eq!(manifest.history.len(), 1);
    }

    #[test]
    fn test_add_feedback() {
        let mut manifest = A2AManifest::new("Test".to_string());
        manifest.add_feedback("Improve clarity".to_string());
        assert_eq!(manifest.feedback.len(), 1);
        assert_eq!(manifest.feedback[0], "Improve clarity");
    }

    #[test]
    fn test_next_round() {
        let mut manifest = A2AManifest::new("Test".to_string());
        assert_eq!(manifest.round, 0);
        manifest.next_round();
        assert_eq!(manifest.round, 1);
    }

    #[test]
    fn test_privacy_flags() {
        let mut manifest = A2AManifest::new("Test".to_string());
        assert!(!manifest.is_privacy_enabled());
        manifest.enable_privacy();
        assert!(manifest.is_privacy_enabled());
    }

    #[test]
    fn test_metadata() {
        let mut manifest = A2AManifest::new("Test".to_string());
        manifest.add_metadata("key".to_string(), serde_json::json!("value"));
        assert!(manifest.metadata.contains_key("key"));
    }
}
