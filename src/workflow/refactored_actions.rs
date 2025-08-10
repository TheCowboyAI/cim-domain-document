//! Refactored Workflow Actions - Template Implementation
//!
//! This module shows the refactored approach for workflow actions that resolves
//! the compilation issues while maintaining functionality.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::Document;
use crate::value_objects::{DocumentId, DocumentState};
use super::{WorkflowResult, WorkflowError, NodeId, WorkflowInstanceId};

/// Actions that can be executed during workflow transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowAction {
    /// Set document state
    SetState(DocumentState),
    /// Send notification to user
    SendNotification {
        recipient: String,
        template: String,
        variables: HashMap<String, String>,
    },
    /// Assign task to user
    AssignTask {
        assignee: Uuid,
        task_type: String,
        description: String,
    },
    /// Create timer for delayed action
    SetTimer {
        duration: chrono::Duration,
        action: Box<WorkflowAction>,
    },
    /// Execute external integration
    CallIntegration {
        service: String,
        method: String,
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// Result of action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Success,
    Partial { completed: Vec<String>, failed: Vec<String> },
    Failed { error: String },
    Deferred { retry_after: DateTime<Utc> },
}

/// Context passed to action executors - now using owned data
#[derive(Debug, Clone)]
pub struct ActionContext {
    /// Workflow instance ID
    pub workflow_instance_id: WorkflowInstanceId,
    /// Document being processed
    pub document_id: DocumentId,
    /// User initiating the action
    pub user_id: Uuid,
    /// Workflow variables (owned copy)
    pub variables: HashMap<String, serde_json::Value>,
    /// When the action was triggered
    pub trigger_time: DateTime<Utc>,
    /// Current workflow node
    pub current_node: NodeId,
    /// Target node (for transitions)
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

/// Trait for executing workflow actions - now object-safe with async_trait
#[async_trait]
pub trait ActionExecutor: Send + Sync + std::fmt::Debug {
    async fn execute_action(
        &self,
        action: &WorkflowAction,
        context: &mut ActionContext,
    ) -> WorkflowResult<ActionResult>;
}

/// Service interfaces - also made object-safe
#[async_trait]
pub trait DocumentService: Send + Sync {
    async fn update_state(&self, document_id: DocumentId, state: DocumentState) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_document(&self, document_id: DocumentId) -> Result<Document, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait NotificationService: Send + Sync + std::fmt::Debug {
    async fn send_notification(&self, notification: Notification) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait IntegrationService: Send + Sync + std::fmt::Debug {
    async fn execute_integration(&self, request: IntegrationRequest) -> Result<IntegrationResponse, Box<dyn std::error::Error + Send + Sync>>;
}

/// Notification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub recipient: String,
    pub template: String,
    pub variables: HashMap<String, String>,
    pub priority: NotificationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Integration request/response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationRequest {
    pub service: String,
    pub method: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

/// Default implementation that works with the new architecture
#[derive(Debug)]
pub struct DefaultActionExecutor {
    document_service: Arc<dyn DocumentService>,
    notification_service: Option<Arc<dyn NotificationService>>,
    integration_service: Option<Arc<dyn IntegrationService>>,
}

impl DefaultActionExecutor {
    pub fn new(
        document_service: Arc<dyn DocumentService>,
        notification_service: Option<Arc<dyn NotificationService>>,
        integration_service: Option<Arc<dyn IntegrationService>>,
    ) -> Self {
        Self {
            document_service,
            notification_service,
            integration_service,
        }
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
            WorkflowAction::SetState(state) => {
                self.document_service
                    .update_state(context.document_id, *state)
                    .await
                    .map_err(|e| WorkflowError::ActionFailed {
                        action: format!("{:?}", action),
                        error: e.to_string(),
                    })?;
                
                // Update context variables with new state
                context.set_variable("document.state".to_string(), serde_json::to_value(state)?);
                
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::SendNotification { recipient, template, variables } => {
                if let Some(service) = &self.notification_service {
                    let notification = Notification {
                        recipient: recipient.clone(),
                        template: template.clone(),
                        variables: variables.clone(),
                        priority: NotificationPriority::Normal,
                    };
                    
                    service.send_notification(notification).await
                        .map_err(|e| WorkflowError::ActionFailed {
                            action: format!("{:?}", action),
                            error: e.to_string(),
                        })?;
                    
                    Ok(ActionResult::Success)
                } else {
                    Ok(ActionResult::Failed {
                        error: "Notification service not available".to_string(),
                    })
                }
            }
            
            WorkflowAction::AssignTask { assignee, task_type, description } => {
                // Update variables to track task assignment
                context.set_variable("task.assignee".to_string(), serde_json::to_value(assignee)?);
                context.set_variable("task.type".to_string(), serde_json::Value::String(task_type.clone()));
                context.set_variable("task.description".to_string(), serde_json::Value::String(description.clone()));
                context.set_variable("task.assigned_at".to_string(), serde_json::to_value(Utc::now())?);
                
                Ok(ActionResult::Success)
            }
            
            WorkflowAction::SetTimer { duration, action: _ } => {
                // Calculate future execution time
                let execute_at = context.trigger_time + *duration;
                context.set_variable("timer.execute_at".to_string(), serde_json::to_value(execute_at)?);
                
                Ok(ActionResult::Deferred { retry_after: execute_at })
            }
            
            WorkflowAction::CallIntegration { service, method, parameters } => {
                if let Some(integration_service) = &self.integration_service {
                    let request = IntegrationRequest {
                        service: service.clone(),
                        method: method.clone(),
                        parameters: parameters.clone(),
                    };
                    
                    let response = integration_service.execute_integration(request).await
                        .map_err(|e| WorkflowError::ActionFailed {
                            action: format!("{:?}", action),
                            error: e.to_string(),
                        })?;
                    
                    // Store response in context
                    context.set_variable("integration.response".to_string(), serde_json::to_value(response.clone())?);
                    
                    if response.success {
                        Ok(ActionResult::Success)
                    } else {
                        Ok(ActionResult::Failed {
                            error: response.error.unwrap_or_else(|| "Unknown integration error".to_string()),
                        })
                    }
                } else {
                    Ok(ActionResult::Failed {
                        error: "Integration service not available".to_string(),
                    })
                }
            }
        }
    }
}

/// Mock implementations for testing
#[cfg(test)]
pub mod mocks {
    use super::*;
    use async_trait::async_trait;
    
    #[derive(Debug)]
    pub struct MockDocumentService;
    
    #[async_trait]
    impl DocumentService for MockDocumentService {
        async fn update_state(&self, _document_id: DocumentId, _state: DocumentState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        async fn get_document(&self, _document_id: DocumentId) -> Result<Document, Box<dyn std::error::Error + Send + Sync>> {
            // Return a mock document
            todo!("Implement mock document creation")
        }
    }
    
    #[derive(Debug)]
    pub struct MockNotificationService;
    
    #[async_trait]
    impl NotificationService for MockNotificationService {
        async fn send_notification(&self, _notification: Notification) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }
    
    #[derive(Debug)]
    pub struct MockIntegrationService;
    
    #[async_trait]
    impl IntegrationService for MockIntegrationService {
        async fn execute_integration(&self, _request: IntegrationRequest) -> Result<IntegrationResponse, Box<dyn std::error::Error + Send + Sync>> {
            Ok(IntegrationResponse {
                success: true,
                data: serde_json::Value::Null,
                error: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mocks::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_set_state_action() {
        // Arrange
        let document_service = Arc::new(MockDocumentService);
        let executor = DefaultActionExecutor::new(document_service, None, None);
        
        let mut context = ActionContext::new(
            WorkflowInstanceId::new(),
            DocumentId::new(),
            Uuid::new_v4(),
            HashMap::new(),
            NodeId::new("test"),
        );
        
        let action = WorkflowAction::SetState(DocumentState::InReview);
        
        // Act
        let result = executor.execute_action(&action, &mut context).await;
        
        // Assert
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ActionResult::Success));
        
        // Check that state was set in context variables
        let state_var = context.get_variable("document.state");
        assert!(state_var.is_some());
    }
    
    #[tokio::test]
    async fn test_send_notification_action() {
        // Arrange
        let document_service = Arc::new(MockDocumentService);
        let notification_service = Arc::new(MockNotificationService);
        let executor = DefaultActionExecutor::new(
            document_service, 
            Some(notification_service), 
            None
        );
        
        let mut context = ActionContext::new(
            WorkflowInstanceId::new(),
            DocumentId::new(),
            Uuid::new_v4(),
            HashMap::new(),
            NodeId::new("test"),
        );
        
        let action = WorkflowAction::SendNotification {
            recipient: "test@example.com".to_string(),
            template: "document_updated".to_string(),
            variables: {
                let mut vars = HashMap::new();
                vars.insert("document_title".to_string(), "Test Document".to_string());
                vars
            },
        };
        
        // Act
        let result = executor.execute_action(&action, &mut context).await;
        
        // Assert
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ActionResult::Success));
    }
    
    #[tokio::test]
    async fn test_action_context_variables() {
        // Test context variable manipulation
        let mut context = ActionContext::new(
            WorkflowInstanceId::new(),
            DocumentId::new(),
            Uuid::new_v4(),
            HashMap::new(),
            NodeId::new("test"),
        );
        
        // Test setting variables
        context.set_variable("test_key".to_string(), serde_json::Value::String("test_value".to_string()));
        
        // Test getting variables
        let value = context.get_variable("test_key");
        assert!(value.is_some());
        assert_eq!(value.unwrap(), &serde_json::Value::String("test_value".to_string()));
    }
}