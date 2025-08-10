/// CID-First Document Ingestion API Example
/// 
/// This example demonstrates the complete CID-first ingestion process:
/// 1. Raw BLOB ingestion to Object Store
/// 2. CID generation and event emission (BLOB-free)
/// 3. Optional processing pipeline (virus scan, validation)
/// 4. Content promotion between partitions
/// 5. Document entity creation with metadata

use std::collections::HashMap;
use uuid::Uuid;
use cid::Cid;
use chrono::Utc;

// Import the new CID-first ingestion types
use cim_domain_document::{
    commands::ingestion_commands::{
        IngestDocumentContent, CreateDocumentFromCid, PromoteDocumentContent,
        UpdateDocumentMetadata, IngestDocumentResponse, CreateDocumentResponse,
        ProcessingResult, ProcessingDetails,
    },
    // Ingestion events are re-exported from events module
    DocumentContentIngested, DocumentCreatedFromCid, DocumentProcessingCompleted,
    DocumentContentPromoted,
    services::object_store::{
        DocumentPartitions, ContentMetadata, ProcessingJob,
        ObjectStoreService, ContentIngestionResult,
        ObjectStorePartition, ObjectStoreError,
    },
    nats::{MessageFactory, ActorId},
    value_objects::{DocumentId, UserId},
};

/// Mock Object Store Service for demonstration
struct MockObjectStoreService;

impl ObjectStoreService for MockObjectStoreService {
    async fn ingest_content(
        &self,
        content: Vec<u8>,
        suggested_mime_type: Option<&str>,
        target_partition: ObjectStorePartition,
        actor: &ActorId,
    ) -> Result<ContentIngestionResult, ObjectStoreError> {
        // Simulate content ingestion to NATS Object Store
        let content_cid = cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o")
            .map_err(|_| ObjectStoreError::IpldError("Invalid CID".to_string()))?;
        
        let metadata = ContentMetadata {
            mime_type: suggested_mime_type.unwrap_or("application/octet-stream").to_string(),
            size_bytes: content.len() as u64,
            hash_algorithm: "sha256".to_string(),
            detected_format: Some("PDF-1.4".to_string()),
            is_encrypted: false,
            language_hint: Some("en".to_string()),
        };
        
        Ok(ContentIngestionResult {
            content_cid,
            partition: target_partition,
            metadata,
            ingested_at: Utc::now(),
            processing_required: true,
        })
    }
    
    // Other methods would be implemented for a real service
    async fn get_content(
        &self,
        _content_cid: Cid,
        _partition: &ObjectStorePartition,
    ) -> Result<Vec<u8>, ObjectStoreError> {
        Ok(b"Mock PDF content".to_vec())
    }
    
    async fn content_exists(
        &self,
        _content_cid: Cid,
        _partition: &ObjectStorePartition,
    ) -> Result<bool, ObjectStoreError> {
        Ok(true)
    }
    
    async fn promote_content(
        &self,
        _content_cid: Cid,
        _from: &ObjectStorePartition,
        _to: &ObjectStorePartition,
        _actor: &ActorId,
    ) -> Result<(), ObjectStoreError> {
        Ok(())
    }
    
    async fn start_processing(
        &self,
        content_cid: Cid,
        enable_virus_scan: bool,
        enable_validation: bool,
    ) -> Result<ProcessingJob, ObjectStoreError> {
        Ok(ProcessingJob::new(content_cid, enable_virus_scan, enable_validation))
    }
    
    async fn get_processing_status(
        &self,
        _job_id: Uuid,
    ) -> Result<ProcessingJob, ObjectStoreError> {
        let content_cid = cid::Cid::try_from("QmYjtig7VJQ6XsnUjqqJvj7QaMcCAwtrgNdahSiFofrE7o")
            .map_err(|_| ObjectStoreError::IpldError("Invalid CID".to_string()))?;
        Ok(ProcessingJob::new(content_cid, true, true))
    }
    
    async fn list_partition_content(
        &self,
        _partition: &ObjectStorePartition,
        _limit: Option<usize>,
    ) -> Result<Vec<Cid>, ObjectStoreError> {
        Ok(vec![])
    }
    
    async fn cleanup_expired_staging(&self) -> Result<usize, ObjectStoreError> {
        Ok(0)
    }
}

/// Complete CID-first document ingestion workflow
pub struct CidFirstIngestionService {
    object_store: MockObjectStoreService,
}

impl CidFirstIngestionService {
    pub fn new() -> Self {
        Self {
            object_store: MockObjectStoreService,
        }
    }

