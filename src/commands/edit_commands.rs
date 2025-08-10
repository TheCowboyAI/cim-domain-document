//! Document Editing Commands
//!
//! This module defines commands for creating document successors through
//! direct replacement and differential patching patterns.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use cid::Cid;
use cim_domain::{Command as DomainCommand, EntityId};
use crate::value_objects::{
    DocumentId, CreateSuccessorRequest, SuccessorCreationType, 
    EditMetadata, EditorInfo, PatchFormat, ContentChange
};

/// Create a successor document through direct replacement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentSuccessor {
    /// Document to create successor for
    pub document_id: DocumentId,
    /// Request details for successor creation
    pub request: CreateSuccessorRequest,
}

impl DomainCommand for CreateDocumentSuccessor {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl crate::commands::Command for CreateDocumentSuccessor {}

/// Edit document with direct content replacement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditDocumentDirect {
    /// Document to edit
    pub document_id: DocumentId,
    /// Current CID to replace
    pub current_cid: Cid,
    /// New content to replace with
    pub new_content: Vec<u8>,
    /// Content type of new content
    pub content_type: String,
    /// Who is performing the edit
    pub edited_by: Uuid,
    /// Optional description of changes
    pub description: Option<String>,
    /// Editor used for the edit
    pub editor_info: Option<EditorInfo>,
}

impl DomainCommand for EditDocumentDirect {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl crate::commands::Command for EditDocumentDirect {}

impl EditDocumentDirect {
    pub fn new(
        document_id: DocumentId,
        current_cid: Cid,
        new_content: Vec<u8>,
        content_type: String,
        edited_by: Uuid,
    ) -> Self {
        Self {
            document_id,
            current_cid,
            new_content,
            content_type,
            edited_by,
            description: None,
            editor_info: None,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn with_editor_info(mut self, editor_info: EditorInfo) -> Self {
        self.editor_info = Some(editor_info);
        self
    }
}

/// Edit document using differential patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditDocumentPatch {
    /// Document to edit
    pub document_id: DocumentId,
    /// CID of content to patch
    pub base_cid: Cid,
    /// Patch data to apply
    pub patch_data: Vec<u8>,
    /// Format of the patch
    pub patch_format: PatchFormat,
    /// Who is performing the edit
    pub edited_by: Uuid,
    /// Description of changes
    pub description: Option<String>,
    /// Editor information
    pub editor_info: Option<EditorInfo>,
}

impl DomainCommand for EditDocumentPatch {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl crate::commands::Command for EditDocumentPatch {}

impl EditDocumentPatch {
    pub fn new(
        document_id: DocumentId,
        base_cid: Cid,
        patch_data: Vec<u8>,
        patch_format: PatchFormat,
        edited_by: Uuid,
    ) -> Self {
        Self {
            document_id,
            base_cid,
            patch_data,
            patch_format,
            edited_by,
            description: None,
            editor_info: None,
        }
    }
}

/// Edit document using structured changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditDocumentStructured {
    /// Document to edit
    pub document_id: DocumentId,
    /// CID of content to edit
    pub base_cid: Cid,
    /// List of structured changes to apply
    pub changes: Vec<ContentChange>,
    /// Summary of changes
    pub change_summary: String,
    /// Who is performing the edit
    pub edited_by: Uuid,
    /// Editor information
    pub editor_info: Option<EditorInfo>,
}

impl DomainCommand for EditDocumentStructured {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl crate::commands::Command for EditDocumentStructured {}

/// Request content for external editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEditAccess {
    /// Document to request edit access for
    pub document_id: DocumentId,
    /// CID of specific version to edit
    pub version_cid: Option<Cid>,
    /// User requesting access
    pub requested_by: Uuid,
    /// Intended editor/tool
    pub editor_info: Option<EditorInfo>,
    /// Expected edit duration
    pub expected_duration: Option<chrono::Duration>,
}

impl DomainCommand for RequestEditAccess {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl crate::commands::Command for RequestEditAccess {}

/// Cancel edit session and release locks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelEditSession {
    /// Document being edited
    pub document_id: DocumentId,
    /// Edit session to cancel
    pub session_id: Uuid,
    /// User cancelling the session
    pub cancelled_by: Uuid,
    /// Reason for cancellation
    pub reason: Option<String>,
}

impl DomainCommand for CancelEditSession {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl crate::commands::Command for CancelEditSession {}

/// Automated document transformation command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformDocument {
    /// Document to transform
    pub document_id: DocumentId,
    /// Source CID for transformation
    pub source_cid: Cid,
    /// Type of transformation
    pub transformation_type: TransformationType,
    /// Parameters for transformation
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    /// System/service performing transformation
    pub processor: String,
    /// Description of transformation
    pub description: Option<String>,
}

impl DomainCommand for TransformDocument {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl crate::commands::Command for TransformDocument {}

/// Types of automated transformations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformationType {
    /// Grammar and spelling correction
    GrammarCorrection,
    /// Language translation
    Translation { target_language: String },
    /// Format conversion
    FormatConversion { target_format: String },
    /// Content summarization
    Summarization { max_length: Option<u32> },
    /// Content enhancement/enrichment
    ContentEnhancement,
    /// OCR processing
    OpticalCharacterRecognition,
    /// Image processing
    ImageProcessing { operations: Vec<String> },
    /// Custom transformation
    Custom { transformation_name: String },
}

/// Merge multiple document edits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeDocumentEdits {
    /// Document with conflicting edits
    pub document_id: DocumentId,
    /// Base CID for merge
    pub base_cid: Cid,
    /// CIDs of versions to merge
    pub merge_cids: Vec<Cid>,
    /// Merge strategy to use
    pub merge_strategy: MergeStrategy,
    /// Who is performing the merge
    pub merged_by: Uuid,
    /// Merge description
    pub description: Option<String>,
}

impl DomainCommand for MergeDocumentEdits {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl crate::commands::Command for MergeDocumentEdits {}

/// Merge strategies for resolving conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Automatic merge where possible, fail on conflicts
    AutoMerge,
    /// Always prefer first version in conflicts
    PreferFirst,
    /// Always prefer last version in conflicts
    PreferLast,
    /// Manual resolution with conflict markers
    ManualResolve,
    /// Custom merge strategy
    Custom { strategy_name: String },
}

/// Rollback document to previous version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackDocument {
    /// Document to rollback
    pub document_id: DocumentId,
    /// Current CID to rollback from
    pub current_cid: Cid,
    /// Target CID to rollback to
    pub target_cid: Cid,
    /// Who is performing rollback
    pub rolled_back_by: Uuid,
    /// Reason for rollback
    pub reason: String,
    /// Whether to create new successor or restore directly
    pub create_successor: bool,
}

