//! Domain verification tests to ensure all components are properly implemented

use cim_domain::{AggregateRoot, EntityId};
use cim_domain_document::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Test that verifies all major commands can be created
#[test]
fn test_all_commands_can_be_created() {
    let doc_id = DocumentId::new();
    let user_id = Uuid::new_v4();

    // Test CreateDocument command
    let _create_cmd = commands::CreateDocument {
        document_id: doc_id.clone(),
        document_type: DocumentType::Report,
        title: "Test Document".to_string(),
        author_id: user_id,
        metadata: HashMap::new(),
    };

    // Test UpdateContent command
    let _update_cmd = commands::UpdateContent {
        document_id: doc_id.clone(),
        content_blocks: vec![ContentBlock {
            id: "block1".to_string(),
            block_type: "paragraph".to_string(),
            title: Some("Introduction".to_string()),
            content: "This is the content".to_string(),
            metadata: HashMap::new(),
        }],
        change_summary: "Added introduction".to_string(),
        updated_by: user_id,
    };

    // Test ShareDocument command
    let _share_cmd = commands::ShareDocument {
        document_id: doc_id.clone(),
        share_with: user_id,
        access_level: AccessLevel::Read,
        shared_by: user_id,
    };

    // Test ChangeState command
    let _state_cmd = commands::ChangeState {
        document_id: doc_id.clone(),
        new_state: DocumentState::InReview,
        reason: "Ready for review".to_string(),
        changed_by: user_id,
    };

    // Test ArchiveDocument command
    let _archive_cmd = commands::ArchiveDocument {
        document_id: user_id,
        reason: "No longer needed".to_string(),
        retention_days: Some(365),
        archived_by: user_id,
    };
}

/// Test that verifies all events can be created
#[test]
fn test_all_events_can_be_created() {
    let doc_id = DocumentId::new();
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Test DocumentCreated event
    let _created_event = events::DocumentCreated {
        document_id: doc_id.clone(),
        document_type: DocumentType::Report,
        title: "Test Document".to_string(),
        author_id: user_id,
        metadata: HashMap::new(),
        created_at: now,
    };

    // Test ContentUpdated event
    let _updated_event = events::ContentUpdated {
        document_id: doc_id.clone(),
        content_blocks: vec![],
        change_summary: "Updated content".to_string(),
        updated_by: user_id,
        updated_at: now,
    };

    // Test StateChanged event
    let _state_event = events::StateChanged {
        document_id: doc_id.clone(),
        old_state: DocumentState::Draft,
        new_state: DocumentState::InReview,
        reason: "Ready for review".to_string(),
        changed_by: user_id,
        changed_at: now,
    };

    // Test DocumentShared event
    let _shared_event = events::DocumentShared {
        document_id: doc_id.clone(),
        shared_with: std::collections::HashSet::new(),
        permissions: vec!["read".to_string()],
        shared_by: user_id.to_string(),
        shared_at: now,
    };
}

/// Test document aggregate creation and basic operations
#[test]
fn test_document_aggregate_operations() {
    let doc_id = EntityId::<DocumentMarker>::new();
    let info = DocumentInfoComponent {
        title: "Test Document".to_string(),
        description: Some("A test document".to_string()),
        mime_type: "text/plain".to_string(),
        filename: Some("test.txt".to_string()),
        size_bytes: 1024,
        language: Some("en".to_string()),
    };

    let content_cid =
        cid::Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();

    // Create document
    let document = Document::new(doc_id, info.clone(), content_cid);

    // Verify basic properties
    assert_eq!(document.id(), doc_id);
    assert_eq!(document.version(), 0);

    // Verify components
    let doc_info = document.get_component::<DocumentInfoComponent>().unwrap();
    assert_eq!(doc_info.title, "Test Document");
    assert_eq!(doc_info.mime_type, "text/plain");

    let content_addr = document.get_component::<ContentAddressComponent>().unwrap();
    assert_eq!(content_addr.content_cid, content_cid);
}

/// Test value objects
#[test]
fn test_value_objects() {
    // Test DocumentId
    let _doc_id = DocumentId::new();
    let uuid = Uuid::new_v4();
    let doc_id_from_uuid = DocumentId::from(uuid);
    assert_eq!(doc_id_from_uuid.as_uuid(), &uuid);

    // Test DocumentVersion
    let version = DocumentVersion::new(1, 2, 3);
    assert_eq!(version.to_string(), "1.2.3");

    // Test AccessLevel
    let _read_access = AccessLevel::Read;
    let _write_access = AccessLevel::Write;
    let _admin_access = AccessLevel::Admin;

    // Test DocumentState
    let _draft_state = DocumentState::Draft;
    let _approved_state = DocumentState::Approved;

    // Test ContentBlock
    let block = ContentBlock {
        id: "block1".to_string(),
        block_type: "paragraph".to_string(),
        title: Some("Title".to_string()),
        content: "Content".to_string(),
        metadata: HashMap::new(),
    };
    assert_eq!(block.id, "block1");
}

