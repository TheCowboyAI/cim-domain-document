//! CIM-compliant Workflow Engine with NATS-first Architecture
//!
//! This module implements a workflow engine that follows CIM principles:
//! - MANDATORY correlation/causation IDs on all events
//! - NATS-first communication (no direct method calls)
//! - Pure event-driven pattern (no CRUD operations)
//! - CID chains for cryptographic integrity

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::nats::{
    DocumentSubject, SubjectNamespace, DocumentAggregate, SubjectScope,
    SubjectOperation, EventType, CommandType,
    MessageIdentity, ActorId, MessageFactory, CimMessage,
};
use crate::value_objects::DocumentId;
use crate::workflow::{
    WorkflowId, WorkflowInstanceId, WorkflowNodeId, WorkflowEdgeId, WorkflowStatus,
    SimpleWorkflowEngine,
};
use crate::workflow::cim_events::{
    CimWorkflowEvent, WorkflowEventType, WorkflowStartedEvent, WorkflowTransitionedEvent,
    WorkflowCompletedEvent, NodeEnteredEvent, NodeExitedEvent,
    NodeExitReason,
};

// ===== CIM-COMPLIANT WORKFLOW COMMANDS =====

/// Start a new workflow instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflowCommand {
    pub workflow_id: WorkflowId,
    pub document_id: DocumentId,
    pub initial_context: HashMap<String, serde_json::Value>,
    pub requested_by: Uuid,
}

/// Transition workflow to next node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionWorkflowCommand {
    pub instance_id: WorkflowInstanceId,
    pub to_node: WorkflowNodeId,
    pub transition_reason: String,
    pub context_updates: HashMap<String, serde_json::Value>,
    pub requested_by: Uuid,
}

/// Complete workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteWorkflowCommand {
    pub instance_id: WorkflowInstanceId,
    pub completion_reason: String,
    pub final_context: HashMap<String, serde_json::Value>,
    pub completed_by: Uuid,
}

/// Cancel workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelWorkflowCommand {
    pub instance_id: WorkflowInstanceId,
    pub cancellation_reason: String,
    pub cancelled_by: Uuid,
}

/// Pause workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseWorkflowCommand {
    pub instance_id: WorkflowInstanceId,
    pub pause_reason: String,
    pub resume_conditions: Vec<String>,
    pub paused_by: Uuid,
}

/// Resume workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeWorkflowCommand {
    pub instance_id: WorkflowInstanceId,
    pub resume_reason: String,
    pub resumed_by: Uuid,
}

// ===== CIM-COMPLIANT WORKFLOW QUERIES =====

/// Get workflow instance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWorkflowStatusQuery {
    pub instance_id: WorkflowInstanceId,
    pub requested_by: Uuid,
}

/// Get workflow execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWorkflowHistoryQuery {
    pub instance_id: WorkflowInstanceId,
    pub include_node_details: bool,
    pub requested_by: Uuid,
}

/// List active workflows for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDocumentWorkflowsQuery {
    pub document_id: DocumentId,
    pub status_filter: Option<Vec<WorkflowStatus>>,
    pub requested_by: Uuid,
}

// ===== WORKFLOW QUERY RESPONSES =====

/// Response to workflow status query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatusResponse {
    pub instance_id: WorkflowInstanceId,
    pub workflow_id: WorkflowId,
    pub document_id: DocumentId,
    pub current_node: WorkflowNodeId,
    pub status: WorkflowStatus,
    pub context: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Response to workflow history query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowHistoryResponse {
    pub instance_id: WorkflowInstanceId,
    pub events: Vec<WorkflowHistoryEntry>,
    pub total_events: usize,
}

/// Single entry in workflow history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowHistoryEntry {
    pub event_id: Uuid,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub actor: Option<ActorId>,
    pub node_id: Option<WorkflowNodeId>,
    pub description: String,
    pub correlation_id: crate::nats::CorrelationId,
    pub causation_id: crate::nats::CausationId,
}

