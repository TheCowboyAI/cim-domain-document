# Document Workflow State Machine Design

## Overview

This document defines the design for comprehensive document workflow state machines using cim-graph integration to replace the current basic state enum system with a robust, configurable workflow engine.

## Current State Analysis

### Existing Components
- **DocumentState enum**: Basic 5-state workflow (Draft → InReview → Approved/Rejected → Archived)
- **DocumentStatus enum**: Lifecycle states in LifecycleComponent
- **ChangeState command**: Manual state transitions
- **StateChanged event**: State transition recording

### Limitations of Current System
- No transition validation (any state can change to any other state)
- No workflow enforcement or business rules
- No automated actions or timers
- No parallel workflows or complex routing
- Hard-coded state definitions

## New Workflow State Machine Architecture

### 1. Core Workflow Components

#### WorkflowDefinition
```rust
pub struct WorkflowDefinition {
    pub id: WorkflowId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub graph: WorkflowGraph,
    pub variables: HashMap<String, WorkflowVariable>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub is_active: bool,
}

pub struct WorkflowGraph {
    pub nodes: HashMap<NodeId, WorkflowNode>,
    pub edges: HashMap<EdgeId, WorkflowEdge>,
    pub start_nodes: Vec<NodeId>,
    pub end_nodes: Vec<NodeId>,
}
```

#### WorkflowNode Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowNode {
    Start(StartNode),
    Task(TaskNode),
    Decision(DecisionNode),
    Parallel(ParallelNode),
    Join(JoinNode),
    Timer(TimerNode),
    End(EndNode),
}

