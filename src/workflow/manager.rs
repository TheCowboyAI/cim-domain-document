//! Workflow Manager
//!
//! This module provides a high-level workflow management interface that coordinates
//! all workflow components including persistence, event integration, validation, and execution.

use super::*;
use crate::events::DocumentDomainEvent;
use crate::value_objects::DocumentState;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Comprehensive workflow manager that coordinates all workflow operations
#[derive(Debug)]
pub struct WorkflowManager {
    /// Persistent workflow engine
    engine: Arc<PersistentWorkflowEngine>,
    /// Event integration service
    event_integration: Arc<RwLock<WorkflowEventIntegration>>,
    /// Business rule validator
    validator: Arc<WorkflowValidator>,
    /// Audit service
    audit_service: Arc<WorkflowAuditService>,
    /// Active workflow monitoring
    active_workflows: Arc<RwLock<std::collections::HashMap<DocumentId, Vec<WorkflowInstanceId>>>>,
}

impl WorkflowManager {
    /// Create new workflow manager with in-memory storage
    pub fn new() -> Self {
        let repository = Box::new(InMemoryWorkflowRepository::default());
        Self::with_repository(repository)
    }

    /// Create workflow manager with custom repository
    pub fn with_repository(repository: Box<dyn WorkflowRepository>) -> Self {
        Self {
            engine: Arc::new(PersistentWorkflowEngine::new(repository)),
            event_integration: Arc::new(RwLock::new(WorkflowEventIntegration::new())),
            validator: Arc::new(WorkflowValidator::new()),
            audit_service: Arc::new(WorkflowAuditService::new()),
            active_workflows: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Initialize the workflow manager with default workflow definitions
    pub async fn initialize(&self) -> WorkflowResult<()> {
        // Register default workflow definitions
        self.register_default_workflows().await?;
        
        // Recover any incomplete workflows
        let recovered = self.engine.recover_workflows().await?;
        if !recovered.is_empty() {
            println!("Recovered {} incomplete workflow instances", recovered.len());
            
            // Update active workflows tracking
            for instance_id in recovered {
                if let Ok(Some(instance)) = self.engine.get_instance(instance_id).await {
                    let mut active = self.active_workflows.write().await;
                    active.entry(instance.document_id)
                        .or_insert_with(Vec::new)
                        .push(instance_id);
                }
            }
        }
        
        Ok(())
    }

    /// Handle document domain event and trigger appropriate workflows
    pub async fn handle_document_event(&self, event: &DocumentDomainEvent) -> WorkflowResult<Vec<WorkflowInstanceId>> {
        // Let event integration handle the event
        let mut integration = self.event_integration.write().await;
        let started_workflows = integration.handle_document_event(event).await?;

        // Update active workflows tracking
        if !started_workflows.is_empty() {
            let document_id = self.extract_document_id_from_event(event);
            if let Some(doc_id) = document_id {
                let mut active = self.active_workflows.write().await;
                let entry = active.entry(doc_id).or_insert_with(Vec::new);
                entry.extend(started_workflows.iter());
            }
        }

        // Record audit events for started workflows
        for &instance_id in &started_workflows {
            self.audit_service.record_event(
                instance_id,
                crate::workflow::persistence::WorkflowEventType::WorkflowStarted,
                None,
                Some(WorkflowNodeId::Start),
                Uuid::nil(), // System event
                std::collections::HashMap::new(),
            ).await;
        }

        Ok(started_workflows)
    }

    /// Start workflow manually
    pub async fn start_workflow(
        &self,
        workflow_type: &str,
        document_id: DocumentId,
        user_id: Uuid,
    ) -> WorkflowResult<WorkflowInstanceId> {
        // Map workflow type to workflow ID
        let workflow_id = self.resolve_workflow_id(workflow_type)?;
        
        // Start the workflow
        let instance_id = self.engine.start_workflow(workflow_id, document_id.clone(), user_id).await?;

        // Update active workflows tracking
        let mut active = self.active_workflows.write().await;
        active.entry(document_id)
            .or_insert_with(Vec::new)
            .push(instance_id);

        // Record audit event
        self.audit_service.record_event(
            instance_id,
            crate::workflow::persistence::WorkflowEventType::WorkflowStarted,
            None,
            Some(WorkflowNodeId::Start),
            user_id,
            std::collections::HashMap::new(),
        ).await;

        Ok(instance_id)
    }

    /// Execute workflow transition with validation
    pub async fn transition_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        to_node: WorkflowNodeId,
        user_id: Uuid,
        user_permissions: Vec<Permission>,
    ) -> WorkflowResult<()> {
        // Get current instance
        let instance = self.engine.get_instance(instance_id)
            .await?
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: instance_id.as_uuid().to_string(),
            })?;

        // Validate transition (simplified - in real system would get document state)
        let document_state = DocumentState::Draft; // Would be retrieved from document service
        let context = instance.context.clone();
        