/// Response to list workflows query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentWorkflowsResponse {
    pub document_id: DocumentId,
    pub workflows: Vec<WorkflowSummary>,
    pub total_count: usize,
}

/// Summary of a workflow instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummary {
    pub instance_id: WorkflowInstanceId,
    pub workflow_id: WorkflowId,
    pub current_node: WorkflowNodeId,
    pub status: WorkflowStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ===== CIM WORKFLOW ENGINE =====

/// NATS-first workflow engine with mandatory correlation/causation IDs
/// 
/// This engine communicates ONLY through NATS subjects - no direct method calls.
/// All operations are event-driven and include required CIM identity metadata.
pub struct CimWorkflowEngine {
    /// Internal workflow engine for business logic
    engine: SimpleWorkflowEngine,
    /// Active workflow instances with correlation tracking
    correlation_tracker: HashMap<crate::nats::CorrelationId, Vec<WorkflowInstanceId>>,
}

impl CimWorkflowEngine {
    /// Create a new CIM-compliant workflow engine
    pub fn new() -> Self {
        Self {
            engine: SimpleWorkflowEngine::new(),
            correlation_tracker: HashMap::new(),
        }
    }

    /// Process a start workflow command
    /// 
    /// This method demonstrates the CIM pattern:
    /// 1. Command arrives with correlation/causation IDs
    /// 2. Business logic executes
    /// 3. Events are published with proper correlation chain
    /// 4. No direct return values - everything through events
    pub async fn process_start_workflow_command(
        &mut self,
        command: CimMessage<StartWorkflowCommand>,
    ) -> Result<Vec<CimWorkflowEvent>, WorkflowError> {
        // Validate command correlation
        command.metadata.validate().map_err(|e| WorkflowError::IdentityError(e))?;

        let cmd = &command.payload;
        let parent_identity = &command.metadata.identity;

        // Create workflow instance using internal engine
        let instance_id = self.engine.start_workflow(
            cmd.workflow_id.clone(),
            cmd.document_id.clone(),
            cmd.requested_by,
        ).map_err(WorkflowError::InternalError)?;

        // Track correlation
        self.correlation_tracker
            .entry(parent_identity.correlation_id.clone())
            .or_insert_with(Vec::new)
            .push(instance_id);

        // Create CIM-compliant events
        let mut events = Vec::new();

        // Workflow started event
        let workflow_started = WorkflowStartedEvent {
            instance_id,
            workflow_id: cmd.workflow_id.clone(),
            document_id: cmd.document_id.clone(),
            start_node: WorkflowNodeId::Start,
            context: cmd.initial_context.clone(),
            started_by: cmd.requested_by,
            event_integrity: None, // TODO: Generate integrity data with CID chain
        };

        let started_event = CimWorkflowEvent::new_caused_by(
            instance_id,
            cmd.document_id.clone(),
            WorkflowEventType::Started(workflow_started),
            parent_identity,
            Some(ActorId::system("cim-workflow-engine")),
        );

        events.push(started_event);

        // Node entered event (entering start node)
        let node_entered = NodeEnteredEvent {
            instance_id,
            node_id: WorkflowNodeId::Start,
            entry_timestamp: chrono::Utc::now(),
            required_permissions: vec![],
            assigned_users: vec![cmd.requested_by],
            sla_deadline: None,
            event_integrity: None, // TODO: Generate integrity data with CID chain
        };

        let node_event = CimWorkflowEvent::new_caused_by(
            instance_id,
            cmd.document_id.clone(),
            WorkflowEventType::NodeEntered(node_entered),
            parent_identity,
            Some(ActorId::system("cim-workflow-engine")),
        );

        events.push(node_event);

        Ok(events)
    }

