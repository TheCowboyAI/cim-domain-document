//! Document domain events

use crate::value_objects::*;
use serde::{Deserialize, Serialize};
use cid::Cid;
use std::time::SystemTime;
use std::collections::HashSet;
use chrono;

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
#[derive(Debug, Clone)]
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
}
