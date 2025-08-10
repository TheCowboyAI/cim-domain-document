//! Document Editing Events
//!
//! This module defines events that are published when documents are edited,
//! creating successor versions through the CID chain pattern.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use cid::Cid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::value_objects::{
    DocumentId, DocumentSuccessor, CidChain,
    ContentChange, PatchFormat, MergeStrategy, EditType, EditMetadata
};

// Import TransformationType from commands module
use crate::commands::edit_commands::TransformationType;

/// Document successor was created through edit operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSuccessorCreated {
    /// Document that was edited
    pub document_id: DocumentId,
    /// The created successor
    pub successor: DocumentSuccessor,
    /// Updated CID chain
    pub updated_chain: CidChain,
    /// New version identifier
    pub new_version: String,
    /// Who performed the edit
    pub edited_by: Uuid,
    /// When the edit occurred
    pub edited_at: DateTime<Utc>,
    /// Size change from predecessor
    pub size_delta: i64,
}

impl DocumentSuccessorCreated {
    pub fn new(
        document_id: DocumentId,
        successor: DocumentSuccessor,
        updated_chain: CidChain,
        new_version: String,
        edited_by: Uuid,
    ) -> Self {
        Self {
            document_id,
            successor,
            updated_chain,
            new_version,
            edited_by,
            edited_at: Utc::now(),
            size_delta: 0, // Will be calculated by handler
        }
    }
}

/// Document was edited using direct replacement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentEditedDirect {
    /// Document that was edited
    pub document_id: DocumentId,
    /// Previous CID before edit
    pub previous_cid: Cid,
    /// New CID after edit
    pub new_cid: Cid,
    /// Content type of new content
    pub content_type: String,
    /// Size of new content
    pub content_size: u64,
    /// Edit metadata
    pub edit_metadata: EditMetadata,
    /// When the edit occurred
    pub edited_at: DateTime<Utc>,
}

/// Document was edited using differential patch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentEditedPatch {
    /// Document that was edited
    pub document_id: DocumentId,
    /// Base CID that was patched
    pub base_cid: Cid,
    /// Resulting CID after patch application
    pub result_cid: Cid,
    /// CID of the patch itself
    pub patch_cid: Cid,
    /// Format of the patch
    pub patch_format: PatchFormat,
    /// Size of patch data
    pub patch_size: u64,
    /// Edit metadata
    pub edit_metadata: EditMetadata,
    /// When the edit occurred
    pub edited_at: DateTime<Utc>,
}

/// Document was edited using structured changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentEditedStructured {
    /// Document that was edited
    pub document_id: DocumentId,
    /// Base CID before changes
    pub base_cid: Cid,
    /// Result CID after changes
    pub result_cid: Cid,
    /// Structured changes that were applied
    pub changes: Vec<ContentChange>,
    /// Summary of changes
    pub change_summary: String,
    /// Number of changes applied
    pub change_count: u32,
    /// Edit metadata
    pub edit_metadata: EditMetadata,
    /// When the edit occurred
    pub edited_at: DateTime<Utc>,
}

/// Edit access was requested for document
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditAccessRequested {
    /// Document access was requested for
    pub document_id: DocumentId,
    /// Specific version CID requested (if any)
    pub version_cid: Option<Cid>,
    /// User who requested access
    pub requested_by: Uuid,
    /// Generated session ID for edit
    pub session_id: Uuid,
    /// Editor that will be used
    pub editor_name: Option<String>,
    /// Expected duration of edit
    pub expected_duration: Option<chrono::Duration>,
    /// When access was requested
    pub requested_at: DateTime<Utc>,
}

/// Edit access was granted
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditAccessGranted {
    /// Document access was granted for
    pub document_id: DocumentId,
    /// Edit session ID
    pub session_id: Uuid,
    /// User granted access
    pub granted_to: Uuid,
    /// CID of content that can be edited
    pub editable_cid: Cid,
    /// Access expiration time
    pub expires_at: DateTime<Utc>,
    /// Content export format/URL for editing
    pub edit_url: Option<String>,
    /// When access was granted
    pub granted_at: DateTime<Utc>,
}

/// Edit session was cancelled
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditSessionCancelled {
    /// Document being edited
    pub document_id: DocumentId,
    /// Edit session that was cancelled
    pub session_id: Uuid,
    /// User who cancelled (may be different from editor)
    pub cancelled_by: Uuid,
    /// Original editor
    pub original_editor: Uuid,
    /// Reason for cancellation
    pub reason: Option<String>,
    /// Whether any changes were lost
    pub changes_lost: bool,
    /// When session was cancelled
    pub cancelled_at: DateTime<Utc>,
}