    /// Process a workflow transition command
    pub async fn process_transition_command(
        &mut self,
        command: CimMessage<TransitionWorkflowCommand>,
    ) -> Result<Vec<CimWorkflowEvent>, WorkflowError> {
        command.metadata.validate().map_err(|e| WorkflowError::IdentityError(e))?;

        let cmd = &command.payload;
        let parent_identity = &command.metadata.identity;

        // Get current instance and clone needed data
        let (from_node, document_id) = {
            let instance = self.engine.get_instance(cmd.instance_id)
                .ok_or(WorkflowError::InstanceNotFound(cmd.instance_id))?;
            (instance.current_node.clone(), instance.document_id.clone())
        };

        // Execute transition
        self.engine.transition_workflow(cmd.instance_id, cmd.to_node.clone())
            .map_err(WorkflowError::InternalError)?;

        // Update context if provided
        for (key, value) in &cmd.context_updates {
            self.engine.update_context(cmd.instance_id, key.clone(), value.clone())
                .map_err(WorkflowError::InternalError)?;
        }

        let mut events = Vec::new();

        // Node exited event
        let node_exited = NodeExitedEvent {
            instance_id: cmd.instance_id,
            node_id: from_node.clone(),
            exit_timestamp: chrono::Utc::now(),
            time_spent: chrono::Duration::seconds(0), // Would calculate actual time
            exit_reason: NodeExitReason::Transitioned(cmd.to_node.clone()),
            completed_by: Some(cmd.requested_by),
            event_integrity: None, // TODO: Generate integrity data with CID chain
        };

        let exit_event = CimWorkflowEvent::new_caused_by(
            cmd.instance_id,
            document_id.clone(),
            WorkflowEventType::NodeExited(node_exited),
            parent_identity,
            Some(ActorId::user(cmd.requested_by)),
        );

        events.push(exit_event);

        // Workflow transitioned event
        let transitioned = WorkflowTransitionedEvent {
            instance_id: cmd.instance_id,
            from_node,
            to_node: cmd.to_node.clone(),
            transition_edge: WorkflowEdgeId::Custom("manual".to_string()),
            conditions_met: vec![],
            actions_executed: vec![],
            transitioned_by: cmd.requested_by,
            context_changes: cmd.context_updates.clone(),
            event_integrity: None, // TODO: Generate integrity data with CID chain
        };

        let transition_event = CimWorkflowEvent::new_caused_by(
            cmd.instance_id,
            document_id.clone(),
            WorkflowEventType::Transitioned(transitioned),
            parent_identity,
            Some(ActorId::user(cmd.requested_by)),
        );

        events.push(transition_event);

        // Node entered event
        let node_entered = NodeEnteredEvent {
            instance_id: cmd.instance_id,
            node_id: cmd.to_node.clone(),
            entry_timestamp: chrono::Utc::now(),
            required_permissions: vec![],
            assigned_users: vec![cmd.requested_by],
            sla_deadline: None,
            event_integrity: None, // TODO: Generate integrity data with CID chain
        };

        let enter_event = CimWorkflowEvent::new_caused_by(
            cmd.instance_id,
            document_id,
            WorkflowEventType::NodeEntered(node_entered),
            parent_identity,
            Some(ActorId::user(cmd.requested_by)),
        );

        events.push(enter_event);

        Ok(events)
    }

