//! CIM-compliant Workflow Events with MANDATORY Correlation/Causation IDs
//!
//! This module defines workflow events that follow CIM principles with required
//! message identity for proper event tracing and system coherence.
//! Enhanced with CID chain integrity for cryptographic event verification.

use serde::{Deserialize, Serialize};
use crate::nats::{CimDomainEvent, MessageIdentity, ActorId, EventMetadata};
use crate::value_objects::DocumentId;
use crate::workflow::{
    WorkflowId, WorkflowInstanceId, WorkflowNodeId, WorkflowEdgeId, 
    WorkflowStatus, Permission, WorkflowActionType, WorkflowCondition,
};
use crate::workflow::event_integrity::{WorkflowEventIntegrity, WorkflowEventChain};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ===== WORKFLOW LIFECYCLE EVENTS =====

/// Workflow instance was started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStartedEvent {
    pub instance_id: WorkflowInstanceId,
    pub workflow_id: WorkflowId,
    pub document_id: DocumentId,
    pub start_node: WorkflowNodeId,
    pub context: HashMap<String, serde_json::Value>,
    pub started_by: Uuid,
    /// CID chain integrity for event verification (optional for backward compatibility)
    pub event_integrity: Option<WorkflowEventIntegrity>,
}

/// Workflow transitioned between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransitionedEvent {
    pub instance_id: WorkflowInstanceId,
    pub from_node: WorkflowNodeId,
    pub to_node: WorkflowNodeId,
    pub transition_edge: WorkflowEdgeId,
    pub conditions_met: Vec<WorkflowCondition>,
    pub actions_executed: Vec<WorkflowActionType>,
    pub transitioned_by: Uuid,
    pub context_changes: HashMap<String, serde_json::Value>,
    /// CID chain integrity for event verification (optional for backward compatibility)
    pub event_integrity: Option<WorkflowEventIntegrity>,
}

/// Workflow completed successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCompletedEvent {
    pub instance_id: WorkflowInstanceId,
    pub end_node: WorkflowNodeId,
    pub final_status: WorkflowStatus,
    pub completion_reason: String,
    pub final_context: HashMap<String, serde_json::Value>,
    pub completed_by: Option<Uuid>,
    /// CID chain integrity for event verification (optional for backward compatibility)
    pub event_integrity: Option<WorkflowEventIntegrity>,
}

/// Workflow failed with error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFailedEvent {
    pub instance_id: WorkflowInstanceId,
    pub current_node: WorkflowNodeId,
    pub failure_reason: String,
    pub error_code: Option<String>,
    pub context_at_failure: HashMap<String, serde_json::Value>,
    pub failed_action: Option<WorkflowActionType>,
    /// CID chain integrity for event verification (optional for backward compatibility)
    pub event_integrity: Option<WorkflowEventIntegrity>,
}

/// Workflow was cancelled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCancelledEvent {
    pub instance_id: WorkflowInstanceId,
    pub current_node: WorkflowNodeId,
    pub cancellation_reason: String,
    pub cancelled_by: Uuid,
    pub context_at_cancellation: HashMap<String, serde_json::Value>,
    /// CID chain integrity for event verification (optional for backward compatibility)
    pub event_integrity: Option<WorkflowEventIntegrity>,
}

/// Workflow was paused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPausedEvent {
    pub instance_id: WorkflowInstanceId,
    pub current_node: WorkflowNodeId,
    pub pause_reason: String,
    pub paused_by: Uuid,
    pub resume_conditions: Vec<String>,
    /// CID chain integrity for event verification (optional for backward compatibility)
    pub event_integrity: Option<WorkflowEventIntegrity>,
}

/// Workflow was resumed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResumedEvent {
    pub instance_id: WorkflowInstanceId,
    pub current_node: WorkflowNodeId,
    pub resume_reason: String,
    pub resumed_by: Uuid,
    pub conditions_satisfied: Vec<String>,
    /// CID chain integrity for event verification (optional for backward compatibility)
    pub event_integrity: Option<WorkflowEventIntegrity>,
}

// ===== NODE-LEVEL EVENTS =====

/// Workflow node was entered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEnteredEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub entry_timestamp: DateTime<Utc>,
    pub required_permissions: Vec<Permission>,
    pub assigned_users: Vec<Uuid>,
    pub sla_deadline: Option<DateTime<Utc>>,
    /// CID chain integrity for event verification (optional for backward compatibility)
    pub event_integrity: Option<WorkflowEventIntegrity>,
}

/// Workflow node was exited
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExitedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub exit_timestamp: DateTime<Utc>,
    pub time_spent: chrono::Duration,
    pub exit_reason: NodeExitReason,
    pub completed_by: Option<Uuid>,
    /// CID chain integrity for event verification (optional for backward compatibility)
    pub event_integrity: Option<WorkflowEventIntegrity>,
}

