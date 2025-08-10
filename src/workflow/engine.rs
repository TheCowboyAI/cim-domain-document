//! Workflow Execution Engine
//!
//! This module implements the core workflow execution engine that orchestrates
//! workflow instances, state transitions, and action execution.

use super::*;
use crate::Document;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Main workflow execution engine
#[derive(Debug)]
pub struct WorkflowEngine {
    /// Workflow definitions repository
    pub definitions: Arc<RwLock<HashMap<WorkflowId, WorkflowDefinition>>>,
    /// Active workflow instances
    pub instances: Arc<RwLock<HashMap<WorkflowInstanceId, WorkflowInstance>>>,
    /// State transition validator
    pub validator: StateTransitionValidator,
    /// Action executor
    pub action_executor: Arc<dyn ActionExecutor + Send + Sync>,
    /// Automation engine
    pub automation_engine: Arc<Mutex<WorkflowAutomationEngine>>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            validator: StateTransitionValidator::new(),
            action_executor: Arc::new(DefaultActionExecutor::new()),
            automation_engine: Arc::new(Mutex::new(WorkflowAutomationEngine::new())),
        }
    }
    
    pub fn with_action_executor(
        mut self,
        executor: Arc<dyn ActionExecutor + Send + Sync>,
    ) -> Self {
        self.action_executor = executor;
        self
    }
    
    /// Register a new workflow definition
    pub async fn register_workflow(&self, definition: WorkflowDefinition) -> WorkflowResult<()> {
        definition.validate()?;
        
        let mut definitions = self.definitions.write().await;
        definitions.insert(definition.id.clone(), definition);
        Ok(())
    }
    
    /// Get workflow definition by ID
    pub async fn get_workflow_definition(&self, workflow_id: &WorkflowId) -> Option<WorkflowDefinition> {
        let definitions = self.definitions.read().await;
        definitions.get(workflow_id).cloned()
    }
    
    /// Start a new workflow instance
    pub async fn start_workflow(
        &self,
        workflow_id: WorkflowId,
        document_id: DocumentId,
        started_by: Uuid,
        initial_context: Option<WorkflowContext>,
    ) -> WorkflowResult<WorkflowInstanceId> {
        let definition = self.get_workflow_definition(&workflow_id).await
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: workflow_id.as_uuid().to_string(),
            })?;
        
        if !definition.is_active {
            return Err(WorkflowError::EngineError {
                message: "Workflow is not active".to_string(),
            });
        }
        
        // Create new workflow instance
        let mut instance = WorkflowInstance::new(
            workflow_id,
            document_id,
            started_by,
            definition.graph.start_nodes.clone(),
        );
        
        // Set initial context
        if let Some(context) = initial_context {
            instance.context = context;
        }
        
        let instance_id = instance.id.clone();
        
        // Store instance
        let mut instances = self.instances.write().await;
        instances.insert(instance_id.clone(), instance);
        
        // Trigger workflow started automation
        if let Some(mut instance) = instances.get_mut(&instance_id) {
            let automation_engine = self.automation_engine.lock().await;
            // Note: In a real implementation, we'd need the actual document and user
            // This is a simplified version for demonstration
        }
        
        Ok(instance_id)
    }
    
    /// Execute transition from one node to another
    pub async fn execute_transition(
        &self,
        instance_id: WorkflowInstanceId,
        from_node: NodeId,
        to_node: NodeId,
        triggered_by: Uuid,
        document: &Document,
        user: &User,
        transition_data: Option<HashMap<String, serde_json::Value>>,
    ) -> WorkflowResult<()> {
        // Get workflow instance
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(&instance_id)
            .ok_or_else(|| WorkflowError::EngineError {
                message: format!("Workflow instance not found: {}", instance_id.as_uuid()),
            })?;
        
        // Get workflow definition
        let definition = self.get_workflow_definition(&instance.workflow_id).await
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: instance.workflow_id.as_uuid().to_string(),
            })?;
        
        // Validate transition
        self.validator.can_transition(
            &definition,
            instance,
            &from_node,
            &to_node,
            document,
            user,
        )?;
        
        // Check if instance is at the from_node
        if !instance.is_at_node(&from_node) {
            return Err(WorkflowError::InvalidTransition {
                from: from_node.as_str().to_string(),
                to: to_node.as_str().to_string(),
                reason: "Workflow is not at the source node".to_string(),
            });
        }
        
        // Execute transition
        self.do_transition(
            instance,
            &definition,
            from_node.clone(),
            to_node.clone(),
            triggered_by,
            document,
            user,
            transition_data,
        ).await?;
        
        Ok(())
    }
    
    /// Internal method to perform the actual transition
    async fn do_transition(
        &self,
        instance: &mut WorkflowInstance,
        definition: &WorkflowDefinition,
        from_node: NodeId,
        to_node: NodeId,
        triggered_by: Uuid,
        document: &Document,
        user: &User,
        transition_data: Option<HashMap<String, serde_json::Value>>,
    ) -> WorkflowResult<()> {
        let transition_time = Utc::now();
        
        // Remove from current node
        instance.current_nodes.retain(|n| *n != from_node);
        
        // Add to target node
        if !instance.current_nodes.contains(&to_node) {
            instance.current_nodes.push(to_node.clone());
        }
        
        // Record transition
        let transition = WorkflowTransition {
            id: Uuid::new_v4(),
            from_node: from_node.clone(),
            to_node: to_node.clone(),
            transitioned_at: transition_time,
            transitioned_by: triggered_by,
            reason: None,
            data: transition_data.unwrap_or_default(),
        };
        instance.add_transition(transition);
        
        // Execute actions on target node
        if let Some(target_node) = definition.graph.nodes.get(&to_node) {
            let actions = self.get_node_actions(target_node);
            
            for action in actions {
                let mut action_context = ActionContext {
                    workflow_instance: instance,
                    workflow_definition: definition,
                    document,
                    user,
                    current_node: &to_node,
                    trigger_time: transition_time,
                    variables: &mut instance.context.variables,
                };
                
                match self.action_executor.execute_action(&action, &mut action_context).await {
                    Ok(ActionResult::Success) => {},
                    Ok(ActionResult::Warning(msg)) => {
                        eprintln!("Action warning: {}", msg);
                    },
                    Ok(ActionResult::Error(msg)) => {
                        return Err(WorkflowError::ActionFailed {
                            action: format!("{:?}", action),
                            error: msg,
                        });
                    },
                    Ok(ActionResult::RequiresIntervention(msg)) => {
                        // Suspend workflow for manual intervention
                        instance.status = WorkflowStatus::Suspended;
                        return Err(WorkflowError::ActionFailed {
                            action: format!("{:?}", action),
                            error: format!("Requires intervention: {}", msg),
                        });
                    },
                    Ok(ActionResult::Pending) => {
                        // Action is pending, continue workflow
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
        
        // Check if workflow is complete
        if definition.graph.end_nodes.iter().any(|end_node| instance.is_at_node(end_node)) {
            instance.status = WorkflowStatus::Completed;
            instance.completed_at = Some(transition_time);
        }
        
        // Trigger automations
        let automation_engine = self.automation_engine.lock().await;
        let _ = automation_engine.process_trigger(
            WorkflowTrigger::NodeCompleted(from_node),
            instance,
            definition,
            document,
            user,
            &to_node,
        ).await;
        
        Ok(())
    }
    
    /// Get actions associated with a node
    fn get_node_actions(&self, node: &WorkflowNode) -> Vec<WorkflowAction> {
        match node {
            WorkflowNode::Start(start) => start.actions.clone(),
            WorkflowNode::Task(task) => task.actions.clone(),
            WorkflowNode::Decision(decision) => decision.actions.clone(),
            WorkflowNode::Parallel(parallel) => parallel.actions.clone(),
            WorkflowNode::Join(join) => join.actions.clone(),
            WorkflowNode::Timer(timer) => timer.actions_on_timeout.clone(),
            WorkflowNode::End(end) => end.actions.clone(),
        }
    }
    
    /// Get workflow instance by ID
    pub async fn get_workflow_instance(&self, instance_id: &WorkflowInstanceId) -> Option<WorkflowInstance> {
        let instances = self.instances.read().await;
        instances.get(instance_id).cloned()
    }
    
    /// Get all workflow instances for a document
    pub async fn get_document_workflows(&self, document_id: &DocumentId) -> Vec<WorkflowInstance> {
        let instances = self.instances.read().await;
        instances.values()
            .filter(|instance| instance.document_id == *document_id)
            .cloned()
            .collect()
    }
    
    /// Cancel a workflow instance
    pub async fn cancel_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        cancelled_by: Uuid,
        reason: String,
    ) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(&instance_id)
            .ok_or_else(|| WorkflowError::EngineError {
                message: format!("Workflow instance not found: {}", instance_id.as_uuid()),
            })?;
        
        instance.status = WorkflowStatus::Cancelled;
        instance.completed_at = Some(Utc::now());
        instance.context.set_variable("cancelled_by".to_string(), serde_json::Value::String(cancelled_by.to_string()));
        instance.context.set_variable("cancellation_reason".to_string(), serde_json::Value::String(reason));
        
        Ok(())
    }
    
    /// Suspend a workflow instance
    pub async fn suspend_workflow(&self, instance_id: WorkflowInstanceId) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(&instance_id)
            .ok_or_else(|| WorkflowError::EngineError {
                message: format!("Workflow instance not found: {}", instance_id.as_uuid()),
            })?;
        
        if instance.status == WorkflowStatus::Running {
            instance.status = WorkflowStatus::Suspended;
        }
        
        Ok(())
    }
    
    /// Resume a suspended workflow instance
    pub async fn resume_workflow(&self, instance_id: WorkflowInstanceId) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(&instance_id)
            .ok_or_else(|| WorkflowError::EngineError {
                message: format!("Workflow instance not found: {}", instance_id.as_uuid()),
            })?;
        
        if instance.status == WorkflowStatus::Suspended {
            instance.status = WorkflowStatus::Running;
        }
        
        Ok(())
    }
    
    /// Get workflow analytics for a definition
    pub async fn get_workflow_analytics(&self, workflow_id: &WorkflowId) -> WorkflowAnalytics {
        let instances = self.instances.read().await;
        let workflow_instances: Vec<_> = instances.values()
            .filter(|instance| instance.workflow_id == *workflow_id)
            .collect();
        
        let total_instances = workflow_instances.len() as u64;
        let completed_instances = workflow_instances.iter()
            .filter(|instance| matches!(instance.status, WorkflowStatus::Completed))
            .count() as u64;
        
        let average_completion_time = if completed_instances > 0 {
            let total_duration: i64 = workflow_instances.iter()
                .filter_map(|instance| {
                    instance.completed_at.map(|completed| {
                        (completed - instance.started_at).num_seconds()
                    })
                })
                .sum();
            
            Duration::seconds(total_duration / completed_instances as i64)
        } else {
            Duration::zero()
        };
        
        // Calculate bottleneck nodes (nodes with longest average duration)
        let mut node_durations: HashMap<NodeId, Vec<i64>> = HashMap::new();
        
        for instance in &workflow_instances {
            for transition in &instance.history {
                let duration = if let Some(next_transition) = instance.history.iter()
                    .find(|t| t.from_node == transition.to_node)
                {
                    (next_transition.transitioned_at - transition.transitioned_at).num_seconds()
                } else if let Some(completed_at) = instance.completed_at {
                    (completed_at - transition.transitioned_at).num_seconds()
                } else {
                    continue;
                };
                
                node_durations.entry(transition.to_node.clone())
                    .or_insert_with(Vec::new)
                    .push(duration);
            }
        }
        
        let bottleneck_nodes = node_durations.into_iter()
            .map(|(node_id, durations)| {
                let avg_duration = durations.iter().sum::<i64>() / durations.len() as i64;
                (node_id, Duration::seconds(avg_duration))
            })
            .collect();
        
        // Calculate SLA compliance rate (simplified)
        let sla_compliance_rate = 0.95; // Placeholder
        
        // Calculate escalation frequency
        let escalation_frequency = HashMap::new(); // Placeholder
        
        WorkflowAnalytics {
            total_instances,
            completed_instances,
            average_completion_time,
            bottleneck_nodes,
            sla_compliance_rate,
            escalation_frequency,
        }
    }
    
    /// Clean up completed workflow instances
    pub async fn cleanup_completed_workflows(&self, retention_period: Duration) -> WorkflowResult<u32> {
        let cutoff_time = Utc::now() - retention_period;
        let mut instances = self.instances.write().await;
        
        let initial_count = instances.len();
        
        instances.retain(|_, instance| {
            if matches!(instance.status, WorkflowStatus::Completed | WorkflowStatus::Cancelled) {
                if let Some(completed_at) = instance.completed_at {
                    return completed_at > cutoff_time;
                }
            }
            true
        });
        
        let cleaned_count = initial_count - instances.len();
        Ok(cleaned_count as u32)
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAnalytics {
    pub total_instances: u64,
    pub completed_instances: u64,
    pub average_completion_time: Duration,
    pub bottleneck_nodes: Vec<(NodeId, Duration)>,
    pub sla_compliance_rate: f64,
    pub escalation_frequency: HashMap<NodeId, u32>,
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
    
    fn create_simple_workflow() -> WorkflowDefinition {
        let user_id = Uuid::new_v4();
        let mut workflow = WorkflowDefinition::new(
            "Simple Test Workflow".to_string(),
            "A simple test workflow".to_string(),
            user_id,
        );
        
        // Add start node
        let start_node = WorkflowNode::Start(StartNode {
            id: NodeId::new("start"),
            name: "Start".to_string(),
            actions: vec![],
            metadata: HashMap::new(),
        });
        
        // Add end node
        let end_node = WorkflowNode::End(EndNode {
            id: NodeId::new("end"),
            name: "End".to_string(),
            actions: vec![],
            completion_status: CompletionStatus::Success,
            metadata: HashMap::new(),
        });
        
        workflow.graph.add_node(NodeId::new("start"), start_node);
        workflow.graph.add_node(NodeId::new("end"), end_node);
        workflow.graph.add_edge(
            EdgeId::new("start_to_end"),
            NodeId::new("start"),
            NodeId::new("end"),
            None,
        );
        
        workflow
    }

    #[tokio::test]
    async fn test_workflow_engine_creation() {
        let engine = WorkflowEngine::new();
        
        assert!(engine.definitions.read().await.is_empty());
        assert!(engine.instances.read().await.is_empty());
    }
    
    #[tokio::test]
    async fn test_workflow_registration() {
        let engine = WorkflowEngine::new();
        let workflow = create_simple_workflow();
        let workflow_id = workflow.id.clone();
        
        let result = engine.register_workflow(workflow).await;
        assert!(result.is_ok());
        
        let retrieved = engine.get_workflow_definition(&workflow_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Simple Test Workflow");
    }
    
    #[tokio::test]
    async fn test_workflow_start() {
        let engine = WorkflowEngine::new();
        let workflow = create_simple_workflow();
        let workflow_id = workflow.id.clone();
        
        engine.register_workflow(workflow).await.unwrap();
        
        let document_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        
        let instance_id = engine.start_workflow(
            workflow_id.clone(),
            document_id.clone(),
            user_id,
            None,
        ).await.unwrap();
        
        let instance = engine.get_workflow_instance(&instance_id).await;
        assert!(instance.is_some());
        
        let instance = instance.unwrap();
        assert_eq!(instance.workflow_id, workflow_id);
        assert_eq!(instance.document_id, document_id);
        assert_eq!(instance.started_by, user_id);
        assert_eq!(instance.status, WorkflowStatus::Running);
        assert_eq!(instance.current_nodes, vec![NodeId::new("start")]);
    }
    
    #[tokio::test]
    async fn test_workflow_transition() {
        let engine = WorkflowEngine::new();
        let workflow = create_simple_workflow();
        let workflow_id = workflow.id.clone();
        
        engine.register_workflow(workflow).await.unwrap();
        
        let document = create_test_document();
        let user = create_test_user();
        
        let instance_id = engine.start_workflow(
            workflow_id,
            document.id().clone(),
            user.id,
            None,
        ).await.unwrap();
        
        // Execute transition from start to end
        let result = engine.execute_transition(
            instance_id.clone(),
            NodeId::new("start"),
            NodeId::new("end"),
            user.id,
            &document,
            &user,
            None,
        ).await;
        
        assert!(result.is_ok());
        
        let instance = engine.get_workflow_instance(&instance_id).await.unwrap();
        assert_eq!(instance.status, WorkflowStatus::Completed);
        assert_eq!(instance.current_nodes, vec![NodeId::new("end")]);
        assert_eq!(instance.history.len(), 1);
    }
    
    #[tokio::test]
    async fn test_workflow_cancellation() {
        let engine = WorkflowEngine::new();
        let workflow = create_simple_workflow();
        let workflow_id = workflow.id.clone();
        
        engine.register_workflow(workflow).await.unwrap();
        
        let user_id = Uuid::new_v4();
        let instance_id = engine.start_workflow(
            workflow_id,
            DocumentId::new(),
            user_id,
            None,
        ).await.unwrap();
        
        let result = engine.cancel_workflow(
            instance_id.clone(),
            user_id,
            "Test cancellation".to_string(),
        ).await;
        
        assert!(result.is_ok());
        
        let instance = engine.get_workflow_instance(&instance_id).await.unwrap();
        assert_eq!(instance.status, WorkflowStatus::Cancelled);
        assert!(instance.completed_at.is_some());
    }
}