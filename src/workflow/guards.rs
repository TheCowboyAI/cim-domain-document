//! Workflow Guards and Validation System
//!
//! This module implements guard conditions and business rule validation
//! for workflow state transitions.

use super::*;
use crate::{Document, value_objects::DocumentState};

/// Guard conditions that must be satisfied for workflow transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Guard {
    /// User must have specific role
    RequireRole(String),
    /// User must have specific permission
    RequirePermission(Permission),
    /// Document size constraints
    DocumentSizeLimit { min: Option<u64>, max: Option<u64> },
    /// Time window constraints
    TimeWindow(TimeWindow),
    /// Minimum number of approvals required
    ApprovalCount { required: u32, current: u32 },
    /// Document must be in specific state
    DocumentState(DocumentState),
    /// Custom guard function
    Custom(String),
    /// Multiple guards that must all pass
    All(Vec<Guard>),
    /// At least one guard must pass
    Any(Vec<Guard>),
    /// Guard must not pass
    Not(Box<Guard>),
}

impl Guard {
    pub fn role(role: impl Into<String>) -> Self {
        Self::RequireRole(role.into())
    }
    
    pub fn permission(permission: Permission) -> Self {
        Self::RequirePermission(permission)
    }
    
    pub fn document_state(state: DocumentState) -> Self {
        Self::DocumentState(state)
    }
    
    pub fn size_limit(min: Option<u64>, max: Option<u64>) -> Self {
        Self::DocumentSizeLimit { min, max }
    }
    
    pub fn approval_count(required: u32) -> Self {
        Self::ApprovalCount { required, current: 0 }
    }
    
    pub fn time_window(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self::TimeWindow(TimeWindow { start, end })
    }
    
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }
    
    pub fn all(guards: Vec<Guard>) -> Self {
        Self::All(guards)
    }
    
    pub fn any(guards: Vec<Guard>) -> Self {
        Self::Any(guards)
    }
    
    pub fn not(guard: Guard) -> Self {
        Self::Not(Box::new(guard))
    }
}

/// Context provided to guards for evaluation
#[derive(Debug, Clone)]
pub struct GuardContext<'a> {
    /// Document being processed
    pub document: &'a Document,
    /// User attempting the transition
    pub user: &'a User,
    /// Current workflow instance
    pub workflow_instance: &'a WorkflowInstance,
    /// Current timestamp
    pub current_time: DateTime<Utc>,
    /// Additional context variables
    pub variables: &'a HashMap<String, serde_json::Value>,
}

/// User information for guard evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<Permission>,
    pub department: Option<String>,
    pub manager_id: Option<Uuid>,
}

impl User {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
    
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
}

/// Result of guard evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardResult {
    /// Guard allows the transition
    Allow,
    /// Guard denies the transition with reason
    Deny(String),
    /// Guard requires additional conditions to be met
    RequireAdditional(Vec<Requirement>),
}

impl GuardResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, GuardResult::Allow)
    }
    
    pub fn is_denied(&self) -> bool {
        matches!(self, GuardResult::Deny(_))
    }
    
    pub fn combine(results: Vec<GuardResult>) -> GuardResult {
        let mut denials = Vec::new();
        let mut requirements = Vec::new();
        
        for result in results {
            match result {
                GuardResult::Deny(reason) => denials.push(reason),
                GuardResult::RequireAdditional(mut reqs) => requirements.append(&mut reqs),
                GuardResult::Allow => {}
            }
        }
        
        if !denials.is_empty() {
            GuardResult::Deny(denials.join("; "))
        } else if !requirements.is_empty() {
            GuardResult::RequireAdditional(requirements)
        } else {
            GuardResult::Allow
        }
    }
}

/// Requirements that must be fulfilled
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Requirement {
    /// Additional approval needed
    AdditionalApproval(Uuid),
    /// Document modification needed
    DocumentUpdate(String),
    /// Wait for specific time
    WaitUntil(DateTime<Utc>),
    /// Complete specific task
    CompleteTask(String),
    /// Upload additional documents
    UploadDocuments(Vec<String>),
}

/// Guard evaluator trait for extensible guard system
pub trait GuardEvaluator {
    /// Evaluate a guard against the provided context
    fn evaluate(&self, guard: &Guard, context: &GuardContext) -> GuardResult;
}

/// Default implementation of guard evaluator
#[derive(Debug, Clone)]
pub struct DefaultGuardEvaluator {
    /// Custom guard functions
    pub custom_guards: HashMap<String, Box<dyn Fn(&GuardContext) -> GuardResult + Send + Sync>>,
}

impl DefaultGuardEvaluator {
    pub fn new() -> Self {
        Self {
            custom_guards: HashMap::new(),
        }
    }
    