impl DomainCommand for RollbackDocument {
    type Aggregate = crate::Document;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl crate::commands::Command for RollbackDocument {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{ChangeType, ChangeLocation};
    
    fn create_test_cid(_data: &str) -> Cid {
        Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap()
    }

    #[test]
    fn test_edit_document_direct_creation() {
        let document_id = DocumentId::new();
        let current_cid = create_test_cid("original content");
        let new_content = "edited content".as_bytes().to_vec();
        let edited_by = Uuid::new_v4();
        
        let command = EditDocumentDirect::new(
            document_id.clone(),
            current_cid.clone(),
            new_content.clone(),
            "text/plain".to_string(),
            edited_by,
        )
        .with_description("Grammar corrections".to_string());
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.current_cid, current_cid);
        assert_eq!(command.new_content, new_content);
        assert_eq!(command.description, Some("Grammar corrections".to_string()));
    }
    
    #[test]
    fn test_edit_document_patch_creation() {
        let document_id = DocumentId::new();
        let base_cid = create_test_cid("base content");
        let patch_data = b"patch data".to_vec();
        let edited_by = Uuid::new_v4();
        
        let command = EditDocumentPatch::new(
            document_id.clone(),
            base_cid.clone(),
            patch_data.clone(),
            PatchFormat::UnifiedDiff,
            edited_by,
        );
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.base_cid, base_cid);
        assert_eq!(command.patch_data, patch_data);
        assert_eq!(command.patch_format, PatchFormat::UnifiedDiff);
    }
    
    #[test]
    fn test_edit_document_structured_creation() {
        let document_id = DocumentId::new();
        let base_cid = create_test_cid("base content");
        let edited_by = Uuid::new_v4();
        
        let changes = vec![
            ContentChange {
                change_type: ChangeType::Update,
                location: ChangeLocation {
                    line: Some(5),
                    column: Some(10),
                    byte_offset: Some(100),
                    structural_path: None,
                    block_id: None,
                },
                old_content: Some("old text".to_string()),
                new_content: Some("new text".to_string()),
                change_size: 8,
            }
        ];
        
        let command = EditDocumentStructured {
            document_id: document_id.clone(),
            base_cid: base_cid.clone(),
            changes: changes.clone(),
            change_summary: "Updated paragraph 5".to_string(),
            edited_by,
            editor_info: None,
        };
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.changes.len(), 1);
        assert_eq!(command.change_summary, "Updated paragraph 5");
    }
    
    #[test]
    fn test_transform_document_creation() {
        let document_id = DocumentId::new();
        let source_cid = create_test_cid("source content");
        
        let command = TransformDocument {
            document_id: document_id.clone(),
            source_cid: source_cid.clone(),
            transformation_type: TransformationType::Translation { 
                target_language: "es".to_string() 
            },
            parameters: std::collections::HashMap::new(),
            processor: "translation_service_v1".to_string(),
            description: Some("Spanish translation".to_string()),
        };
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(
            command.transformation_type,
            TransformationType::Translation { target_language: "es".to_string() }
        );
    }
    
    #[test]
    fn test_merge_document_edits_creation() {
        let document_id = DocumentId::new();
        let base_cid = create_test_cid("base");
        let cid1 = create_test_cid("edit1");
        let cid2 = create_test_cid("edit2");
        let merged_by = Uuid::new_v4();
        
        let command = MergeDocumentEdits {
            document_id: document_id.clone(),
            base_cid: base_cid.clone(),
            merge_cids: vec![cid1.clone(), cid2.clone()],
            merge_strategy: MergeStrategy::AutoMerge,
            merged_by,
            description: Some("Merging concurrent edits".to_string()),
        };
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.merge_cids, vec![cid1, cid2]);
        assert_eq!(command.merge_strategy, MergeStrategy::AutoMerge);
    }
    
    #[test]
    fn test_rollback_document_creation() {
        let document_id = DocumentId::new();
        let current_cid = create_test_cid("current");
        let target_cid = create_test_cid("previous");
        let rolled_back_by = Uuid::new_v4();
        
        let command = RollbackDocument {
            document_id: document_id.clone(),
            current_cid: current_cid.clone(),
            target_cid: target_cid.clone(),
            rolled_back_by,
            reason: "Reverting problematic changes".to_string(),
            create_successor: true,
        };
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.current_cid, current_cid);
        assert_eq!(command.target_cid, target_cid);
        assert!(command.create_successor);
    }
    
    #[test]
    fn test_create_document_successor_command() {
        // US-024: Test document successor creation command
        let document_id = DocumentId::new();
        let predecessor_cid = create_test_cid("original");
        let created_by = Uuid::new_v4();
        
        let request = CreateSuccessorRequest {
            document_id: document_id.clone(),
            predecessor_cid,
            successor_type: SuccessorCreationType::DirectReplacement { 
                content: b"New content".to_vec() 
            },
            created_by,
            description: Some("Updated content".to_string()),
            editor_info: None,
        };
        
        let command = CreateDocumentSuccessor {
            document_id: document_id.clone(),
            request,
        };
        
        assert_eq!(command.document_id, document_id);
        assert!(matches!(
            command.request.successor_type,
            SuccessorCreationType::DirectReplacement { .. }
        ));
        assert_eq!(command.request.description, Some("Updated content".to_string()));
    }
    
    #[test]
    fn test_request_edit_access_command() {
        // US-024: Test edit access request command
        let document_id = DocumentId::new();
        let requested_by = Uuid::new_v4();
        let version_cid = create_test_cid("version");
        
        let command = RequestEditAccess {
            document_id: document_id.clone(),
            version_cid: Some(version_cid.clone()),
            requested_by,
            editor_info: Some(EditorInfo {
                name: "vim".to_string(),
                version: Some("8.2".to_string()),
                platform: Some("linux".to_string()),
                metadata: std::collections::HashMap::new(),
            }),
            expected_duration: Some(chrono::Duration::hours(2)),
        };
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.version_cid, Some(version_cid));
        assert_eq!(command.requested_by, requested_by);
        assert!(command.editor_info.is_some());
        assert!(command.expected_duration.is_some());
    }
    
    #[test]
    fn test_cancel_edit_session_command() {
        // US-024: Test edit session cancellation command
        let document_id = DocumentId::new();
        let session_id = Uuid::new_v4();
        let cancelled_by = Uuid::new_v4();
        
        let command = CancelEditSession {
            document_id: document_id.clone(),
            session_id,
            cancelled_by,
            reason: Some("Session timeout".to_string()),
        };
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.session_id, session_id);
        assert_eq!(command.cancelled_by, cancelled_by);
        assert_eq!(command.reason, Some("Session timeout".to_string()));
    }
    
    #[test]
    fn test_transformation_types() {
        // US-024: Test transformation type variants
        let grammar_correction = TransformationType::GrammarCorrection;
        let translation = TransformationType::Translation { 
            target_language: "es".to_string() 
        };
        let format_conversion = TransformationType::FormatConversion { 
            target_format: "pdf".to_string() 
        };
        let summarization = TransformationType::Summarization { 
            max_length: Some(500) 
        };
        
        assert!(matches!(grammar_correction, TransformationType::GrammarCorrection));
        if let TransformationType::Translation { target_language } = translation {
            assert_eq!(target_language, "es");
        }
        if let TransformationType::FormatConversion { target_format } = format_conversion {
            assert_eq!(target_format, "pdf");
        }
        if let TransformationType::Summarization { max_length } = summarization {
            assert_eq!(max_length, Some(500));
        }
    }
    
    #[test]
    fn test_merge_strategies() {
        // US-024: Test merge strategy variants
        let auto_merge = MergeStrategy::AutoMerge;
        let prefer_first = MergeStrategy::PreferFirst;
        let prefer_last = MergeStrategy::PreferLast;
        let manual_resolve = MergeStrategy::ManualResolve;
        let custom = MergeStrategy::Custom { 
            strategy_name: "three_way_merge".to_string() 
        };
        
        assert!(matches!(auto_merge, MergeStrategy::AutoMerge));
        assert!(matches!(prefer_first, MergeStrategy::PreferFirst));
        assert!(matches!(prefer_last, MergeStrategy::PreferLast));
        assert!(matches!(manual_resolve, MergeStrategy::ManualResolve));
        if let MergeStrategy::Custom { strategy_name } = custom {
            assert_eq!(strategy_name, "three_way_merge");
        }
    }
    
    #[test]
    fn test_edit_command_serialization() {
        // US-024: Test edit command serialization
        let document_id = DocumentId::new();
        let current_cid = create_test_cid("original");
        let edited_by = Uuid::new_v4();
        
        let command = EditDocumentDirect::new(
            document_id.clone(),
            current_cid,
            b"Updated content".to_vec(),
            "text/plain".to_string(),
            edited_by,
        ).with_description("Grammar fixes".to_string());
        
        // Test serialization
        let serialized = serde_json::to_string(&command).unwrap();
        assert!(serialized.contains("Grammar fixes"));
        assert!(serialized.contains("text/plain"));
        
        // Test deserialization
        let deserialized: EditDocumentDirect = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.document_id, command.document_id);
        assert_eq!(deserialized.description, command.description);
        assert_eq!(deserialized.content_type, command.content_type);
    }
    
    #[test]
    fn test_edit_command_builder_pattern() {
        // US-024: Test builder pattern for edit commands
        let document_id = DocumentId::new();
        let current_cid = create_test_cid("original");
        let edited_by = Uuid::new_v4();
        
        let editor_info = EditorInfo {
            name: "vscode".to_string(),
            version: Some("1.70.0".to_string()),
            platform: Some("darwin".to_string()),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("extension_version".to_string(), "0.1.0".to_string());
                map
            },
        };
        
        let command = EditDocumentDirect::new(
            document_id.clone(),
            current_cid,
            b"New content".to_vec(),
            "text/markdown".to_string(),
            edited_by,
        )
        .with_description("Updated documentation".to_string())
        .with_editor_info(editor_info.clone());
        
        assert_eq!(command.description, Some("Updated documentation".to_string()));
        assert_eq!(command.editor_info, Some(editor_info));
        assert_eq!(command.content_type, "text/markdown");
    }
    
    #[test]
    fn test_edit_command_aggregate_ids() {
        // US-024: Test that edit commands properly extract aggregate IDs
        let document_id = DocumentId::new();
        
        let direct_edit = EditDocumentDirect::new(
            document_id.clone(),
            create_test_cid("test"),
            b"content".to_vec(),
            "text/plain".to_string(),
            Uuid::new_v4(),
        );
        
        let patch_edit = EditDocumentPatch::new(
            document_id.clone(),
            create_test_cid("base"),
            b"patch".to_vec(),
            PatchFormat::UnifiedDiff,
            Uuid::new_v4(),
        );
        
        let structured_edit = EditDocumentStructured {
            document_id: document_id.clone(),
            base_cid: create_test_cid("base"),
            changes: vec![],
            change_summary: "Test changes".to_string(),
            edited_by: Uuid::new_v4(),
            editor_info: None,
        };
        
        // All should have the same aggregate ID
        let expected_id = *document_id.as_uuid();
        assert_eq!(*direct_edit.aggregate_id().unwrap().as_uuid(), expected_id);
        assert_eq!(*patch_edit.aggregate_id().unwrap().as_uuid(), expected_id);
        assert_eq!(*structured_edit.aggregate_id().unwrap().as_uuid(), expected_id);
    }
}