/// Document was automatically transformed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTransformed {
    /// Document that was transformed
    pub document_id: DocumentId,
    /// Source CID before transformation
    pub source_cid: Cid,
    /// Result CID after transformation
    pub result_cid: Cid,
    /// Type of transformation applied
    pub transformation_type: TransformationType,
    /// Parameters used for transformation
    pub parameters: HashMap<String, serde_json::Value>,
    /// System that performed transformation
    pub processor: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Success indicators and quality metrics
    pub metrics: TransformationMetrics,
    /// When transformation occurred
    pub transformed_at: DateTime<Utc>,
}

/// Metrics about transformation quality and performance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformationMetrics {
    /// Whether transformation was successful
    pub success: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence_score: Option<f64>,
    /// Quality score (0.0 - 1.0) 
    pub quality_score: Option<f64>,
    /// Number of changes made
    pub changes_count: u32,
    /// Size change percentage
    pub size_change_percent: f64,
    /// Any warnings or issues
    pub warnings: Vec<String>,
}

/// Multiple document edits were merged
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentEditsMerged {
    /// Document with merged edits
    pub document_id: DocumentId,
    /// Base CID used for merge
    pub base_cid: Cid,
    /// CIDs that were merged
    pub merged_cids: Vec<Cid>,
    /// Result CID after merge
    pub result_cid: Cid,
    /// Strategy used for merge
    pub merge_strategy: MergeStrategy,
    /// Number of conflicts encountered
    pub conflict_count: u32,
    /// How conflicts were resolved
    pub conflict_resolutions: Vec<ConflictResolution>,
    /// Who performed the merge
    pub merged_by: Uuid,
    /// When merge occurred
    pub merged_at: DateTime<Utc>,
}

/// How a specific merge conflict was resolved
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// Location of conflict
    pub location: String,
    /// Type of conflict
    pub conflict_type: String,
    /// How it was resolved
    pub resolution: String,
    /// Source of resolution (auto/manual)
    pub resolution_source: ResolutionSource,
}

/// Source of conflict resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionSource {
    /// Automatically resolved by system
    Automatic,
    /// Manually resolved by user
    Manual,
    /// Resolved using predefined rule
    Rule(String),
}

/// Document was rolled back to previous version
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentRolledBack {
    /// Document that was rolled back
    pub document_id: DocumentId,
    /// CID before rollback
    pub from_cid: Cid,
    /// CID after rollback
    pub to_cid: Cid,
    /// Version rolled back to
    pub target_version: String,
    /// Reason for rollback
    pub reason: String,
    /// Who performed rollback
    pub rolled_back_by: Uuid,
    /// Whether rollback created new successor
    pub created_successor: bool,
    /// Steps skipped in rollback
    pub versions_skipped: u32,
    /// When rollback occurred
    pub rolled_back_at: DateTime<Utc>,
}

/// CID chain integrity was verified
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CidChainVerified {
    /// Document whose chain was verified
    pub document_id: DocumentId,
    /// Chain that was verified
    pub verified_chain: CidChain,
    /// Verification result
    pub verification_result: ChainVerificationResult,
    /// When verification occurred
    pub verified_at: DateTime<Utc>,
}

/// Result of CID chain verification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChainVerificationResult {
    /// Whether chain is valid
    pub is_valid: bool,
    /// Any issues found
    pub issues: Vec<ChainIssue>,
    /// Performance metrics
    pub verification_time_ms: u64,
    /// Number of links verified
    pub links_verified: u32,
}

/// Issue found in CID chain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChainIssue {
    /// Type of issue
    pub issue_type: ChainIssueType,
    /// Position in chain where issue occurred
    pub position: u64,
    /// Description of issue
    pub description: String,
    /// Severity level
    pub severity: IssueSeverity,
}

/// Types of chain integrity issues
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainIssueType {
    /// Missing CID in storage
    MissingContent,
    /// Hash verification failed
    HashMismatch,
    /// Chain link is broken
    BrokenLink,
    /// Duplicate CID in chain
    DuplicateContent,
    /// Metadata inconsistency
    MetadataInconsistent,
}

/// Severity of chain issue
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Low severity - chain still usable
    Low,
    /// Medium severity - some functions affected
    Medium,
    /// High severity - chain integrity compromised
    High,
    /// Critical - chain unusable
    Critical,
}