pub struct TaskNode {
    pub id: NodeId,
    pub name: String,
    pub node_type: TaskType,
    pub assignees: Vec<Uuid>,
    pub duration_sla: Option<Duration>,
    pub guards: Vec<Guard>,
    pub actions: Vec<Action>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub enum TaskType {
    Manual,      // Human task
    Automatic,   // System task
    Review,      // Approval task
    Notification,// Send notification
    Integration, // External system call
}
```

#### WorkflowEdge
```rust
pub struct WorkflowEdge {
    pub id: EdgeId,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub condition: Option<Condition>,
    pub weight: i32,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct Condition {
    pub expression: String,        // "document.size > 1MB"
    pub variables: Vec<String>,    // ["document.size"]
    pub condition_type: ConditionType,
}

pub enum ConditionType {
    Boolean,     // Simple true/false
    Expression,  // Complex expression evaluation
    Guard,       // Custom guard function
    Timer,       // Time-based condition
}
```

### 2. Workflow Instance Management

#### WorkflowInstance
```rust
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

pub enum WorkflowStatus {
    Running,
    Suspended,
    Completed,
    Failed,
    Cancelled,
    Escalated,
}

pub struct WorkflowContext {
    pub variables: HashMap<String, serde_json::Value>,
    pub permissions: HashMap<Uuid, Vec<Permission>>,
    pub sla_deadlines: HashMap<NodeId, DateTime<Utc>>,
    pub escalation_rules: Vec<EscalationRule>,
}
```

### 3. Pre-defined Workflow Templates

#### Standard Document Approval Workflow
```rust
pub fn create_standard_approval_workflow() -> WorkflowDefinition {
    let mut graph = WorkflowGraph::new();
    
    // Nodes
    let start = StartNode { id: "start".into(), name: "Start".into() };
    let draft = TaskNode {
        id: "draft".into(),
        name: "Draft Creation".into(),
        node_type: TaskType::Manual,
        assignees: vec![], // Dynamic assignment
        duration_sla: Some(Duration::days(7)),
        guards: vec![Guard::RequireRole("author".into())],
        actions: vec![Action::SetState("Draft".into())],
        metadata: HashMap::new(),
    };
    
    let review = TaskNode {
        id: "review".into(),
        name: "Review".into(),
        node_type: TaskType::Review,
        assignees: vec![], // Dynamic assignment based on document type
        duration_sla: Some(Duration::days(3)),
        guards: vec![Guard::RequireRole("reviewer".into())],
        actions: vec![
            Action::SetState("InReview".into()),
            Action::SendNotification("ReviewRequired".into()),
        ],
        metadata: HashMap::new(),
    };
    
    let decision = DecisionNode {
        id: "approval_decision".into(),
        name: "Approval Decision".into(),
        conditions: vec![
            ("approve".into(), Condition::expression("review.decision == 'approve'")),
            ("reject".into(), Condition::expression("review.decision == 'reject'")),
            ("revise".into(), Condition::expression("review.decision == 'revise'")),
        ],
    };
    
    // Add nodes to graph
    graph.add_node("start".into(), WorkflowNode::Start(start));
    graph.add_node("draft".into(), WorkflowNode::Task(draft));
    graph.add_node("review".into(), WorkflowNode::Task(review));
    graph.add_node("decision".into(), WorkflowNode::Decision(decision));
    
    // Edges
    graph.add_edge("start_to_draft".into(), "start".into(), "draft".into(), None);
    graph.add_edge("draft_to_review".into(), "draft".into(), "review".into(), 
                   Some(Condition::expression("document.status == 'submitted'")));
    
    WorkflowDefinition {
        id: WorkflowId::new(),
        name: "Standard Document Approval".into(),
        version: "1.0".into(),
        description: "Standard approval workflow for business documents".into(),
        graph,
        variables: HashMap::new(),
        created_at: Utc::now(),
        created_by: Uuid::nil(), // System
        is_active: true,
    }
}
```

#### Contract Review Workflow
```rust
pub fn create_contract_review_workflow() -> WorkflowDefinition {
    // Multi-stage approval with legal review, finance approval, and executive sign-off
    // Parallel review paths for different stakeholders
    // Automated compliance checks
    // Integration with external signature systems
}

pub fn create_compliance_workflow() -> WorkflowDefinition {
    // Automated compliance scanning
    // Risk assessment routing
    // Regulatory approval chains
    // Audit trail requirements
}
```

### 4. Guards and Business Rules

#### Guard System
```rust
pub enum Guard {
    RequireRole(String),
    RequirePermission(Permission),
    DocumentSizeLimit(u64),
    TimeWindow(TimeWindow),
    ApprovalCount(u32),
    Custom(String), // Custom guard function name
}

pub struct GuardContext {
    pub document: Document,
    pub user: User,
    pub workflow_instance: WorkflowInstance,
    pub current_time: DateTime<Utc>,
}

pub trait GuardEvaluator {
    fn evaluate(&self, guard: &Guard, context: &GuardContext) -> GuardResult;
}

pub enum GuardResult {
    Allow,
    Deny(String), // Denial reason
    RequireAdditional(Vec<Requirement>),
}
```

#### Business Rules Engine
```rust
pub struct BusinessRulesEngine {
    pub rules: HashMap<String, BusinessRule>,
}

pub struct BusinessRule {
    pub id: String,
    pub name: String,
    pub condition: Condition,
    pub actions: Vec<Action>,
    pub priority: i32,
    pub is_active: bool,
}

pub enum Action {
    SetState(String),
    AssignTask(Uuid),
    SendNotification(String),
    SetDeadline(Duration),
    EscalateToManager,
    IntegrateWithSystem(String),
    UpdateMetadata(HashMap<String, serde_json::Value>),
}
```

### 5. Automation and Timers

#### Timer-Based Workflow Actions
```rust
pub struct TimerNode {
    pub id: NodeId,
    pub name: String,
    pub timer_type: TimerType,
    pub duration: Duration,
    pub actions_on_timeout: Vec<Action>,
    pub escalation_rules: Vec<EscalationRule>,
}

pub enum TimerType {
    Deadline,     // Hard deadline - workflow fails if exceeded
    SLA,         // Soft deadline - escalate but continue
    Reminder,    // Send notifications
    Cleanup,     // Automatic cleanup actions
}

pub struct EscalationRule {
    pub trigger_after: Duration,
    pub escalate_to: Vec<Uuid>,
    pub actions: Vec<Action>,
    pub repeat_interval: Option<Duration>,
}
```

#### Automated Workflow Triggers
```rust
pub enum WorkflowTrigger {
    DocumentUploaded,
    StateChanged(DocumentState),
    TimerExpired(NodeId),
    ExternalEvent(String),
    UserAction(String),
    ScheduledTime(DateTime<Utc>),
}

pub struct WorkflowAutomationEngine {
    pub triggers: HashMap<WorkflowTrigger, Vec<AutomatedAction>>,
    pub schedulers: HashMap<WorkflowInstanceId, Vec<ScheduledTask>>,
}
```

### 6. Integration with Existing Domain

#### Enhanced Commands
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflow {
    pub document_id: DocumentId,
    pub workflow_id: WorkflowId,
    pub initiated_by: Uuid,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionWorkflow {
    pub workflow_instance_id: WorkflowInstanceId,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub transition_data: HashMap<String, serde_json::Value>,
    pub triggered_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTask {
    pub workflow_instance_id: WorkflowInstanceId,
    pub node_id: NodeId,
    pub completion_data: HashMap<String, serde_json::Value>,
    pub completed_by: Uuid,
}
```

#### Enhanced Events
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStarted {
    pub workflow_instance_id: WorkflowInstanceId,
    pub document_id: DocumentId,
    pub workflow_id: WorkflowId,
    pub started_by: Uuid,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransitioned {
    pub workflow_instance_id: WorkflowInstanceId,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub transition_reason: String,
    pub transitioned_by: Uuid,
    pub transitioned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompleted {
    pub workflow_instance_id: WorkflowInstanceId,
    pub node_id: NodeId,
    pub completed_by: Uuid,
    pub completion_data: HashMap<String, serde_json::Value>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEscalated {
    pub workflow_instance_id: WorkflowInstanceId,
    pub node_id: NodeId,
    pub escalated_to: Vec<Uuid>,
    pub escalation_reason: String,
    pub escalated_at: DateTime<Utc>,
}
```

### 7. Workflow Queries and Analytics

#### Workflow Query Models
```rust
pub struct WorkflowInstanceView {
    pub instance_id: WorkflowInstanceId,
    pub document_id: DocumentId,
    pub workflow_name: String,
    pub current_nodes: Vec<NodeInfo>,
    pub status: WorkflowStatus,
    pub progress_percentage: f64,
    pub sla_status: SLAStatus,
    pub assigned_users: Vec<UserInfo>,
}

pub struct WorkflowAnalytics {
    pub total_instances: u64,
    pub completed_instances: u64,
    pub average_completion_time: Duration,
    pub bottleneck_nodes: Vec<(NodeId, Duration)>,
    pub sla_compliance_rate: f64,
    pub escalation_frequency: HashMap<NodeId, u32>,
}
```

## Implementation Phases

### Phase 1: Foundation (Weeks 1-2)
1. **Workflow Definition System**
   - Create workflow definition structures
   - Implement graph-based workflow representation
   - Add workflow template system

2. **Basic State Machine Engine**
   - Implement workflow instance management
   - Add basic state transitions
   - Create workflow execution engine

### Phase 2: Business Logic (Weeks 3-4)
1. **Guards and Validation**
   - Implement guard system
   - Add business rules engine
   - Create transition validation

2. **Actions and Automation**
   - Add action execution system
   - Implement automated workflow triggers
   - Create notification system

### Phase 3: Advanced Features (Weeks 5-6)
1. **Timers and Escalation**
   - Implement timer-based actions
   - Add escalation rules
   - Create SLA monitoring

2. **Analytics and Monitoring**
   - Add workflow analytics
   - Implement performance monitoring
   - Create workflow optimization tools

### Phase 4: Integration (Weeks 7-8)
1. **Domain Integration**
   - Integrate with existing document domain
   - Update commands and events
   - Migrate existing workflows

2. **Testing and Documentation**
   - Comprehensive testing suite
   - Performance optimization
   - Documentation and examples

## Benefits of New Architecture

1. **Flexibility**: Configurable workflows without code changes
2. **Scalability**: Parallel workflows and complex routing
3. **Automation**: Timer-based actions and smart escalation
4. **Compliance**: Audit trails and governance controls
5. **Performance**: Optimized state transitions and bulk operations
6. **Analytics**: Workflow performance insights and bottleneck detection

## Migration Strategy

1. **Backward Compatibility**: Existing DocumentState enum remains functional
2. **Gradual Migration**: New workflows use new system, legacy continues
3. **Data Migration**: Convert existing state history to workflow format
4. **Feature Parity**: Ensure new system supports all existing functionality
5. **Testing**: Extensive testing with production-like scenarios

This design provides a comprehensive, scalable workflow state machine system that can handle complex document management scenarios while maintaining the existing domain's event-driven architecture and domain-driven design principles.