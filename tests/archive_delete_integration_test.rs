//! Integration tests for document archive functionality

use cim_domain_document::commands::*;
use cim_domain_document::value_objects::*;
use cim_domain_document::events::*;
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn test_document_lifecycle() {
    // Create a document
    let document_id = DocumentId::new();
    let create_cmd = CreateDocument {
        document_id: document_id.clone(),
        title: "Important Document".to_string(),
        document_type: DocumentType::Report,
        author_id: Uuid::new_v4(),
        metadata: HashMap::new(),
    };

    // Archive the document
    let archive_cmd = ArchiveDocument {
        document_id: *document_id.as_uuid(),
        reason: "End of year archival".to_string(),
        retention_days: Some(2555), // 7 years
        archived_by: Uuid::new_v4(),
    };

    // Verify archive command
    assert_eq!(archive_cmd.retention_days, Some(2555));
    assert_eq!(archive_cmd.reason, "End of year archival");
}

#[tokio::test]
async fn test_document_deleted_event() {
    let document_id = DocumentId::new();
    let user_id = Uuid::new_v4();

    // Test the deleted event structure
    let event = DocumentDeleted {
        document_id: document_id.clone(),
        hard_delete: false,
        reason: Some("No longer needed".to_string()),
        deleted_by: user_id,
        deleted_at: chrono::Utc::now(),
    };

    assert!(!event.hard_delete);
    assert!(event.reason.is_some());
}

#[tokio::test]
async fn test_archive_metadata() {
    let document_id = DocumentId::new();
    
    // Archive with retention period
    let archive_cmd = ArchiveDocument {
        document_id: *document_id.as_uuid(),
        reason: "Compliance requirement".to_string(),
        retention_days: Some(3650), // 10 years for compliance
        archived_by: Uuid::new_v4(),
    };

    assert_eq!(archive_cmd.retention_days, Some(3650));
    assert_eq!(archive_cmd.reason, "Compliance requirement");
}

#[tokio::test]
async fn test_document_restored_event() {
    let document_id = DocumentId::new();
    
    // Test the restored event structure
    let event = DocumentRestored {
        document_id: document_id.clone(),
        restored_from: RestorationSource::Archive,
        restored_by: Uuid::new_v4(),
        restored_at: chrono::Utc::now(),
        reason: Some("Needed for compliance audit".to_string()),
    };

    assert!(matches!(event.restored_from, RestorationSource::Archive));
    assert!(event.reason.is_some());
}

#[tokio::test]
async fn test_archive_delete_workflow() {
    let document_id = DocumentId::new();
    let admin_id = Uuid::new_v4();

    // Step 1: Archive document
    let archive_cmd = ArchiveDocument {
        document_id: *document_id.as_uuid(),
        reason: "Quarterly cleanup".to_string(),
        retention_days: Some(90), // 3 months
        archived_by: admin_id,
    };

    // Step 2: After retention period, create deletion event
    let delete_event = DocumentDeleted {
        document_id: document_id.clone(),
        hard_delete: true,
        reason: Some("Retention period expired".to_string()),
        deleted_by: admin_id,
        deleted_at: chrono::Utc::now(),
    };

    // Verify workflow
    assert_eq!(archive_cmd.reason, "Quarterly cleanup");
    assert_eq!(delete_event.reason, Some("Retention period expired".to_string()));
} 