/// Reason for exiting a workflow node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeExitReason {
    /// Normal completion
    Completed,
    /// Transitioned to another node
    Transitioned(WorkflowNodeId),
    /// Skipped due to conditions
    Skipped,
    /// Failed during execution
    Failed(String),
    /// Cancelled by user
    Cancelled,
    /// Escalated due to SLA breach
    Escalated,
}

// ===== ACTION AND CONDITION EVENTS =====

/// Workflow action was executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExecutedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub action: WorkflowActionType,
    pub execution_result: ActionResult,
    pub execution_duration: chrono::Duration,
    pub executed_by: ActorId,
}

/// Result of workflow action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    /// Action completed successfully
    Success {
        output: HashMap<String, serde_json::Value>,
    },
    /// Action failed with error
    Failed {
        error: String,
        error_code: Option<String>,
    },
    /// Action was skipped due to conditions
    Skipped {
        reason: String,
    },
}

/// Workflow condition was evaluated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionEvaluatedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub condition: WorkflowCondition,
    pub evaluation_result: bool,
    pub evaluation_context: HashMap<String, serde_json::Value>,
    pub evaluated_by: ActorId,
}

// ===== PERMISSION AND ASSIGNMENT EVENTS =====

/// User was assigned to workflow node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAssignedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub user_id: Uuid,
    pub assigned_permissions: Vec<Permission>,
    pub assignment_reason: String,
    pub assigned_by: Uuid,
    pub due_date: Option<DateTime<Utc>>,
}

/// User assignment was removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUnassignedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub user_id: Uuid,
    pub removal_reason: String,
    pub removed_by: Uuid,
}

/// Permission was granted for workflow operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrantedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub user_id: Uuid,
    pub permission: Permission,
    pub granted_by: Uuid,
    pub grant_reason: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Permission was revoked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRevokedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub user_id: Uuid,
    pub permission: Permission,
    pub revoked_by: Uuid,
    pub revocation_reason: String,
}

// ===== SLA AND ESCALATION EVENTS =====

/// SLA deadline approaching warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaWarningEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub deadline: DateTime<Utc>,
    pub time_remaining: chrono::Duration,
    pub warning_threshold: chrono::Duration,
    pub assigned_users: Vec<Uuid>,
}

/// SLA deadline was breached
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaBreachedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub deadline: DateTime<Utc>,
    pub breach_duration: chrono::Duration,
    pub assigned_users: Vec<Uuid>,
    pub escalation_triggered: bool,
}

/// Workflow was escalated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEscalatedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: WorkflowNodeId,
    pub escalation_reason: EscalationReason,
    pub escalated_to: Vec<Uuid>,
    pub original_assignees: Vec<Uuid>,
    pub escalation_actions: Vec<WorkflowActionType>,
}

/// Reason for workflow escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationReason {
    /// SLA breach
    SlaBreached {
        deadline: DateTime<Utc>,
        breach_duration: chrono::Duration,
    },
    /// Manual escalation
    ManualEscalation {
        reason: String,
        escalated_by: Uuid,
    },
    /// System-detected issue
    SystemDetected {
        issue_type: String,
        detection_reason: String,
    },
}

// ===== CONTEXT AND VARIABLE EVENTS =====

/// Workflow context was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdatedEvent {
    pub instance_id: WorkflowInstanceId,
    pub node_id: Option<WorkflowNodeId>,
    pub variable_changes: HashMap<String, VariableChange>,
    pub updated_by: ActorId,
    pub update_reason: String,
}

/// Change to a workflow variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableChange {
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub change_type: VariableChangeType,
}

/// Type of variable change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableChangeType {
    /// Variable was created
    Created,
    /// Variable was updated
    Updated,
    /// Variable was deleted
    Deleted,
    /// Variable was reset to default
    Reset,
}

// ===== WORKFLOW DEFINITION EVENTS =====

/// Workflow definition was registered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinitionRegisteredEvent {
    pub workflow_id: WorkflowId,
    pub definition_name: String,
    pub version: String,
    pub description: Option<String>,
    pub node_count: usize,
    pub edge_count: usize,
    pub registered_by: Uuid,
}

/// Workflow definition was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinitionUpdatedEvent {
    pub workflow_id: WorkflowId,
    pub old_version: String,
    pub new_version: String,
    pub changes_summary: String,
    pub updated_by: Uuid,
    pub migration_required: bool,
}

/// Workflow definition was deprecated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinitionDeprecatedEvent {
    pub workflow_id: WorkflowId,
    pub version: String,
    pub deprecation_reason: String,
    pub replacement_workflow: Option<WorkflowId>,
    pub deprecated_by: Uuid,
    pub sunset_date: Option<DateTime<Utc>>,
}

