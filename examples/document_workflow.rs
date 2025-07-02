//! Document Workflow Example
//!
//! This example demonstrates:
//! - Creating a document
//! - Adding content and metadata
//! - Versioning documents
//! - Sharing and collaboration
//! - Document workflow states

use cim_domain_document::{
    aggregate::Document,
    commands::{ChangeState, CreateDocument, ShareDocument, UpdateContent},
    events::{ContentUpdated, DocumentCreated, DocumentShared, StateChanged},
    handlers::DocumentCommandHandler,
    queries::{DocumentQueryHandler, GetDocument, GetDocumentHistory},
    value_objects::{AccessLevel, ContentBlock, DocumentId, DocumentState, DocumentType},
};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CIM Document Domain Example ===\n");

    // Initialize handlers
    let command_handler = DocumentCommandHandler::new();
    let query_handler = DocumentQueryHandler::new();

    let document_id = DocumentId::new();
    let author_id = Uuid::new_v4();
    let reviewer_id = Uuid::new_v4();

    // Step 1: Create a new document
    println!("1. Creating document...");
    let create_command = CreateDocument {
        document_id: document_id.clone(),
        document_type: DocumentType::Proposal,
        title: "Project Proposal: AI-Powered Workflow System".to_string(),
        author_id,
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("department".to_string(), "Engineering".to_string());
            meta.insert("priority".to_string(), "High".to_string());
            meta
        },
    };

    let events = command_handler.handle(create_command).await?;
    println!("   Document created! Events: {:?}\n", events.len());

    // Step 2: Add initial content
    println!("2. Adding content sections...");
    let executive_summary = UpdateContent {
        document_id: document_id.clone(),
        content_block: ContentBlock {
            id: Uuid::new_v4(),
            section: "executive_summary".to_string(),
            content: "This proposal outlines the development of an AI-powered workflow system that will automate and optimize business processes across departments.".to_string(),
            format: "markdown".to_string(),
            version: 1,
        },
        author_id,
    };

    let events = command_handler.handle(executive_summary).await?;
    println!("   Executive summary added! Events: {:?}", events.len());

    let technical_details = UpdateContent {
        document_id: document_id.clone(),
        content_block: ContentBlock {
            id: Uuid::new_v4(),
            section: "technical_details".to_string(),
            content: r#"## Technical Architecture

### Core Components:
1. **Event-Driven Engine**: NATS-based messaging for real-time processing
2. **AI Integration**: LLM-powered decision making and automation
3. **Visual Workflow Designer**: Drag-and-drop interface for process creation
4. **Analytics Dashboard**: Real-time metrics and insights

### Technology Stack:
- Backend: Rust with Bevy ECS
- Messaging: NATS JetStream
- AI: OpenAI/Anthropic APIs
- Frontend: React with TypeScript
- Database: PostgreSQL with event sourcing"#
                .to_string(),
            format: "markdown".to_string(),
            version: 1,
        },
        author_id,
    };

    let events = command_handler.handle(technical_details).await?;
    println!("   Technical details added! Events: {:?}\n", events.len());

    // Step 3: Change document state to review
    println!("3. Submitting for review...");
    let submit_for_review = ChangeState {
        document_id: document_id.clone(),
        new_state: DocumentState::InReview,
        reason: Some("Initial draft complete, ready for technical review".to_string()),
        actor_id: author_id,
    };

    let events = command_handler.handle(submit_for_review).await?;
    println!("   Submitted for review! Events: {:?}\n", events.len());

    // Step 4: Share with reviewer
    println!("4. Sharing with reviewer...");
    let share_command = ShareDocument {
        document_id: document_id.clone(),
        user_id: reviewer_id,
        access_level: AccessLevel::Reviewer,
        expires_at: None,
    };

    let events = command_handler.handle(share_command).await?;
    println!("   Shared with reviewer! Events: {:?}\n", events.len());

    // Step 5: Reviewer adds comments (as content update)
    println!("5. Reviewer adding feedback...");
    let review_feedback = UpdateContent {
        document_id: document_id.clone(),
        content_block: ContentBlock {
            id: Uuid::new_v4(),
            section: "review_comments".to_string(),
            content: r#"## Review Comments

**Overall Assessment**: Strong proposal with clear technical vision.

### Suggestions:
1. Add cost-benefit analysis section
2. Include timeline with milestones
3. Detail integration with existing systems
4. Add risk assessment section

**Recommendation**: Approve with minor revisions"#
                .to_string(),
            format: "markdown".to_string(),
            version: 1,
        },
        author_id: reviewer_id,
    };

    let events = command_handler.handle(review_feedback).await?;
    println!("   Review feedback added! Events: {:?}\n", events.len());

    // Step 6: Approve document
    println!("6. Approving document...");
    let approve_command = ChangeState {
        document_id: document_id.clone(),
        new_state: DocumentState::Approved,
        reason: Some("Approved pending minor revisions as noted".to_string()),
        actor_id: reviewer_id,
    };

    let events = command_handler.handle(approve_command).await?;
    println!("   Document approved! Events: {:?}\n", events.len());

    // Step 7: Query document and history
    println!("7. Retrieving document details...");
    let get_document = GetDocument {
        document_id: document_id.clone(),
        include_content: true,
        include_metadata: true,
    };

    let document = query_handler.handle(&get_document).await?;
    println!("   Document: {}", document.title);
    println!("   State: {:?}", document.state);
    println!("   Sections: {}", document.content_blocks.len());
    println!("   Shared with: {} users\n", document.access_list.len());

    // Step 8: Get document history
    println!("8. Retrieving document history...");
    let get_history = GetDocumentHistory {
        document_id: document_id.clone(),
        include_content_changes: true,
    };

    let history = query_handler.handle(&get_history).await?;
    println!("   Document has {} historical events", history.events.len());

    for (idx, event) in history.events.iter().enumerate() {
        println!(
            "   Event {}: {} at {:?}",
            idx + 1,
            event.event_type(),
            event.timestamp()
        );
    }

    // Step 9: Generate document summary
    println!("\n9. Document Summary:");
    println!("   ==================");
    println!("   Title: {}", document.title);
    println!("   Type: {:?}", document.document_type);
    println!("   State: {:?}", document.state);
    println!("   Author: {}", author_id);
    println!("   Sections: {}", document.content_blocks.len());
    println!("   Total Events: {}", history.events.len());
    println!("   Access List: {} users", document.access_list.len());

    println!("\n=== Example completed successfully! ===");
    Ok(())
}

// Helper traits for demo
trait EventHelpers {
    fn event_type(&self) -> &str;
    fn timestamp(&self) -> SystemTime;
}

// Note: In real implementation, these would be part of the domain
impl EventHelpers for cim_domain_document::events::DocumentEvent {
    fn event_type(&self) -> &str {
        match self {
            DocumentEvent::Created(_) => "Created",
            DocumentEvent::Updated(_) => "Content Updated",
            DocumentEvent::Shared(_) => "Shared",
            DocumentEvent::StateChanged(_) => "State Changed",
            _ => "Other",
        }
    }

    fn timestamp(&self) -> SystemTime {
        // In real implementation, events would have timestamps
        SystemTime::now()
    }
}
