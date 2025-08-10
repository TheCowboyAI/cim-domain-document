//! Workflow Persistence Layer
//!
//! This module provides persistence capabilities for workflow instances and definitions,
//! allowing workflows to be saved, restored, and recovered across system restarts.

use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Workflow persistence trait for pluggable storage backends
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    /// Save workflow instance
    async fn save_instance(&self, instance: &WorkflowInstance) -> WorkflowResult<()>;
    
    /// Load workflow instance by ID
    async fn load_instance(&self, instance_id: WorkflowInstanceId) -> WorkflowResult<Option<WorkflowInstance>>;
    
    /// Find workflow instances by document ID
    async fn find_instances_by_document(&self, document_id: DocumentId) -> WorkflowResult<Vec<WorkflowInstance>>;
    
    /// Find workflow instances by status
    async fn find_instances_by_status(&self, status: WorkflowStatus) -> WorkflowResult<Vec<WorkflowInstance>>;
    
    /// Update workflow instance status
    async fn update_instance_status(&self, instance_id: WorkflowInstanceId, status: WorkflowStatus) -> WorkflowResult<()>;
    
    /// Delete workflow instance
    async fn delete_instance(&self, instance_id: WorkflowInstanceId) -> WorkflowResult<()>;
    
    /// Save workflow definition
    async fn save_definition(&self, definition: &WorkflowDefinition) -> WorkflowResult<()>;
    
    /// Load workflow definition by ID
    async fn load_definition(&self, workflow_id: WorkflowId) -> WorkflowResult<Option<WorkflowDefinition>>;
    
    /// List all workflow definitions
    async fn list_definitions(&self) -> WorkflowResult<Vec<WorkflowDefinition>>;
}

/// In-memory workflow repository for testing and simple use cases
#[derive(Debug, Default)]
pub struct InMemoryWorkflowRepository {
    instances: tokio::sync::RwLock<HashMap<WorkflowInstanceId, WorkflowInstance>>,
    definitions: tokio::sync::RwLock<HashMap<WorkflowId, WorkflowDefinition>>,
}

#[async_trait]
impl WorkflowRepository for InMemoryWorkflowRepository {
    async fn save_instance(&self, instance: &WorkflowInstance) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id, instance.clone());
        Ok(())
    }
    
    async fn load_instance(&self, instance_id: WorkflowInstanceId) -> WorkflowResult<Option<WorkflowInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.get(&instance_id).cloned())
    }
    
    async fn find_instances_by_document(&self, document_id: DocumentId) -> WorkflowResult<Vec<WorkflowInstance>> {
        let instances = self.instances.read().await;
        let matching_instances: Vec<WorkflowInstance> = instances
            .values()
            .filter(|instance| instance.document_id == document_id)
            .cloned()
            .collect();
        Ok(matching_instances)
    }
    
    async fn find_instances_by_status(&self, status: WorkflowStatus) -> WorkflowResult<Vec<WorkflowInstance>> {
        let instances = self.instances.read().await;
        let matching_instances: Vec<WorkflowInstance> = instances
            .values()
            .filter(|instance| std::mem::discriminant(&instance.status) == std::mem::discriminant(&status))
            .cloned()
            .collect();
        Ok(matching_instances)
    }
    
    async fn update_instance_status(&self, instance_id: WorkflowInstanceId, status: WorkflowStatus) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.status = status;
            instance.updated_at = Utc::now();
            Ok(())
        } else {
            Err(WorkflowError::WorkflowNotFound {
                workflow_id: instance_id.as_uuid().to_string(),
            })
        }
    }
    
    async fn delete_instance(&self, instance_id: WorkflowInstanceId) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        instances.remove(&instance_id);
        Ok(())
    }
    
    async fn save_definition(&self, definition: &WorkflowDefinition) -> WorkflowResult<()> {
        let mut definitions = self.definitions.write().await;
        definitions.insert(definition.id.clone(), definition.clone());
        Ok(())
    }
    
    async fn load_definition(&self, workflow_id: WorkflowId) -> WorkflowResult<Option<WorkflowDefinition>> {
        let definitions = self.definitions.read().await;
        Ok(definitions.get(&workflow_id).cloned())
    }
    
    async fn list_definitions(&self) -> WorkflowResult<Vec<WorkflowDefinition>> {
        let definitions = self.definitions.read().await;
        Ok(definitions.values().cloned().collect())
    }
}

/// Persistent workflow engine that uses a repository for storage
pub struct PersistentWorkflowEngine {
    repository: Box<dyn WorkflowRepository>,
    graph_cache: tokio::sync::RwLock<HashMap<WorkflowId, simple_workflow::WorkflowGraph>>,
}

impl std::fmt::Debug for PersistentWorkflowEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistentWorkflowEngine")
            .field("graph_cache", &self.graph_cache)
            .finish()
    }
}

