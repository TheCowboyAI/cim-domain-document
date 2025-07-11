//! Document domain events

use crate::value_objects::*;
use serde::{Deserialize, Serialize};
use cid::Cid;
use std::collections::HashSet;
use chrono;
use uuid::Uuid;
use std::collections::HashMap;

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
    pub hard_delete: bool,
    pub reason: Option<String>,
    pub deleted_by: Uuid,
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}

/// Document was archived
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentArchived {
    pub document_id: DocumentId,
    pub reason: String,
    pub archived_by: Uuid,
    pub archived_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// Document was forked
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentForked {
    pub original_id: DocumentId,
    pub fork_id: DocumentId,
    pub fork_point_version: DocumentVersion,
    pub description: String,
    pub forked_by: Uuid,
    pub forked_at: chrono::DateTime<chrono::Utc>,
}

/// Version was tagged
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionTagged {
    pub document_id: DocumentId,
    pub tag: VersionTag,
}

/// Comment was added
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommentAdded {
    pub document_id: DocumentId,
    pub comment: Comment,
}

/// Documents were linked
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentsLinked {
    pub source_id: DocumentId,
    pub target_id: DocumentId,
    pub link_type: LinkType,
    pub description: Option<String>,
    pub linked_by: Uuid,
    pub linked_at: chrono::DateTime<chrono::Utc>,
}

/// Documents were merged
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentsMerged {
    pub target_id: DocumentId,
    pub source_id: DocumentId,
    pub merge_strategy: MergeStrategy,
    pub conflicts: Vec<MergeConflict>,
    pub merged_by: Uuid,
    pub merged_at: chrono::DateTime<chrono::Utc>,
}

/// Version was rolled back
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionRolledBack {
    pub document_id: DocumentId,
    pub from_version: DocumentVersion,
    pub to_version: DocumentVersion,
    pub reason: String,
    pub rolled_back_by: Uuid,
    pub rolled_back_at: chrono::DateTime<chrono::Utc>,
}

/// Entities were extracted
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntitiesExtracted {
    pub document_id: DocumentId,
    pub entities: Vec<ExtractedEntity>,
    pub extraction_options: ExtractionOptions,
    pub extracted_by: Uuid,
    pub extracted_at: chrono::DateTime<chrono::Utc>,
}

/// Summary was generated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryGenerated {
    pub document_id: DocumentId,
    pub summary: DocumentSummary,
    pub requested_by: Uuid,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Document was classified
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentClassified {
    pub document_id: DocumentId,
    pub document_type: DocumentType,
    pub category: String,
    pub subcategories: Vec<String>,
    pub classified_by: String,
    pub classified_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Classification {
    pub category: String,
    pub confidence: f32,
    pub labels: Vec<String>,
}

/// Document content was updated (with CID tracking)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentContentUpdated {
    pub document_id: DocumentId,
    pub new_content_cid: Cid,
    pub previous_content_cid: Cid,
    pub updated_by: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub update_reason: Option<String>,
}

/// Document was tagged
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTagged {
    pub document_id: DocumentId,
    pub tags: Vec<String>,
    pub all_tags: Vec<String>,
    pub tagged_by: String,
    pub tagged_at: chrono::DateTime<chrono::Utc>,
}

/// Document version was created
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentVersionCreated {
    pub document_id: DocumentId,
    pub version_number: String,
    pub content_cid: Cid,
    pub previous_version: String,
    pub change_summary: String,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Document version was restored
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentVersionRestored {
    pub document_id: DocumentId,
    pub restored_version: String,
    pub new_version: String,
    pub previous_version: String,
    pub restored_by: String,
    pub restored_at: chrono::DateTime<chrono::Utc>,
    pub reason: String,
}

/// Template was applied
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateApplied {
    pub document_id: DocumentId,
    pub template_id: TemplateId,
    pub variables: HashMap<String, String>,
    pub applied_by: Uuid,
    pub applied_at: chrono::DateTime<chrono::Utc>,
}

/// Collection was created
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionCreated {
    pub collection_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Document added to collection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentAddedToCollection {
    pub document_id: DocumentId,
    pub collection_id: Uuid,
    pub added_by: Uuid,
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// Document was imported
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentImported {
    pub document_id: DocumentId,
    pub source_format: ImportFormat,
    pub original_filename: Option<String>,
    pub metadata_extracted: HashMap<String, String>,
    pub imported_by: Uuid,
    pub imported_at: chrono::DateTime<chrono::Utc>,
}

/// Document was exported
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentExported {
    pub document_id: DocumentId,
    pub target_format: ExportFormat,
    pub export_size: usize,
    pub included_history: bool,
    pub exported_by: Uuid,
    pub exported_at: chrono::DateTime<chrono::Utc>,
}

/// Document was restored
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentRestored {
    pub document_id: DocumentId,
    pub restored_from: RestorationSource,
    pub restored_by: Uuid,
    pub restored_at: chrono::DateTime<chrono::Utc>,
    pub reason: Option<String>,
}

/// Source of restoration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RestorationSource {
    /// Restored from archive
    Archive,
    /// Restored from soft delete
    SoftDelete,
    /// Restored from backup
    Backup { backup_id: String },
}

/// Versions were compared
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionsCompared {
    pub document_id: DocumentId,
    pub version_a: DocumentVersion,
    pub version_b: DocumentVersion,
    pub comparison_id: Uuid,
    pub compared_by: Uuid,
    pub compared_at: chrono::DateTime<chrono::Utc>,
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
    /// Document was forked
    DocumentForked(DocumentForked),
    /// Version was tagged
    VersionTagged(VersionTagged),
    /// Comment was added
    CommentAdded(CommentAdded),
    /// Documents were linked
    DocumentsLinked(DocumentsLinked),
    /// Documents were merged
    DocumentsMerged(DocumentsMerged),
    /// Version was rolled back
    VersionRolledBack(VersionRolledBack),
    /// Entities were extracted
    EntitiesExtracted(EntitiesExtracted),
    /// Summary was generated
    SummaryGenerated(SummaryGenerated),
    /// Document was classified
    DocumentClassified(DocumentClassified),
    /// Document content was updated (with CID tracking)
    DocumentContentUpdated(DocumentContentUpdated),
    /// Document was tagged
    DocumentTagged(DocumentTagged),
    /// Document version was created
    DocumentVersionCreated(DocumentVersionCreated),
    /// Document version was restored
    DocumentVersionRestored(DocumentVersionRestored),
    /// Template was applied
    TemplateApplied(TemplateApplied),
    /// Collection was created
    CollectionCreated(CollectionCreated),
    /// Document added to collection
    DocumentAddedToCollection(DocumentAddedToCollection),
    /// Document was imported
    DocumentImported(DocumentImported),
    /// Document was exported
    DocumentExported(DocumentExported),
    /// Document was restored
    DocumentRestored(DocumentRestored),
    /// Versions were compared
    VersionsCompared(VersionsCompared),
}