    /// Process a workflow completion command
    pub async fn process_complete_command(
        &mut self,
        command: CimMessage<CompleteWorkflowCommand>,
    ) -> Result<Vec<CimWorkflowEvent>, WorkflowError> {
        command.metadata.validate().map_err(|e| WorkflowError::IdentityError(e))?;

        let cmd = &command.payload;
        let parent_identity = &command.metadata.identity;

        // Get current instance
        let instance = self.engine.get_instance(cmd.instance_id)
            .ok_or(WorkflowError::InstanceNotFound(cmd.instance_id))?;

        let mut events = Vec::new();

        // Node exited event (exiting current node)
        let node_exited = NodeExitedEvent {
            instance_id: cmd.instance_id,
            node_id: instance.current_node.clone(),
            exit_timestamp: chrono::Utc::now(),
            time_spent: chrono::Duration::seconds(0),
            exit_reason: NodeExitReason::Completed,
            completed_by: Some(cmd.completed_by),
            event_integrity: None, // TODO: Generate integrity data with CID chain
        };

        let exit_event = CimWorkflowEvent::new_caused_by(
            cmd.instance_id,
            instance.document_id.clone(),
            WorkflowEventType::NodeExited(node_exited),
            parent_identity,
            Some(ActorId::user(cmd.completed_by)),
        );

        events.push(exit_event);

        // Workflow completed event
        let completed = WorkflowCompletedEvent {
            instance_id: cmd.instance_id,
            end_node: WorkflowNodeId::End,
            final_status: WorkflowStatus::Completed,
            completion_reason: cmd.completion_reason.clone(),
            final_context: cmd.final_context.clone(),
            completed_by: Some(cmd.completed_by),
            event_integrity: None, // TODO: Generate integrity data with CID chain
        };

        let completion_event = CimWorkflowEvent::new_caused_by(
            cmd.instance_id,
            instance.document_id.clone(),
            WorkflowEventType::Completed(completed),
            parent_identity,
            Some(ActorId::user(cmd.completed_by)),
        );

        events.push(completion_event);

        Ok(events)
    }

    /// Get workflow status (query processing)
    pub async fn process_status_query(
        &self,
        query: CimMessage<GetWorkflowStatusQuery>,
    ) -> Result<CimMessage<WorkflowStatusResponse>, WorkflowError> {
        query.metadata.validate().map_err(|e| WorkflowError::IdentityError(e))?;

        let instance = self.engine.get_instance(query.payload.instance_id)
            .ok_or(WorkflowError::InstanceNotFound(query.payload.instance_id))?;

        let response = WorkflowStatusResponse {
            instance_id: instance.id,
            workflow_id: instance.workflow_id.clone(),
            document_id: instance.document_id.clone(),
            current_node: instance.current_node.clone(),
            status: instance.status.clone(),
            context: instance.context.clone(),
            created_at: instance.created_at,
            updated_at: instance.updated_at,
        };

        Ok(MessageFactory::create_caused_by_with_actor(
            response,
            &query.metadata.identity,
            ActorId::system("cim-workflow-engine"),
        ))
    }

    /// Get workflows for a document
    pub async fn process_document_workflows_query(
        &self,
        query: CimMessage<ListDocumentWorkflowsQuery>,
    ) -> Result<CimMessage<DocumentWorkflowsResponse>, WorkflowError> {
        query.metadata.validate().map_err(|e| WorkflowError::IdentityError(e))?;

        // This would query the event store for document workflows
        // For now, return empty response
        let response = DocumentWorkflowsResponse {
            document_id: query.payload.document_id.clone(),
            workflows: Vec::new(),
            total_count: 0,
        };

        Ok(MessageFactory::create_caused_by_with_actor(
            response,
            &query.metadata.identity,
            ActorId::system("cim-workflow-engine"),
        ))
    }

    /// Generate NATS subjects for workflow events
    pub fn event_subject(
        &self, 
        event: &CimWorkflowEvent
    ) -> DocumentSubject {
        // Generate subject based on event type and scope
        DocumentSubject::new(
            SubjectNamespace::Events,
            SubjectScope::Aggregate(DocumentAggregate::Workflow),
            SubjectOperation::Event(EventType::WorkflowTransitioned),
            Some(event.instance_id.as_uuid().to_string()),
        )
    }

    /// Generate NATS subjects for workflow commands
    pub fn command_subject(
        instance_id: WorkflowInstanceId,
        _command_type: &str,
    ) -> DocumentSubject {
        DocumentSubject::new(
            SubjectNamespace::Commands,
            SubjectScope::Aggregate(DocumentAggregate::Workflow),
            SubjectOperation::Command(CommandType::TransitionWorkflow),
            Some(instance_id.as_uuid().to_string()),
        )
    }
}

impl Default for CimWorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ===== ERROR TYPES =====

