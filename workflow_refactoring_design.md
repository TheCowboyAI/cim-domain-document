# Workflow Module Refactoring Design & Plan

## Overview

This document outlines the design and implementation plan for refactoring the workflow module to resolve compilation issues and improve architectural soundness while maintaining the existing functionality.

## Current Issues Analysis

### 1. Async Trait Object Incompatibility
**Problem**: Traits with `async fn` methods cannot be used as trait objects (`dyn Trait`)
```rust
// Current - BROKEN
pub trait ActionExecutor {
    async fn execute_action(&self, action: &WorkflowAction) -> WorkflowResult<ActionResult>;
}
pub struct Engine {
    pub action_executor: Arc<dyn ActionExecutor + Send + Sync>, // ERROR
}
```

### 2. Derive Macro Constraints
**Problem**: Complex fields prevent automatic trait derivation
```rust
// Current - BROKEN
#[derive(Debug, Clone)] // ERROR: dyn traits don't implement Debug/Clone
pub struct ActionContext<'a> {
    pub workflow_instance: &'a mut WorkflowInstance, // ERROR: &mut doesn't implement Clone
}
```

### 3. Missing Type Imports
**Problem**: `Document` type not imported in workflow modules

### 4. Error Handling Gaps
**Problem**: Missing `From` implementations for error conversions

## Refactoring Design

### Phase 1: Make Traits Object-Safe

#### Solution 1A: Use async_trait with Boxed Futures
```rust
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

#[async_trait]
pub trait ActionExecutor: Send + Sync {
    async fn execute_action(
        &self, 
        action: &WorkflowAction, 
        context: &mut ActionContext<'_>
    ) -> WorkflowResult<ActionResult>;
}

// Usage remains the same:
pub struct Engine {
    pub action_executor: Arc<dyn ActionExecutor + Send + Sync>, // Now works
}
```

#### Solution 1B: Use Concrete Types Instead of Trait Objects
```rust
// Alternative: Use concrete types with generics
pub struct Engine<E: ActionExecutor> {
    pub action_executor: Arc<E>,
}

impl<E: ActionExecutor> Engine<E> {
    pub fn new(executor: Arc<E>) -> Self {
        Self { action_executor: executor }
    }
}
```

### Phase 2: Fix Derive Macro Issues

#### Solution 2A: Remove Problematic Derives
```rust
// Instead of deriving, implement manually or remove
pub struct ActionContext<'a> {
    pub workflow_instance: &'a mut WorkflowInstance,
    pub variables: &'a mut HashMap<String, serde_json::Value>,
    pub document: &'a Document,
    pub user_id: Uuid,
    pub trigger_time: DateTime<Utc>,
}

// Manual Debug implementation
impl<'a> std::fmt::Debug for ActionContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionContext")
            .field("user_id", &self.user_id)
            .field("trigger_time", &self.trigger_time)
            .finish_non_exhaustive()
    }
}
```

#### Solution 2B: Use Owned Data Instead of References
```rust
// Convert borrowed data to owned where possible
#[derive(Debug, Clone)]
pub struct ActionContext {
    pub workflow_instance_id: WorkflowInstanceId,
    pub variables: HashMap<String, serde_json::Value>,
    pub document_id: DocumentId,
    pub user_id: Uuid,
    pub trigger_time: DateTime<Utc>,
}
```

### Phase 3: Architectural Improvements

#### 3.1: Service Layer Pattern
```rust
// Separate concerns using service layer
pub struct WorkflowService {
    engine: Arc<WorkflowEngine>,
    action_service: Arc<ActionService>,
    notification_service: Arc<NotificationService>,
}

pub struct ActionService {
    executors: HashMap<String, Box<dyn ActionExecutor>>,
}

impl ActionService {
    pub async fn execute(&self, action: &WorkflowAction) -> WorkflowResult<ActionResult> {
        // Implementation
    }
}
```

#### 3.2: Event-Driven Architecture
```rust
// Use events for loose coupling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    TransitionStarted { instance_id: WorkflowInstanceId, from: NodeId, to: NodeId },
    ActionExecuted { instance_id: WorkflowInstanceId, action: WorkflowAction, result: ActionResult },
    WorkflowCompleted { instance_id: WorkflowInstanceId },
}

pub trait WorkflowEventHandler: Send + Sync {
    fn handle_event(&self, event: WorkflowEvent);
}
```

## Implementation Plan

### Phase 1: Foundation (Week 1)
**Goal**: Make the code compile and tests pass

#### Task 1.1: Fix Imports and Basic Issues (Day 1)
- [ ] Add missing `use crate::Document;` imports
- [ ] Add missing `From` trait implementations
- [ ] Remove unused imports (Instant, timeout)
- [ ] Fix basic compilation errors

#### Task 1.2: Convert Async Traits (Days 2-3)
- [ ] Add `async-trait` dependency to Cargo.toml
- [ ] Convert `ActionExecutor` trait to use `#[async_trait]`
- [ ] Convert `NotificationService` trait to use `#[async_trait]`
- [ ] Convert `IntegrationService` trait to use `#[async_trait]`
- [ ] Convert `GuardEvaluator` trait to use `#[async_trait]`

