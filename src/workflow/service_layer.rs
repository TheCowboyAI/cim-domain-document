//! Service Layer Architecture for Workflow System
//!
//! This module demonstrates the service layer pattern that provides
//! clean separation of concerns and improved testability.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, error, instrument};

use crate::value_objects::{DocumentId, DocumentState};
use super::refactored_actions::{
    ActionExecutor, ActionContext, WorkflowAction, ActionResult,
    DocumentService, NotificationService, IntegrationService
};
use super::{
    WorkflowResult, WorkflowError, WorkflowId, WorkflowInstanceId, NodeId,
    WorkflowDefinition, WorkflowInstance
};

/// Main workflow service - entry point for all workflow operations
#[derive(Debug)]
pub struct WorkflowService {
    engine: Arc<WorkflowEngine>,
    action_service: Arc<ActionService>,
    guard_service: Arc<GuardService>,
    event_bus: Arc<EventBus>,
}

impl WorkflowService {
    pub fn new(
        engine: Arc<WorkflowEngine>,
        action_service: Arc<ActionService>,
        guard_service: Arc<GuardService>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            engine,
            action_service,
            guard_service,
            event_bus,
        }
    }
    
    /// Start a new workflow instance
    #[instrument(skip(self))]
    pub async fn start_workflow(
        &self,
        workflow_id: WorkflowId,
        document_id: DocumentId,
        initiated_by: Uuid,
        initial_variables: HashMap<String, serde_json::Value>,
    ) -> WorkflowResult<WorkflowInstanceId> {
        info!("Starting workflow {} for document {}", workflow_id.as_str(), document_id);
        
        // Create new instance
        let instance_id = WorkflowInstanceId::new();
        let instance = WorkflowInstance::new(
            instance_id,
            workflow_id.clone(),
            document_id,
            initiated_by,
            initial_variables,
        );
        
        // Store instance
        self.engine.create_instance(instance).await?;
        
        // Publish event
        self.event_bus.publish(WorkflowEvent::WorkflowStarted {
            instance_id,
            workflow_id,
            document_id,
            initiated_by,
            started_at: Utc::now(),
        }).await;
        
        info!("Workflow instance {} started successfully", instance_id);
        Ok(instance_id)
    }
    
    /// Execute a state transition
    #[instrument(skip(self))]
    pub async fn execute_transition(
        &self,
        instance_id: WorkflowInstanceId,
        to_node: NodeId,
        user_id: Uuid,
    ) -> WorkflowResult<TransitionResult> {
        info!("Executing transition for instance {} to node {}", instance_id, to_node.as_str());
        
        // Load instance
        let instance = self.engine.get_instance(instance_id).await?;
        
        // Check guards
        let can_transition = self.guard_service.can_transition(
            &instance.workflow_id,
            &instance.current_node,
            &to_node,
            &GuardContext::new(&instance, user_id),
        ).await?;
        
        if !can_transition {
            return Err(WorkflowError::InvalidTransition {
                from: instance.current_node.as_str().to_string(),
                to: to_node.as_str().to_string(),
                reason: "Guard conditions not met".to_string(),
            });
        }
        
        // Execute transition
        let transition_result = self.engine.execute_transition(instance_id, to_node.clone()).await?;
        
        // Execute actions
        let mut action_results = Vec::new();
        for action in &transition_result.actions {
            let mut action_context = ActionContext::new(
                instance_id,
                instance.document_id,
                user_id,
                instance.variables.clone(),
                to_node.clone(),
            );
            
            let result = self.action_service.execute_action(action, &mut action_context).await?;
            action_results.push(result);
            
            // Update instance variables with any changes from action context
            self.engine.update_variables(instance_id, action_context.variables).await?;
        }
        
        // Publish event
        self.event_bus.publish(WorkflowEvent::TransitionCompleted {
            instance_id,
            from_node: instance.current_node,
            to_node: to_node.clone(),
            user_id,
            completed_at: Utc::now(),
            action_results: action_results.clone(),
        }).await;
        
        info!("Transition completed successfully for instance {}", instance_id);
        Ok(TransitionResult {
            instance_id,
            from_node: instance.current_node,
            to_node,
            actions: transition_result.actions,
            action_results,
        })
    }
    
    /// Get active workflow instances for a document
    pub async fn get_active_instances_for_document(
        &self,
        document_id: DocumentId,
    ) -> WorkflowResult<Vec<WorkflowInstance>> {
        self.engine.get_instances_for_document(document_id).await
    }
    
    /// Cancel a workflow instance
    #[instrument(skip(self))]
    pub async fn cancel_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        cancelled_by: Uuid,
        reason: String,
    ) -> WorkflowResult<()> {
        info!("Cancelling workflow instance {} - reason: {}", instance_id, reason);
        
        self.engine.cancel_instance(instance_id, cancelled_by, reason.clone()).await?;
        
        self.event_bus.publish(WorkflowEvent::WorkflowCancelled {
            instance_id,
            cancelled_by,
            reason,
            cancelled_at: Utc::now(),
        }).await;
        
        Ok(())
    }
}

