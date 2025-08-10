//! Document Workflow State Machine Implementation
//!
//! This module implements a comprehensive workflow state machine system for document management
//! using cim-graph principles. It provides configurable workflows, business rule validation,
//! automated actions, and comprehensive audit trails.

pub mod definitions;
pub mod simple_workflow;
pub mod event_integration;
pub mod persistence;
pub mod manager;
pub mod cim_events;
pub mod cim_engine;
pub mod event_integrity;
// TODO: Re-enable complex modules after simplification
// pub mod engine;
// pub mod guards; 
// pub mod actions;
// pub mod templates;

// Import definitions types with specific names
pub use definitions::{WorkflowDefinition, WorkflowGraph as ComplexWorkflowGraph, WorkflowNode as ComplexWorkflowNode, WorkflowEdge as ComplexWorkflowEdge};
// Import simple workflow types
pub use simple_workflow::*;
pub use event_integration::*;
pub use persistence::*;
pub use manager::*;
// CIM events - importing specific types to avoid conflicts
pub use cim_events::{
    CimWorkflowEvent, WorkflowStartedEvent, WorkflowTransitionedEvent,
    WorkflowCompletedEvent, WorkflowFailedEvent, NodeEnteredEvent, NodeExitedEvent,
    NodeExitReason, ActionExecutedEvent, ActionResult, ConditionEvaluatedEvent,
    WorkflowEventType as CimWorkflowEventType, // Renamed to avoid conflict
};
pub use cim_engine::*;
// Event integrity for CID chain verification
pub use event_integrity::{
    WorkflowEventIntegrity, WorkflowEventChain, WorkflowEventLink,
    ChainIntegrityStatus, IntegrityIssue, WorkflowIntegrityService,
    DefaultWorkflowIntegrityService, IntegrityError,
};

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use crate::value_objects::DocumentId;

/// Unique identifier for workflow definitions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(Uuid);

impl WorkflowId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn new_named(name: &str) -> Self {
        // Create deterministic UUID from name for consistent workflow IDs
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Convert hash to UUID format (this is a simple approach)
        let uuid_bytes = [
            (hash >> 56) as u8, (hash >> 48) as u8, (hash >> 40) as u8, (hash >> 32) as u8,
            (hash >> 24) as u8, (hash >> 16) as u8, (hash >> 8) as u8, hash as u8,
            0, 0, 0, 0, 0, 0, 0, 0
        ];
        
        Self(Uuid::from_bytes(uuid_bytes))
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
    
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for workflow instances
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowInstanceId(Uuid);

impl WorkflowInstanceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for WorkflowInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for workflow nodes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for workflow edges
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(String);

impl EdgeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for EdgeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EdgeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Workflow execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is actively running
    Running,
    /// Workflow is temporarily suspended
    Suspended,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with error
    Failed(String),
    /// Workflow was cancelled by user
    Cancelled,
    /// Workflow escalated due to SLA breach
    Escalated,
}

/// Workflow execution context containing runtime data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Dynamic variables available to workflow
    pub variables: HashMap<String, serde_json::Value>,
    /// User permissions within this workflow
    pub permissions: HashMap<Uuid, Vec<Permission>>,
    /// SLA deadlines for each node
    pub sla_deadlines: HashMap<NodeId, DateTime<Utc>>,
    /// Active escalation rules
    pub escalation_rules: Vec<EscalationRule>,
}

impl WorkflowContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            permissions: HashMap::new(),
            sla_deadlines: HashMap::new(),
            escalation_rules: Vec::new(),
        }
    }
    
    pub fn set_variable(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
    }
    
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }
    
    pub fn add_permission(&mut self, user_id: Uuid, permission: Permission) {
        self.permissions.entry(user_id).or_insert_with(Vec::new).push(permission);
    }
    
    pub fn has_permission(&self, user_id: &Uuid, permission: &Permission) -> bool {
        self.permissions.get(user_id)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }
}

impl Default for WorkflowContext {
    fn default() -> Self {
        Self::new()
    }
}

/// User permissions within workflow context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// Can view workflow status
    View,
    /// Can complete tasks
    CompleteTask,
    /// Can review documents
    Review,
    /// Can approve/reject items
    Approve,
    /// Can cancel workflow
    Cancel,
    /// Can modify workflow
    Modify,
    /// Administrative access
    Admin,
    /// Custom permission
    Custom(String),
}

/// Escalation rule for SLA breaches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    /// When to trigger escalation
    pub trigger_after: Duration,
    /// Users to escalate to
    pub escalate_to: Vec<Uuid>,
    /// Actions to take on escalation
    pub actions: Vec<WorkflowAction>,
    /// How often to repeat escalation
    pub repeat_interval: Option<Duration>,
}