// ===== CIM WORKFLOW EVENT WRAPPER =====

/// CIM-compliant workflow event with mandatory correlation/causation IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimWorkflowEvent {
    /// Event metadata with MANDATORY correlation/causation (CIM requirement)
    pub metadata: EventMetadata,
    /// Workflow instance identifier
    pub instance_id: WorkflowInstanceId,
    /// Document this workflow is processing
    pub document_id: DocumentId,
    /// The specific workflow event
    pub event: WorkflowEventType,
}

/// All possible workflow event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEventType {
    // Lifecycle events
    Started(WorkflowStartedEvent),
    Transitioned(WorkflowTransitionedEvent),
    Completed(WorkflowCompletedEvent),
    Failed(WorkflowFailedEvent),
    Cancelled(WorkflowCancelledEvent),
    Paused(WorkflowPausedEvent),
    Resumed(WorkflowResumedEvent),
    
    // Node events
    NodeEntered(NodeEnteredEvent),
    NodeExited(NodeExitedEvent),
    
    // Action and condition events
    ActionExecuted(ActionExecutedEvent),
    ConditionEvaluated(ConditionEvaluatedEvent),
    
    // Permission and assignment events
    UserAssigned(UserAssignedEvent),
    UserUnassigned(UserUnassignedEvent),
    PermissionGranted(PermissionGrantedEvent),
    PermissionRevoked(PermissionRevokedEvent),
    
    // SLA and escalation events
    SlaWarning(SlaWarningEvent),
    SlaBreached(SlaBreachedEvent),
    WorkflowEscalated(WorkflowEscalatedEvent),
    
    // Context events
    ContextUpdated(ContextUpdatedEvent),
    
    // Definition events
    DefinitionRegistered(WorkflowDefinitionRegisteredEvent),
    DefinitionUpdated(WorkflowDefinitionUpdatedEvent),
    DefinitionDeprecated(WorkflowDefinitionDeprecatedEvent),
}

impl CimWorkflowEvent {
    /// Create a workflow event with proper CIM identity
    pub fn new(
        instance_id: WorkflowInstanceId,
        document_id: DocumentId,
        event: WorkflowEventType,
        parent_identity: Option<&MessageIdentity>,
        actor: Option<ActorId>,
    ) -> Self {
        let metadata = match parent_identity {
            Some(parent) => EventMetadata::new_caused_by(parent, actor),
            None => EventMetadata::new_root(actor),
        };

        Self {
            metadata,
            instance_id,
            document_id,
            event,
        }
    }

    /// Create a root workflow event (starts new correlation chain)
    pub fn new_root(
        instance_id: WorkflowInstanceId,
        document_id: DocumentId,
        event: WorkflowEventType,
        actor: Option<ActorId>,
    ) -> Self {
        Self::new(instance_id, document_id, event, None, actor)
    }

    /// Create a caused workflow event (part of existing correlation chain)
    pub fn new_caused_by(
        instance_id: WorkflowInstanceId,
        document_id: DocumentId,
        event: WorkflowEventType,
        parent_identity: &MessageIdentity,
        actor: Option<ActorId>,
    ) -> Self {
        Self::new(instance_id, document_id, event, Some(parent_identity), actor)
    }