/// Centralized action execution service
#[derive(Debug)]
pub struct ActionService {
    executors: HashMap<String, Arc<dyn ActionExecutor>>,
    default_executor: Arc<dyn ActionExecutor>,
}

impl ActionService {
    pub fn new(default_executor: Arc<dyn ActionExecutor>) -> Self {
        Self {
            executors: HashMap::new(),
            default_executor,
        }
    }
    
    pub fn register_executor(&mut self, action_type: String, executor: Arc<dyn ActionExecutor>) {
        self.executors.insert(action_type, executor);
    }
    
    #[instrument(skip(self, context))]
    pub async fn execute_action(
        &self,
        action: &WorkflowAction,
        context: &mut ActionContext,
    ) -> WorkflowResult<ActionResult> {
        // Get appropriate executor
        let executor = self.executors
            .get(&format!("{:?}", action))
            .unwrap_or(&self.default_executor);
        
        // Execute with error handling
        match executor.execute_action(action, context).await {
            Ok(result) => {
                info!("Action executed successfully: {:?} -> {:?}", action, result);
                Ok(result)
            }
            Err(e) => {
                error!("Action execution failed: {:?} - {}", action, e);
                Err(e)
            }
        }
    }
}

/// Guard evaluation service
#[derive(Debug)]
pub struct GuardService {
    // In real implementation, this would contain guard evaluators
}

impl GuardService {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn can_transition(
        &self,
        _workflow_id: &WorkflowId,
        _from_node: &NodeId,
        _to_node: &NodeId,
        _context: &GuardContext,
    ) -> WorkflowResult<bool> {
        // Simplified implementation - in real version would evaluate guards
        Ok(true)
    }
}

/// Event bus for workflow events
#[derive(Debug)]
pub struct EventBus {
    handlers: Arc<RwLock<Vec<Arc<dyn WorkflowEventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn register_handler(&self, handler: Arc<dyn WorkflowEventHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }
    
    pub async fn publish(&self, event: WorkflowEvent) {
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            handler.handle_event(event.clone()).await;
        }
    }
}

/// Workflow engine - manages instances and definitions
#[derive(Debug)]
pub struct WorkflowEngine {
    instances: Arc<RwLock<HashMap<WorkflowInstanceId, WorkflowInstance>>>,
    definitions: Arc<RwLock<HashMap<WorkflowId, WorkflowDefinition>>>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            definitions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn create_instance(&self, instance: WorkflowInstance) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id, instance);
        Ok(())
    }
    
    pub async fn get_instance(&self, instance_id: WorkflowInstanceId) -> WorkflowResult<WorkflowInstance> {
        let instances = self.instances.read().await;
        instances.get(&instance_id)
            .cloned()
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: instance_id.to_string(),
            })
    }
    
    pub async fn execute_transition(
        &self,
        instance_id: WorkflowInstanceId,
        to_node: NodeId,
    ) -> WorkflowResult<EngineTransitionResult> {
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(&instance_id)
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: instance_id.to_string(),
            })?;
        
        let from_node = instance.current_node.clone();
        instance.current_node = to_node.clone();
        instance.updated_at = Utc::now();
        
        // In real implementation, would load workflow definition and get actions
        let actions = vec![]; // Placeholder
        
        Ok(EngineTransitionResult {
            from_node,
            to_node,
            actions,
        })
    }
    
    pub async fn update_variables(
        &self,
        instance_id: WorkflowInstanceId,
        variables: HashMap<String, serde_json::Value>,
    ) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(&instance_id)
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: instance_id.to_string(),
            })?;
        
        instance.variables = variables;
        instance.updated_at = Utc::now();
        Ok(())
    }
    
    pub async fn get_instances_for_document(
        &self,
        document_id: DocumentId,
    ) -> WorkflowResult<Vec<WorkflowInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.values()
            .filter(|instance| instance.document_id == document_id)
            .cloned()
            .collect())
    }
    
    pub async fn cancel_instance(
        &self,
        instance_id: WorkflowInstanceId,
        _cancelled_by: Uuid,
        _reason: String,
    ) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        instances.remove(&instance_id);
        Ok(())
    }
}

