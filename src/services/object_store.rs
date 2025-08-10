//! Object Store Service for CID-First Document Ingestion
//!
//! This service provides domain-partitioned content storage using IPLD and CIDs.
//! It ensures BLOBs never transit through the Event Store and enables content-addressed
//! subscriptions via the Subject Algebra.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use chrono::{DateTime, Utc};
use cid::Cid;
use uuid::Uuid;

use crate::value_objects::DocumentId;
use crate::nats::{MessageIdentity, ActorId};

/// Object Store partitions aligned by Domain and Aggregate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectStorePartition {
    /// Untrusted content awaiting processing
    Staging {
        domain: String,
        retention_hours: u64,
    },
    /// Processed content ready for aggregate use
    Aggregate {
        domain: String,
        aggregate_type: String,
    },
    /// Archived content for compliance
    Archive {
        domain: String,
        compliance_class: String,
        retention_years: u32,
    },
}

impl ObjectStorePartition {
    /// Get the partition identifier for NATS Object Store
    pub fn bucket_name(&self) -> String {
        match self {
            Self::Staging { domain, .. } => format!("{}-staging", domain),
            Self::Aggregate { domain, aggregate_type } => format!("{}-{}-aggregate", domain, aggregate_type),
            Self::Archive { domain, compliance_class, .. } => format!("{}-{}-archive", domain, compliance_class),
        }
    }

    /// Check if content can be promoted from this partition
    pub fn allows_promotion(&self) -> bool {
        matches!(self, Self::Staging { .. })
    }

    /// Get the next partition in the processing pipeline
    pub fn next_partition(&self, aggregate_type: &str) -> Option<ObjectStorePartition> {
        match self {
            Self::Staging { domain, .. } => Some(Self::Aggregate {
                domain: domain.clone(),
                aggregate_type: aggregate_type.to_string(),
            }),
            Self::Aggregate { domain, .. } => Some(Self::Archive {
                domain: domain.clone(),
                compliance_class: "standard".to_string(),
                retention_years: 7,
            }),
            Self::Archive { .. } => None, // Terminal state
        }
    }
}

/// Content metadata detected during ingestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub mime_type: String,
    pub size_bytes: u64,
    pub hash_algorithm: String,
    pub detected_format: Option<String>,
    pub is_encrypted: bool,
    pub language_hint: Option<String>,
}

/// Content ingestion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentIngestionResult {
    pub content_cid: Cid,
    pub partition: ObjectStorePartition,
    pub metadata: ContentMetadata,
    pub ingested_at: DateTime<Utc>,
    pub processing_required: bool,
}

/// Processing job for async content validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJob {
    pub job_id: Uuid,
    pub content_cid: Cid,
    pub stages: Vec<ProcessingStage>,
    pub current_stage: usize,
    pub created_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Individual processing stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStage {
    pub name: String,
    pub required: bool,
    pub timeout: Duration,
    pub retry_count: u32,
}

impl ProcessingJob {
    /// Create a new processing job for content
    pub fn new(content_cid: Cid, enable_virus_scan: bool, enable_validation: bool) -> Self {
        let mut stages = Vec::new();
        
        if enable_virus_scan {
            stages.push(ProcessingStage {
                name: "virus_scan".to_string(),
                required: true,
                timeout: Duration::from_secs(300), // 5 minutes
                retry_count: 2,
            });
        }
        
        if enable_validation {
            stages.push(ProcessingStage {
                name: "format_validation".to_string(),
                required: false,
                timeout: Duration::from_secs(60),
                retry_count: 1,
            });
        }
        
        stages.push(ProcessingStage {
            name: "content_promotion".to_string(),
            required: true,
            timeout: Duration::from_secs(30),
            retry_count: 0,
        });

        Self {
            job_id: Uuid::new_v4(),
            content_cid,
            stages,
            current_stage: 0,
            created_at: Utc::now(),
            estimated_completion: Some(Utc::now() + Duration::from_secs(600)), // 10 minutes
        }
    }
}

/// Object Store service errors
#[derive(Debug, thiserror::Error)]
pub enum ObjectStoreError {
    #[error("Content not found: {content_cid}")]
    ContentNotFound { content_cid: Cid },
    
    #[error("Partition access denied: {partition:?}")]
    PartitionAccessDenied { partition: ObjectStorePartition },
    
    #[error("Content promotion failed: {reason}")]
    PromotionFailed { reason: String },
    
    #[error("Processing timeout: {job_id}")]
    ProcessingTimeout { job_id: Uuid },
    
    #[error("Invalid content format: {format}")]
    InvalidContentFormat { format: String },
    
    #[error("Storage capacity exceeded")]
    StorageCapacityExceeded,
    
