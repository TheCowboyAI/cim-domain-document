//! Document Successor and CID Chain Types
//!
//! This module defines types for document editing operations that create
//! successor documents through CID chains, supporting both direct replacement
//! and differential patching patterns.

use serde::{Deserialize, Serialize};
use cid::Cid;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::DocumentId;

/// Represents a document edit operation creating a successor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSuccessor {
    /// Unique identifier for this successor relationship
    pub id: Uuid,
    /// Original document ID
    pub document_id: DocumentId,
    /// Original document CID
    pub predecessor_cid: Cid,
    /// New document CID after edit
    pub successor_cid: Cid,
    /// Type of edit operation
    pub edit_type: EditType,
    /// Edit metadata
    pub edit_metadata: EditMetadata,
    /// Content addressing information
    pub content_info: SuccessorContentInfo,
    /// When this successor was created
    pub created_at: DateTime<Utc>,
}

impl DocumentSuccessor {
    pub fn new(
        document_id: DocumentId,
        predecessor_cid: Cid,
        successor_cid: Cid,
        edit_type: EditType,
        edited_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            document_id,
            predecessor_cid,
            successor_cid,
            edit_type,
            edit_metadata: EditMetadata::new(edited_by),
            content_info: SuccessorContentInfo::default(),
            created_at: Utc::now(),
        }
    }
}

/// Type of edit operation performed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditType {
    /// Complete document replacement: CID -> SuccessorCID
    DirectReplacement,
    /// Patch-based differential edit: (CID + Diff) -> SuccessorCID
    DifferentialPatch { 
        patch_cid: Cid,
        patch_format: PatchFormat,
    },
    /// Structured edit with specific changes
    StructuredEdit { 
        changes: Vec<ContentChange>,
        change_summary: String,
    },
    /// Machine-generated transformation
    AutomatedTransformation {
        transformation_type: String,
        processor: String,
    },
}

/// Supported patch formats for differential edits
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchFormat {
    /// Unified diff format
    UnifiedDiff,
    /// Binary diff (bsdiff)
    BinaryDiff,
    /// Git-style patch
    GitPatch,
    /// JSON patch (RFC 6902)
    JsonPatch,
    /// Custom format
    Custom(String),
}

/// Metadata about the edit operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditMetadata {
    /// Who performed the edit
    pub edited_by: Uuid,
    /// Whether edit was automated
    pub is_automated: bool,
    /// Edit description/comment
    pub description: Option<String>,
    /// Tool or system used for edit
    pub editor_info: Option<EditorInfo>,
    /// Tags associated with this edit
    pub tags: Vec<String>,
    /// Custom metadata
    pub custom_attributes: HashMap<String, serde_json::Value>,
}

impl EditMetadata {
    pub fn new(edited_by: Uuid) -> Self {
        Self {
            edited_by,
            is_automated: false,
            description: None,
            editor_info: None,
            tags: Vec::new(),
            custom_attributes: HashMap::new(),
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn with_editor(mut self, editor_info: EditorInfo) -> Self {
        self.editor_info = Some(editor_info);
        self
    }
    
    pub fn automated(mut self) -> Self {
        self.is_automated = true;
        self
    }
}

/// Information about the editor used
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorInfo {
    /// Name of the editing tool/application
    pub name: String,
    /// Version of the editor
    pub version: Option<String>,
    /// Platform/OS where edit occurred
    pub platform: Option<String>,
    /// Additional editor-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Content addressing information for successor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuccessorContentInfo {
    /// Hash algorithm used
    pub hash_algorithm: String,
    /// Content encoding
    pub encoding: String,
    /// Content size in bytes
    pub content_size: u64,
    /// Whether content is compressed
    pub is_compressed: bool,
    /// Compression algorithm if used
    pub compression: Option<String>,
    /// Content type/MIME type
    pub content_type: String,
}

impl Default for SuccessorContentInfo {
    fn default() -> Self {
        Self {
            hash_algorithm: "blake2b-256".to_string(),
            encoding: "cbor".to_string(),
            content_size: 0,
            is_compressed: false,
            compression: None,
            content_type: "application/octet-stream".to_string(),
        }
    }
}

/// Specific content change in structured edit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Location in document where change occurred
    pub location: ChangeLocation,
    /// Old content (for updates/deletions)
    pub old_content: Option<String>,
    /// New content (for insertions/updates)
    pub new_content: Option<String>,
    /// Size of change in characters/bytes
    pub change_size: u64,
}

/// Type of content change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Content insertion
    Insert,
    /// Content deletion
    Delete,
    /// Content modification
    Update,
    /// Content move/restructure
    Move,
    /// Metadata change
    MetadataUpdate,
}

/// Location within document where change occurred
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeLocation {
    /// Line number (for text documents)
    pub line: Option<u64>,
    /// Column/character position
    pub column: Option<u64>,
    /// Byte offset from start
    pub byte_offset: Option<u64>,
    /// Structural path (e.g., for JSON/XML)
    pub structural_path: Option<String>,
    /// Block or section identifier
    pub block_id: Option<String>,
}

