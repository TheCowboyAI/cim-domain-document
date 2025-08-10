//! CID-First Document Ingestion Events
//!
//! These events are emitted during the document ingestion process.
//! CRITICAL: No BLOB data is ever stored in events - only CIDs and metadata.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use cid::Cid;
use chrono::{DateTime, Utc};

use crate::value_objects::{DocumentId, UserId};
use crate::services::object_store::{ObjectStorePartition, ProcessingJob, ContentMetadata};
use crate::commands::ingestion_commands::{ProcessingResult, ProcessingDetails};
// Note: Using custom domain event pattern like existing events
use crate::events::DocumentDomainEvent;

// ===== CONTENT INGESTION EVENTS =====

/// Raw content was ingested into Object Store (BLOB-free)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContentIngested {
    /// Content CID created (NO BLOB DATA - just the identifier)
    pub content_cid: Cid,
    
    /// Where content was stored
    pub content_partition: ObjectStorePartition,
    
    /// Content metadata detected during ingestion
    pub content_metadata: ContentMetadata,
    
    /// Processing job created (if async processing enabled)
    pub processing_job_id: Option<Uuid>,
    
    /// When ingestion completed
    pub ingested_at: DateTime<Utc>,
    
    /// Who uploaded the content
    pub uploaded_by: UserId,
}

impl DocumentContentIngested {
    pub fn event_type(&self) -> &'static str {
        "DocumentContentIngested"
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Document entity was created from existing content CID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreatedFromCid {
    /// Document entity identifier
    pub document_id: DocumentId,
    
    /// Content CID this document references (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Where the content is stored
    pub content_partition: ObjectStorePartition,
    
    /// Initial document metadata
    pub filename: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    
    /// Content metadata from ingestion
    pub content_metadata: ContentMetadata,
    
    /// When document entity was created
    pub created_at: DateTime<Utc>,
    
    /// Who created the document
    pub created_by: UserId,
}

impl DocumentCreatedFromCid {
    pub fn event_type(&self) -> &'static str {
        "DocumentCreatedFromCid"
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ===== PROCESSING EVENTS =====

/// Processing job was started for content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProcessingStarted {
    /// Content CID being processed (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Processing job details
    pub processing_job: ProcessingJob,
    
    /// Document that initiated processing (if exists)
    pub document_id: Option<DocumentId>,
    
    /// When processing started
    pub started_at: DateTime<Utc>,
    
    /// Who initiated processing
    pub initiated_by: UserId,
}

impl DocumentProcessingStarted {
    pub fn event_type(&self) -> &'static str {
        "DocumentProcessingStarted"
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Processing stage completed for content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProcessingStageCompleted {
    /// Content CID being processed (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Processing job ID
    pub processing_job_id: Uuid,
    
    /// Stage result
    pub stage_result: ProcessingResult,
    
    /// Next stage to execute (if any)
    pub next_stage: Option<String>,
    
    /// Whether entire processing job is complete
    pub job_complete: bool,
    
    /// When stage completed
    pub completed_at: DateTime<Utc>,
}

impl DocumentProcessingStageCompleted {
    pub fn event_type(&self) -> &'static str {
        "DocumentProcessingStageCompleted"
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Processing job completed (all stages done)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProcessingCompleted {
    /// Content CID that was processed (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Processing job ID
    pub processing_job_id: Uuid,
    
    /// All processing results
    pub processing_results: Vec<ProcessingResult>,
    
    /// Whether processing succeeded overall
    pub success: bool,
    
    /// Failure reason if processing failed
    pub failure_reason: Option<String>,
    
    /// When processing completed
    pub completed_at: DateTime<Utc>,
}

impl DocumentProcessingCompleted {
    pub fn event_type(&self) -> &'static str {
        "DocumentProcessingCompleted"
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ===== CONTENT PROMOTION EVENTS =====

/// Content was promoted between Object Store partitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContentPromoted {
    /// Document that owns this content
    pub document_id: DocumentId,
    
    /// Content CID that was promoted (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Source partition
    pub from_partition: ObjectStorePartition,
    
    /// Destination partition
    pub to_partition: ObjectStorePartition,
    
    /// Promotion reason
    pub promotion_reason: String,
    
    /// Processing results that enabled promotion
    pub processing_results: Vec<ProcessingResult>,
    
    /// When promotion completed
    pub promoted_at: DateTime<Utc>,
    
    /// Who promoted the content
    pub promoted_by: UserId,
    