    /// Get the event type as a string for subjects and logging
    pub fn event_type(&self) -> &'static str {
        match &self.event {
            WorkflowEventType::Started(_) => "started",
            WorkflowEventType::Transitioned(_) => "transitioned",
            WorkflowEventType::Completed(_) => "completed",
            WorkflowEventType::Failed(_) => "failed",
            WorkflowEventType::Cancelled(_) => "cancelled",
            WorkflowEventType::Paused(_) => "paused",
            WorkflowEventType::Resumed(_) => "resumed",
            WorkflowEventType::NodeEntered(_) => "node_entered",
            WorkflowEventType::NodeExited(_) => "node_exited",
            WorkflowEventType::ActionExecuted(_) => "action_executed",
            WorkflowEventType::ConditionEvaluated(_) => "condition_evaluated",
            WorkflowEventType::UserAssigned(_) => "user_assigned",
            WorkflowEventType::UserUnassigned(_) => "user_unassigned",
            WorkflowEventType::PermissionGranted(_) => "permission_granted",
            WorkflowEventType::PermissionRevoked(_) => "permission_revoked",
            WorkflowEventType::SlaWarning(_) => "sla_warning",
            WorkflowEventType::SlaBreached(_) => "sla_breached",
            WorkflowEventType::WorkflowEscalated(_) => "escalated",
            WorkflowEventType::ContextUpdated(_) => "context_updated",
            WorkflowEventType::DefinitionRegistered(_) => "definition_registered",
            WorkflowEventType::DefinitionUpdated(_) => "definition_updated",
            WorkflowEventType::DefinitionDeprecated(_) => "definition_deprecated",
        }
    }

    /// Convert to a generic CIM domain event for NATS publishing
    pub fn to_domain_event(self) -> Result<CimDomainEvent, serde_json::Error> {
        let payload = serde_json::to_value(&self.event)?;
        let event_type_str = self.event_type();
        
        Ok(CimDomainEvent {
            metadata: self.metadata,
            aggregate_id: self.instance_id.as_uuid().to_string(),
            sequence: 0, // Will be set by event store
            event_cid: None, // Will be set by event store
            previous_cid: None, // Will be set by event store
            event_type: format!("workflow.{}", event_type_str),
            payload,
        })
    }

    /// Get correlation ID (MANDATORY in CIM)
    pub fn correlation_id(&self) -> &crate::nats::CorrelationId {
        &self.metadata.identity.correlation_id
    }

    /// Get causation ID (MANDATORY in CIM)
    pub fn causation_id(&self) -> &crate::nats::CausationId {
        &self.metadata.identity.causation_id
    }

    /// Get message ID
    pub fn message_id(&self) -> &crate::nats::MessageId {
        &self.metadata.identity.message_id
    }

    /// Check if this is a root event (starts correlation chain)
    pub fn is_root_event(&self) -> bool {
        self.metadata.identity.is_root()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::WorkflowNodeId;

    #[test]
    fn test_cim_workflow_event_creation() {
        let instance_id = WorkflowInstanceId::new();
        let document_id = DocumentId::new();
        
        let workflow_started = WorkflowStartedEvent {
            instance_id,
            workflow_id: WorkflowId::new(),
            document_id: document_id.clone(),
            start_node: WorkflowNodeId::Start,
            context: HashMap::new(),
            started_by: Uuid::new_v4(),
            event_integrity: None, // TODO: Generate integrity data
        };

        let event = CimWorkflowEvent::new_root(
            instance_id,
            document_id,
            WorkflowEventType::Started(workflow_started),
            Some(ActorId::system("workflow-engine")),
        );

        assert!(event.is_root_event());
        assert_eq!(event.event_type(), "started");
        assert_eq!(event.instance_id, instance_id);
    }

    #[test]
    fn test_workflow_event_correlation() {
        let instance_id = WorkflowInstanceId::new();
        let document_id = DocumentId::new();

        // Create root event
        let started_event = CimWorkflowEvent::new_root(
            instance_id,
            document_id.clone(),
            WorkflowEventType::Started(WorkflowStartedEvent {
                instance_id,
                workflow_id: WorkflowId::new(),
                document_id: document_id.clone(),
                start_node: WorkflowNodeId::Start,
                context: HashMap::new(),
                started_by: Uuid::new_v4(),
                event_integrity: None, // TODO: Generate integrity data
            }),
            Some(ActorId::system("workflow-engine")),
        );

        // Create caused event
        let transitioned_event = CimWorkflowEvent::new_caused_by(
            instance_id,
            document_id,
            WorkflowEventType::Transitioned(WorkflowTransitionedEvent {
                instance_id,
                from_node: WorkflowNodeId::Start,
                to_node: WorkflowNodeId::Draft,
                transition_edge: crate::workflow::WorkflowEdgeId::StartToDraft,
                conditions_met: vec![],
                actions_executed: vec![],
                transitioned_by: Uuid::new_v4(),
                context_changes: HashMap::new(),
            }),
            &started_event.metadata.identity,
            Some(ActorId::user(Uuid::new_v4())),
        );

        // Both events should have same correlation ID
        assert_eq!(started_event.correlation_id(), transitioned_event.correlation_id());
        
        // Caused event should have different causation ID
        assert_ne!(started_event.causation_id(), transitioned_event.causation_id());
        
        // Causation should point to parent message
        assert_eq!(transitioned_event.causation_id().0, started_event.message_id().0);
    }

    #[test]
    fn test_domain_event_conversion() {
        let instance_id = WorkflowInstanceId::new();
        let document_id = DocumentId::new();

        let workflow_event = CimWorkflowEvent::new_root(
            instance_id,
            document_id,
            WorkflowEventType::Completed(WorkflowCompletedEvent {
                instance_id,
                end_node: WorkflowNodeId::End,
                final_status: WorkflowStatus::Completed,
                completion_reason: "All tasks completed".to_string(),
                final_context: HashMap::new(),
                completed_by: Some(Uuid::new_v4()),
            }),
            Some(ActorId::system("workflow-engine")),
        );

        let domain_event = workflow_event.to_domain_event().unwrap();
        
        assert_eq!(domain_event.event_type, "workflow.completed");
        assert_eq!(domain_event.aggregate_id, instance_id.as_uuid().to_string());
        assert!(domain_event.payload.is_object());
    }
}