impl PersistentWorkflowEngine {
    pub fn new(repository: Box<dyn WorkflowRepository>) -> Self {
        Self {
            repository,
            graph_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Start a new workflow instance
    pub async fn start_workflow(
        &self,
        workflow_id: WorkflowId,
        document_id: DocumentId,
        created_by: Uuid,
    ) -> WorkflowResult<WorkflowInstanceId> {
        // Load workflow definition
        let definition = self.repository.load_definition(workflow_id.clone())
            .await?
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: workflow_id.as_uuid().to_string(),
            })?;

        // Create new instance (use first start node if available)
        let start_node = definition.graph.start_nodes.first()
            .map(|_node_id| {
                // Convert NodeId to WorkflowNodeId - simplified conversion
                WorkflowNodeId::Start
            })
            .unwrap_or(WorkflowNodeId::Start);

        let instance = WorkflowInstance::new(
            workflow_id,
            document_id,
            created_by,
            start_node,
        );

        let instance_id = instance.id;

        // Save instance
        self.repository.save_instance(&instance).await?;

        Ok(instance_id)
    }

    /// Transition workflow to new state
    pub async fn transition_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        to_node: WorkflowNodeId,
        transitioned_by: Uuid,
    ) -> WorkflowResult<()> {
        // Load current instance
        let mut instance = self.repository.load_instance(instance_id)
            .await?
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: instance_id.as_uuid().to_string(),
            })?;

        // Load workflow graph
        let graph = self.get_workflow_graph(instance.workflow_id.clone()).await?;

        // Validate transition
        if !graph.can_transition(&instance.current_node, &to_node) {
            return Err(WorkflowError::InvalidTransition {
                from: instance.current_node.as_str().to_string(),
                to: to_node.as_str().to_string(),
                reason: "Transition not allowed in workflow definition".to_string(),
            });
        }

        // Update instance
        instance.current_node = to_node;
        instance.updated_at = Utc::now();
        instance.set_context("last_transitioned_by".to_string(), serde_json::to_value(transitioned_by)?);

        // Check if workflow is complete
        let graph = self.get_workflow_graph(instance.workflow_id.clone()).await?;
        if graph.end_nodes.contains(&instance.current_node) {
            instance.status = WorkflowStatus::Completed;
        }

        // Save updated instance
        self.repository.save_instance(&instance).await?;

        Ok(())
    }

    /// Get workflow instance
    pub async fn get_instance(&self, instance_id: WorkflowInstanceId) -> WorkflowResult<Option<WorkflowInstance>> {
        self.repository.load_instance(instance_id).await
    }

    /// Get active workflow instances for document
    pub async fn get_active_instances_for_document(&self, document_id: DocumentId) -> WorkflowResult<Vec<WorkflowInstance>> {
        let all_instances = self.repository.find_instances_by_document(document_id).await?;
        let active_instances = all_instances
            .into_iter()
            .filter(|instance| matches!(instance.status, WorkflowStatus::Running | WorkflowStatus::Suspended))
            .collect();
        Ok(active_instances)
    }

    /// Cancel workflow instance
    pub async fn cancel_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        cancelled_by: Uuid,
        reason: String,
    ) -> WorkflowResult<()> {
        let mut instance = self.repository.load_instance(instance_id)
            .await?
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: instance_id.as_uuid().to_string(),
            })?;

        instance.status = WorkflowStatus::Cancelled;
        instance.updated_at = Utc::now();
        instance.set_context("cancelled_by".to_string(), serde_json::to_value(cancelled_by)?);
        instance.set_context("cancellation_reason".to_string(), serde_json::Value::String(reason));

        self.repository.save_instance(&instance).await?;
        Ok(())
    }

    /// Register workflow definition
    pub async fn register_workflow_definition(&self, definition: WorkflowDefinition) -> WorkflowResult<()> {
        // Cache the graph
        let mut cache = self.graph_cache.write().await;
        // Convert complex graph to simple graph for execution engine
        let simple_graph = self.convert_complex_to_simple_graph(&definition.graph);
        cache.insert(definition.id.clone(), simple_graph.clone());

        // Save definition
        self.repository.save_definition(&definition).await?;
        Ok(())
    }

    /// Get workflow graph (with caching)
    async fn get_workflow_graph(&self, workflow_id: WorkflowId) -> WorkflowResult<simple_workflow::WorkflowGraph> {
        // Check cache first
        {
            let cache = self.graph_cache.read().await;
            if let Some(graph) = cache.get(&workflow_id) {
                return Ok(graph.clone());
            }
        }

        // Load from repository
        let definition = self.repository.load_definition(workflow_id.clone())
            .await?
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: workflow_id.as_uuid().to_string(),
            })?;

        // Convert complex graph to simple graph for execution engine
        let graph = self.convert_complex_to_simple_graph(&definition.graph);
        let mut cache = self.graph_cache.write().await;
        cache.insert(workflow_id, graph.clone());

        Ok(graph)
    }

    /// Convert complex workflow graph to simple execution graph
    fn convert_complex_to_simple_graph(&self, _complex_graph: &ComplexWorkflowGraph) -> simple_workflow::WorkflowGraph {
        // For now, create a basic graph - this would need proper conversion logic
        simple_workflow::WorkflowGraph::new()
    }

    /// Recover incomplete workflows (useful after system restart)
    pub async fn recover_workflows(&self) -> WorkflowResult<Vec<WorkflowInstanceId>> {
        let running_instances = self.repository.find_instances_by_status(WorkflowStatus::Running).await?;
        let mut recovered = Vec::new();

        for instance in running_instances {
            // Validate instance is still valid
            if let Ok(Some(_)) = self.repository.load_definition(instance.workflow_id).await {
                // Instance is valid, add to recovery list
                recovered.push(instance.id);
            } else {
                // Invalid workflow definition, cancel instance
                self.repository.update_instance_status(
                    instance.id,
                    WorkflowStatus::Failed("Invalid workflow definition".to_string()),
                ).await?;
            }
        }

        Ok(recovered)
    }
}

