//! Workflow Integration Example
//!
//! This example demonstrates the comprehensive workflow system functionality
//! including document event integration, workflow state management, and 
//! audit trails.

use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use cim_domain_document::workflow::*;
use cim_domain_document::events::{DocumentDomainEvent, DocumentCreated};
use cim_domain_document::value_objects::{DocumentId, DocumentType, DocumentState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CIM Document Workflow System Integration Example\n");

    // Initialize the workflow manager
    let workflow_manager = WorkflowManager::new();
    workflow_manager.initialize().await?;
    println!("✅ Workflow manager initialized with default workflows\n");

    // Create a test document
    let document_id = DocumentId::new();
    let user_id = Uuid::new_v4();
    
    println!("📄 Created document: {}", document_id);
    println!("👤 User: {}\n", user_id);

    // Demonstrate 1: Manual workflow start
    println!("=== Manual Workflow Start ===");
    let review_instance = workflow_manager.start_workflow(
        "review",
        document_id.clone(),
        user_id,
    ).await?;
    
    println!("✅ Started review workflow: {}", review_instance.as_uuid());
    
    // Get workflow instance details
    if let Some(instance) = workflow_manager.get_workflow_instance(review_instance).await? {
        println!("   Current node: {:?}", instance.current_node);
        println!("   Status: {:?}", instance.status);
    }
    println!();

    // Demonstrate 2: Event-driven workflow triggering
    println!("=== Event-Driven Workflow Triggering ===");
    
    // Create a document creation event
    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), "Important Document".to_string());
    metadata.insert("category".to_string(), "proposal".to_string());
    
    let creation_event = DocumentDomainEvent::DocumentCreated(DocumentCreated {
        document_id: DocumentId::new(),
        document_type: DocumentType::Text,
        title: "Important Document".to_string(),
        author_id: user_id,
        metadata,
        created_at: Utc::now(),
    });

    // Handle the event - this should automatically trigger workflows
    let triggered_workflows = workflow_manager.handle_document_event(&creation_event).await?;
    
    println!("✅ Document creation event triggered {} workflows", triggered_workflows.len());
    for workflow_id in &triggered_workflows {
        println!("   Workflow: {}", workflow_id.as_uuid());
    }
    println!();

    // Demonstrate 3: Workflow transitions with validation
    println!("=== Workflow Transitions with Validation ===");
    
    let user_permissions = vec![Permission::Review, Permission::View];
    
    // Transition the review workflow to InReview state
    match workflow_manager.transition_workflow(
        review_instance,
        WorkflowNodeId::InReview,
        user_id,
        user_permissions.clone(),
    ).await {
        Ok(()) => {
            println!("✅ Successfully transitioned workflow to InReview");
            
            // Get updated instance details
            if let Some(instance) = workflow_manager.get_workflow_instance(review_instance).await? {
                println!("   Updated node: {:?}", instance.current_node);
                println!("   Status: {:?}", instance.status);
            }
        }
        Err(e) => {
            println!("❌ Transition failed: {}", e);
        }
    }
    println!();

    // Demonstrate 4: Audit trail
    println!("=== Workflow Audit Trail ===");
    let audit_trail = workflow_manager.get_audit_trail(review_instance).await;
    
    println!("📋 Audit trail for workflow {} ({} entries):", 
             review_instance.as_uuid(), audit_trail.len());
    
    for (i, entry) in audit_trail.iter().enumerate() {
        println!("   {}. {} at {}", 
                 i + 1,
                 format!("{:?}", entry.event_type),
                 entry.performed_at.format("%H:%M:%S"));
        if let (Some(from), Some(to)) = (&entry.from_node, &entry.to_node) {
            println!("      Transition: {:?} → {:?}", from, to);
        }
    }
    println!();

    // Demonstrate 5: Workflow statistics and monitoring
    println!("=== Workflow Statistics ===");
    let stats = workflow_manager.get_workflow_statistics().await?;
    
    println!("📊 Active workflows:");
    println!("   Documents with workflows: {}", stats.total_active_documents);
    println!("   Total active workflows: {}", stats.total_active_workflows);
    println!();

    // Demonstrate 6: Get active workflows for a document
    println!("=== Active Workflows for Document ===");
    let active_workflows = workflow_manager.get_active_workflows_for_document(document_id.clone()).await?;
    
    println!("🔄 Active workflows for document {}:", document_id);
    for workflow in &active_workflows {
        println!("   {} - {:?} ({})", 
                 workflow.id.as_uuid(),
                 workflow.current_node,
                 format!("{:?}", workflow.status));
    }
    
    if active_workflows.is_empty() {
        println!("   No active workflows");
    }
    println!();

    // Demonstrate 7: Workflow cancellation
    println!("=== Workflow Cancellation ===");
    if let Some(workflow_to_cancel) = active_workflows.first() {
        workflow_manager.cancel_workflow(
            workflow_to_cancel.id,
            user_id,
            "Example cancellation".to_string(),
        ).await?;
        
        println!("✅ Cancelled workflow: {}", workflow_to_cancel.id.as_uuid());
        
        // Verify cancellation in audit trail
        let updated_trail = workflow_manager.get_audit_trail(workflow_to_cancel.id).await;
        if let Some(last_entry) = updated_trail.last() {
            println!("   Last entry: {:?}", last_entry.event_type);
        }
    }

    println!("\n🎉 Workflow integration example completed successfully!");
    println!("\n📋 Summary of demonstrated features:");
    println!("   ✓ Manual workflow initiation");
    println!("   ✓ Event-driven workflow triggering");  
    println!("   ✓ State transitions with validation");
    println!("   ✓ Comprehensive audit trails");
    println!("   ✓ Workflow monitoring and statistics");
    println!("   ✓ Active workflow management");
    println!("   ✓ Workflow cancellation");

    Ok(())
}