/// Record of a workflow state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    /// Unique identifier for this transition
    pub id: Uuid,
    /// Source node
    pub from_node: NodeId,
    /// Destination node
    pub to_node: NodeId,
    /// When transition occurred
    pub transitioned_at: DateTime<Utc>,
    /// Who triggered the transition
    pub transitioned_by: Uuid,
    /// Reason for transition
    pub reason: Option<String>,
    /// Additional transition data
    pub data: HashMap<String, serde_json::Value>,
}

/// Workflow variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVariable {
    /// Variable name
    pub name: String,
    /// Variable type
    pub var_type: VariableType,
    /// Default value
    pub default_value: Option<serde_json::Value>,
    /// Whether variable is required
    pub required: bool,
    /// Variable description
    pub description: Option<String>,
}

/// Supported workflow variable types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Number,
    Boolean,
    DateTime,
    Duration,
    UserId,
    DocumentId,
    Json,
}

/// Time window specification for guards and conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Start time
    pub start: DateTime<Utc>,
    /// End time
    pub end: DateTime<Utc>,
}

impl TimeWindow {
    pub fn contains(&self, time: DateTime<Utc>) -> bool {
        time >= self.start && time <= self.end
    }
}

/// SLA status tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SLAStatus {
    /// Within SLA bounds
    OnTrack,
    /// Approaching SLA deadline
    AtRisk,
    /// SLA deadline breached
    Breached,
    /// No SLA defined
    NoSLA,
}

/// Simple workflow action for compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAction {
    pub action_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Simple guard for compatibility  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guard {
    pub condition: String,
}

/// User information for workflow assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub role: String,
}

/// Node information for workflow display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: NodeId,
    pub name: String,
    pub status: NodeStatus,
    pub assigned_users: Vec<UserInfo>,
    pub sla_deadline: Option<DateTime<Utc>>,
    pub sla_status: SLAStatus,
}

/// Status of individual workflow nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is waiting to be activated
    Pending,
    /// Node is currently active
    Active,
    /// Node has been completed
    Completed,
    /// Node was skipped
    Skipped,
    /// Node failed
    Failed(String),
}

/// Error types for workflow operations
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum WorkflowError {
    #[error("Workflow not found: {workflow_id}")]
    WorkflowNotFound { workflow_id: String },
    
    #[error("Invalid transition from {from} to {to}: {reason}")]
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },
    
    #[error("Guard evaluation failed: {guard} - {reason}")]
    GuardFailed {
        guard: String,
        reason: String,
    },
    
    #[error("Action execution failed: {action} - {error}")]
    ActionFailed {
        action: String,
        error: String,
    },
    
    #[error("Permission denied: {permission} required")]
    PermissionDenied { permission: String },
    
    #[error("SLA breach: {node} deadline {deadline}")]
    SLABreach {
        node: String,
        deadline: DateTime<Utc>,
    },
    
    #[error("Invalid workflow definition: {reason}")]
    InvalidDefinition { reason: String },
    
    #[error("Workflow engine error: {message}")]
    EngineError { message: String },
}

impl From<serde_json::Error> for WorkflowError {
    fn from(err: serde_json::Error) -> Self {
        WorkflowError::EngineError {
            message: format!("JSON serialization error: {}", err),
        }
    }
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_id_creation() {
        let id1 = WorkflowId::new();
        let id2 = WorkflowId::new();
        assert_ne!(id1, id2);
        
        let uuid = Uuid::new_v4();
        let id3 = WorkflowId::from_uuid(uuid);
        assert_eq!(id3.as_uuid(), &uuid);
    }
    
    #[test]
    fn test_workflow_context() {
        let mut context = WorkflowContext::new();
        
        // Test variables
        context.set_variable("test_var".into(), serde_json::json!("test_value"));
        assert_eq!(context.get_variable("test_var"), Some(&serde_json::json!("test_value")));
        
        // Test permissions
        let user_id = Uuid::new_v4();
        context.add_permission(user_id, Permission::View);
        assert!(context.has_permission(&user_id, &Permission::View));
        assert!(!context.has_permission(&user_id, &Permission::Admin));
    }
    
    #[test]
    fn test_time_window() {
        let start = Utc::now();
        let end = start + Duration::hours(2);
        let window = TimeWindow { start, end };
        
        assert!(window.contains(start + Duration::hours(1)));
        assert!(!window.contains(start + Duration::hours(3)));
    }
    
    #[test]
    fn test_node_id_creation() {
        let node1 = NodeId::new("test_node");
        let node2 = NodeId::from("test_node");
        assert_eq!(node1, node2);
        assert_eq!(node1.as_str(), "test_node");
    }
}