/// Workflow events
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    WorkflowStarted {
        instance_id: WorkflowInstanceId,
        workflow_id: WorkflowId,
        document_id: DocumentId,
        initiated_by: Uuid,
        started_at: DateTime<Utc>,
    },
    TransitionCompleted {
        instance_id: WorkflowInstanceId,
        from_node: NodeId,
        to_node: NodeId,
        user_id: Uuid,
        completed_at: DateTime<Utc>,
        action_results: Vec<ActionResult>,
    },
    WorkflowCancelled {
        instance_id: WorkflowInstanceId,
        cancelled_by: Uuid,
        reason: String,
        cancelled_at: DateTime<Utc>,
    },
}

/// Event handler trait
#[async_trait]
pub trait WorkflowEventHandler: Send + Sync {
    async fn handle_event(&self, event: WorkflowEvent);
}

/// Supporting types
#[derive(Debug, Clone)]
pub struct TransitionResult {
    pub instance_id: WorkflowInstanceId,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub actions: Vec<WorkflowAction>,
    pub action_results: Vec<ActionResult>,
}

#[derive(Debug, Clone)]
pub struct EngineTransitionResult {
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub actions: Vec<WorkflowAction>,
}

#[derive(Debug, Clone)]
pub struct GuardContext {
    pub instance: WorkflowInstance,
    pub user_id: Uuid,
    pub evaluated_at: DateTime<Utc>,
}

impl GuardContext {
    pub fn new(instance: &WorkflowInstance, user_id: Uuid) -> Self {
        Self {
            instance: instance.clone(),
            user_id,
            evaluated_at: Utc::now(),
        }
    }
}

/// Example event handler for logging
#[derive(Debug)]
pub struct LoggingEventHandler;

#[async_trait]
impl WorkflowEventHandler for LoggingEventHandler {
    async fn handle_event(&self, event: WorkflowEvent) {
        match event {
            WorkflowEvent::WorkflowStarted { instance_id, workflow_id, .. } => {
                info!("📋 Workflow started: {} (instance: {})", workflow_id.as_str(), instance_id);
            }
            WorkflowEvent::TransitionCompleted { instance_id, from_node, to_node, .. } => {
                info!("🔄 Transition completed: {} → {} (instance: {})", 
                      from_node.as_str(), to_node.as_str(), instance_id);
            }
            WorkflowEvent::WorkflowCancelled { instance_id, reason, .. } => {
                warn!("❌ Workflow cancelled: {} - {}", instance_id, reason);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::refactored_actions::mocks::*;
    use tokio_test;
    
    async fn create_test_workflow_service() -> WorkflowService {
        let engine = Arc::new(WorkflowEngine::new());
        
        let document_service = Arc::new(MockDocumentService);
        let default_executor = Arc::new(super::super::refactored_actions::DefaultActionExecutor::new(
            document_service,
            None,
            None,
        ));
        let action_service = Arc::new(ActionService::new(default_executor));
        
        let guard_service = Arc::new(GuardService::new());
        let event_bus = Arc::new(EventBus::new());
        
        // Register logging event handler
        let logging_handler = Arc::new(LoggingEventHandler);
        event_bus.register_handler(logging_handler).await;
        
        WorkflowService::new(engine, action_service, guard_service, event_bus)
    }
    
    #[tokio::test]
    async fn test_start_workflow() {
        // Arrange
        let service = create_test_workflow_service().await;
        let workflow_id = WorkflowId::new("test_workflow");
        let document_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        
        // Act
        let instance_id = service.start_workflow(
            workflow_id.clone(),
            document_id,
            user_id,
            HashMap::new(),
        ).await.unwrap();
        
        // Assert
        assert!(!instance_id.to_string().is_empty());
        
        // Verify instance was created
        let instances = service.get_active_instances_for_document(document_id).await.unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].workflow_id, workflow_id);
    }
    
    #[tokio::test]
    async fn test_cancel_workflow() {
        // Arrange
        let service = create_test_workflow_service().await;
        let workflow_id = WorkflowId::new("test_workflow");
        let document_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        
        let instance_id = service.start_workflow(
            workflow_id,
            document_id,
            user_id,
            HashMap::new(),
        ).await.unwrap();
        
        // Act
        service.cancel_workflow(
            instance_id,
            user_id,
            "Test cancellation".to_string(),
        ).await.unwrap();
        
        // Assert
        let instances = service.get_active_instances_for_document(document_id).await.unwrap();
        assert_eq!(instances.len(), 0);
    }
}