/// Errors in CIM workflow processing
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Message identity error: {0}")]
    IdentityError(#[from] crate::nats::IdentityError),

    #[error("Internal workflow engine error: {0}")]
    InternalError(crate::workflow::WorkflowError),

    #[error("Workflow instance not found: {0:?}")]
    InstanceNotFound(WorkflowInstanceId),

    #[error("Invalid workflow state transition")]
    InvalidTransition,

    #[error("Permission denied for workflow operation")]
    PermissionDenied,

    #[error("Workflow operation timeout")]
    Timeout,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_workflow_command() {
        let mut engine = CimWorkflowEngine::new();
        
        let start_cmd = StartWorkflowCommand {
            workflow_id: WorkflowId::new(),
            document_id: DocumentId::new(),
            initial_context: HashMap::new(),
            requested_by: Uuid::new_v4(),
        };

        let command_msg = MessageFactory::create_root_with_actor(
            start_cmd,
            ActorId::user(Uuid::new_v4()),
        );

        let events = engine.process_start_workflow_command(command_msg)
            .await
            .expect("Failed to process start command");

        assert_eq!(events.len(), 2); // Started + NodeEntered events
        assert_eq!(events[0].event_type(), "started");
        assert_eq!(events[1].event_type(), "node_entered");

        // Both events should have same correlation ID
        assert_eq!(events[0].correlation_id(), events[1].correlation_id());
    }

    #[tokio::test]
    async fn test_workflow_correlation_chain() {
        let mut engine = CimWorkflowEngine::new();
        
        // Start workflow
        let start_cmd = StartWorkflowCommand {
            workflow_id: WorkflowId::new(),
            document_id: DocumentId::new(),
            initial_context: HashMap::new(),
            requested_by: Uuid::new_v4(),
        };

        let start_msg = MessageFactory::create_root(start_cmd);
        let start_events = engine.process_start_workflow_command(start_msg)
            .await
            .unwrap();

        let instance_id = match &start_events[0].event {
            WorkflowEventType::Started(event) => event.instance_id,
            _ => panic!("Expected WorkflowStarted event"),
        };

        // Transition workflow
        let transition_cmd = TransitionWorkflowCommand {
            instance_id,
            to_node: WorkflowNodeId::Draft,
            transition_reason: "Moving to draft".to_string(),
            context_updates: HashMap::new(),
            requested_by: Uuid::new_v4(),
        };

        let transition_msg = MessageFactory::create_caused_by(
            transition_cmd,
            &start_events[0].metadata.identity,
        );

        let transition_events = engine.process_transition_command(transition_msg)
            .await
            .unwrap();

        // All events should have same correlation ID
        assert_eq!(start_events[0].correlation_id(), transition_events[0].correlation_id());
        
        // Transition events should be caused by start events
        assert_eq!(
            transition_events[0].causation_id().0,
            start_events[0].message_id().0
        );
    }

    #[tokio::test]
    async fn test_workflow_query_processing() {
        let mut engine = CimWorkflowEngine::new();
        
        // Start a workflow first
        let start_cmd = StartWorkflowCommand {
            workflow_id: WorkflowId::new(),
            document_id: DocumentId::new(),
            initial_context: HashMap::new(),
            requested_by: Uuid::new_v4(),
        };

        let start_msg = MessageFactory::create_root(start_cmd);
        let start_events = engine.process_start_workflow_command(start_msg)
            .await
            .unwrap();

        let instance_id = match &start_events[0].event {
            WorkflowEventType::Started(event) => event.instance_id,
            _ => panic!("Expected WorkflowStarted event"),
        };

        // Query workflow status
        let status_query = GetWorkflowStatusQuery {
            instance_id,
            requested_by: Uuid::new_v4(),
        };

        let query_msg = MessageFactory::create_root(status_query);
        let response = engine.process_status_query(query_msg)
            .await
            .unwrap();

        assert_eq!(response.payload.instance_id, instance_id);
        assert_eq!(response.payload.current_node, WorkflowNodeId::Start);
    }
}