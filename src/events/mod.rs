//! Document domain events

use crate::value_objects::*;
use serde::{Deserialize, Serialize};
use cid::Cid;
use std::time::SystemTime;
use std::collections::HashSet;
use chrono;
use uuid::Uuid;

/// Document was created
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCreated {
    pub document_id: DocumentId,
    pub document_type: DocumentType,
    pub title: String,
    pub author_id: Uuid,
    pub metadata: std::collections::HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Document content was updated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentUpdated {
    pub document_id: DocumentId,
    pub content_blocks: Vec<ContentBlock>,
    pub change_summary: String,
    pub updated_by: Uuid,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Document state changed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateChanged {
    pub document_id: DocumentId,
    pub old_state: DocumentState,
    pub new_state: DocumentState,
    pub reason: String,
    pub changed_by: Uuid,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

/// Document was uploaded
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentUploaded {
    pub document_id: DocumentId,
    pub path: std::path::PathBuf,
    pub content_cid: Cid,
    pub metadata: DocumentMetadata,
    pub document_type: DocumentType,
    pub uploaded_by: String,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
}

/// Document metadata was updated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentMetadataUpdated {
    pub document_id: DocumentId,
    pub metadata: DocumentMetadata,
    pub updated_by: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Document was shared
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentShared {
    pub document_id: DocumentId,
    pub shared_with: HashSet<String>,
    pub permissions: Vec<String>,
    pub shared_by: String,
    pub shared_at: chrono::DateTime<chrono::Utc>,
}

/// Document was deleted
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentDeleted {
    pub document_id: DocumentId,
    pub timestamp: SystemTime,
}

/// Document was archived
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentArchived {
    pub document_id: DocumentId,
    pub reason: String,
    pub archived_by: String,
    pub archived_at: chrono::DateTime<chrono::Utc>,
}

// Implement as_any for event handlers
impl DocumentUploaded {
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DocumentMetadataUpdated {
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DocumentShared {
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DocumentDeleted {
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DocumentArchived {
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Domain event enum for all document events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentDomainEvent {
    /// Document was uploaded
    DocumentUploaded(DocumentUploaded),
    /// Document metadata was updated
    DocumentMetadataUpdated(DocumentMetadataUpdated),
    /// Document was shared
    DocumentShared(DocumentShared),
    /// Document was deleted
    DocumentDeleted(DocumentDeleted),
    /// Document was archived
    DocumentArchived(DocumentArchived),
    /// Document was created
    DocumentCreated(DocumentCreated),
    /// Document content was updated
    ContentUpdated(ContentUpdated),
    /// Document state changed
    StateChanged(StateChanged),
}
