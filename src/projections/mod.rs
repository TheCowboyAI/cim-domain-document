//! Document projections

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