/// Test services
#[test]
fn test_document_services() {
    use cim_domain_document::services::{ImportExportService, TemplateService};

    // Test template service
    let mut template_service = TemplateService::new();
    let template = DocumentTemplate {
        id: TemplateId::new(),
        name: "Test Template".to_string(),
        description: None,
        content: "Hello {{name}}!".to_string(),
        required_variables: vec![TemplateVariable {
            name: "name".to_string(),
            description: None,
            var_type: VariableType::Text,
            default_value: Some("World".to_string()),
            required: false,
        }],
        category: "test".to_string(),
        version: DocumentVersion::new(1, 0, 0),
    };

    template_service
        .register_template(template.clone())
        .unwrap();

    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "Alice".to_string());

    let result = template_service
        .apply_template(&template.id, &vars)
        .unwrap();
    assert_eq!(result, "Hello Alice!");

    // Test import/export service
    let markdown = "# Test\n\nThis is a test.";
    let imported = ImportExportService::import_document(
        markdown.as_bytes(),
        &ImportFormat::Markdown,
        &ImportOptions::default(),
    )
    .unwrap();

    assert_eq!(imported.title, "Test");
    assert!(imported.content.contains("This is a test"));
}

/// Test projections
#[test]
fn test_document_projections() {
    let user_id = Uuid::new_v4();

    // Test DocumentView
    let view = projections::DocumentView {
        document_id: user_id,
        title: "Test Document".to_string(),
        mime_type: "text/plain".to_string(),
        status: "draft".to_string(),
        owner_name: Some("Test User".to_string()),
        size_bytes: 1024,
        created_at: chrono::Utc::now().to_string(),
        tags: vec!["test".to_string()],
    };

    assert_eq!(view.title, "Test Document");
    assert_eq!(view.status, "draft");

    // Test DocumentFullView
    let doc_id = DocumentId::new();
    let full_view = projections::DocumentFullView {
        id: doc_id.clone(),
        title: "Test Document".to_string(),
        content: "This is the content".to_string(),
        version: DocumentVersion::new(1, 0, 0),
        doc_type: DocumentType::Report,
        tags: vec!["test".to_string()],
        author: user_id,
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    assert_eq!(full_view.title, "Test Document");
    assert_eq!(full_view.version.to_string(), "1.0.0");
}

/// Test query handlers
#[test]
fn test_query_handlers() {
    use cim_domain_document::queries::*;

    let doc_id = DocumentId::new();

    // Test SearchDocuments query
    let _search_query = SearchDocuments {
        query: "test".to_string(),
        tags: vec![],
        mime_types: vec![],
        limit: Some(10),
    };

    // Test GetDocument query
    let _get_query = GetDocument {
        document_id: doc_id.clone(),
        include_content: true,
        include_metadata: true,
    };

    // Test GetDocumentHistory query
    let _history_query = GetDocumentHistory {
        document_id: doc_id,
        include_content_changes: true,
    };

    // Mermaid graph showing test coverage:
    // ```mermaid
    // graph TD
    //     A[Document Domain Tests] --> B[Commands]
    //     A --> C[Events]
    //     A --> D[Aggregates]
    //     A --> E[Value Objects]
    //     A --> F[Services]
    //     A --> G[Projections]
    //     A --> H[Queries]
    //
    //     B --> B1[CreateDocument]
    //     B --> B2[UpdateContent]
    //     B --> B3[ShareDocument]
    //     B --> B4[ChangeState]
    //     B --> B5[ArchiveDocument]
    //
    //     C --> C1[DocumentCreated]
    //     C --> C2[ContentUpdated]
    //     C --> C3[StateChanged]
    //     C --> C4[DocumentShared]
    //
    //     D --> D1[Document Aggregate]
    //     D --> D2[Components]
    //
    //     E --> E1[DocumentId]
    //     E --> E2[DocumentVersion]
    //     E --> E3[AccessLevel]
    //     E --> E4[ContentBlock]
    //
    //     F --> F1[TemplateService]
    //     F --> F2[ImportExportService]
    //
    //     G --> G1[DocumentView]
    //
    //     H --> H1[SearchDocuments]
    //     H --> H2[GetDocument]
    //     H --> H3[GetDocumentHistory]
    // ```
}