        if let Err(errors) = self.validator.validate_transition(
            &instance.current_node,
            &to_node,
            &user_permissions,
            document_state,
            &context,
        ) {
            return Err(WorkflowError::GuardFailed {
                guard: "business_rules".to_string(),
                reason: errors.join(", "),
            });
        }

        // Record transition start
        self.audit_service.record_event(
            instance_id,
            WorkflowEventType::TransitionExecuted,
            Some(instance.current_node.clone()),
            Some(to_node.clone()),
            user_id,
            context.clone(),
        ).await;

        // Execute transition
        self.engine.transition_workflow(instance_id, to_node.clone(), user_id).await?;

        // Check if workflow completed
        let updated_instance = self.engine.get_instance(instance_id).await?.unwrap();
        if matches!(updated_instance.status, WorkflowStatus::Completed) {
            // Remove from active workflows
            let mut active = self.active_workflows.write().await;
            if let Some(instances) = active.get_mut(&updated_instance.document_id) {
                instances.retain(|&id| id != instance_id);
                if instances.is_empty() {
                    active.remove(&updated_instance.document_id);
                }
            }

            // Record completion
            self.audit_service.record_event(
                instance_id,
                WorkflowEventType::WorkflowCompleted,
                Some(to_node),
                None,
                user_id,
                updated_instance.context,
            ).await;
        }

        Ok(())
    }

    /// Cancel workflow
    pub async fn cancel_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        user_id: Uuid,
        reason: String,
    ) -> WorkflowResult<()> {
        // Get current instance for document ID
        let instance = self.engine.get_instance(instance_id)
            .await?
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: instance_id.as_uuid().to_string(),
            })?;
        
        let document_id = instance.document_id;

        // Cancel in engine
        self.engine.cancel_workflow(instance_id, user_id, reason.clone()).await?;

        // Remove from active workflows
        let mut active = self.active_workflows.write().await;
        if let Some(instances) = active.get_mut(&document_id) {
            instances.retain(|&id| id != instance_id);
            if instances.is_empty() {
                active.remove(&document_id);
            }
        }

        // Record cancellation
        let mut context = std::collections::HashMap::new();
        context.insert("reason".to_string(), serde_json::Value::String(reason));
        
        self.audit_service.record_event(
            instance_id,
            WorkflowEventType::WorkflowCancelled,
            Some(instance.current_node),
            None,
            user_id,
            context,
        ).await;

        Ok(())
    }

    /// Get workflow instance details
    pub async fn get_workflow_instance(&self, instance_id: WorkflowInstanceId) -> WorkflowResult<Option<WorkflowInstance>> {
        self.engine.get_instance(instance_id).await
    }

    /// Get active workflows for document
    pub async fn get_active_workflows_for_document(&self, document_id: DocumentId) -> WorkflowResult<Vec<WorkflowInstance>> {
        self.engine.get_active_instances_for_document(document_id).await
    }

    /// Get workflow audit trail
    pub async fn get_audit_trail(&self, instance_id: WorkflowInstanceId) -> Vec<WorkflowAuditEntry> {
        self.audit_service.get_audit_trail(instance_id).await
    }

    /// Get workflow statistics
    pub async fn get_workflow_statistics(&self) -> WorkflowResult<WorkflowStatistics> {
        let active = self.active_workflows.read().await;
        let total_active_documents = active.len();
        let total_active_workflows: usize = active.values().map(|v| v.len()).sum();

        Ok(WorkflowStatistics {
            total_active_documents,
            total_active_workflows,
            workflows_by_type: std::collections::HashMap::new(), // Would be calculated in real implementation
        })
    }

    /// Register workflow definition
    pub async fn register_workflow_definition(&self, definition: WorkflowDefinition) -> WorkflowResult<()> {
        self.engine.register_workflow_definition(definition).await
    }

    /// Private helper methods
    async fn register_default_workflows(&self) -> WorkflowResult<()> {
        // Create review workflow definition
        let review_workflow = self.create_review_workflow_definition();
        self.engine.register_workflow_definition(review_workflow).await?;

        // Create approval workflow definition  
        let approval_workflow = self.create_approval_workflow_definition();
        self.engine.register_workflow_definition(approval_workflow).await?;

        Ok(())
    }

    fn create_review_workflow_definition(&self) -> WorkflowDefinition {
        let mut definition = WorkflowDefinition::new(
            "Document Review".to_string(),
            "Standard document review workflow".to_string(),
            Uuid::nil(),
        );

        // Set well-known ID for review workflow
        definition.id = WorkflowId::new_named("document_review");
        // Note: We'll need to convert simple workflow graph to complex definition graph
        // For now, use empty complex graph
        definition.category = "Review".to_string();
        definition.tags = vec!["review".to_string(), "approval".to_string()];

        definition
    }

    fn create_approval_workflow_definition(&self) -> WorkflowDefinition {
        let mut definition = WorkflowDefinition::new(
            "Document Approval".to_string(),
            "Standard document approval workflow".to_string(),
            Uuid::nil(),
        );

        // Set well-known ID for approval workflow
        definition.id = WorkflowId::new_named("document_approval");
        // Note: We'll need to convert simple workflow graph to complex definition graph
        // For now, use empty complex graph
        definition.category = "Approval".to_string();
        definition.tags = vec!["approval".to_string(), "publishing".to_string()];

        definition
    }

    fn resolve_workflow_id(&self, workflow_type: &str) -> WorkflowResult<WorkflowId> {
        match workflow_type {
            "review" => Ok(WorkflowId::new_named("document_review")),
            "approval" => Ok(WorkflowId::new_named("document_approval")),
            _ => Err(WorkflowError::WorkflowNotFound {
                workflow_id: workflow_type.to_string(),
            }),
        }
    }

    fn extract_document_id_from_event(&self, event: &DocumentDomainEvent) -> Option<DocumentId> {
        match event {
            DocumentDomainEvent::DocumentCreated(e) => Some(e.document_id.clone()),
            DocumentDomainEvent::DocumentUploaded(e) => Some(e.document_id.clone()),
            DocumentDomainEvent::StateChanged(e) => Some(e.document_id.clone()),
            DocumentDomainEvent::DocumentEditedDirect(e) => Some(e.document_id.clone()),
            DocumentDomainEvent::DocumentEditedPatch(e) => Some(e.document_id.clone()),
            DocumentDomainEvent::DocumentEditedStructured(e) => Some(e.document_id.clone()),
            _ => None,
        }
    }
}