/// CID chain representing document version history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CidChain {
    /// Document this chain belongs to
    pub document_id: DocumentId,
    /// Root/original CID
    pub root_cid: Cid,
    /// Current/latest CID
    pub head_cid: Cid,
    /// Chain of successors from root to head
    pub chain: Vec<CidChainLink>,
    /// Total length of chain
    pub chain_length: u64,
    /// When chain was last updated
    pub updated_at: DateTime<Utc>,
}

impl CidChain {
    pub fn new(document_id: DocumentId, root_cid: Cid) -> Self {
        Self {
            document_id,
            root_cid: root_cid.clone(),
            head_cid: root_cid,
            chain: Vec::new(),
            chain_length: 1,
            updated_at: Utc::now(),
        }
    }
    
    /// Add a new successor to the chain
    pub fn add_successor(&mut self, successor: DocumentSuccessor) -> Result<(), ChainError> {
        // Verify predecessor matches current head
        if successor.predecessor_cid != self.head_cid {
            return Err(ChainError::InvalidPredecessor {
                expected: self.head_cid.clone(),
                actual: successor.predecessor_cid.clone(),
            });
        }
        
        let link = CidChainLink {
            predecessor_cid: successor.predecessor_cid.clone(),
            successor_cid: successor.successor_cid.clone(),
            edit_type: successor.edit_type.clone(),
            created_at: successor.created_at,
            metadata_summary: successor.edit_metadata.description.clone(),
        };
        
        self.chain.push(link);
        self.head_cid = successor.successor_cid;
        self.chain_length += 1;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Get all CIDs in the chain from root to head
    pub fn get_all_cids(&self) -> Vec<Cid> {
        let mut cids = vec![self.root_cid.clone()];
        for link in &self.chain {
            cids.push(link.successor_cid.clone());
        }
        cids
    }
    
    /// Get CID at specific position in chain (0 = root)
    pub fn get_cid_at_position(&self, position: u64) -> Option<Cid> {
        if position == 0 {
            Some(self.root_cid.clone())
        } else if position <= self.chain_length - 1 {
            self.chain.get((position - 1) as usize)
                .map(|link| link.successor_cid.clone())
        } else {
            None
        }
    }
    
    /// Find position of specific CID in chain
    pub fn find_cid_position(&self, target_cid: &Cid) -> Option<u64> {
        if *target_cid == self.root_cid {
            return Some(0);
        }
        
        for (index, link) in self.chain.iter().enumerate() {
            if link.successor_cid == *target_cid {
                return Some((index + 1) as u64);
            }
        }
        
        None
    }
}

/// Link in the CID chain representing one edit operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CidChainLink {
    /// CID before edit
    pub predecessor_cid: Cid,
    /// CID after edit
    pub successor_cid: Cid,
    /// Type of edit that created this link
    pub edit_type: EditType,
    /// When edit occurred
    pub created_at: DateTime<Utc>,
    /// Summary of what changed
    pub metadata_summary: Option<String>,
}

/// Errors that can occur in CID chain operations
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum ChainError {
    #[error("Invalid predecessor CID: expected {expected}, got {actual}")]
    InvalidPredecessor { expected: Cid, actual: Cid },
    
    #[error("Chain corruption detected at position {position}")]
    ChainCorruption { position: u64 },
    
    #[error("CID not found in chain: {cid}")]
    CidNotFound { cid: Cid },
    
    #[error("Chain is empty")]
    EmptyChain,
    
    #[error("Position {position} is out of bounds for chain length {length}")]
    PositionOutOfBounds { position: u64, length: u64 },
}

/// Request to create a document successor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateSuccessorRequest {
    /// Document to create successor for
    pub document_id: DocumentId,
    /// Current CID to create successor from
    pub predecessor_cid: Cid,
    /// Type of successor creation
    pub successor_type: SuccessorCreationType,
    /// Who is creating the successor
    pub created_by: Uuid,
    /// Optional description of changes
    pub description: Option<String>,
    /// Editor information
    pub editor_info: Option<EditorInfo>,
}

/// How the successor should be created
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuccessorCreationType {
    /// Upload complete replacement content
    DirectReplacement { content: Vec<u8> },
    /// Apply diff patch to create successor
    DifferentialPatch { 
        patch_data: Vec<u8>,
        patch_format: PatchFormat,
    },
    /// Specify structured changes
    StructuredChanges { changes: Vec<ContentChange> },
}

