/// CIM Workflow Engine Demonstration
/// 
/// This example shows how the CIM-compliant workflow engine works with
/// MANDATORY correlation/causation IDs and NATS-first architecture.

use std::collections::HashMap;
use uuid::Uuid;

// Import CIM workflow types
use cim_domain_document::{
    workflow::{
        WorkflowId, WorkflowNodeId,
        CimWorkflowEngine, StartWorkflowCommand, TransitionWorkflowCommand,
        CimWorkflowEventType,
    },
    nats::{MessageFactory, ActorId},
    value_objects::DocumentId,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 CIM Workflow Engine Demo");
    println!("=============================");
    
    // Create CIM-compliant workflow engine
    let mut engine = CimWorkflowEngine::new();
    
    // 1. Start a new workflow with root correlation
    println!("\n1️⃣ Starting new workflow...");
    let start_command = StartWorkflowCommand {
        workflow_id: WorkflowId::new(),
        document_id: DocumentId::new(),
        initial_context: HashMap::new(),
        requested_by: Uuid::new_v4(),
    };
    
    // Create root message (starts new correlation chain)
    let start_message = MessageFactory::create_root_with_actor(
        start_command,
        ActorId::user(Uuid::new_v4()),
    );
    
    println!("📋 Root correlation ID: {}", start_message.identity().correlation_id);
    println!("📋 Root message ID: {}", start_message.identity().message_id);
    
    // Process start command
    let start_events = engine.process_start_workflow_command(start_message)
        .await?;
    
    println!("✅ Generated {} workflow events", start_events.len());
    
    // Extract instance ID for next command
    let instance_id = match &start_events[0].event {
        CimWorkflowEventType::Started(event) => {
            println!("🚀 Workflow started with instance: {}", event.instance_id.as_uuid());
            event.instance_id
        },
        _ => panic!("Expected WorkflowStarted event"),
    };
    
    // 2. Transition workflow (part of same correlation chain)
    println!("\n2️⃣ Transitioning workflow...");
    let transition_command = TransitionWorkflowCommand {
        instance_id,
        to_node: WorkflowNodeId::Draft,
        transition_reason: "Moving to draft stage".to_string(),
        context_updates: HashMap::new(),
        requested_by: Uuid::new_v4(),
    };
    
    // Create caused message (inherits correlation from start events)
    let transition_message = MessageFactory::create_caused_by_with_actor(
        transition_command,
        &start_events[0].metadata.identity,
        ActorId::user(Uuid::new_v4()),
    );
    
    println!("🔗 Same correlation ID: {}", transition_message.identity().correlation_id);
    println!("🔗 Caused by message: {}", transition_message.identity().causation_id);
    
    // Process transition command
    let transition_events = engine.process_transition_command(transition_message)
        .await?;
    
    println!("✅ Generated {} transition events", transition_events.len());
    
    // 3. Verify correlation chain integrity
    println!("\n3️⃣ Verifying correlation chain...");
    
    // All events should have the same correlation ID
    let root_correlation = start_events[0].correlation_id();
    for event in &transition_events {
        assert_eq!(event.correlation_id(), root_correlation);
        println!("✅ Event {} maintains correlation chain", event.event_type());
    }
    
    // Transition events should be caused by start events
    assert_eq!(
        transition_events[0].causation_id().0,
        start_events[0].message_id().0
    );
    println!("✅ Causation chain verified");
    
    // 4. Demonstrate Subject Algebra (would be used for NATS publishing)
    println!("\n4️⃣ Demonstrating Subject Algebra...");
    
    for event in &start_events {
        let subject = engine.event_subject(event);
        println!("📡 Event subject: {}", subject.to_string());
    }
    
    println!("\n🎉 CIM Workflow Engine Demo Complete!");
    println!("✅ MANDATORY correlation/causation IDs implemented");
    println!("✅ Event-driven workflow transitions");
    println!("✅ Subject Algebra for NATS communication");
    println!("✅ CIM-compliant message identity");
    
    Ok(())
}