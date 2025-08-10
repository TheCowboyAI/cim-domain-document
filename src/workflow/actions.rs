//! Workflow Actions and Automation System
//!
//! This module implements workflow actions, automated triggers, and timer-based
//! workflow automation for document state machines.

use async_trait::async_trait;
use super::*;
use crate::value_objects::DocumentState;
use crate::Document;
use std::sync::Arc;
use tokio::time::{Duration as TokioDuration, interval};
use tracing::{info, warn, error, instrument};

/// Actions that can be executed during workflow transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowAction {
    /// Set document state
    SetState(DocumentState),
    /// Assign task to user
    AssignTask { user_id: Uuid, task_description: String },
    /// Send notification
    SendNotification { 
        notification_type: NotificationType,
        recipients: Vec<Uuid>,
        message: String,
        template: Option<String>,
    },
    /// Set deadline for workflow step
    SetDeadline { 
        node_id: NodeId,
        deadline: DateTime<Utc>,
    },
    /// Escalate to manager
    EscalateToManager { 
        user_id: Uuid,
        reason: String,
    },
    /// Call external system
    IntegrateWithSystem { 
        system_name: String,
        action: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    /// Update workflow context variables
    UpdateContext {
        variables: HashMap<String, serde_json::Value>,
    },
    /// Log workflow event
    LogEvent { 
        level: LogLevel,
        message: String,
        metadata: HashMap<String, serde_json::Value>,
    },
    /// Wait for specified duration
    Wait { duration: Duration },
    /// Cancel workflow
    CancelWorkflow { reason: String },
    /// Complete workflow
    CompleteWorkflow { 
        status: CompletionStatus,
        message: Option<String>,
    },
    /// Create new workflow instance
    StartChildWorkflow {
        workflow_id: WorkflowId,
        context: HashMap<String, serde_json::Value>,
    },
    /// Custom action
    Custom {
        action_name: String,
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// Types of notifications that can be sent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    Email,
    SMS,
    InApp,
    Slack,
    Teams,
    Webhook,
    Custom(String),
}

/// Logging levels for workflow events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// Context for action execution - now using owned data
#[derive(Debug, Clone)]
pub struct ActionContext {
    pub workflow_instance_id: WorkflowInstanceId,
    pub document_id: DocumentId,
    pub user_id: Uuid,
    pub variables: HashMap<String, serde_json::Value>,
    pub trigger_time: DateTime<Utc>,
    pub current_node: NodeId,
    pub target_node: Option<NodeId>,
}

impl ActionContext {
    pub fn new(
        workflow_instance_id: WorkflowInstanceId,
        document_id: DocumentId,
        user_id: Uuid,
        variables: HashMap<String, serde_json::Value>,
        current_node: NodeId,
    ) -> Self {
        Self {
            workflow_instance_id,
            document_id,
            user_id,
            variables,
            trigger_time: Utc::now(),
            current_node,
            target_node: None,
        }
    }
    
    pub fn with_target_node(mut self, target: NodeId) -> Self {
        self.target_node = Some(target);
        self
    }
    
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }
    
    pub fn set_variable(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
    }
}

/// Result of action execution
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Action completed successfully
    Success,
    /// Action completed with warnings
    Warning(String),
    /// Action failed with error
    Error(String),
    /// Action requires manual intervention
    RequiresIntervention(String),
    /// Action is pending completion
    Pending,
}

/// Trait for executing workflow actions - now object-safe with async_trait
#[async_trait]
pub trait ActionExecutor: Send + Sync + std::fmt::Debug {
    async fn execute_action(
        &self,
        action: &WorkflowAction,
        context: &mut ActionContext,
    ) -> WorkflowResult<ActionResult>;
}

/// Default implementation of action executor
#[derive(Debug)]
pub struct DefaultActionExecutor {
    /// Notification service
    pub notification_service: Option<Arc<dyn NotificationService>>,
    /// External integration service
    pub integration_service: Option<Arc<dyn IntegrationService>>,
}

