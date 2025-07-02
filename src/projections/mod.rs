//! Document projections

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::value_objects::{DocumentId, DocumentVersion, DocumentType};
use std::collections::HashMap;

/// Document view for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentView {
    /// Document's unique identifier
    pub document_id: Uuid,
    /// Title of the document
    pub title: String,
    /// MIME type of the document
    pub mime_type: String,
    /// Current status of the document
    pub status: String,
    /// Name of the document owner
    pub owner_name: Option<String>,
    /// Size of the document in bytes
    pub size_bytes: u64,
    /// Creation timestamp
    pub created_at: String,
    /// Tags associated with the document
    pub tags: Vec<String>,
}

/// Extended document view with full content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFullView {
    /// Document ID
    pub id: DocumentId,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Version
    pub version: DocumentVersion,
    /// Document type
    pub doc_type: DocumentType,
    /// Tags
    pub tags: Vec<String>,
    /// Author
    pub author: Uuid,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Document history view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHistoryView {
    /// Document ID
    pub document_id: DocumentId,
    /// List of versions
    pub versions: Vec<VersionEntry>,
}

/// Version entry in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Version number
    pub version: DocumentVersion,
    /// Change summary
    pub summary: String,
    /// Author of change
    pub author: Uuid,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Search result view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSearchView {
    /// Document ID
    pub document_id: DocumentId,
    /// Title
    pub title: String,
    /// Content snippet
    pub snippet: String,
    /// Relevance score
    pub score: f32,
    /// Highlight positions
    pub highlights: Vec<(usize, usize)>,
}