#### Task 1.3: Fix Derive Issues (Days 4-5)
- [ ] Remove problematic `#[derive(Debug, Clone)]` annotations
- [ ] Implement manual `Debug` implementations where needed
- [ ] Convert `ActionContext` to use owned data
- [ ] Fix guard evaluation function signatures

### Phase 2: Architectural Improvements (Week 2)
**Goal**: Improve design patterns and testability

#### Task 2.1: Service Layer Implementation (Days 1-3)
- [ ] Create `WorkflowService` as main entry point
- [ ] Extract `ActionService` for action execution
- [ ] Extract `GuardService` for condition evaluation
- [ ] Implement dependency injection pattern

#### Task 2.2: Event-Driven Refactoring (Days 4-5)
- [ ] Define comprehensive `WorkflowEvent` enum
- [ ] Implement event publishing mechanism
- [ ] Convert tight coupling to event-based communication
- [ ] Add event handlers for logging, metrics, notifications

### Phase 3: Integration and Testing (Week 3)
**Goal**: Ensure robust integration with document editing

#### Task 3.1: Document Editing Integration (Days 1-2)
- [ ] Create workflow triggers for document edit events
- [ ] Implement approval workflows for document changes
- [ ] Add CID chain validation workflows
- [ ] Create automated quality checks

#### Task 3.2: Comprehensive Testing (Days 3-4)
- [ ] Unit tests for all refactored components
- [ ] Integration tests for workflow execution
- [ ] Performance tests for concurrent workflows
- [ ] Error handling and resilience tests

#### Task 3.3: Documentation and Examples (Day 5)
- [ ] Update API documentation
- [ ] Create workflow definition examples
- [ ] Write integration guides
- [ ] Performance tuning guidelines

## Detailed Technical Design

### 1. Async Trait Refactoring

#### Before:
```rust
pub trait ActionExecutor {
    async fn execute_action(&self, action: &WorkflowAction) -> WorkflowResult<ActionResult>;
}
```

#### After:
```rust
#[async_trait]
pub trait ActionExecutor: Send + Sync + std::fmt::Debug {
    async fn execute_action(
        &self,
        action: &WorkflowAction,
        context: &ActionContext,
    ) -> WorkflowResult<ActionResult>;
}

// Implement for concrete types
#[derive(Debug)]
pub struct DefaultActionExecutor {
    document_service: Arc<dyn DocumentService>,
    notification_service: Option<Arc<dyn NotificationService>>,
}

#[async_trait]
impl ActionExecutor for DefaultActionExecutor {
    async fn execute_action(
        &self,
        action: &WorkflowAction,
        context: &ActionContext,
    ) -> WorkflowResult<ActionResult> {
        match action {
            WorkflowAction::SetState(state) => {
                self.document_service.update_state(context.document_id, *state).await?;
                Ok(ActionResult::Success)
            }
            WorkflowAction::SendNotification { recipient, template } => {
                if let Some(service) = &self.notification_service {
                    let notification = Notification {
                        recipient: recipient.clone(),
                        template: template.clone(),
                        context: context.clone(),
                    };
                    service.send_notification(notification).await?;
                }
                Ok(ActionResult::Success)
            }
            // ... other actions
        }
    }
}
```

### 2. Context Refactoring

#### Before:
```rust
#[derive(Debug, Clone)] // ERROR
pub struct ActionContext<'a> {
    pub workflow_instance: &'a mut WorkflowInstance, // ERROR
    pub variables: &'a mut HashMap<String, serde_json::Value>, // ERROR
    pub document: &'a Document,
}
```

#### After:
```rust
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
    pub fn new(instance: &WorkflowInstance, document_id: DocumentId, user_id: Uuid) -> Self {
        Self {
            workflow_instance_id: instance.id,
            document_id,
            user_id,
            variables: instance.variables.clone(),
            trigger_time: Utc::now(),
            current_node: instance.current_node,
            target_node: None,
        }
    }
    
    pub fn with_target_node(mut self, target: NodeId) -> Self {
        self.target_node = Some(target);
        self
    }
}
```

### 3. Service Layer Architecture

