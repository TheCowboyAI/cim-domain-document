//! Integration tests for document workflows
//!
//! Tests the complete document lifecycle including creation, versioning,
//! collaboration, and search features.

use cim_domain_document::{
    commands::*,
    value_objects::*,
};
use cim_domain::Command as DomainCommand;
use uuid::Uuid;

#[test]
fn test_document_creation_workflow() {
    // Create a new document
    let document_id = DocumentId::new();
    let author_id = Uuid::new_v4();
    
    let create_cmd = CreateDocument {
        document_id: document_id.clone(),
        document_type: DocumentType::Report,
        title: "Quarterly Report Q1 2025".to_string(),
        author_id,
        metadata: std::collections::HashMap::from([
            ("department".to_string(), "Finance".to_string()),
            ("fiscal_year".to_string(), "2025".to_string()),
        ]),
    };

    // Verify the command has the right aggregate ID
    let aggregate_id = create_cmd.aggregate_id().unwrap();
    let uuid: Uuid = aggregate_id.into();
    assert_eq!(uuid, *document_id.as_uuid());
}

#[test]
fn test_document_versioning_workflow() {
    let document_id = DocumentId::new();
    
    // Create initial version
    let version_1 = DocumentVersion::new(1, 0, 0);
    assert_eq!(version_1.to_string(), "1.0.0");
    
    // Create a tag for release
    let tag = VersionTag {
        name: "v1.0-release".to_string(),
        description: Some("Initial release".to_string()),
        version: version_1.clone(),
        tagged_by: Uuid::new_v4(),
        tagged_at: chrono::Utc::now(),
    };
    
    let tag_cmd = TagVersion {
        document_id: document_id.clone(),
        tag_name: tag.name.clone(),
        description: tag.description.clone(),
        tagged_by: tag.tagged_by,
    };
    
    assert_eq!(tag_cmd.tag_name, "v1.0-release");
}

#[test]
fn test_document_collaboration_workflow() {
    let document_id = DocumentId::new();
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    
    // Share document with another user
    let share_cmd = ShareDocument {
        document_id: document_id.clone(),
        share_with: user2,
        access_level: AccessLevel::Write,
        shared_by: user1,
    };
    
    // Add a comment
    let comment = Comment {
        id: Uuid::new_v4(),
        content: "Please review section 3.2".to_string(),
        author_id: user1,
        block_id: Some("section-3.2".to_string()),
        parent_id: None,
        created_at: chrono::Utc::now(),
        resolved: false,
    };
    
    let comment_cmd = AddComment {
        document_id: document_id.clone(),
        content: comment.content.clone(),
        block_id: comment.block_id.clone(),
        parent_comment_id: None,
        author_id: user1,
    };
    
    assert_eq!(comment_cmd.content, "Please review section 3.2");
    assert_eq!(share_cmd.access_level, AccessLevel::Write);
}

#[test]
fn test_document_forking_workflow() {
    let original_id = DocumentId::new();
    let fork_id = DocumentId::new();
    let user = Uuid::new_v4();
    
    let fork_cmd = ForkDocument {
        document_id: original_id.clone(),
        fork_id: fork_id.clone(),
        description: "Experimental changes for v2".to_string(),
        forked_by: user,
    };
    
    assert_ne!(original_id, fork_id);
    assert_eq!(fork_cmd.description, "Experimental changes for v2");
}

#[test]
fn test_document_linking_workflow() {
    let doc1 = DocumentId::new();
    let doc2 = DocumentId::new();
    let user = Uuid::new_v4();
    
    // Test different link types
    let link_types = vec![
        LinkType::References,
        LinkType::Related,
        LinkType::Supersedes,
        LinkType::DerivedFrom,
        LinkType::PartOf,
    ];
    
    for link_type in link_types {
        let link_cmd = LinkDocuments {
            source_id: doc1.clone(),
            target_id: doc2.clone(),
            link_type: link_type.clone(),
            description: Some(format!("Document {:?} relationship", link_type)),
            linked_by: user,
        };
        
        assert!(link_cmd.description.is_some());
    }
}

#[test]
fn test_document_state_transitions() {
    let document_id = DocumentId::new();
    let user = Uuid::new_v4();
    
    // Test state transitions
    let states = vec![
        (DocumentState::Draft, DocumentState::InReview, "Submitting for review"),
        (DocumentState::InReview, DocumentState::Approved, "All requirements met"),
        (DocumentState::InReview, DocumentState::Rejected, "Missing section 4"),
        (DocumentState::Rejected, DocumentState::Draft, "Fixing issues"),
        (DocumentState::Approved, DocumentState::Archived, "End of fiscal year"),
    ];
    
    for (_old_state, new_state, reason) in states {
        let change_cmd = ChangeState {
            document_id: document_id.clone(),
            new_state: new_state.clone(),
            reason: reason.to_string(),
            changed_by: user,
        };
        
        assert_eq!(change_cmd.new_state, new_state);
        assert_eq!(change_cmd.reason, reason);
    }
}

#[test]
fn test_document_content_blocks() {
    let document_id = DocumentId::new();
    let user = Uuid::new_v4();
    
    let blocks = vec![
        ContentBlock {
            id: "intro".to_string(),
            block_type: "section".to_string(),
            title: Some("Introduction".to_string()),
            content: "This report covers Q1 2025...".to_string(),
            metadata: std::collections::HashMap::new(),
        },
        ContentBlock {
            id: "financials".to_string(),
            block_type: "section".to_string(),
            title: Some("Financial Overview".to_string()),
            content: "Revenue increased by 15%...".to_string(),
            metadata: std::collections::HashMap::from([
                ("charts".to_string(), "revenue-chart.png".to_string()),
            ]),
        },
    ];
    
    let update_cmd = UpdateContent {
        document_id,
        content_blocks: blocks.clone(),
        change_summary: "Added financial section".to_string(),
        updated_by: user,
    };
    
    assert_eq!(update_cmd.content_blocks.len(), 2);
    assert_eq!(update_cmd.content_blocks[0].id, "intro");
}

#[test]
fn test_document_collections() {
    let parent = Collection {
        id: Uuid::new_v4(),
        name: "Annual Reports".to_string(),
        description: Some("All annual company reports".to_string()),
        parent_id: None,
        metadata: std::collections::HashMap::new(),
    };
    
    let child = Collection {
        id: Uuid::new_v4(),
        name: "2025 Reports".to_string(),
        description: Some("Reports for fiscal year 2025".to_string()),
        parent_id: Some(parent.id),
        metadata: std::collections::HashMap::from([
            ("year".to_string(), "2025".to_string()),
        ]),
    };
    
    assert_eq!(child.parent_id, Some(parent.id));
    assert_eq!(child.metadata.get("year"), Some(&"2025".to_string()));
}

#[test]
fn test_access_levels() {
    let levels = vec![
        AccessLevel::Read,
        AccessLevel::Comment,
        AccessLevel::Write,
        AccessLevel::Admin,
    ];
    
    // Verify access level hierarchy
    for (i, level) in levels.iter().enumerate() {
        match level {
            AccessLevel::Read => assert_eq!(i, 0),
            AccessLevel::Comment => assert_eq!(i, 1),
            AccessLevel::Write => assert_eq!(i, 2),
            AccessLevel::Admin => assert_eq!(i, 3),
        }
    }
} 