impl DefaultActionExecutor {
    pub fn new() -> Self {
        Self {
            notification_service: None,
            integration_service: None,
        }
    }
    
    pub fn with_notification_service(
        mut self,
        service: Arc<dyn NotificationService>,
    ) -> Self {
        self.notification_service = Some(service);
        self
    }
    
    pub fn with_integration_service(
        mut self,
        service: Arc<dyn IntegrationService>,
    ) -> Self {
        self.integration_service = Some(service);
        self
    }
}

impl Default for DefaultActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ActionExecutor for DefaultActionExecutor {
    async fn execute_action(
        &self,
        action: &WorkflowAction,
        context: &mut ActionContext,
    ) -> WorkflowResult<ActionResult> {
        match action {
            WorkflowAction::SetState(new_state) => {
                context.set_variable("document.state".to_string(), serde_json::to_value(new_state)?);
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::AssignTask { user_id, task_description } => {
                // Add task assignment to context
                let assignment = serde_json::json!({
                    "user_id": user_id,
                    "task": task_description,
                    "assigned_at": context.trigger_time,
                    "node_id": context.current_node
                });
                
                let mut assignments = context.get_variable("assignments")
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                assignments.push(assignment);
                
                context.set_variable("assignments".to_string(), serde_json::Value::Array(assignments));
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::SendNotification { notification_type, recipients, message, template } => {
                if let Some(service) = &self.notification_service {
                    let notification = Notification {
                        notification_type: notification_type.clone(),
                        recipients: recipients.clone(),
                        message: message.clone(),
                        template: template.clone(),
                        metadata: HashMap::new(),
                    };
                    
                    match service.send_notification(notification).await {
                        Ok(_) => Ok(ActionResult::Success),
                        Err(e) => Ok(ActionResult::Error(e.to_string())),
                    }
                } else {
                    Ok(ActionResult::Warning("No notification service configured".to_string()))
                }
            }
            
            WorkflowAction::SetDeadline { node_id, deadline } => {
                let deadline_info = serde_json::json!({
                    "node_id": node_id,
                    "deadline": deadline
                });
                context.set_variable("sla_deadline".to_string(), deadline_info);
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::EscalateToManager { user_id, reason } => {
                // Find user's manager and escalate
                let escalation = serde_json::json!({
                    "user_id": user_id,
                    "reason": reason,
                    "escalated_at": context.trigger_time,
                    "node_id": context.current_node
                });
                
                let mut escalations = context.get_variable("escalations")
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                escalations.push(escalation);
                
                context.set_variable("escalations".to_string(), serde_json::Value::Array(escalations));
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::IntegrateWithSystem { system_name, action: sys_action, parameters } => {
                if let Some(service) = &self.integration_service {
                    let request = IntegrationRequest {
                        system: system_name.clone(),
                        action: sys_action.clone(),
                        parameters: parameters.clone(),
                        context: serde_json::json!({
                            "workflow_instance_id": context.workflow_instance_id,
                            "document_id": context.document_id,
                            "node_id": context.current_node
                        }),
                    };
                    
                    match service.execute_integration(request).await {
                        Ok(response) => {
                            context.set_variable("integration_response".to_string(), response.result);
                            Ok(ActionResult::Success)
                        }
                        Err(e) => Ok(ActionResult::Error(e.to_string())),
                    }
                } else {
                    Ok(ActionResult::Warning("No integration service configured".to_string()))
                }
            }
            
            WorkflowAction::UpdateContext { variables } => {
                for (key, value) in variables {
                    context.set_variable(key.clone(), value.clone());
                }
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::LogEvent { level, message, metadata } => {
                // In a real implementation, this would use a proper logging system
                println!("[{:?}] {}: {} - {:?}", level, context.current_node.as_str(), message, metadata);
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::Wait { duration } => {
                // For async wait, we would need to schedule this in the workflow engine
                // For now, we just record the wait requirement
                context.set_variable("wait_until".to_string(), 
                    serde_json::to_value(context.trigger_time + *duration)?);
                Ok(ActionResult::Pending)
            }
            
            WorkflowAction::CancelWorkflow { reason } => {
                // Record cancellation in context - engine will handle status update
                context.set_variable("cancellation_reason".to_string(), 
                    serde_json::Value::String(reason.clone()));
                context.set_variable("workflow_status".to_string(), 
                    serde_json::Value::String("cancelled".to_string()));
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::CompleteWorkflow { status, message } => {
                // Record completion in context - engine will handle status update
                context.set_variable("workflow_status".to_string(), 
                    serde_json::Value::String("completed".to_string()));
                context.set_variable("completed_at".to_string(), 
                    serde_json::to_value(context.trigger_time)?);
                
                if let Some(msg) = message {
                    context.set_variable("completion_message".to_string(), 
                        serde_json::Value::String(msg.clone()));
                }
                
                context.set_variable("completion_status".to_string(), 
                    serde_json::to_value(status)?);
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::StartChildWorkflow { workflow_id, context: child_context } => {
                // This would typically be handled by the workflow engine
                let child_workflow = serde_json::json!({
                    "parent_instance_id": context.workflow_instance_id,
                    "workflow_id": workflow_id,
                    "context": child_context,
                    "started_at": context.trigger_time
                });
                
                let mut child_workflows = context.get_variable("child_workflows")
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                child_workflows.push(child_workflow);
                
                context.set_variable("child_workflows".to_string(), 
                    serde_json::Value::Array(child_workflows));
                Ok(ActionResult::Pending)
            }
            
            WorkflowAction::Custom { action_name, parameters } => {
                // Record custom action execution - would be handled by specialized executors
                context.set_variable("custom_action".to_string(), serde_json::json!({
                    "action_name": action_name,
                    "parameters": parameters,
                    "executed_at": context.trigger_time
                }));
                Ok(ActionResult::Success)
            }
        }
    }
}

/// Workflow automation triggers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowTrigger {
    /// Document was uploaded
    DocumentUploaded,
    /// Document state changed
    StateChanged(DocumentState),
    /// Timer expired for specific node
    TimerExpired(NodeId),
    /// External event received
    ExternalEvent(String),
    /// User performed specific action
    UserAction(String),
    /// Scheduled time reached
    ScheduledTime(DateTime<Utc>),
    /// SLA deadline breached
    SLABreach(NodeId),
    /// Node completed
    NodeCompleted(NodeId),
    /// Workflow started
    WorkflowStarted,
    /// Custom trigger
    Custom(String),
}

/// Automated workflow action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedAction {
    /// Trigger that activates this action
    pub trigger: WorkflowTrigger,
    /// Conditions that must be met (optional)
    pub conditions: Vec<Condition>,
    /// Actions to execute
    pub actions: Vec<WorkflowAction>,
    /// Delay before executing actions
    pub delay: Option<Duration>,
    /// Priority for execution ordering
    pub priority: i32,
    /// Whether this automation is active
    pub is_active: bool,
}

impl AutomatedAction {
    pub fn new(trigger: WorkflowTrigger) -> Self {
        Self {
            trigger,
            conditions: Vec::new(),
            actions: Vec::new(),
            delay: None,
            priority: 0,
            is_active: true,
        }
    }
    
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }
    
    pub fn with_action(mut self, action: WorkflowAction) -> Self {
        self.actions.push(action);
        self
    }
    
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
    
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Scheduled workflow task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub workflow_instance_id: WorkflowInstanceId,
    pub node_id: NodeId,
    pub scheduled_time: DateTime<Utc>,
    pub task_type: ScheduledTaskType,
    pub actions: Vec<WorkflowAction>,
    pub is_active: bool,
}

/// Types of scheduled tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduledTaskType {
    /// Reminder notification
    Reminder,
    /// SLA deadline check
    SLADeadline,
    /// Escalation trigger
    Escalation,
    /// Automatic state transition
    AutoTransition,
    /// Cleanup task
    Cleanup,
    /// Custom scheduled task
    Custom(String),
}

/// Workflow automation engine
#[derive(Debug)]
pub struct WorkflowAutomationEngine {
    /// Registered automated actions by trigger
    pub triggers: HashMap<WorkflowTrigger, Vec<AutomatedAction>>,
    /// Active scheduled tasks
    pub scheduled_tasks: HashMap<WorkflowInstanceId, Vec<ScheduledTask>>,
    /// Action executor
    pub action_executor: Arc<dyn ActionExecutor>,
}

impl WorkflowAutomationEngine {
    pub fn new() -> Self {
        Self {
            triggers: HashMap::new(),
            scheduled_tasks: HashMap::new(),
            action_executor: Arc::new(DefaultActionExecutor::new()),
        }
    }
    
    pub fn with_action_executor(
        mut self,
        executor: Arc<dyn ActionExecutor>,
    ) -> Self {
        self.action_executor = executor;
        self
    }
    
    pub fn register_automation(&mut self, automation: AutomatedAction) {
        self.triggers
            .entry(automation.trigger.clone())
            .or_insert_with(Vec::new)
            .push(automation);
    }
    
    pub fn schedule_task(&mut self, task: ScheduledTask) {
        self.scheduled_tasks
            .entry(task.workflow_instance_id)
            .or_insert_with(Vec::new)
            .push(task);
    }
    
    /// Process trigger and execute matching automations
    /// TODO: Re-enable after WorkflowInstance, WorkflowDefinition, User types are defined
    /*
    pub async fn process_trigger(
        &self,
        trigger: WorkflowTrigger,
    ) -> WorkflowResult<Vec<ActionResult>> {
        // Placeholder implementation until dependencies are resolved
        Ok(Vec::new())
    }
    */
    
    /// Process scheduled tasks that are due
    pub async fn process_scheduled_tasks(
        &mut self,
        current_time: DateTime<Utc>,
    ) -> WorkflowResult<Vec<(WorkflowInstanceId, NodeId, Vec<ActionResult>)>> {
        let mut results = Vec::new();
        
        for (instance_id, tasks) in &mut self.scheduled_tasks {
            let due_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.is_active && task.scheduled_time <= current_time)
                .cloned()
                .collect();
            
            for task in due_tasks {
                // Mark task as processed
                if let Some(task_ref) = tasks.iter_mut().find(|t| t.id == task.id) {
                    task_ref.is_active = false;
                }
                
                // This would need actual workflow instance, document, and user context
                // For now, we'll just record the results
                let task_results = Vec::new(); // Placeholder
                results.push((instance_id.clone(), task.node_id.clone(), task_results));
            }
        }
        
        Ok(results)
    }
    
    /// Start automation engine with periodic scheduled task processing
    pub async fn start_engine(&mut self) -> WorkflowResult<()> {
        let mut interval = interval(TokioDuration::from_secs(60)); // Check every minute
        
        loop {
            interval.tick().await;
            
            let current_time = Utc::now();
            match self.process_scheduled_tasks(current_time).await {
                Ok(results) => {
                    // Log successful task processing
                    for (instance_id, node_id, action_results) in results {
                        println!("Processed scheduled tasks for instance {} at node {}: {} actions", 
                                instance_id.as_uuid(), node_id.as_str(), action_results.len());
                    }
                }
                Err(e) => {
                    eprintln!("Error processing scheduled tasks: {}", e);
                }
            }
        }
    }
}

impl Default for WorkflowAutomationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification service trait - object-safe with async_trait
#[async_trait]
pub trait NotificationService: Send + Sync + std::fmt::Debug {
    async fn send_notification(&self, notification: Notification) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Notification structure
#[derive(Debug, Clone)]
pub struct Notification {
    pub notification_type: NotificationType,
    pub recipients: Vec<Uuid>,
    pub message: String,
    pub template: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// External system integration service - object-safe with async_trait
#[async_trait]
pub trait IntegrationService: Send + Sync + std::fmt::Debug {
    async fn execute_integration(&self, request: IntegrationRequest) -> Result<IntegrationResponse, Box<dyn std::error::Error + Send + Sync>>;
}

/// Integration request structure
#[derive(Debug, Clone)]
pub struct IntegrationRequest {
    pub system: String,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: serde_json::Value,
}

/// Integration response structure
#[derive(Debug, Clone)]
pub struct IntegrationResponse {
    pub success: bool,
    pub result: serde_json::Value,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::DocumentInfoComponent;

    fn create_test_document() -> Document {
        let mut doc = Document::new(DocumentId::new());
        let info = DocumentInfoComponent {
            title: "Test Document".to_string(),
            description: None,
            mime_type: "text/plain".to_string(),
            filename: None,
            size_bytes: 1024,
            language: None,
        };
        doc.add_component(info, &Uuid::new_v4(), None).unwrap();
        doc
    }
    
    fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            roles: vec!["author".to_string()],
            permissions: vec![Permission::View],
            department: None,
            manager_id: None,
        }
    }

    #[tokio::test]
    async fn test_action_execution() {
        let executor = DefaultActionExecutor::new();
        let document = create_test_document();
        let user = create_test_user();
        let workflow_def = WorkflowDefinition::new(
            "Test Workflow".to_string(),
            "Test".to_string(),
            user.id,
        );
        let mut instance = WorkflowInstance::new(
            workflow_def.id.clone(),
            DocumentId::new(),
            user.id,
            vec![NodeId::new("start")],
        );
        let node_id = NodeId::new("start");
        let mut variables = HashMap::new();
        
        let mut context = ActionContext {
            workflow_instance: &mut instance,
            workflow_definition: &workflow_def,
            document: &document,
            user: &user,
            current_node: &node_id,
            trigger_time: Utc::now(),
            variables: &mut variables,
        };
        
        // Test SetState action
        let action = WorkflowAction::SetState(DocumentState::InReview);
        let result = executor.execute_action(&action, &mut context).await.unwrap();
        assert!(matches!(result, ActionResult::Success));
        
        // Check that state was set in variables
        assert!(context.variables.contains_key("document.state"));
    }
    
    #[test]
    fn test_automated_action_builder() {
        let automation = AutomatedAction::new(WorkflowTrigger::DocumentUploaded)
            .with_condition(Condition::boolean("true".to_string()))
            .with_action(WorkflowAction::SetState(DocumentState::Draft))
            .with_delay(Duration::hours(1))
            .with_priority(10);
        
        assert_eq!(automation.trigger, WorkflowTrigger::DocumentUploaded);
        assert_eq!(automation.conditions.len(), 1);
        assert_eq!(automation.actions.len(), 1);
        assert_eq!(automation.delay, Some(Duration::hours(1)));
        assert_eq!(automation.priority, 10);
        assert!(automation.is_active);
    }
    
    #[test]
    fn test_scheduled_task_creation() {
        let task = ScheduledTask {
            id: Uuid::new_v4(),
            workflow_instance_id: WorkflowInstanceId::new(),
            node_id: NodeId::new("review"),
            scheduled_time: Utc::now() + Duration::hours(24),
            task_type: ScheduledTaskType::Reminder,
            actions: vec![
                WorkflowAction::SendNotification {
                    notification_type: NotificationType::Email,
                    recipients: vec![Uuid::new_v4()],
                    message: "Review reminder".to_string(),
                    template: Some("review_reminder".to_string()),
                }
            ],
            is_active: true,
        };
        
        assert!(task.is_active);
        assert_eq!(task.task_type, ScheduledTaskType::Reminder);
        assert_eq!(task.actions.len(), 1);
    }
    
    #[test]
    fn test_workflow_automation_engine() {
        let mut engine = WorkflowAutomationEngine::new();
        
        let automation = AutomatedAction::new(WorkflowTrigger::DocumentUploaded)
            .with_action(WorkflowAction::SetState(DocumentState::Draft));
        
        engine.register_automation(automation);
        
        assert!(engine.triggers.contains_key(&WorkflowTrigger::DocumentUploaded));
        assert_eq!(engine.triggers[&WorkflowTrigger::DocumentUploaded].len(), 1);
    }
}