/// Workflow audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAuditEntry {
    pub id: Uuid,
    pub workflow_instance_id: WorkflowInstanceId,
    pub event_type: WorkflowEventType,
    pub from_node: Option<WorkflowNodeId>,
    pub to_node: Option<WorkflowNodeId>,
    pub performed_by: Uuid,
    pub performed_at: DateTime<Utc>,
    pub context: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

/// Types of workflow events for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEventType {
    WorkflowStarted,
    NodeEntered,
    NodeExited,
    TransitionExecuted,
    ActionExecuted,
    WorkflowCompleted,
    WorkflowCancelled,
    WorkflowFailed,
    ContextUpdated,
}

/// Workflow audit service
#[derive(Debug)]
pub struct WorkflowAuditService {
    entries: tokio::sync::RwLock<Vec<WorkflowAuditEntry>>,
}

impl WorkflowAuditService {
    pub fn new() -> Self {
        Self {
            entries: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    /// Record workflow event
    pub async fn record_event(
        &self,
        workflow_instance_id: WorkflowInstanceId,
        event_type: WorkflowEventType,
        from_node: Option<WorkflowNodeId>,
        to_node: Option<WorkflowNodeId>,
        performed_by: Uuid,
        context: HashMap<String, serde_json::Value>,
    ) {
        let entry = WorkflowAuditEntry {
            id: Uuid::new_v4(),
            workflow_instance_id,
            event_type,
            from_node,
            to_node,
            performed_by,
            performed_at: Utc::now(),
            context,
            metadata: HashMap::new(),
        };

        let mut entries = self.entries.write().await;
        entries.push(entry);
    }

    /// Get audit trail for workflow instance
    pub async fn get_audit_trail(&self, workflow_instance_id: WorkflowInstanceId) -> Vec<WorkflowAuditEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|entry| entry.workflow_instance_id == workflow_instance_id)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_repository() {
        let repo = InMemoryWorkflowRepository::default();
        
        // Create test instance
        let instance = WorkflowInstance::new(
            WorkflowId::new(),
            DocumentId::new(),
            Uuid::new_v4(),
            WorkflowNodeId::Start,
        );
        
        let instance_id = instance.id;
        
        // Save and load
        repo.save_instance(&instance).await.unwrap();
        let loaded = repo.load_instance(instance_id).await.unwrap();
        
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, instance_id);
    }

    #[tokio::test]
    async fn test_persistent_workflow_engine() {
        let repo = Box::new(InMemoryWorkflowRepository::default());
        let engine = PersistentWorkflowEngine::new(repo);
        
        // Register workflow definition
        let definition = WorkflowDefinition::new(
            "Test Workflow".to_string(),
            "Test workflow for persistence".to_string(),
            Uuid::new_v4(),
        );
        let workflow_id = definition.id.clone();
        
        engine.register_workflow_definition(definition).await.unwrap();
        
        // Start workflow
        let document_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        
        let instance_id = engine.start_workflow(workflow_id, document_id.clone(), user_id).await.unwrap();
        
        // Verify instance was created
        let instance = engine.get_instance(instance_id).await.unwrap();
        assert!(instance.is_some());
        
        let instance = instance.unwrap();
        assert_eq!(instance.document_id, document_id);
        assert_eq!(instance.current_node, WorkflowNodeId::Start);
    }

    #[tokio::test]
    async fn test_workflow_audit_service() {
        let audit_service = WorkflowAuditService::new();
        let instance_id = WorkflowInstanceId::new();
        let user_id = Uuid::new_v4();
        
        // Record event
        audit_service.record_event(
            instance_id,
            WorkflowEventType::WorkflowStarted,
            None,
            Some(WorkflowNodeId::Start),
            user_id,
            HashMap::new(),
        ).await;
        
        // Get audit trail
        let trail = audit_service.get_audit_trail(instance_id).await;
        assert_eq!(trail.len(), 1);
        assert!(matches!(trail[0].event_type, WorkflowEventType::WorkflowStarted));
    }
}