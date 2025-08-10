//! Workflow Definition Structures
//!
//! This module defines the core structures for workflow definitions including
//! nodes, edges, and graph representations.

use super::*;
// use crate::value_objects::DocumentId; // Not used in this module
use std::collections::{HashMap, HashSet};

/// Complete workflow definition including graph structure and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Unique workflow identifier
    pub id: WorkflowId,
    /// Human-readable workflow name
    pub name: String,
    /// Version identifier for workflow evolution
    pub version: String,
    /// Detailed description of workflow purpose
    pub description: String,
    /// The workflow graph structure
    pub graph: WorkflowGraph,
    /// Variable definitions available to this workflow
    pub variables: HashMap<String, WorkflowVariable>,
    /// When this workflow was created
    pub created_at: DateTime<Utc>,
    /// Who created this workflow
    pub created_by: Uuid,
    /// Whether this workflow is active and can be used
    pub is_active: bool,
    /// Workflow category for organization
    pub category: String,
    /// Tags for searchability
    pub tags: Vec<String>,
}

impl WorkflowDefinition {
    pub fn new(
        name: String,
        description: String,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: WorkflowId::new(),
            name,
            version: "1.0".to_string(),
            description,
            graph: WorkflowGraph::new(),
            variables: HashMap::new(),
            created_at: Utc::now(),
            created_by,
            is_active: true,
            category: "General".to_string(),
            tags: Vec::new(),
        }
    }
    
    /// Validate workflow definition for correctness
    pub fn validate(&self) -> WorkflowResult<()> {
        self.graph.validate()?;
        
        // Ensure at least one start node
        if self.graph.start_nodes.is_empty() {
            return Err(WorkflowError::InvalidDefinition {
                reason: "Workflow must have at least one start node".to_string(),
            });
        }
        
        // Ensure at least one end node
        if self.graph.end_nodes.is_empty() {
            return Err(WorkflowError::InvalidDefinition {
                reason: "Workflow must have at least one end node".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Get all nodes that can be reached from start nodes
    pub fn get_reachable_nodes(&self) -> HashSet<NodeId> {
        let mut reachable = HashSet::new();
        let mut to_visit = self.graph.start_nodes.clone();
        
        while let Some(node_id) = to_visit.pop() {
            if reachable.insert(node_id.clone()) {
                // Add all nodes reachable from this node
                for edge in self.graph.edges.values() {
                    if edge.from_node == node_id {
                        to_visit.push(edge.to_node.clone());
                    }
                }
            }
        }
        
        reachable
    }
}

/// Graph structure representing workflow flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGraph {
    /// All nodes in the workflow
    pub nodes: HashMap<NodeId, WorkflowNode>,
    /// All edges connecting nodes
    pub edges: HashMap<EdgeId, WorkflowEdge>,
    /// Starting points for workflow execution
    pub start_nodes: Vec<NodeId>,
    /// Terminal nodes that complete workflow
    pub end_nodes: Vec<NodeId>,
}

impl WorkflowGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            start_nodes: Vec::new(),
            end_nodes: Vec::new(),
        }
    }
    
    pub fn add_node(&mut self, id: NodeId, node: WorkflowNode) {
        match &node {
            WorkflowNode::Start(_) => {
                if !self.start_nodes.contains(&id) {
                    self.start_nodes.push(id.clone());
                }
            }
            WorkflowNode::End(_) => {
                if !self.end_nodes.contains(&id) {
                    self.end_nodes.push(id.clone());
                }
            }
            _ => {}
        }
        self.nodes.insert(id, node);
    }
    
    pub fn add_edge(&mut self, id: EdgeId, from: NodeId, to: NodeId, condition: Option<Condition>) {
        let edge = WorkflowEdge {
            id: id.clone(),
            from_node: from,
            to_node: to,
            condition,
            weight: 0,
            metadata: HashMap::new(),
        };
        self.edges.insert(id, edge);
    }
    
    pub fn get_outgoing_edges(&self, node_id: &NodeId) -> Vec<&WorkflowEdge> {
        self.edges.values()
            .filter(|edge| edge.from_node == *node_id)
            .collect()
    }
    
    pub fn get_incoming_edges(&self, node_id: &NodeId) -> Vec<&WorkflowEdge> {
        self.edges.values()
            .filter(|edge| edge.to_node == *node_id)
            .collect()
    }
    
    /// Validate graph structure for correctness
    pub fn validate(&self) -> WorkflowResult<()> {
        // Check all edges reference valid nodes
        for edge in self.edges.values() {
            if !self.nodes.contains_key(&edge.from_node) {
                return Err(WorkflowError::InvalidDefinition {
                    reason: format!("Edge references non-existent from_node: {}", edge.from_node.as_str()),
                });
            }
            if !self.nodes.contains_key(&edge.to_node) {
                return Err(WorkflowError::InvalidDefinition {
                    reason: format!("Edge references non-existent to_node: {}", edge.to_node.as_str()),
                });
            }
        }
        
        // Check for isolated nodes (except start/end nodes)
        for (node_id, node) in &self.nodes {
            let has_incoming = self.get_incoming_edges(node_id).len() > 0;
            let has_outgoing = self.get_outgoing_edges(node_id).len() > 0;
            
            match node {
                WorkflowNode::Start(_) => {
                    if !has_outgoing {
                        return Err(WorkflowError::InvalidDefinition {
                            reason: format!("Start node {} has no outgoing edges", node_id.as_str()),
                        });
                    }
                }
                WorkflowNode::End(_) => {
                    if !has_incoming {
                        return Err(WorkflowError::InvalidDefinition {
                            reason: format!("End node {} has no incoming edges", node_id.as_str()),
                        });
                    }
                }
                _ => {
                    if !has_incoming && !has_outgoing {
                        return Err(WorkflowError::InvalidDefinition {
                            reason: format!("Node {} is isolated (no incoming or outgoing edges)", node_id.as_str()),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for WorkflowGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow nodes representing different types of workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowNode {
    /// Starting point of workflow
    Start(StartNode),
    /// Human or system task
    Task(TaskNode),
    /// Decision point with branching
    Decision(DecisionNode),
    /// Split into parallel paths
    Parallel(ParallelNode),
    /// Join parallel paths back together
    Join(JoinNode),
    /// Timer-based delays or deadlines
    Timer(TimerNode),
    /// Terminal node
    End(EndNode),
}

impl WorkflowNode {
    pub fn id(&self) -> &NodeId {
        match self {
            WorkflowNode::Start(n) => &n.id,
            WorkflowNode::Task(n) => &n.id,
            WorkflowNode::Decision(n) => &n.id,
            WorkflowNode::Parallel(n) => &n.id,
            WorkflowNode::Join(n) => &n.id,
            WorkflowNode::Timer(n) => &n.id,
            WorkflowNode::End(n) => &n.id,
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            WorkflowNode::Start(n) => &n.name,
            WorkflowNode::Task(n) => &n.name,
            WorkflowNode::Decision(n) => &n.name,
            WorkflowNode::Parallel(n) => &n.name,
            WorkflowNode::Join(n) => &n.name,
            WorkflowNode::Timer(n) => &n.name,
            WorkflowNode::End(n) => &n.name,
        }
    }
}

/// Start node - entry point to workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartNode {
    pub id: NodeId,
    pub name: String,
    /// Actions to execute when workflow starts
    pub actions: Vec<WorkflowAction>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task node - represents work to be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: NodeId,
    pub name: String,
    /// Type of task (manual, automatic, etc.)
    pub task_type: TaskType,
    /// Users assigned to this task
    pub assignees: Vec<Uuid>,
    /// How long this task should take
    pub duration_sla: Option<Duration>,
    /// Guards that must pass before task can be completed
    pub guards: Vec<Guard>,
    /// Actions to execute when task is completed
    pub actions: Vec<WorkflowAction>,
    /// Additional task metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of tasks in workflow
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// Human task requiring user interaction
    Manual,
    /// Automated system task
    Automatic,
    /// Review/approval task
    Review,
    /// Notification task
    Notification,
    /// Integration with external system
    Integration,
}

/// Decision node - branching logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionNode {
    pub id: NodeId,
    pub name: String,
    /// Named conditions for different branches
    pub conditions: Vec<(String, Condition)>,
    /// Actions to execute during decision evaluation
    pub actions: Vec<WorkflowAction>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Parallel node - split execution into multiple paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelNode {
    pub id: NodeId,
    pub name: String,
    /// Minimum number of branches that must complete
    pub min_branches: usize,
    /// Actions to execute when splitting
    pub actions: Vec<WorkflowAction>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Join node - merge parallel paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinNode {
    pub id: NodeId,
    pub name: String,
    /// Strategy for joining parallel branches
    pub join_strategy: JoinStrategy,
    /// Actions to execute when joining
    pub actions: Vec<WorkflowAction>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Strategies for joining parallel branches
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinStrategy {
    /// Wait for all branches to complete
    WaitAll,
    /// Wait for any branch to complete
    WaitAny,
    /// Wait for specific number of branches
    WaitCount(usize),
    /// Wait for specific branches by ID
    WaitSpecific(Vec<NodeId>),
}

/// Timer node - time-based workflow control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerNode {
    pub id: NodeId,
    pub name: String,
    /// Type of timer behavior
    pub timer_type: TimerType,
    /// Duration to wait or deadline to enforce
    pub duration: Duration,
    /// Actions on timer expiration
    pub actions_on_timeout: Vec<WorkflowAction>,
    /// Escalation rules for SLA breaches
    pub escalation_rules: Vec<EscalationRule>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Timer behavior types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerType {
    /// Hard deadline - workflow fails if exceeded
    Deadline,
    /// Soft deadline - escalate but continue
    SLA,
    /// Send reminder notifications
    Reminder,
    /// Automatic cleanup actions
    Cleanup,
    /// Simple delay
    Delay,
}

/// End node - workflow termination point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndNode {
    pub id: NodeId,
    pub name: String,
    /// Final actions to execute
    pub actions: Vec<WorkflowAction>,
    /// Workflow completion status
    pub completion_status: CompletionStatus,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Workflow completion status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionStatus {
    /// Workflow completed successfully
    Success,
    /// Workflow completed with warnings
    Warning,
    /// Workflow completed with errors
    Error,
    /// Workflow was cancelled
    Cancelled,
}

/// Edge connecting workflow nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    /// Unique edge identifier
    pub id: EdgeId,
    /// Source node
    pub from_node: NodeId,
    /// Destination node
    pub to_node: NodeId,
    /// Condition that must be met for transition
    pub condition: Option<Condition>,
    /// Edge weight for prioritization
    pub weight: i32,
    /// Additional edge metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl WorkflowEdge {
    pub fn new(id: EdgeId, from: NodeId, to: NodeId) -> Self {
        Self {
            id,
            from_node: from,
            to_node: to,
            condition: None,
            weight: 0,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }
    
    pub fn with_weight(mut self, weight: i32) -> Self {
        self.weight = weight;
        self
    }
}

/// Condition for edge transitions or decision branching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Expression to evaluate (e.g., "document.size > 1MB")
    pub expression: String,
    /// Variables referenced in expression
    pub variables: Vec<String>,
    /// Type of condition evaluation
    pub condition_type: ConditionType,
}

impl Condition {
    pub fn boolean(expression: String) -> Self {
        Self {
            variables: Vec::new(),
            expression,
            condition_type: ConditionType::Boolean,
        }
    }
    
    pub fn expression(expression: String) -> Self {
        // Simple variable extraction (look for ${var} patterns)
        let variables = extract_variables(&expression);
        Self {
            expression,
            variables,
            condition_type: ConditionType::Expression,
        }
    }
    
    pub fn guard(guard_name: String) -> Self {
        Self {
            expression: guard_name.clone(),
            variables: vec![guard_name],
            condition_type: ConditionType::Guard,
        }
    }
    
    pub fn timer(duration: Duration) -> Self {
        Self {
            expression: format!("timer:{}", duration.num_seconds()),
            variables: Vec::new(),
            condition_type: ConditionType::Timer,
        }
    }
}

/// Types of condition evaluation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionType {
    /// Simple boolean evaluation
    Boolean,
    /// Complex expression evaluation
    Expression,
    /// Custom guard function
    Guard,
    /// Time-based condition
    Timer,
}

/// Extract variable references from expression string
fn extract_variables(expression: &str) -> Vec<String> {
    // Simple implementation - look for ${variable} or variable patterns
    let mut variables = Vec::new();
    
    // Pattern: ${variable_name}
    if let Ok(regex) = regex::Regex::new(r"\$\{([^}]+)\}") {
        for cap in regex.captures_iter(expression) {
            if let Some(var) = cap.get(1) {
                variables.push(var.as_str().to_string());
            }
        }
    }
    
    // Pattern: document.field, user.name, etc.
    if let Ok(regex) = regex::Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*\.[a-zA-Z_][a-zA-Z0-9_]*)\b") {
        for cap in regex.captures_iter(expression) {
            if let Some(var) = cap.get(1) {
                let var_name = var.as_str().to_string();
                if !variables.contains(&var_name) {
                    variables.push(var_name);
                }
            }
        }
    }
    
    variables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_definition_creation() {
        let user_id = Uuid::new_v4();
        let workflow = WorkflowDefinition::new(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
            user_id,
        );
        
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.created_by, user_id);
        assert!(workflow.is_active);
        assert_eq!(workflow.version, "1.0");
    }
    
    #[test]
    fn test_workflow_graph_operations() {
        let mut graph = WorkflowGraph::new();
        
        let start_node = WorkflowNode::Start(StartNode {
            id: NodeId::new("start"),
            name: "Start".to_string(),
            actions: vec![],
            metadata: HashMap::new(),
        });
        
        let end_node = WorkflowNode::End(EndNode {
            id: NodeId::new("end"),
            name: "End".to_string(),
            actions: vec![],
            completion_status: CompletionStatus::Success,
            metadata: HashMap::new(),
        });
        
        graph.add_node(NodeId::new("start"), start_node);
        graph.add_node(NodeId::new("end"), end_node);
        graph.add_edge(
            EdgeId::new("start_to_end"),
            NodeId::new("start"),
            NodeId::new("end"),
            None,
        );
        
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.start_nodes.len(), 1);
        assert_eq!(graph.end_nodes.len(), 1);
        
        assert!(graph.validate().is_ok());
    }
    
    #[test]
    fn test_condition_variable_extraction() {
        let variables = extract_variables("document.size > 1000 && user.role == 'admin'");
        assert!(variables.contains(&"document.size".to_string()));
        assert!(variables.contains(&"user.role".to_string()));
        
        let variables2 = extract_variables("${workflow.state} == 'approved'");
        assert!(variables2.contains(&"workflow.state".to_string()));
    }
    
    #[test]
    fn test_workflow_edge_builder() {
        let edge = WorkflowEdge::new(
            EdgeId::new("test_edge"),
            NodeId::new("from"),
            NodeId::new("to"),
        )
        .with_condition(Condition::boolean("true".to_string()))
        .with_weight(10);
        
        assert!(edge.condition.is_some());
        assert_eq!(edge.weight, 10);
    }
    
    #[test]
    fn test_graph_validation_fails_for_isolated_nodes() {
        let mut graph = WorkflowGraph::new();
        
        let isolated_task = WorkflowNode::Task(TaskNode {
            id: NodeId::new("isolated"),
            name: "Isolated Task".to_string(),
            task_type: TaskType::Manual,
            assignees: vec![],
            duration_sla: None,
            guards: vec![],
            actions: vec![],
            metadata: HashMap::new(),
        });
        
        graph.add_node(NodeId::new("isolated"), isolated_task);
        
        assert!(graph.validate().is_err());
    }
}