    /// Register a custom guard function
    pub fn register_custom_guard<F>(&mut self, name: String, guard_fn: F)
    where
        F: Fn(&GuardContext) -> GuardResult + Send + Sync + 'static,
    {
        self.custom_guards.insert(name, Box::new(guard_fn));
    }
}

impl Default for DefaultGuardEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl GuardEvaluator for DefaultGuardEvaluator {
    fn evaluate(&self, guard: &Guard, context: &GuardContext) -> GuardResult {
        match guard {
            Guard::RequireRole(role) => {
                if context.user.has_role(role) {
                    GuardResult::Allow
                } else {
                    GuardResult::Deny(format!("User must have role: {}", role))
                }
            }
            
            Guard::RequirePermission(permission) => {
                if context.user.has_permission(permission) {
                    GuardResult::Allow
                } else {
                    GuardResult::Deny(format!("User must have permission: {:?}", permission))
                }
            }
            
            Guard::DocumentSizeLimit { min, max } => {
                // Get document size from component
                if let Some(info) = context.document.get_component::<crate::aggregate::DocumentInfoComponent>() {
                    let size = info.size_bytes;
                    
                    if let Some(min_size) = min {
                        if size < *min_size {
                            return GuardResult::Deny(format!("Document size {} is below minimum {}", size, min_size));
                        }
                    }
                    
                    if let Some(max_size) = max {
                        if size > *max_size {
                            return GuardResult::Deny(format!("Document size {} exceeds maximum {}", size, max_size));
                        }
                    }
                    
                    GuardResult::Allow
                } else {
                    GuardResult::Deny("Document size information not available".to_string())
                }
            }
            
            Guard::TimeWindow(window) => {
                if window.contains(context.current_time) {
                    GuardResult::Allow
                } else {
                    GuardResult::Deny(format!("Current time {} is outside allowed window", context.current_time))
                }
            }
            
            Guard::ApprovalCount { required, current } => {
                if current >= required {
                    GuardResult::Allow
                } else {
                    let needed = required - current;
                    GuardResult::RequireAdditional(
                        (0..needed).map(|_| Requirement::AdditionalApproval(Uuid::nil())).collect()
                    )
                }
            }
            
            Guard::DocumentState(required_state) => {
                // Get current document state from workflow context
                if let Some(current_state_value) = context.variables.get("document.state") {
                    if let Ok(current_state) = serde_json::from_value::<DocumentState>(current_state_value.clone()) {
                        if current_state == *required_state {
                            GuardResult::Allow
                        } else {
                            GuardResult::Deny(format!("Document must be in state {:?}, currently {:?}", required_state, current_state))
                        }
                    } else {
                        GuardResult::Deny("Invalid document state format".to_string())
                    }
                } else {
                    GuardResult::Deny("Document state not available in context".to_string())
                }
            }
            
            Guard::Custom(name) => {
                if let Some(guard_fn) = self.custom_guards.get(name) {
                    guard_fn(context)
                } else {
                    GuardResult::Deny(format!("Unknown custom guard: {}", name))
                }
            }
            
            Guard::All(guards) => {
                let results: Vec<GuardResult> = guards.iter()
                    .map(|g| self.evaluate(g, context))
                    .collect();
                
                // All must pass
                for result in &results {
                    if !result.is_allowed() {
                        return result.clone();
                    }
                }
                GuardResult::Allow
            }
            
            Guard::Any(guards) => {
                let results: Vec<GuardResult> = guards.iter()
                    .map(|g| self.evaluate(g, context))
                    .collect();
                
                // At least one must pass
                for result in &results {
                    if result.is_allowed() {
                        return GuardResult::Allow;
                    }
                }
                
                // None passed, combine all denial reasons
                GuardResult::combine(results)
            }
            
            Guard::Not(guard) => {
                let result = self.evaluate(guard, context);
                match result {
                    GuardResult::Allow => GuardResult::Deny("Guard must not pass".to_string()),
                    GuardResult::Deny(_) => GuardResult::Allow,
                    GuardResult::RequireAdditional(_) => GuardResult::Allow, // Requirements not met = good for NOT
                }
            }
        }
    }
}

/// Workflow instance state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: WorkflowInstanceId,
    pub workflow_id: WorkflowId,
    pub document_id: DocumentId,
    pub current_nodes: Vec<NodeId>,
    pub status: WorkflowStatus,
    pub context: WorkflowContext,
    pub history: Vec<WorkflowTransition>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub started_by: Uuid,
}

impl WorkflowInstance {
    pub fn new(
        workflow_id: WorkflowId,
        document_id: DocumentId,
        started_by: Uuid,
        initial_nodes: Vec<NodeId>,
    ) -> Self {
        Self {
            id: WorkflowInstanceId::new(),
            workflow_id,
            document_id,
            current_nodes: initial_nodes,
            status: WorkflowStatus::Running,
            context: WorkflowContext::new(),
            history: Vec::new(),
            started_at: Utc::now(),
            completed_at: None,
            started_by,
        }
    }
    
