//! CID-First Document Ingestion Commands
//!
//! These commands implement proper BLOB handling with Object Store partitioning
//! and ensure the Event Store never contains BLOB data - only CIDs.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use cid::Cid;
use chrono::{DateTime, Utc};

use crate::value_objects::{DocumentId, UserId};
use crate::services::object_store::{ObjectStorePartition, ProcessingJob, ContentMetadata};
use crate::commands::{Command, DomainCommand};
use cim_domain::EntityId;

/// Primary ingestion command - accepts raw BLOB data
/// This is the ONLY command that contains actual content bytes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestDocumentContent {
    /// Raw document bytes (PDF, DOCX, etc.) - THE ONLY BLOB IN THE SYSTEM
    pub content: Vec<u8>,
    
    /// Optional metadata hints (not stored with content)
    pub suggested_filename: Option<String>,
    pub content_type_hint: Option<String>,
    
    /// Processing preferences
    pub target_partition: ObjectStorePartition,
    pub enable_virus_scanning: bool,
    pub enable_format_validation: bool,
    pub enable_auto_promotion: bool,
    
    /// Actor information
    pub uploaded_by: UserId,
    
    /// Correlation for tracking
    pub correlation_id: Option<Uuid>,
}

impl DomainCommand for IngestDocumentContent {
    type Aggregate = crate::Document;
    
    // No aggregate ID yet - this creates the document
    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        None
    }
}

impl Command for IngestDocumentContent {}

/// Command to create document entity after content ingestion
/// This separates content storage from document entity creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentFromCid {
    /// Document entity identifier
    pub document_id: DocumentId,
    
    /// Content CID from Object Store (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Which partition contains the content
    pub content_partition: ObjectStorePartition,
    
    /// Content metadata detected during ingestion
    pub content_metadata: ContentMetadata,
    
    /// Initial document metadata (separate from content)
    pub filename: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    
    /// Actor information
    pub created_by: UserId,
}

impl DomainCommand for CreateDocumentFromCid {
    type Aggregate = crate::Document;
    
    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl Command for CreateDocumentFromCid {}

/// Command to promote content between Object Store partitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteDocumentContent {
    /// Document that owns this content
    pub document_id: DocumentId,
    
    /// Content CID to promote (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Source partition
    pub from_partition: ObjectStorePartition,
    
    /// Destination partition
    pub to_partition: ObjectStorePartition,
    
    /// Reason for promotion
    pub promotion_reason: String,
    
    /// Processing results that enabled promotion
    pub processing_results: Vec<ProcessingResult>,
    
    /// Actor information
    pub promoted_by: UserId,
}

impl DomainCommand for PromoteDocumentContent {
    type Aggregate = crate::Document;
    
    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl Command for PromoteDocumentContent {}

/// Command to update document metadata (separate from content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentMetadata {
    /// Document to update
    pub document_id: DocumentId,
    
    /// Content CID this metadata refers to (for correlation)
    pub content_cid: Option<Cid>,
    
    /// Metadata updates
    pub filename: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub custom_properties: std::collections::HashMap<String, serde_json::Value>,
    
    /// Actor information  
    pub updated_by: UserId,
}

impl DomainCommand for UpdateDocumentMetadata {
    type Aggregate = crate::Document;
    
    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl Command for UpdateDocumentMetadata {}

/// Command to quarantine content due to processing failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineDocumentContent {
    /// Document that owns this content
    pub document_id: DocumentId,
    
    /// Content CID to quarantine (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Current partition of content
    pub current_partition: ObjectStorePartition,
    
    /// Quarantine reason
    pub quarantine_reason: String,
    
    /// Threat details if from virus scan
    pub threats_detected: Vec<String>,
    
    /// When quarantine expires (for cleanup)
    pub expires_at: DateTime<Utc>,
    
    /// Actor information (usually system)
    pub quarantined_by: UserId,
}

impl DomainCommand for QuarantineDocumentContent {
    type Aggregate = crate::Document;
    
    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(*self.document_id.as_uuid()))
    }
}

impl Command for QuarantineDocumentContent {}

// ===== SUPPORTING TYPES =====

/// Result of processing stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub stage_name: String,
    pub success: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub details: ProcessingDetails,
}

/// Specific processing stage results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingDetails {
    VirusScan {
        threats_found: Vec<String>,
        scanner_version: String,
        definitions_updated: DateTime<Utc>,
    },
    FormatValidation {
        format_valid: bool,
        detected_format: String,
        format_version: Option<String>,
        validation_errors: Vec<String>,
    },
    ContentAnalysis {
        language_detected: Option<String>,
        text_extractable: bool,
        page_count: Option<u32>,
        embedded_objects: u32,
    },
}

// ===== INGESTION RESPONSES =====