/// Workflow statistics
#[derive(Debug, Clone)]
pub struct WorkflowStatistics {
    pub total_active_documents: usize,
    pub total_active_workflows: usize,
    pub workflows_by_type: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::DocumentCreated;
    use crate::value_objects::DocumentMetadata;

    #[tokio::test]
    async fn test_workflow_manager_initialization() {
        let manager = WorkflowManager::new();
        manager.initialize().await.unwrap();

        // Should have registered default workflows
        let stats = manager.get_workflow_statistics().await.unwrap();
        assert_eq!(stats.total_active_workflows, 0); // No active workflows yet
    }

    #[tokio::test]
    async fn test_workflow_manager_start_workflow() {
        let manager = WorkflowManager::new();
        manager.initialize().await.unwrap();

        let document_id = DocumentId::new();
        let user_id = Uuid::new_v4();

        // Start review workflow
        let instance_id = manager.start_workflow(
            "review",
            document_id.clone(),
            user_id,
        ).await.unwrap();

        // Verify workflow was started
        let instance = manager.get_workflow_instance(instance_id).await.unwrap();
        assert!(instance.is_some());

        let instance = instance.unwrap();
        assert_eq!(instance.document_id, document_id);
        assert_eq!(instance.current_node, WorkflowNodeId::Start);

        // Check statistics
        let stats = manager.get_workflow_statistics().await.unwrap();
        assert_eq!(stats.total_active_workflows, 1);
        assert_eq!(stats.total_active_documents, 1);
    }

    #[tokio::test]
    async fn test_workflow_manager_event_handling() {
        let manager = WorkflowManager::new();
        manager.initialize().await.unwrap();

        // Create document event
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), "Test Document".to_string());
        metadata.insert("mime_type".to_string(), "text/plain".to_string());
        
        let event = DocumentDomainEvent::DocumentCreated(DocumentCreated {
            document_id: DocumentId::new(),
            document_type: crate::value_objects::DocumentType::Text,
            title: "Test Document".to_string(),
            author_id: Uuid::new_v4(),
            metadata,
            created_at: chrono::Utc::now(),
        });

        // Handle event
        let started_workflows = manager.handle_document_event(&event).await.unwrap();
        assert!(!started_workflows.is_empty());

        // Check that audit trail was created
        let audit_trail = manager.get_audit_trail(started_workflows[0]).await;
        assert_eq!(audit_trail.len(), 1);
        assert!(matches!(audit_trail[0].event_type, WorkflowEventType::WorkflowStarted));
    }

    #[tokio::test]
    async fn test_workflow_transition_with_validation() {
        let manager = WorkflowManager::new();
        manager.initialize().await.unwrap();

        let document_id = DocumentId::new();
        let user_id = Uuid::new_v4();

        // Start workflow
        let instance_id = manager.start_workflow("review", document_id, user_id).await.unwrap();

        // Transition with proper permissions
        let permissions = vec![Permission::Review];
        manager.transition_workflow(
            instance_id,
            WorkflowNodeId::InReview,
            user_id,
            permissions,
        ).await.unwrap();

        // Verify transition
        let instance = manager.get_workflow_instance(instance_id).await.unwrap().unwrap();
        assert_eq!(instance.current_node, WorkflowNodeId::InReview);

        // Check audit trail
        let audit_trail = manager.get_audit_trail(instance_id).await;
        assert_eq!(audit_trail.len(), 2); // Start + Transition
    }
}