    #[error("IPLD encoding error: {0}")]
    IpldError(String),
    
    #[error("NATS Object Store error: {0}")]
    NatsError(String),
}

/// Object Store service for domain-partitioned content storage
pub trait ObjectStoreService: Send + Sync {
    /// Ingest raw content into staging partition
    async fn ingest_content(
        &self,
        content: Vec<u8>,
        suggested_mime_type: Option<&str>,
        target_partition: ObjectStorePartition,
        actor: &ActorId,
    ) -> Result<ContentIngestionResult, ObjectStoreError>;

    /// Get content by CID from specific partition
    async fn get_content(
        &self,
        content_cid: Cid,
        partition: &ObjectStorePartition,
    ) -> Result<Vec<u8>, ObjectStoreError>;

    /// Check if content exists in partition
    async fn content_exists(
        &self,
        content_cid: Cid,
        partition: &ObjectStorePartition,
    ) -> Result<bool, ObjectStoreError>;

    /// Promote content from one partition to another
    async fn promote_content(
        &self,
        content_cid: Cid,
        from: &ObjectStorePartition,
        to: &ObjectStorePartition,
        actor: &ActorId,
    ) -> Result<(), ObjectStoreError>;

    /// Start processing job for content
    async fn start_processing(
        &self,
        content_cid: Cid,
        enable_virus_scan: bool,
        enable_validation: bool,
    ) -> Result<ProcessingJob, ObjectStoreError>;

    /// Get processing job status
    async fn get_processing_status(
        &self,
        job_id: Uuid,
    ) -> Result<ProcessingJob, ObjectStoreError>;

    /// List content in partition (for cleanup/monitoring)
    async fn list_partition_content(
        &self,
        partition: &ObjectStorePartition,
        limit: Option<usize>,
    ) -> Result<Vec<Cid>, ObjectStoreError>;

    /// Cleanup expired staging content
    async fn cleanup_expired_staging(&self) -> Result<usize, ObjectStoreError>;
}

/// Default partitions for the Document domain
pub struct DocumentPartitions;

impl DocumentPartitions {
    pub fn staging() -> ObjectStorePartition {
        ObjectStorePartition::Staging {
            domain: "document".to_string(),
            retention_hours: 48, // 2 days retention
        }
    }

    pub fn aggregate() -> ObjectStorePartition {
        ObjectStorePartition::Aggregate {
            domain: "document".to_string(),
            aggregate_type: "document".to_string(),
        }
    }

    pub fn archive(compliance_class: &str, retention_years: u32) -> ObjectStorePartition {
        ObjectStorePartition::Archive {
            domain: "document".to_string(),
            compliance_class: compliance_class.to_string(),
            retention_years,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_bucket_names() {
        let staging = DocumentPartitions::staging();
        assert_eq!(staging.bucket_name(), "document-staging");

        let aggregate = DocumentPartitions::aggregate();
        assert_eq!(aggregate.bucket_name(), "document-document-aggregate");

        let archive = DocumentPartitions::archive("confidential", 10);
        assert_eq!(archive.bucket_name(), "document-confidential-archive");
    }

    #[test]
    fn test_partition_promotion_flow() {
        let staging = DocumentPartitions::staging();
        assert!(staging.allows_promotion());

        let aggregate = staging.next_partition("document").unwrap();
        assert_eq!(aggregate.bucket_name(), "document-document-aggregate");

        let archive = aggregate.next_partition("document").unwrap();
        assert!(matches!(archive, ObjectStorePartition::Archive { .. }));

        assert!(archive.next_partition("document").is_none()); // Terminal
    }

    #[test]
    fn test_processing_job_creation() {
        let content_cid = cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o").unwrap();
        let job = ProcessingJob::new(content_cid, true, true);

        assert_eq!(job.content_cid, content_cid);
        assert_eq!(job.current_stage, 0);
        assert_eq!(job.stages.len(), 3); // virus_scan + validation + promotion
        assert_eq!(job.stages[0].name, "virus_scan");
        assert_eq!(job.stages[1].name, "format_validation");
        assert_eq!(job.stages[2].name, "content_promotion");
    }

    #[test]
    fn test_content_metadata() {
        let metadata = ContentMetadata {
            mime_type: "application/pdf".to_string(),
            size_bytes: 1024000,
            hash_algorithm: "sha256".to_string(),
            detected_format: Some("PDF-1.4".to_string()),
            is_encrypted: false,
            language_hint: Some("en".to_string()),
        };

        assert_eq!(metadata.mime_type, "application/pdf");
        assert!(!metadata.is_encrypted);
        assert_eq!(metadata.size_bytes, 1024000);
    }
}