    /// Whether source content was cleaned up
    pub source_cleaned: bool,
}

impl DocumentContentPromoted {
    pub fn event_type(&self) -> &'static str {
        "DocumentContentPromoted"
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ===== QUARANTINE EVENTS =====

/// Content was quarantined due to processing issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContentQuarantined {
    /// Document that owns this content
    pub document_id: DocumentId,
    
    /// Content CID that was quarantined (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Current partition of content
    pub current_partition: ObjectStorePartition,
    
    /// Quarantine reason
    pub quarantine_reason: String,
    
    /// Threat details if from virus scan
    pub threats_detected: Vec<String>,
    
    /// Processing job that triggered quarantine
    pub processing_job_id: Option<Uuid>,
    
    /// When quarantine expires
    pub expires_at: DateTime<Utc>,
    
    /// When quarantine was applied
    pub quarantined_at: DateTime<Utc>,
    
    /// Who quarantined the content
    pub quarantined_by: UserId,
}

impl DocumentContentQuarantined {
    pub fn event_type(&self) -> &'static str {
        "DocumentContentQuarantined"
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Quarantined content was released
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContentReleased {
    /// Document that owns this content
    pub document_id: DocumentId,
    
    /// Content CID that was released (NO BLOB DATA)
    pub content_cid: Cid,
    
    /// Partition where content resides
    pub content_partition: ObjectStorePartition,
    
    /// Release reason
    pub release_reason: String,
    
    /// Whether content can now be promoted
    pub promotion_allowed: bool,
    
    /// When content was released
    pub released_at: DateTime<Utc>,
    
    /// Who released the content
    pub released_by: UserId,
}

impl DocumentContentReleased {
    pub fn event_type(&self) -> &'static str {
        "DocumentContentReleased"
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ===== CLEANUP EVENTS =====

/// Expired staging content was cleaned up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStagingContentCleaned {
    /// Content CIDs that were cleaned (NO BLOB DATA)
    pub cleaned_content_cids: Vec<Cid>,
    
    /// Staging partition that was cleaned
    pub staging_partition: ObjectStorePartition,
    
    /// Total bytes freed
    pub bytes_freed: u64,
    
    /// Cleanup reason
    pub cleanup_reason: String,
    
    /// When cleanup occurred
    pub cleaned_at: DateTime<Utc>,
    
