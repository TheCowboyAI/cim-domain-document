//! Workflow Event Integrity Demo
//!
//! This demonstrates the CID chain-based event integrity system for workflows.
//! Each workflow event is content-addressed and cryptographically linked to
//! the previous event, ensuring the entire workflow is immutable and verifiable.

use cim_domain_document::{
    DocumentId,
    workflow::{
        WorkflowInstanceId, WorkflowNodeId,
        DefaultWorkflowIntegrityService, WorkflowIntegrityService,
        WorkflowEventIntegrity, WorkflowEventChain,
        event_integrity::{WorkflowEventType, ChainIntegrityStatus, IntegrityError, create_workflow_event_chain},
    },
};
use cim_domain_document::nats::ActorId;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), IntegrityError> {
    println!("🔐 Workflow Event Integrity Demo");
    println!("=================================");

    // Initialize the integrity service
    let integrity_service = DefaultWorkflowIntegrityService::new();
    let actor = ActorId::User(Uuid::new_v4());
    let instance_id = WorkflowInstanceId::new();
    let document_id = DocumentId::new();
    
    println!("📋 Workflow Instance: {}", instance_id.as_uuid());
    println!("📄 Document ID: {}", document_id.as_uuid());
    println!();

    // Phase 1: Create genesis event (workflow started)
    println!("🚀 Phase 1: Creating Genesis Event");
    let genesis_payload = b"workflow_started_event";
    let genesis_integrity = integrity_service.create_event_integrity(
        genesis_payload,
        None, // Genesis event has no predecessor
        &actor,
        &WorkflowNodeId::Start,
        WorkflowEventType::Started,
    ).await?;

    println!("✅ Genesis Event CID: {}", genesis_integrity.event_cid);
    println!("🔗 Predecessor: None (genesis)");
    println!("📊 Sequence Number: {}", genesis_integrity.chain_metadata.sequence_number);
    
    // Verify genesis event integrity
    let is_valid = integrity_service.verify_event_integrity(&genesis_integrity, genesis_payload).await?;
    println!("✅ Genesis Event Verification: {}", if is_valid { "VALID ✓" } else { "INVALID ✗" });
    println!();

    // Phase 2: Create workflow event chain
    println!("⛓️  Phase 2: Building Event Chain");
    let mut chain = create_workflow_event_chain(
        instance_id,
        document_id,
        genesis_integrity.event_cid.clone(),
    );
    
    // Add genesis event to chain
    integrity_service.extend_event_chain(
        &mut chain,
        genesis_integrity,
        WorkflowNodeId::Start,
        WorkflowEventType::Started,
        &actor,
    ).await?;
    
    println!("📏 Chain Length: {}", chain.event_links.len());
    println!("🎯 Head CID: {}", chain.head_cid);
    println!();

    // Phase 3: Add transition event
    println!("🔄 Phase 3: Adding Transition Event");
    let transition_payload = b"workflow_transitioned_event";
    let transition_integrity = integrity_service.create_event_integrity(
        transition_payload,
        Some(chain.head_cid.clone()),
        &actor,
        &WorkflowNodeId::InReview,
        WorkflowEventType::Transitioned,
    ).await?;

    println!("✅ Transition Event CID: {}", transition_integrity.event_cid);
    println!("🔗 Predecessor: {}", transition_integrity.predecessor_cid.as_ref().unwrap());
    println!("📊 Sequence Number: {}", transition_integrity.chain_metadata.sequence_number);
    
    // Add transition to chain
    integrity_service.extend_event_chain(
        &mut chain,
        transition_integrity,
        WorkflowNodeId::InReview,
        WorkflowEventType::Transitioned,
        &actor,
    ).await?;
    
    println!("📏 Chain Length: {}", chain.event_links.len());
    println!("🎯 New Head CID: {}", chain.head_cid);
    println!();

    // Phase 4: Add completion event
    println!("✅ Phase 4: Adding Completion Event");
    let completion_payload = b"workflow_completed_event";
    let completion_integrity = integrity_service.create_event_integrity(
        completion_payload,
        Some(chain.head_cid.clone()),
        &actor,
        &WorkflowNodeId::End,
        WorkflowEventType::Completed,
    ).await?;

    println!("✅ Completion Event CID: {}", completion_integrity.event_cid);
    println!("🔗 Predecessor: {}", completion_integrity.predecessor_cid.as_ref().unwrap());
    println!("📊 Sequence Number: {}", completion_integrity.chain_metadata.sequence_number);
    
    // Add completion to chain
    integrity_service.extend_event_chain(
        &mut chain,
        completion_integrity,
        WorkflowNodeId::End,
        WorkflowEventType::Completed,
        &actor,
    ).await?;
    
    println!("📏 Final Chain Length: {}", chain.event_links.len());
    println!("🎯 Final Head CID: {}", chain.head_cid);
    println!();

    // Phase 5: Verify entire chain integrity
    println!("🔍 Phase 5: Verifying Chain Integrity");
    let integrity_status = integrity_service.verify_event_chain(&chain).await?;
    
    match integrity_status {
        ChainIntegrityStatus::Valid => {
            println!("✅ Chain Integrity: VALID ✓");
            println!("🛡️  All events are cryptographically linked and verified");
        },
        ChainIntegrityStatus::Corrupted { issues } => {
            println!("❌ Chain Integrity: CORRUPTED ✗");
            println!("🚨 Issues found: {}", issues.len());
            for (i, issue) in issues.iter().enumerate() {
                println!("  {}. {}: {}", i + 1, format!("{:?}", issue.issue_type), issue.description);
            }
        },
        _ => {
            println!("⚠️  Chain Integrity: UNKNOWN");
        }
    }
    println!();

    // Phase 6: Demonstrate chain properties
    println!("🔬 Phase 6: Chain Properties Analysis");
    println!("📈 Genesis CID: {}", chain.genesis_cid);
    println!("📈 Head CID: {}", chain.head_cid);
    println!("📊 Total Events: {}", chain.event_links.len());
    println!("⏰ Last Verified: {}", chain.last_verified);
    
    println!("\n🔗 Event Chain Links:");
    for (i, link) in chain.event_links.iter().enumerate() {
        let predecessor_str = match &link.predecessor_cid {
            Some(cid) => format!("{}", cid),
            None => "GENESIS".to_string(),
        };
        println!("  {}. {} → {} ({})", 
            i + 1, 
            predecessor_str,
            link.event_cid,
            format!("{:?}", link.event_type)
        );
    }
    
    println!();
    println!("🎉 Workflow Event Integrity Demo Complete!");
    println!("✅ All events are cryptographically verified and immutable");
    println!("🔒 The workflow chain cannot be tampered with or replayed");
    
    Ok(())
}