/// Document edit failed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentEditFailed {
    /// Document that failed to edit
    pub document_id: DocumentId,
    /// CID that was being edited
    pub target_cid: Cid,
    /// Type of edit that failed
    pub edit_type: String,
    /// Who attempted the edit
    pub attempted_by: Uuid,
    /// Error that occurred
    pub error_code: String,
    /// Error description
    pub error_message: String,
    /// Any partial changes that were rolled back
    pub rolled_back_changes: Vec<String>,
    /// When failure occurred
    pub failed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{EditType, EditMetadata};

    fn create_test_cid(_data: &str) -> Cid {
        Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap()
    }

    #[test]
    fn test_document_successor_created_event() {
        let document_id = DocumentId::new();
        let predecessor_cid = create_test_cid("original");
        let successor_cid = create_test_cid("edited");
        let edited_by = Uuid::new_v4();
        
        let successor = DocumentSuccessor::new(
            document_id.clone(),
            predecessor_cid,
            successor_cid,
            EditType::DirectReplacement,
            edited_by,
        );
        
        let chain = CidChain::new(document_id.clone(), predecessor_cid);
        
        let event = DocumentSuccessorCreated::new(
            document_id.clone(),
            successor,
            chain,
            "1.1".to_string(),
            edited_by,
        );
        
        assert_eq!(event.document_id, document_id);
        assert_eq!(event.new_version, "1.1");
        assert_eq!(event.edited_by, edited_by);
    }
    
    #[test]
    fn test_document_edited_direct_event() {
        let document_id = DocumentId::new();
        let previous_cid = create_test_cid("original");
        let new_cid = create_test_cid("edited");
        let edited_by = Uuid::new_v4();
        
        let event = DocumentEditedDirect {
            document_id: document_id.clone(),
            previous_cid: previous_cid.clone(),
            new_cid: new_cid.clone(),
            content_type: "text/plain".to_string(),
            content_size: 1024,
            edit_metadata: EditMetadata::new(edited_by),
            edited_at: Utc::now(),
        };
        
        assert_eq!(event.document_id, document_id);
        assert_eq!(event.previous_cid, previous_cid);
        assert_eq!(event.new_cid, new_cid);
        assert_eq!(event.content_type, "text/plain");
    }
    
    #[test]
    fn test_document_transformed_event() {
        let document_id = DocumentId::new();
        let source_cid = create_test_cid("source");
        let result_cid = create_test_cid("transformed");
        
        let metrics = TransformationMetrics {
            success: true,
            confidence_score: Some(0.95),
            quality_score: Some(0.88),
            changes_count: 42,
            size_change_percent: 5.2,
            warnings: vec!["Minor formatting inconsistency".to_string()],
        };
        
        let event = DocumentTransformed {
            document_id: document_id.clone(),
            source_cid: source_cid.clone(),
            result_cid: result_cid.clone(),
            transformation_type: TransformationType::GrammarCorrection,
            parameters: HashMap::new(),
            processor: "grammar_ai_v2".to_string(),
            processing_time_ms: 2500,
            metrics,
            transformed_at: Utc::now(),
        };
        
        assert_eq!(event.document_id, document_id);
        assert_eq!(event.source_cid, source_cid);
        assert_eq!(event.result_cid, result_cid);
        assert!(event.metrics.success);
        assert_eq!(event.metrics.changes_count, 42);
    }
    
    #[test]
    fn test_document_edits_merged_event() {
        let document_id = DocumentId::new();
        let base_cid = create_test_cid("base");
        let cid1 = create_test_cid("edit1");
        let cid2 = create_test_cid("edit2");
        let result_cid = create_test_cid("merged");
        let merged_by = Uuid::new_v4();
        
        let conflict_resolution = ConflictResolution {
            location: "line 42".to_string(),
            conflict_type: "text_overlap".to_string(),
            resolution: "preferred_version_1".to_string(),
            resolution_source: ResolutionSource::Automatic,
        };
        
        let event = DocumentEditsMerged {
            document_id: document_id.clone(),
            base_cid: base_cid.clone(),
            merged_cids: vec![cid1.clone(), cid2.clone()],
            result_cid: result_cid.clone(),
            merge_strategy: MergeStrategy::AutoMerge,
            conflict_count: 1,
            conflict_resolutions: vec![conflict_resolution],
            merged_by,
            merged_at: Utc::now(),
        };
        
        assert_eq!(event.document_id, document_id);
        assert_eq!(event.merged_cids, vec![cid1, cid2]);
        assert_eq!(event.conflict_count, 1);
        assert_eq!(event.conflict_resolutions.len(), 1);
    }
    
    #[test]
    fn test_cid_chain_verified_event() {
        let document_id = DocumentId::new();
        let root_cid = create_test_cid("root");
        let chain = CidChain::new(document_id.clone(), root_cid);
        
        let verification_result = ChainVerificationResult {
            is_valid: true,
            issues: vec![],
            verification_time_ms: 150,
            links_verified: 5,
        };
        
        let event = CidChainVerified {
            document_id: document_id.clone(),
            verified_chain: chain,
            verification_result,
            verified_at: Utc::now(),
        };
        
        assert_eq!(event.document_id, document_id);
        assert!(event.verification_result.is_valid);
        assert_eq!(event.verification_result.links_verified, 5);
    }
}