    pub fn is_at_node(&self, node_id: &NodeId) -> bool {
        self.current_nodes.contains(node_id)
    }
    
    pub fn add_transition(&mut self, transition: WorkflowTransition) {
        self.history.push(transition);
    }
    
    pub fn get_transitions_from_node(&self, node_id: &NodeId) -> Vec<&WorkflowTransition> {
        self.history.iter()
            .filter(|t| t.from_node == *node_id)
            .collect()
    }
    
    pub fn get_transitions_to_node(&self, node_id: &NodeId) -> Vec<&WorkflowTransition> {
        self.history.iter()
            .filter(|t| t.to_node == *node_id)
            .collect()
    }
}

/// State transition validator
#[derive(Debug)]
pub struct StateTransitionValidator {
    pub guard_evaluator: Box<dyn GuardEvaluator + Send + Sync>,
}

impl StateTransitionValidator {
    pub fn new() -> Self {
        Self {
            guard_evaluator: Box::new(DefaultGuardEvaluator::new()),
        }
    }
    
    pub fn with_evaluator(guard_evaluator: Box<dyn GuardEvaluator + Send + Sync>) -> Self {
        Self {
            guard_evaluator,
        }
    }
    
    /// Validate if transition from one node to another is allowed
    pub fn can_transition(
        &self,
        workflow_def: &WorkflowDefinition,
        workflow_instance: &WorkflowInstance,
        from_node: &NodeId,
        to_node: &NodeId,
        document: &Document,
        user: &User,
    ) -> WorkflowResult<()> {
        // Check if edge exists in workflow definition
        let edge = self.find_edge(workflow_def, from_node, to_node)?;
        
        // Evaluate edge condition if present
        if let Some(condition) = &edge.condition {
            let context = GuardContext {
                document,
                user,
                workflow_instance,
                current_time: Utc::now(),
                variables: &workflow_instance.context.variables,
            };
            
            let condition_result = self.evaluate_condition(condition, &context)?;
            if !condition_result {
                return Err(WorkflowError::InvalidTransition {
                    from: from_node.as_str().to_string(),
                    to: to_node.as_str().to_string(),
                    reason: "Condition not met".to_string(),
                });
            }
        }
        
        // Evaluate guards on target node
        if let Some(target_node) = workflow_def.graph.nodes.get(to_node) {
            let guards = self.get_node_guards(target_node);
            
            let context = GuardContext {
                document,
                user,
                workflow_instance,
                current_time: Utc::now(),
                variables: &workflow_instance.context.variables,
            };
            
            for guard in guards {
                let result = self.guard_evaluator.evaluate(&guard, &context);
                match result {
                    GuardResult::Allow => continue,
                    GuardResult::Deny(reason) => {
                        return Err(WorkflowError::GuardFailed {
                            guard: format!("{:?}", guard),
                            reason,
                        });
                    }
                    GuardResult::RequireAdditional(requirements) => {
                        return Err(WorkflowError::GuardFailed {
                            guard: format!("{:?}", guard),
                            reason: format!("Additional requirements: {:?}", requirements),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Find edge between two nodes
    fn find_edge(
        &self,
        workflow_def: &WorkflowDefinition,
        from_node: &NodeId,
        to_node: &NodeId,
    ) -> WorkflowResult<&WorkflowEdge> {
        workflow_def.graph.edges.values()
            .find(|edge| edge.from_node == *from_node && edge.to_node == *to_node)
            .ok_or_else(|| WorkflowError::InvalidTransition {
                from: from_node.as_str().to_string(),
                to: to_node.as_str().to_string(),
                reason: "No edge exists between nodes".to_string(),
            })
    }
    
    /// Get guards associated with a node
    fn get_node_guards(&self, node: &WorkflowNode) -> Vec<Guard> {
        match node {
            WorkflowNode::Task(task) => task.guards.clone(),
            WorkflowNode::Decision(_) => vec![], // Decision nodes don't have guards, only conditions
            _ => vec![], // Other nodes don't have guards by default
        }
    }
    
    /// Evaluate condition expression
    fn evaluate_condition(
        &self,
        condition: &Condition,
        context: &GuardContext,
    ) -> WorkflowResult<bool> {
        match condition.condition_type {
            ConditionType::Boolean => {
                // Simple boolean evaluation
                match condition.expression.as_str() {
                    "true" => Ok(true),
                    "false" => Ok(false),
                    _ => Ok(false), // Default to false for unknown expressions
                }
            }
            ConditionType::Expression => {
                // For now, simple expression evaluation
                // In a real implementation, you'd use an expression parser
                self.evaluate_simple_expression(&condition.expression, context)
            }
            ConditionType::Guard => {
                // Treat as custom guard
                let guard = Guard::Custom(condition.expression.clone());
                let result = self.guard_evaluator.evaluate(&guard, context);
                Ok(result.is_allowed())
            }
            ConditionType::Timer => {
                // Timer conditions are handled by the workflow engine
                Ok(true) // Assume timer condition is met for now
            }
        }
    }
    
    /// Simple expression evaluator (placeholder for more sophisticated parser)
    fn evaluate_simple_expression(
        &self,
        expression: &str,
        context: &GuardContext,
    ) -> WorkflowResult<bool> {
        // Very basic expression evaluation
        // In production, you'd want a proper expression parser/evaluator
        
        if expression.contains("==") {
            let parts: Vec<&str> = expression.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim().trim_matches('"').trim_matches('\'');
                
                if let Some(value) = context.variables.get(left) {
                    if let Some(value_str) = value.as_str() {
                        return Ok(value_str == right);
                    }
                }
            }
        }
        
        // Default to false for unhandled expressions
        Ok(false)
    }
}

impl Default for StateTransitionValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::DocumentInfoComponent;

    fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            roles: vec!["author".to_string()],
            permissions: vec![Permission::View, Permission::CompleteTask],
            department: Some("Engineering".to_string()),
            manager_id: None,
        }
    }
    
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

    #[test]
    fn test_guard_role_evaluation() {
        let evaluator = DefaultGuardEvaluator::new();
        let user = create_test_user();
        let document = create_test_document();
        let instance = WorkflowInstance::new(
            WorkflowId::new(),
            DocumentId::new(),
            user.id,
            vec![NodeId::new("start")],
        );
        
        let context = GuardContext {
            document: &document,
            user: &user,
            workflow_instance: &instance,
            current_time: Utc::now(),
            variables: &HashMap::new(),
        };
        
        let guard = Guard::RequireRole("author".to_string());
        let result = evaluator.evaluate(&guard, &context);
        assert_eq!(result, GuardResult::Allow);
        
        let guard = Guard::RequireRole("admin".to_string());
        let result = evaluator.evaluate(&guard, &context);
        assert!(result.is_denied());
    }
    
    #[test]
    fn test_guard_size_limit_evaluation() {
        let evaluator = DefaultGuardEvaluator::new();
        let user = create_test_user();
        let document = create_test_document();
        let instance = WorkflowInstance::new(
            WorkflowId::new(),
            DocumentId::new(),
            user.id,
            vec![NodeId::new("start")],
        );
        
        let context = GuardContext {
            document: &document,
            user: &user,
            workflow_instance: &instance,
            current_time: Utc::now(),
            variables: &HashMap::new(),
        };
        
        let guard = Guard::DocumentSizeLimit { min: None, max: Some(2048) };
        let result = evaluator.evaluate(&guard, &context);
        assert_eq!(result, GuardResult::Allow);
        
        let guard = Guard::DocumentSizeLimit { min: None, max: Some(512) };
        let result = evaluator.evaluate(&guard, &context);
        assert!(result.is_denied());
    }
    
    #[test]
    fn test_guard_combination() {
        let evaluator = DefaultGuardEvaluator::new();
        let user = create_test_user();
        let document = create_test_document();
        let instance = WorkflowInstance::new(
            WorkflowId::new(),
            DocumentId::new(),
            user.id,
            vec![NodeId::new("start")],
        );
        
        let context = GuardContext {
            document: &document,
            user: &user,
            workflow_instance: &instance,
            current_time: Utc::now(),
            variables: &HashMap::new(),
        };
        
        // All guards must pass
        let guard = Guard::All(vec![
            Guard::RequireRole("author".to_string()),
            Guard::RequirePermission(Permission::View),
        ]);
        let result = evaluator.evaluate(&guard, &context);
        assert_eq!(result, GuardResult::Allow);
        
        // Any guard can pass
        let guard = Guard::Any(vec![
            Guard::RequireRole("admin".to_string()),
            Guard::RequireRole("author".to_string()),
        ]);
        let result = evaluator.evaluate(&guard, &context);
        assert_eq!(result, GuardResult::Allow);
    }
    
    #[test]
    fn test_workflow_instance_creation() {
        let workflow_id = WorkflowId::new();
        let document_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        let nodes = vec![NodeId::new("start")];
        
        let instance = WorkflowInstance::new(workflow_id.clone(), document_id.clone(), user_id, nodes.clone());
        
        assert_eq!(instance.workflow_id, workflow_id);
        assert_eq!(instance.document_id, document_id);
        assert_eq!(instance.started_by, user_id);
        assert_eq!(instance.current_nodes, nodes);
        assert_eq!(instance.status, WorkflowStatus::Running);
    }
}