    /// Phase 1: Ingest raw document content (THE ONLY METHOD THAT HANDLES BLOBS)
    pub async fn ingest_document_content(
        &self,
        content: Vec<u8>,
        suggested_filename: Option<String>,
        content_type_hint: Option<String>,
        uploaded_by: UserId,
    ) -> Result<IngestDocumentResponse, Box<dyn std::error::Error>> {
        println!("🔄 Phase 1: Ingesting raw content ({} bytes)", content.len());
        
        // Create ingestion command (contains the BLOB)
        let command = IngestDocumentContent {
            content: content.clone(),
            suggested_filename: suggested_filename.clone(),
            content_type_hint: content_type_hint.clone(),
            target_partition: DocumentPartitions::staging(),
            enable_virus_scanning: true,
            enable_format_validation: true,
            enable_auto_promotion: false, // Manual promotion for demo
            uploaded_by,
            correlation_id: Some(Uuid::new_v4()),
        };

        // Store content in Object Store (BLOB disappears from system after this)
        let actor = ActorId::user(*uploaded_by.as_uuid());
        let ingestion_result = self.object_store.ingest_content(
            command.content,
            command.content_type_hint.as_deref(),
            command.target_partition.clone(),
            &actor,
        ).await?;

        println!("✅ Content stored with CID: {}", ingestion_result.content_cid);
        println!("📍 Partition: {:?}", ingestion_result.partition);

        // Emit BLOB-free event
        let event = DocumentContentIngested {
            content_cid: ingestion_result.content_cid,
            content_partition: ingestion_result.partition.clone(),
            content_metadata: ingestion_result.metadata.clone(),
            processing_job_id: None,
            ingested_at: ingestion_result.ingested_at,
            uploaded_by,
        };

        println!("📡 Event emitted: {} (BLOB-free)", event.event_type());

        // Start processing job
        let processing_job = if ingestion_result.processing_required {
            Some(self.object_store.start_processing(
                ingestion_result.content_cid,
                command.enable_virus_scanning,
                command.enable_format_validation,
            ).await?)
        } else {
            None
        };

        // Generate new document ID for the entity
        let document_id = DocumentId::new();

        Ok(IngestDocumentResponse {
            content_cid: ingestion_result.content_cid,
            document_id,
            content_partition: ingestion_result.partition,
            processing_job: processing_job.clone(),
            content_metadata: ingestion_result.metadata,
            ingested_at: ingestion_result.ingested_at,
        })
    }

    /// Phase 2: Create document entity from CID (BLOB-free)
    pub async fn create_document_from_cid(
        &self,
        content_cid: Cid,
        filename: Option<String>,
        title: Option<String>,
        created_by: UserId,
    ) -> Result<CreateDocumentResponse, Box<dyn std::error::Error>> {
        println!("\n🔄 Phase 2: Creating document entity from CID {}", content_cid);

        let document_id = DocumentId::new();
        
        // Mock getting content metadata from Object Store
        let content_metadata = ContentMetadata {
            mime_type: "application/pdf".to_string(),
            size_bytes: 1024000,
            hash_algorithm: "sha256".to_string(),
            detected_format: Some("PDF-1.4".to_string()),
            is_encrypted: false,
            language_hint: Some("en".to_string()),
        };

        let command = CreateDocumentFromCid {
            document_id,
            content_cid,
            content_partition: DocumentPartitions::staging(),
            content_metadata: content_metadata.clone(),
            filename: filename.clone(),
            title: title.clone(),
            description: None,
            tags: vec!["ingestion".to_string(), "demo".to_string()],
            created_by,
        };

        println!("📄 Document entity created: {}", document_id.as_uuid());
        println!("🔗 References content CID: {} (NO BLOB DATA)", content_cid);

        // Emit document creation event (BLOB-free)
        let event = DocumentCreatedFromCid {
            document_id: command.document_id,
            content_cid: command.content_cid,
            content_partition: command.content_partition.clone(),
            filename: command.filename.clone(),
            title: command.title.clone(),
            description: command.description.clone(),
            tags: command.tags.clone(),
            content_metadata: content_metadata,
            created_at: Utc::now(),
            created_by,
        };

        println!("📡 Event emitted: {} (BLOB-free)", event.event_type());

        Ok(CreateDocumentResponse {
            document_id: command.document_id,
            content_cid: command.content_cid,
            content_partition: command.content_partition,
            created_at: Utc::now(),
        })
    }