/// Response from creating a document successor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateSuccessorResponse {
    /// The created successor
    pub successor: DocumentSuccessor,
    /// Updated CID chain
    pub cid_chain: CidChain,
    /// New document version created
    pub new_version: String,
    /// Any warnings or notes
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cid(_data: &str) -> Cid {
        Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap()
    }

    #[test]
    fn test_document_successor_creation() {
        let document_id = DocumentId::new();
        let predecessor_cid = create_test_cid("original content");
        let successor_cid = create_test_cid("edited content");
        let edited_by = Uuid::new_v4();
        
        let successor = DocumentSuccessor::new(
            document_id.clone(),
            predecessor_cid.clone(),
            successor_cid.clone(),
            EditType::DirectReplacement,
            edited_by,
        );
        
        assert_eq!(successor.document_id, document_id);
        assert_eq!(successor.predecessor_cid, predecessor_cid);
        assert_eq!(successor.successor_cid, successor_cid);
        assert_eq!(successor.edit_metadata.edited_by, edited_by);
        assert!(!successor.edit_metadata.is_automated);
    }
    
    #[test]
    fn test_edit_metadata_builder() {
        let edited_by = Uuid::new_v4();
        let metadata = EditMetadata::new(edited_by)
            .with_description("Grammar corrections".to_string())
            .automated();
        
        assert_eq!(metadata.edited_by, edited_by);
        assert_eq!(metadata.description, Some("Grammar corrections".to_string()));
        assert!(metadata.is_automated);
    }
    
    #[test]
    fn test_cid_chain_creation() {
        let document_id = DocumentId::new();
        let root_cid = create_test_cid("original");
        
        let chain = CidChain::new(document_id.clone(), root_cid.clone());
        
        assert_eq!(chain.document_id, document_id);
        assert_eq!(chain.root_cid, root_cid);
        assert_eq!(chain.head_cid, root_cid);
        assert_eq!(chain.chain_length, 1);
        assert!(chain.chain.is_empty());
    }
    
    #[test]
    fn test_cid_chain_add_successor() {
        let document_id = DocumentId::new();
        let root_cid = create_test_cid("original");
        let successor_cid = create_test_cid("edited");
        
        let mut chain = CidChain::new(document_id.clone(), root_cid.clone());
        
        let successor = DocumentSuccessor::new(
            document_id,
            root_cid.clone(),
            successor_cid.clone(),
            EditType::DirectReplacement,
            Uuid::new_v4(),
        );
        
        let result = chain.add_successor(successor);
        assert!(result.is_ok());
        
        assert_eq!(chain.head_cid, successor_cid);
        assert_eq!(chain.chain_length, 2);
        assert_eq!(chain.chain.len(), 1);
    }
    
    #[test]
    fn test_cid_chain_invalid_predecessor() {
        let document_id = DocumentId::new();
        let root_cid = create_test_cid("original");
        let wrong_cid = create_test_cid("wrong");
        let successor_cid = create_test_cid("edited");
        
        let mut chain = CidChain::new(document_id.clone(), root_cid);
        
        let successor = DocumentSuccessor::new(
            document_id,
            wrong_cid,
            successor_cid,
            EditType::DirectReplacement,
            Uuid::new_v4(),
        );
        
        let result = chain.add_successor(successor);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ChainError::InvalidPredecessor { .. }));
    }
    
    #[test]
    fn test_cid_chain_navigation() {
        let document_id = DocumentId::new();
        let cid1 = create_test_cid("version1");
        let cid2 = create_test_cid("version2");
        let cid3 = create_test_cid("version3");
        
        let mut chain = CidChain::new(document_id.clone(), cid1.clone());
        
        // Add version 2
        let successor2 = DocumentSuccessor::new(
            document_id.clone(),
            cid1.clone(),
            cid2.clone(),
            EditType::DirectReplacement,
            Uuid::new_v4(),
        );
        chain.add_successor(successor2).unwrap();
        
        // Add version 3
        let successor3 = DocumentSuccessor::new(
            document_id,
            cid2.clone(),
            cid3.clone(),
            EditType::DirectReplacement,
            Uuid::new_v4(),
        );
        chain.add_successor(successor3).unwrap();
        
        // Test navigation
        assert_eq!(chain.get_cid_at_position(0), Some(cid1.clone()));
        assert_eq!(chain.get_cid_at_position(1), Some(cid2.clone()));
        assert_eq!(chain.get_cid_at_position(2), Some(cid3.clone()));
        assert_eq!(chain.get_cid_at_position(3), None);
        
        // Test position finding
        assert_eq!(chain.find_cid_position(&cid1), Some(0));
        assert_eq!(chain.find_cid_position(&cid2), Some(1));
        assert_eq!(chain.find_cid_position(&cid3), Some(2));
        
        let unknown_cid = create_test_cid("unknown");
        assert_eq!(chain.find_cid_position(&unknown_cid), None);
        
        // Test get all CIDs
        let all_cids = chain.get_all_cids();
        assert_eq!(all_cids, vec![cid1, cid2, cid3]);
    }
    
    #[test]
    fn test_content_change_structure() {
        let change = ContentChange {
            change_type: ChangeType::Update,
            location: ChangeLocation {
                line: Some(42),
                column: Some(10),
                byte_offset: Some(1024),
                structural_path: Some("/document/paragraph[2]".to_string()),
                block_id: Some("intro".to_string()),
            },
            old_content: Some("Hello World".to_string()),
            new_content: Some("Hello Universe".to_string()),
            change_size: 11,
        };
        
        assert_eq!(change.change_type, ChangeType::Update);
        assert_eq!(change.location.line, Some(42));
        assert_eq!(change.old_content, Some("Hello World".to_string()));
        assert_eq!(change.new_content, Some("Hello Universe".to_string()));
    }
}