/// Response to content ingestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestDocumentResponse {
    /// Immutable content identifier
    pub content_cid: Cid,
    
    /// Document entity created
    pub document_id: DocumentId,
    
    /// Where content was stored
    pub content_partition: ObjectStorePartition,
    
    /// Processing job if async processing enabled
    pub processing_job: Option<ProcessingJob>,
    
    /// Content metadata detected during ingestion
    pub content_metadata: ContentMetadata,
    
    /// When ingestion completed
    pub ingested_at: DateTime<Utc>,
}

/// Response to document creation from CID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    /// Document entity identifier
    pub document_id: DocumentId,
    
    /// Content CID (immutable reference)
    pub content_cid: Cid,
    
    /// Current content location
    pub content_partition: ObjectStorePartition,
    
    /// When document entity was created
    pub created_at: DateTime<Utc>,
}

/// Response to content promotion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteContentResponse {
    /// Content CID that was promoted
    pub content_cid: Cid,
    
    /// Source partition
    pub from_partition: ObjectStorePartition,
    
    /// Destination partition
    pub to_partition: ObjectStorePartition,
    
    /// When promotion completed
    pub promoted_at: DateTime<Utc>,
    
    /// Whether source content was cleaned up
    pub source_cleaned: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::object_store::DocumentPartitions;

    #[test]
    fn test_ingest_document_content_command() {
        let content = b"PDF content bytes".to_vec();
        let user_id = UserId::new();
        
        let command = IngestDocumentContent {
            content: content.clone(),
            suggested_filename: Some("test.pdf".to_string()),
            content_type_hint: Some("application/pdf".to_string()),
            target_partition: DocumentPartitions::staging(),
            enable_virus_scanning: true,
            enable_format_validation: true,
            enable_auto_promotion: false,
            uploaded_by: user_id,
            correlation_id: Some(Uuid::new_v4()),
        };
        
        assert_eq!(command.content, content);
        assert_eq!(command.suggested_filename, Some("test.pdf".to_string()));
        assert!(command.enable_virus_scanning);
        assert!(!command.enable_auto_promotion);
    }

    #[test]
    fn test_create_document_from_cid_command() {
        let document_id = DocumentId::new();
        let content_cid = cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o").unwrap();
        let user_id = UserId::new();
        
        let metadata = ContentMetadata {
            mime_type: "application/pdf".to_string(),
            size_bytes: 1024,
            hash_algorithm: "sha256".to_string(),
            detected_format: Some("PDF-1.4".to_string()),
            is_encrypted: false,
            language_hint: Some("en".to_string()),
        };
        
        let command = CreateDocumentFromCid {
            document_id,
            content_cid,
            content_partition: DocumentPartitions::aggregate(),
            content_metadata: metadata.clone(),
            filename: Some("report.pdf".to_string()),
            title: Some("Quarterly Report".to_string()),
            description: None,
            tags: vec!["report".to_string(), "quarterly".to_string()],
            created_by: user_id,
        };
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.content_cid, content_cid);
        assert_eq!(command.filename, Some("report.pdf".to_string()));
        assert_eq!(command.tags.len(), 2);
        assert_eq!(command.content_metadata.mime_type, "application/pdf");
    }

    #[test]
    fn test_promote_content_command() {
        let document_id = DocumentId::new();
        let content_cid = cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o").unwrap();
        let user_id = UserId::new();
        
        let processing_result = ProcessingResult {
            stage_name: "virus_scan".to_string(),
            success: true,
            started_at: Utc::now(),
            completed_at: Utc::now(),
            details: ProcessingDetails::VirusScan {
                threats_found: vec![],
                scanner_version: "ClamAV 0.103".to_string(),
                definitions_updated: Utc::now(),
            },
        };
        
        let command = PromoteDocumentContent {
            document_id,
            content_cid,
            from_partition: DocumentPartitions::staging(),
            to_partition: DocumentPartitions::aggregate(),
            promotion_reason: "All processing stages passed".to_string(),
            processing_results: vec![processing_result],
            promoted_by: user_id,
        };
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.content_cid, content_cid);
        assert_eq!(command.processing_results.len(), 1);
        assert!(command.processing_results[0].success);
    }

    #[test]
    fn test_quarantine_command() {
        let document_id = DocumentId::new();
        let content_cid = cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o").unwrap();
        let user_id = UserId::new();
        
        let command = QuarantineDocumentContent {
            document_id,
            content_cid,
            current_partition: DocumentPartitions::staging(),
            quarantine_reason: "Virus detected".to_string(),
            threats_detected: vec!["Trojan.Generic".to_string()],
            expires_at: Utc::now() + chrono::Duration::days(30),
            quarantined_by: user_id,
        };
        
        assert_eq!(command.document_id, document_id);
        assert_eq!(command.threats_detected.len(), 1);
        assert_eq!(command.quarantine_reason, "Virus detected");
    }
}