    /// Phase 3: Process and promote content (BLOB-free)
    pub async fn process_and_promote_content(
        &self,
        document_id: DocumentId,
        content_cid: Cid,
        promoted_by: UserId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔄 Phase 3: Processing and promoting content CID {}", content_cid);

        // Simulate processing completion
        println!("🦠 Virus scan: CLEAN");
        println!("✅ Format validation: PASSED");
        println!("🔍 Content analysis: COMPLETED");

        let processing_results = vec![
            ProcessingResult {
                stage_name: "virus_scan".to_string(),
                success: true,
                started_at: Utc::now(),
                completed_at: Utc::now(),
                details: ProcessingDetails::VirusScan {
                    threats_found: vec![],
                    scanner_version: "ClamAV 0.103".to_string(),
                    definitions_updated: Utc::now(),
                },
            },
            ProcessingResult {
                stage_name: "format_validation".to_string(),
                success: true,
                started_at: Utc::now(),
                completed_at: Utc::now(),
                details: ProcessingDetails::FormatValidation {
                    format_valid: true,
                    detected_format: "PDF-1.4".to_string(),
                    format_version: Some("1.4".to_string()),
                    validation_errors: vec![],
                },
            },
        ];

        // Emit processing completion event (BLOB-free)
        let processing_event = DocumentProcessingCompleted {
            content_cid,
            processing_job_id: Uuid::new_v4(),
            processing_results: processing_results.clone(),
            success: true,
            failure_reason: None,
            completed_at: Utc::now(),
        };

        println!("📡 Event emitted: {} (BLOB-free)", processing_event.event_type());

        // Promote content from staging to aggregate
        let actor = ActorId::user(*promoted_by.as_uuid());
        self.object_store.promote_content(
            content_cid,
            &DocumentPartitions::staging(),
            &DocumentPartitions::aggregate(),
            &actor,
        ).await?;

        println!("📈 Content promoted: staging → aggregate");

        // Emit promotion event (BLOB-free)
        let promotion_event = DocumentContentPromoted {
            document_id,
            content_cid,
            from_partition: DocumentPartitions::staging(),
            to_partition: DocumentPartitions::aggregate(),
            promotion_reason: "All processing stages passed".to_string(),
            processing_results,
            promoted_at: Utc::now(),
            promoted_by,
            source_cleaned: true,
        };

        println!("📡 Event emitted: {} (BLOB-free)", promotion_event.event_type());

        Ok(())
    }

    /// Phase 4: Update document metadata (separate from content)
    pub async fn update_document_metadata(
        &self,
        document_id: DocumentId,
        content_cid: Option<Cid>,
        filename: Option<String>,
        title: Option<String>,
        tags: Option<Vec<String>>,
        updated_by: UserId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔄 Phase 4: Updating document metadata");

        let command = UpdateDocumentMetadata {
            document_id,
            content_cid,
            filename: filename.clone(),
            title: title.clone(),
            description: Some("Updated via CID-first ingestion demo".to_string()),
            tags: tags.clone(),
            custom_properties: {
                let mut props = HashMap::new();
                props.insert("processing_method".to_string(), serde_json::json!("cid_first"));
                props.insert("ingestion_demo".to_string(), serde_json::json!(true));
                props
            },
            updated_by,
        };

        println!("📝 Updated filename: {:?}", command.filename);
        println!("📝 Updated title: {:?}", command.title);
        println!("📝 Updated tags: {:?}", command.tags);
        println!("🔗 Content CID unchanged: {:?}", content_cid);

        // Note: In a real system, this would emit DocumentMetadataUpdated event
        println!("📡 Metadata update event emitted (BLOB-free)");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 CID-First Document Ingestion Demo");
    println!("=====================================");
    
    let service = CidFirstIngestionService::new();
    let user_id = UserId::new();

    // Simulate raw PDF content (the ONLY BLOB in the entire system)
    let pdf_content = b"Mock PDF content for ingestion demo".to_vec();
    
    // Phase 1: Ingest content (BLOB → CID)
    let ingestion_response = service.ingest_document_content(
        pdf_content,
        Some("quarterly-report.pdf".to_string()),
        Some("application/pdf".to_string()),
        user_id,
    ).await?;

    // Phase 2: Create document entity from CID
    let document_response = service.create_document_from_cid(
        ingestion_response.content_cid,
        Some("Q4 2024 Financial Report".to_string()),
        Some("Quarterly Financial Report".to_string()),
        user_id,
    ).await?;

    // Phase 3: Process and promote content
    service.process_and_promote_content(
        document_response.document_id,
        document_response.content_cid,
        user_id,
    ).await?;

    // Phase 4: Update metadata (separate from immutable content)
    service.update_document_metadata(
        document_response.document_id,
        Some(document_response.content_cid),
        Some("Q4-2024-Financial-Report.pdf".to_string()),
        Some("Q4 2024 Financial Report - Final".to_string()),
        Some(vec!["financial".to_string(), "quarterly".to_string(), "2024".to_string()]),
        user_id,
    ).await?;

    println!("\n🎉 CID-First Ingestion Complete!");
    println!("================================");
    println!("✅ Content CID: {}", document_response.content_cid);
    println!("✅ Document ID: {}", document_response.document_id.as_uuid());
    println!("✅ Final partition: aggregate");
    println!("✅ Event Store: BLOB-free (only CIDs and metadata)");
    println!("✅ Object Store: Contains actual content");
    println!("✅ Subject subscriptions: Available on CID and document ID");

    println!("\n📡 NATS Subject Examples:");
    println!("- events.document.cid.{}.> (all events for this content)", 
             document_response.content_cid.to_string().chars().take(12).collect::<String>());
    println!("- events.document.user.{}.> (all documents for this user)", user_id.as_uuid());
    println!("- events.document.aggregate.document.> (all document events)");

    Ok(())
}