```rust
// Main service interface
pub struct WorkflowService {
    engine: Arc<WorkflowEngine>,
    action_service: Arc<ActionService>,
    guard_service: Arc<GuardService>,
    event_bus: Arc<EventBus>,
}

impl WorkflowService {
    pub async fn execute_transition(
        &self,
        instance_id: WorkflowInstanceId,
        to_node: NodeId,
        user_id: Uuid,
    ) -> WorkflowResult<TransitionResult> {
        // 1. Load workflow instance
        let instance = self.engine.get_instance(instance_id).await?;
        
        // 2. Evaluate guards
        let context = GuardContext::new(&instance, user_id);
        self.guard_service.can_transition(&instance.workflow_id, &instance.current_node, &to_node, &context).await?;
        
        // 3. Execute transition
        let transition_result = self.engine.execute_transition(instance_id, to_node).await?;
        
        // 4. Execute actions
        for action in &transition_result.actions {
            let action_context = ActionContext::new(&instance, instance.document_id, user_id);
            let result = self.action_service.execute_action(action, &action_context).await?;
            
            // 5. Publish events
            self.event_bus.publish(WorkflowEvent::ActionExecuted {
                instance_id,
                action: action.clone(),
                result,
            }).await;
        }
        
        Ok(transition_result)
    }
}

// Specialized action service
pub struct ActionService {
    document_service: Arc<dyn DocumentService>,
    notification_service: Arc<dyn NotificationService>,
    integration_service: Arc<dyn IntegrationService>,
}

impl ActionService {
    pub async fn execute_action(
        &self,
        action: &WorkflowAction,
        context: &ActionContext,
    ) -> WorkflowResult<ActionResult> {
        // Centralized action execution logic
        match action {
            WorkflowAction::SetState(state) => {
                self.document_service.update_state(context.document_id, *state).await
                    .map_err(|e| WorkflowError::ActionFailed {
                        action: format!("{:?}", action),
                        error: e.to_string(),
                    })?;
                Ok(ActionResult::Success)
            }
            // ... other actions
        }
    }
}
```

### 4. Integration with Document Editing

```rust
// Workflow triggers for document editing events
pub struct DocumentEditingWorkflowIntegration {
    workflow_service: Arc<WorkflowService>,
}

impl DocumentEditingWorkflowIntegration {
    pub async fn handle_document_edited(&self, event: DocumentEditedDirect) -> Result<(), Box<dyn std::error::Error>> {
        // Start approval workflow for document edits
        let workflow_id = WorkflowId::new("document_approval");
        let variables = {
            let mut vars = HashMap::new();
            vars.insert("document_id".to_string(), serde_json::to_value(event.document_id)?);
            vars.insert("editor".to_string(), serde_json::to_value(event.edit_metadata.edited_by)?);
            vars.insert("edit_type".to_string(), serde_json::Value::String("direct".to_string()));
            vars
        };
        
        let instance = self.workflow_service.start_workflow(
            workflow_id,
            event.document_id,
            event.edit_metadata.edited_by,
            variables,
        ).await?;
        
        Ok(())
    }
    
    pub async fn handle_cid_chain_verified(&self, event: CidChainVerified) -> Result<(), Box<dyn std::error::Error>> {
        if !event.verification_result.is_valid {
            // Start chain repair workflow
            let workflow_id = WorkflowId::new("chain_repair");
            let variables = {
                let mut vars = HashMap::new();
                vars.insert("document_id".to_string(), serde_json::to_value(event.document_id)?);
                vars.insert("issues".to_string(), serde_json::to_value(event.verification_result.issues)?);
                vars
            };
            
            self.workflow_service.start_workflow(
                workflow_id,
                event.document_id,
                Uuid::nil(), // System user
                variables,
            ).await?;
        }
        
        Ok(())
    }
}
```

## Migration Strategy

### 1. Incremental Migration Approach
- **Phase 1**: Make existing code compile (minimal changes)
- **Phase 2**: Improve architecture (larger refactoring)
- **Phase 3**: Add new features and optimizations

### 2. Backward Compatibility
- Maintain existing public APIs during transition
- Use deprecation warnings for old interfaces
- Provide migration guides for API changes

### 3. Testing Strategy
- Keep existing tests passing during refactoring
- Add integration tests for new architecture
- Performance benchmarks to ensure no regressions

## Risk Mitigation

### 1. Technical Risks
- **Async trait complexity**: Use well-tested `async-trait` crate
- **Performance regression**: Benchmark critical paths
- **Memory usage**: Profile heap allocations

### 2. Implementation Risks
- **Scope creep**: Stick to defined phases and goals
- **Breaking changes**: Maintain API compatibility
- **Testing gaps**: Comprehensive test coverage

### 3. Timeline Risks
- **Dependency issues**: Verify crate compatibility early
- **Complexity underestimation**: Buffer time for unexpected issues
- **Integration problems**: Test with real document editing scenarios

## Success Metrics

### 1. Compilation Success
- [ ] All modules compile without errors
- [ ] All tests pass
- [ ] No warnings for unused imports/variables

### 2. Architecture Quality
- [ ] Clear separation of concerns
- [ ] Testable components with dependency injection
- [ ] Event-driven communication between modules

### 3. Performance Targets
- [ ] Workflow execution time < 100ms (95th percentile)
- [ ] Memory usage increase < 20% vs current implementation
- [ ] Support for 1000+ concurrent workflow instances

### 4. Integration Success
- [ ] Document editing workflows work end-to-end
- [ ] CID chain validation workflows complete successfully
- [ ] Error handling and recovery mechanisms function properly

## Dependencies and Requirements

### New Crate Dependencies
```toml
[dependencies]
async-trait = "0.1"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Development Dependencies
```toml
[dev-dependencies]
tokio-test = "0.4"
criterion = "0.5"
mockall = "0.11"
```

This comprehensive refactoring plan addresses all the identified issues while providing a clear path forward for improving the workflow module's architecture and integration with the document editing system.