    /// System actor that performed cleanup
    pub cleaned_by: UserId,
}

impl DocumentStagingContentCleaned {
    pub fn event_type(&self) -> &'static str {
        "DocumentStagingContentCleaned"
    }
    
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ===== DOMAIN EVENT ENUM EXTENSIONS =====

// These would be added to the main DocumentDomainEvent enum
impl From<DocumentContentIngested> for DocumentDomainEvent {
    fn from(event: DocumentContentIngested) -> Self {
        // This would be added to the main enum
        // DocumentDomainEvent::DocumentContentIngested(event)
        todo!("Add to main DocumentDomainEvent enum")
    }
}

impl From<DocumentCreatedFromCid> for DocumentDomainEvent {
    fn from(event: DocumentCreatedFromCid) -> Self {
        // DocumentDomainEvent::DocumentCreatedFromCid(event)
        todo!("Add to main DocumentDomainEvent enum")
    }
}

impl From<DocumentProcessingStarted> for DocumentDomainEvent {
    fn from(event: DocumentProcessingStarted) -> Self {
        // DocumentDomainEvent::DocumentProcessingStarted(event)
        todo!("Add to main DocumentDomainEvent enum")
    }
}

impl From<DocumentProcessingStageCompleted> for DocumentDomainEvent {
    fn from(event: DocumentProcessingStageCompleted) -> Self {
        // DocumentDomainEvent::DocumentProcessingStageCompleted(event)
        todo!("Add to main DocumentDomainEvent enum")
    }
}

impl From<DocumentProcessingCompleted> for DocumentDomainEvent {
    fn from(event: DocumentProcessingCompleted) -> Self {
        // DocumentDomainEvent::DocumentProcessingCompleted(event)
        todo!("Add to main DocumentDomainEvent enum")
    }
}

impl From<DocumentContentPromoted> for DocumentDomainEvent {
    fn from(event: DocumentContentPromoted) -> Self {
        // DocumentDomainEvent::DocumentContentPromoted(event)
        todo!("Add to main DocumentDomainEvent enum")
    }
}

impl From<DocumentContentQuarantined> for DocumentDomainEvent {
    fn from(event: DocumentContentQuarantined) -> Self {
        // DocumentDomainEvent::DocumentContentQuarantined(event)
        todo!("Add to main DocumentDomainEvent enum")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::object_store::DocumentPartitions;

    #[test]
    fn test_document_content_ingested_event() {
        let content_cid = cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o").unwrap();
        let user_id = UserId::new();
        let now = Utc::now();
        
        let metadata = ContentMetadata {
            mime_type: "application/pdf".to_string(),
            size_bytes: 1024,
            hash_algorithm: "sha256".to_string(),
            detected_format: Some("PDF-1.4".to_string()),
            is_encrypted: false,
            language_hint: Some("en".to_string()),
        };
        
        let event = DocumentContentIngested {
            content_cid,
            content_partition: DocumentPartitions::staging(),
            content_metadata: metadata,
            processing_job_id: Some(Uuid::new_v4()),
            ingested_at: now,
            uploaded_by: user_id,
        };
        
        assert_eq!(event.content_cid, content_cid);
        assert_eq!(event.event_type(), "DocumentContentIngested");
        assert!(event.processing_job_id.is_some());
    }

    #[test]
    fn test_document_created_from_cid_event() {
        let document_id = DocumentId::new();
        let content_cid = cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o").unwrap();
        let user_id = UserId::new();
        
        let metadata = ContentMetadata {
            mime_type: "application/pdf".to_string(),
            size_bytes: 2048,
            hash_algorithm: "sha256".to_string(),
            detected_format: Some("PDF-1.4".to_string()),
            is_encrypted: false,
            language_hint: Some("en".to_string()),
        };
        
        let event = DocumentCreatedFromCid {
            document_id,
            content_cid,
            content_partition: DocumentPartitions::aggregate(),
            filename: Some("test.pdf".to_string()),
            title: Some("Test Document".to_string()),
            description: None,
            tags: vec!["test".to_string()],
            content_metadata: metadata,
            created_at: Utc::now(),
            created_by: user_id,
        };
        
        assert_eq!(event.document_id, document_id);
        assert_eq!(event.content_cid, content_cid);
        assert_eq!(event.event_type(), "DocumentCreatedFromCid");
        assert_eq!(event.filename, Some("test.pdf".to_string()));
    }

    #[test]
    fn test_document_content_promoted_event() {
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
        
        let event = DocumentContentPromoted {
            document_id,
            content_cid,
            from_partition: DocumentPartitions::staging(),
            to_partition: DocumentPartitions::aggregate(),
            promotion_reason: "All checks passed".to_string(),
            processing_results: vec![processing_result],
            promoted_at: Utc::now(),
            promoted_by: user_id,
            source_cleaned: true,
        };
        
        assert_eq!(event.document_id, document_id);
        assert_eq!(event.content_cid, content_cid);
        assert_eq!(event.event_type(), "DocumentContentPromoted");
        assert!(event.source_cleaned);
    }

    #[test]
    fn test_document_content_quarantined_event() {
        let document_id = DocumentId::new();
        let content_cid = cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o").unwrap();
        let user_id = UserId::new();
        
        let event = DocumentContentQuarantined {
            document_id,
            content_cid,
            current_partition: DocumentPartitions::staging(),
            quarantine_reason: "Virus detected".to_string(),
            threats_detected: vec!["Trojan.Generic".to_string(), "Backdoor.Win32".to_string()],
            processing_job_id: Some(Uuid::new_v4()),
            expires_at: Utc::now() + chrono::Duration::days(30),
            quarantined_at: Utc::now(),
            quarantined_by: user_id,
        };
        
        assert_eq!(event.document_id, document_id);
        assert_eq!(event.threats_detected.len(), 2);
        assert_eq!(event.event_type(), "DocumentContentQuarantined");
    }

    #[test]
    fn test_staging_content_cleaned_event() {
        let content_cids = vec![
            cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o").unwrap(),
            cid::Cid::try_from("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG").unwrap(),
        ];
        let user_id = UserId::new();
        
        let event = DocumentStagingContentCleaned {
            cleaned_content_cids: content_cids.clone(),
            staging_partition: DocumentPartitions::staging(),
            bytes_freed: 2048000, // 2MB freed
            cleanup_reason: "Retention period expired".to_string(),
            cleaned_at: Utc::now(),
            cleaned_by: user_id,
        };
        
        assert_eq!(event.cleaned_content_cids.len(), 2);
        assert_eq!(event.bytes_freed, 2048000);
        assert_eq!(event.event_type(), "DocumentStagingContentCleaned");
    }
}