//! Document domain events

use crate::value_objects::*;
use serde::{Deserialize, Serialize};
use cid::Cid;
use std::collections::HashSet;
use chrono;
use uuid::Uuid;
use std::collections::HashMap;

// Re-export edit events
pub use edit_events::*;
pub use ingestion_events::*;

mod edit_events;
mod ingestion_events;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;
    use chrono::Utc;
    use cid::Cid;

    // Helper to create test CID
    fn create_test_cid() -> Cid {
        Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap()
    }

    // Helper to create test metadata
    fn create_test_metadata() -> DocumentMetadata {
        DocumentMetadata {
            title: "Test Document".to_string(),
            description: Some("Test description".to_string()),
            tags: vec!["test".to_string()],
            custom_attributes: HashMap::new(),
            mime_type: Some("text/plain".to_string()),
            size_bytes: Some(1024),
            language: Some("en".to_string()),
            category: Some("test".to_string()),
            subcategories: Some(vec!["unit".to_string()]),
            filename: Some("test.txt".to_string()),
        }
    }

    #[test]
    fn test_document_created_event() {
        // US-008: Test event creation and structure
        let doc_id = DocumentId::new();
        let author_id = Uuid::new_v4();
        let now = Utc::now();
        let mut metadata = HashMap::new();
        metadata.insert("department".to_string(), "Engineering".to_string());

        let event = DocumentCreated {
            document_id: doc_id.clone(),
            document_type: DocumentType::Report,
            title: "Test Report".to_string(),
            author_id,
            metadata: metadata.clone(),
            created_at: now,
        };

        // Verify event structure
        assert_eq!(event.document_id, doc_id);
        assert!(matches!(event.document_type, DocumentType::Report));
        assert_eq!(event.title, "Test Report");
        assert_eq!(event.author_id, author_id);
        assert_eq!(event.metadata, metadata);
        assert_eq!(event.created_at, now);
    }

    #[test]
    fn test_document_uploaded_event() {
        // US-008: Test document upload event
        let doc_id = DocumentId::new();
        let content_cid = create_test_cid();
        let metadata = create_test_metadata();
        let now = Utc::now();

        let event = DocumentUploaded {
            document_id: doc_id.clone(),
            path: std::path::PathBuf::from("/test/document.pdf"),
            content_cid,
            metadata: metadata.clone(),
            document_type: DocumentType::Pdf,
            uploaded_by: "user123".to_string(),
            uploaded_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.content_cid, content_cid);
        assert_eq!(event.metadata, metadata);
        assert!(matches!(event.document_type, DocumentType::Pdf));
        assert_eq!(event.uploaded_by, "user123");
        assert_eq!(event.uploaded_at, now);
    }

    #[test]
    fn test_content_updated_event() {
        // US-008: Test content update event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        
        let content_blocks = vec![
            ContentBlock {
                id: "block1".to_string(),
                block_type: "paragraph".to_string(),
                title: Some("Updated Section".to_string()),
                content: "Updated content".to_string(),
                metadata: HashMap::new(),
            }
        ];

        let event = ContentUpdated {
            document_id: doc_id.clone(),
            content_blocks: content_blocks.clone(),
            change_summary: "Updated introduction".to_string(),
            updated_by: user_id,
            updated_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.content_blocks.len(), 1);
        assert_eq!(event.content_blocks[0].id, "block1");
        assert_eq!(event.change_summary, "Updated introduction");
        assert_eq!(event.updated_by, user_id);
        assert_eq!(event.updated_at, now);
    }

    #[test]
    fn test_state_changed_event() {
        // US-008: Test state change event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let event = StateChanged {
            document_id: doc_id.clone(),
            old_state: DocumentState::Draft,
            new_state: DocumentState::InReview,
            reason: "Ready for review".to_string(),
            changed_by: user_id,
            changed_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert!(matches!(event.old_state, DocumentState::Draft));
        assert!(matches!(event.new_state, DocumentState::InReview));
        assert_eq!(event.reason, "Ready for review");
        assert_eq!(event.changed_by, user_id);
        assert_eq!(event.changed_at, now);
    }

    #[test]
    fn test_document_shared_event() {
        // US-008: Test document sharing event
        let doc_id = DocumentId::new();
        let now = Utc::now();
        
        let mut shared_with = HashSet::new();
        shared_with.insert("user1".to_string());
        shared_with.insert("user2".to_string());
        
        let permissions = vec!["read".to_string(), "comment".to_string()];

        let event = DocumentShared {
            document_id: doc_id.clone(),
            shared_with: shared_with.clone(),
            permissions: permissions.clone(),
            shared_by: "owner".to_string(),
            shared_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.shared_with, shared_with);
        assert_eq!(event.permissions, permissions);
        assert_eq!(event.shared_by, "owner");
        assert_eq!(event.shared_at, now);
    }

    #[test]
    fn test_document_archived_event() {
        // US-008: Test document archiving event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        
        let mut metadata = HashMap::new();
        metadata.insert("retention_period".to_string(), "7_years".to_string());

        let event = DocumentArchived {
            document_id: doc_id.clone(),
            reason: "Project completed".to_string(),
            archived_by: user_id,
            archived_at: now,
            metadata: metadata.clone(),
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.reason, "Project completed");
        assert_eq!(event.archived_by, user_id);
        assert_eq!(event.archived_at, now);
        assert_eq!(event.metadata, metadata);
    }

    #[test]
    fn test_document_deleted_event() {
        // US-008: Test document deletion event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let event = DocumentDeleted {
            document_id: doc_id.clone(),
            hard_delete: false,
            reason: Some("No longer needed".to_string()),
            deleted_by: user_id,
            deleted_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert!(!event.hard_delete);
        assert_eq!(event.reason, Some("No longer needed".to_string()));
        assert_eq!(event.deleted_by, user_id);
        assert_eq!(event.deleted_at, now);
    }

    #[test]
    fn test_document_forked_event() {
        // US-008: Test document forking event
        let original_id = DocumentId::new();
        let fork_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let version = DocumentVersion::new(1, 2, 3);
        let now = Utc::now();

        let event = DocumentForked {
            original_id: original_id.clone(),
            fork_id: fork_id.clone(),
            fork_point_version: version.clone(),
            description: "Fork for feature development".to_string(),
            forked_by: user_id,
            forked_at: now,
        };

        assert_eq!(event.original_id, original_id);
        assert_eq!(event.fork_id, fork_id);
        assert_eq!(event.fork_point_version, version);
        assert_eq!(event.description, "Fork for feature development");
        assert_eq!(event.forked_by, user_id);
        assert_eq!(event.forked_at, now);
    }

    #[test]
    fn test_version_tagged_event() {
        // US-008: Test version tagging event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let tag = VersionTag {
            name: "v1.0.0".to_string(),
            description: Some("First release".to_string()),
            version: DocumentVersion::new(1, 0, 0),
            tagged_by: user_id,
            tagged_at: now,
        };

        let event = VersionTagged {
            document_id: doc_id.clone(),
            tag: tag.clone(),
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.tag, tag);
    }

    #[test]
    fn test_comment_added_event() {
        // US-008: Test comment addition event
        let doc_id = DocumentId::new();
        let comment_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let now = Utc::now();

        let comment = Comment {
            id: comment_id,
            content: "This needs revision".to_string(),
            author_id,
            block_id: Some("block1".to_string()),
            parent_id: None,
            created_at: now,
            resolved: false,
        };

        let event = CommentAdded {
            document_id: doc_id.clone(),
            comment: comment.clone(),
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.comment, comment);
    }

    #[test]
    fn test_documents_linked_event() {
        // US-008: Test document linking event
        let source_id = DocumentId::new();
        let target_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let event = DocumentsLinked {
            source_id: source_id.clone(),
            target_id: target_id.clone(),
            link_type: LinkType::References,
            description: Some("Reference material".to_string()),
            linked_by: user_id,
            linked_at: now,
        };

        assert_eq!(event.source_id, source_id);
        assert_eq!(event.target_id, target_id);
        assert!(matches!(event.link_type, LinkType::References));
        assert_eq!(event.description, Some("Reference material".to_string()));
        assert_eq!(event.linked_by, user_id);
        assert_eq!(event.linked_at, now);
    }

    #[test]
    fn test_documents_merged_event() {
        // US-008: Test document merging event
        let target_id = DocumentId::new();
        let source_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let conflicts = vec![
            MergeConflict {
                id: Uuid::new_v4(),
                block_id: "block1".to_string(),
                target_content: "Target content".to_string(),
                source_content: "Source content".to_string(),
                base_content: Some("Base content".to_string()),
                conflict_type: ConflictType::ContentModified,
            }
        ];

        let event = DocumentsMerged {
            target_id: target_id.clone(),
            source_id: source_id.clone(),
            merge_strategy: MergeStrategy::ThreeWay,
            conflicts: conflicts.clone(),
            merged_by: user_id,
            merged_at: now,
        };

        assert_eq!(event.target_id, target_id);
        assert_eq!(event.source_id, source_id);
        assert!(matches!(event.merge_strategy, MergeStrategy::ThreeWay));
        assert_eq!(event.conflicts.len(), 1);
        assert_eq!(event.merged_by, user_id);
        assert_eq!(event.merged_at, now);
    }

    #[test]
    fn test_version_rolled_back_event() {
        // US-008: Test version rollback event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let from_version = DocumentVersion::new(2, 1, 0);
        let to_version = DocumentVersion::new(1, 5, 3);

        let event = VersionRolledBack {
            document_id: doc_id.clone(),
            from_version: from_version.clone(),
            to_version: to_version.clone(),
            reason: "Critical bug found".to_string(),
            rolled_back_by: user_id,
            rolled_back_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.from_version, from_version);
        assert_eq!(event.to_version, to_version);
        assert_eq!(event.reason, "Critical bug found");
        assert_eq!(event.rolled_back_by, user_id);
        assert_eq!(event.rolled_back_at, now);
    }

    #[test]
    fn test_entities_extracted_event() {
        // US-008: Test entity extraction event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let entities = vec![
            ExtractedEntity {
                text: "John Doe".to_string(),
                entity_type: EntityType::Person,
                confidence: 0.95,
                start_offset: 10,
                end_offset: 18,
                metadata: HashMap::new(),
            }
        ];

        let options = ExtractionOptions::default();

        let event = EntitiesExtracted {
            document_id: doc_id.clone(),
            entities: entities.clone(),
            extraction_options: options.clone(),
            extracted_by: user_id,
            extracted_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.entities.len(), 1);
        assert_eq!(event.entities[0].text, "John Doe");
        assert!(matches!(event.entities[0].entity_type, EntityType::Person));
        assert_eq!(event.extracted_by, user_id);
        assert_eq!(event.extracted_at, now);
    }

    #[test]
    fn test_summary_generated_event() {
        // US-008: Test summary generation event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let summary = DocumentSummary {
            text: "This is a test document summary.".to_string(),
            key_points: vec!["Key point 1".to_string(), "Key point 2".to_string()],
            length: SummaryLength::Standard,
            language: "en".to_string(),
            generated_at: now,
            quality_score: Some(0.85),
        };

        let event = SummaryGenerated {
            document_id: doc_id.clone(),
            summary: summary.clone(),
            requested_by: user_id,
            generated_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.summary, summary);
        assert_eq!(event.requested_by, user_id);
        assert_eq!(event.generated_at, now);
    }

    #[test]
    fn test_template_applied_event() {
        // US-008: Test template application event
        let doc_id = DocumentId::new();
        let template_id = TemplateId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let mut variables = HashMap::new();
        variables.insert("title".to_string(), "Monthly Report".to_string());
        variables.insert("date".to_string(), "2024-01-15".to_string());

        let event = TemplateApplied {
            document_id: doc_id.clone(),
            template_id,
            variables: variables.clone(),
            applied_by: user_id,
            applied_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.template_id, template_id);
        assert_eq!(event.variables, variables);
        assert_eq!(event.applied_by, user_id);
        assert_eq!(event.applied_at, now);
    }

    #[test]
    fn test_document_imported_event() {
        // US-008: Test document import event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let mut metadata = HashMap::new();
        metadata.insert("original_format".to_string(), "docx".to_string());

        let event = DocumentImported {
            document_id: doc_id.clone(),
            source_format: ImportFormat::Word,
            original_filename: Some("document.docx".to_string()),
            metadata_extracted: metadata.clone(),
            imported_by: user_id,
            imported_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert!(matches!(event.source_format, ImportFormat::Word));
        assert_eq!(event.original_filename, Some("document.docx".to_string()));
        assert_eq!(event.metadata_extracted, metadata);
        assert_eq!(event.imported_by, user_id);
        assert_eq!(event.imported_at, now);
    }

    #[test]
    fn test_document_exported_event() {
        // US-008: Test document export event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let event = DocumentExported {
            document_id: doc_id.clone(),
            target_format: ExportFormat::Pdf,
            export_size: 2048,
            included_history: true,
            exported_by: user_id,
            exported_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert!(matches!(event.target_format, ExportFormat::Pdf));
        assert_eq!(event.export_size, 2048);
        assert!(event.included_history);
        assert_eq!(event.exported_by, user_id);
        assert_eq!(event.exported_at, now);
    }

    #[test]
    fn test_document_restored_event() {
        // US-008: Test document restoration event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let event = DocumentRestored {
            document_id: doc_id.clone(),
            restored_from: RestorationSource::Archive,
            restored_by: user_id,
            restored_at: now,
            reason: Some("Needed for audit".to_string()),
        };

        assert_eq!(event.document_id, doc_id);
        assert!(matches!(event.restored_from, RestorationSource::Archive));
        assert_eq!(event.restored_by, user_id);
        assert_eq!(event.restored_at, now);
        assert_eq!(event.reason, Some("Needed for audit".to_string()));
    }

    #[test]
    fn test_restoration_source_variants() {
        // US-008: Test restoration source enum variants
        let archive = RestorationSource::Archive;
        let soft_delete = RestorationSource::SoftDelete;
        let backup = RestorationSource::Backup { 
            backup_id: "backup_123".to_string() 
        };

        assert!(matches!(archive, RestorationSource::Archive));
        assert!(matches!(soft_delete, RestorationSource::SoftDelete));
        if let RestorationSource::Backup { backup_id } = backup {
            assert_eq!(backup_id, "backup_123");
        } else {
            panic!("Expected Backup variant");
        }
    }

    #[test]
    fn test_versions_compared_event() {
        // US-008: Test version comparison event
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let comparison_id = Uuid::new_v4();
        let now = Utc::now();

        let version_a = DocumentVersion::new(1, 0, 0);
        let version_b = DocumentVersion::new(1, 1, 0);

        let event = VersionsCompared {
            document_id: doc_id.clone(),
            version_a: version_a.clone(),
            version_b: version_b.clone(),
            comparison_id,
            compared_by: user_id,
            compared_at: now,
        };

        assert_eq!(event.document_id, doc_id);
        assert_eq!(event.version_a, version_a);
        assert_eq!(event.version_b, version_b);
        assert_eq!(event.comparison_id, comparison_id);
        assert_eq!(event.compared_by, user_id);
        assert_eq!(event.compared_at, now);
    }

    #[test]
    fn test_event_serialization() {
        // US-009: Test event serialization and deserialization
        let doc_id = DocumentId::new();
        let now = Utc::now();

        let event = DocumentCreated {
            document_id: doc_id.clone(),
            document_type: DocumentType::Text,
            title: "Test Document".to_string(),
            author_id: Uuid::new_v4(),
            metadata: HashMap::new(),
            created_at: now,
        };

        // Test serialization
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("Test Document"));
        assert!(serialized.contains("Text"));

        // Test deserialization
        let deserialized: DocumentCreated = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.document_id, event.document_id);
        assert_eq!(deserialized.title, event.title);
        assert!(matches!(deserialized.document_type, DocumentType::Text));
    }

    #[test]
    fn test_event_serialization_with_complex_data() {
        // US-009: Test serialization with complex nested data
        let doc_id = DocumentId::new();
        let now = Utc::now();

        let entities = vec![
            ExtractedEntity {
                text: "Complex Entity".to_string(),
                entity_type: EntityType::Custom("CustomType".to_string()),
                confidence: 0.89,
                start_offset: 0,
                end_offset: 14,
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("category".to_string(), "test".to_string());
                    map
                },
            }
        ];

        let event = EntitiesExtracted {
            document_id: doc_id.clone(),
            entities,
            extraction_options: ExtractionOptions::default(),
            extracted_by: Uuid::new_v4(),
            extracted_at: now,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: EntitiesExtracted = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.entities.len(), 1);
        assert_eq!(deserialized.entities[0].text, "Complex Entity");
        assert_eq!(deserialized.entities[0].confidence, 0.89);
    }

    #[test]
    fn test_event_ordering_consistency() {
        // US-010: Test event ordering with timestamps
        let doc_id = DocumentId::new();
        let base_time = Utc::now();

        let events = vec![
            (base_time, "DocumentCreated"),
            (base_time + chrono::Duration::seconds(1), "DocumentUploaded"),
            (base_time + chrono::Duration::seconds(2), "ContentUpdated"),
            (base_time + chrono::Duration::seconds(3), "StateChanged"),
            (base_time + chrono::Duration::seconds(4), "DocumentArchived"),
        ];

        // Verify chronological ordering
        for i in 1..events.len() {
            assert!(events[i].0 > events[i-1].0, 
                "Event {} should come after {}", events[i].1, events[i-1].1);
        }
    }

    #[test]
    fn test_event_metadata_consistency() {
        // US-009: Test that events maintain metadata consistency
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        // Events should maintain consistent document ID
        let events: Vec<DocumentId> = vec![
            DocumentCreated {
                document_id: doc_id.clone(),
                document_type: DocumentType::Text,
                title: "Test".to_string(),
                author_id: user_id,
                metadata: HashMap::new(),
                created_at: now,
            }.document_id,
            
            ContentUpdated {
                document_id: doc_id.clone(),
                content_blocks: vec![],
                change_summary: "Update".to_string(),
                updated_by: user_id,
                updated_at: now,
            }.document_id,
            
            StateChanged {
                document_id: doc_id.clone(),
                old_state: DocumentState::Draft,
                new_state: DocumentState::InReview,
                reason: "Review".to_string(),
                changed_by: user_id,
                changed_at: now,
            }.document_id,
        ];

        // All events should have the same document ID
        for event_doc_id in events {
            assert_eq!(event_doc_id, doc_id);
        }
    }

    #[test]
    fn test_as_any_implementations() {
        // US-009: Test as_any trait implementations for event handlers
        let doc_id = DocumentId::new();
        let now = Utc::now();

        let upload_event = DocumentUploaded {
            document_id: doc_id.clone(),
            path: std::path::PathBuf::from("/test/file.txt"),
            content_cid: create_test_cid(),
            metadata: create_test_metadata(),
            document_type: DocumentType::Text,
            uploaded_by: "user".to_string(),
            uploaded_at: now,
        };

        // Should not panic - as_any is implemented
        let _any_ref = upload_event.as_any();

        let delete_event = DocumentDeleted {
            document_id: doc_id,
            hard_delete: false,
            reason: None,
            deleted_by: Uuid::new_v4(),
            deleted_at: now,
        };

        let _any_ref = delete_event.as_any();
    }

    #[test]
    fn test_event_trait_implementations() {
        // US-009: Test that events implement required traits
        let doc_id = DocumentId::new();
        let now = Utc::now();

        let event = DocumentCreated {
            document_id: doc_id,
            document_type: DocumentType::Note,
            title: "Test".to_string(),
            author_id: Uuid::new_v4(),
            metadata: HashMap::new(),
            created_at: now,
        };

        // Test Debug
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("DocumentCreated"));

        // Test Clone
        let cloned = event.clone();
        assert_eq!(cloned.title, event.title);

        // Test PartialEq
        assert_eq!(event, cloned);
    }

    #[test]
    fn test_edge_case_events() {
        // US-010: Test edge cases in event creation
        let doc_id = DocumentId::new();
        let now = Utc::now();

        // Event with empty collections
        let event = DocumentShared {
            document_id: doc_id.clone(),
            shared_with: HashSet::new(),
            permissions: vec![],
            shared_by: "".to_string(),
            shared_at: now,
        };

        assert!(event.shared_with.is_empty());
        assert!(event.permissions.is_empty());
        assert_eq!(event.shared_by, "");

        // Event with optional None values
        let delete_event = DocumentDeleted {
            document_id: doc_id,
            hard_delete: true,
            reason: None,
            deleted_by: Uuid::new_v4(),
            deleted_at: now,
        };

        assert!(delete_event.hard_delete);
        assert!(delete_event.reason.is_none());
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
    
    // Document editing events
    /// Document successor was created
    DocumentSuccessorCreated(DocumentSuccessorCreated),
    /// Document was edited with direct replacement
    DocumentEditedDirect(DocumentEditedDirect),
    /// Document was edited with differential patch
    DocumentEditedPatch(DocumentEditedPatch),
    /// Document was edited with structured changes
    DocumentEditedStructured(DocumentEditedStructured),
    /// Edit access was requested
    EditAccessRequested(EditAccessRequested),
    /// Edit access was granted
    EditAccessGranted(EditAccessGranted),
    /// Edit session was cancelled
    EditSessionCancelled(EditSessionCancelled),
    /// Document was automatically transformed
    DocumentTransformed(DocumentTransformed),
    /// Document edits were merged
    DocumentEditsMerged(DocumentEditsMerged),
    /// Document was rolled back
    DocumentRolledBack(DocumentRolledBack),
    /// CID chain was verified
    CidChainVerified(CidChainVerified),
    /// Document edit failed
    DocumentEditFailed(DocumentEditFailed),
}
