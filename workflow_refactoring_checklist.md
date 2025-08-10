# Workflow Refactoring Implementation Checklist

## Phase 1: Foundation (Make it Compile)

### ✅ Completed Items
- [x] Added missing `Document` type import in actions.rs
- [x] Added `From<serde_json::Error>` implementation for WorkflowError
- [x] Removed unused imports (Instant, timeout)

### 🔲 Remaining Items for Phase 1

#### Day 1: Dependencies and Basic Setup
- [ ] Add `async-trait = "0.1"` to Cargo.toml
- [ ] Add `tracing = "0.1"` and `tracing-subscriber = "0.3"` for better logging
- [ ] Add `mockall = "0.11"` to dev-dependencies for testing

#### Day 2: Convert Async Traits
- [ ] Convert `ActionExecutor` trait to use `#[async_trait]`
- [ ] Convert `NotificationService` trait to use `#[async_trait]`  
- [ ] Convert `IntegrationService` trait to use `#[async_trait]`
- [ ] Convert `GuardEvaluator` trait to use `#[async_trait]`
- [ ] Update all trait implementations with `#[async_trait]`

#### Day 3: Fix Derive Issues
- [ ] Remove `#[derive(Debug, Clone)]` from `ActionContext`
- [ ] Implement manual `Debug` for `ActionContext`
- [ ] Convert `ActionContext` to use owned data instead of references
- [ ] Remove `#[derive(Debug)]` from structs with non-debuggable fields
- [ ] Remove `#[derive(Clone)]` from structs with non-cloneable fields

#### Day 4: Guard System Fixes
- [ ] Fix `DefaultGuardEvaluator` to implement `Debug` trait
- [ ] Remove function pointers from `custom_guards` HashMap
- [ ] Use enum-based approach for custom guards instead
- [ ] Fix lifetime issues in guard evaluation

#### Day 5: Engine and Actions Fixes  
- [ ] Fix `WorkflowEngine` constructor to avoid trait object issues
- [ ] Update action execution to work with new context structure
- [ ] Fix automation engine compilation issues
- [ ] Ensure all tests compile (may not pass yet, but should compile)

## Phase 2: Architectural Improvements

### Week 2: Service Layer and Event System

#### Day 1-2: Service Layer
- [ ] Create `WorkflowService` as main entry point
- [ ] Extract `ActionService` for centralized action execution
- [ ] Extract `GuardService` for condition evaluation  
- [ ] Implement dependency injection pattern

#### Day 3-4: Event-Driven Architecture
- [ ] Define `WorkflowEvent` enum with all event types
- [ ] Implement `EventBus` for publishing/subscribing to events
- [ ] Convert direct method calls to event-based communication
- [ ] Add event handlers for logging, metrics, notifications

#### Day 5: Integration Points
- [ ] Create workflow triggers for document editing events
- [ ] Implement document state change workflows
- [ ] Add CID chain validation workflow triggers
- [ ] Create automated quality check workflows

## Phase 3: Integration and Testing

### Week 3: Testing and Documentation

#### Day 1-2: Document Integration
- [ ] Test document editing → workflow triggers
- [ ] Test workflow state changes → document updates
- [ ] Test CID chain validation → repair workflows
- [ ] End-to-end integration testing

#### Day 3-4: Comprehensive Testing
- [ ] Unit tests for all refactored components
- [ ] Mock implementations for external dependencies
- [ ] Performance tests for workflow execution
- [ ] Error handling and edge case testing

#### Day 5: Documentation and Polish
- [ ] Update README with new architecture
- [ ] Create example workflow definitions
- [ ] Write API documentation
- [ ] Performance optimization based on test results

## Quick Start Implementation Guide

### Step 1: Add Dependencies (5 minutes)

Add to `Cargo.toml`:
```toml
[dependencies]
async-trait = "0.1"
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
mockall = "0.11"
tokio-test = "0.4"
```

### Step 2: Convert First Trait (30 minutes)

Replace in `src/workflow/actions.rs`:
```rust
// OLD - BROKEN
pub trait ActionExecutor {
    async fn execute_action(&self, action: &WorkflowAction) -> WorkflowResult<ActionResult>;
}

// NEW - WORKING
use async_trait::async_trait;

#[async_trait]
pub trait ActionExecutor: Send + Sync + std::fmt::Debug {
    async fn execute_action(
        &self,
        action: &WorkflowAction,
        context: &ActionContext,
    ) -> WorkflowResult<ActionResult>;
}
```

### Step 3: Fix Context Structure (45 minutes)

Replace `ActionContext` in `src/workflow/actions.rs`:
```rust
// OLD - BROKEN
#[derive(Debug, Clone)]
pub struct ActionContext<'a> {
    pub workflow_instance: &'a mut WorkflowInstance,
    pub variables: &'a mut HashMap<String, serde_json::Value>,
    pub document: &'a Document,
}

// NEW - WORKING
#[derive(Debug, Clone)]
pub struct ActionContext {
    pub workflow_instance_id: WorkflowInstanceId,
    pub document_id: DocumentId,
    pub user_id: Uuid,
    pub variables: HashMap<String, serde_json::Value>,
    pub trigger_time: DateTime<Utc>,
    pub current_node: NodeId,
}

impl ActionContext {
    pub fn new(
        instance_id: WorkflowInstanceId,
        document_id: DocumentId, 
        user_id: Uuid,
        variables: HashMap<String, serde_json::Value>,
        current_node: NodeId,
    ) -> Self {
        Self {
            workflow_instance_id: instance_id,
            document_id,
            user_id,
            variables,
            trigger_time: Utc::now(),
            current_node,
        }
    }
}
```

### Step 4: Update Implementations (60 minutes)

Update all trait implementations:
```rust
// Update DefaultActionExecutor
#[async_trait]
impl ActionExecutor for DefaultActionExecutor {
    async fn execute_action(
        &self,
        action: &WorkflowAction,
        context: &ActionContext,
    ) -> WorkflowResult<ActionResult> {
        match action {
            WorkflowAction::SetState(state) => {
                // Implementation using context.document_id instead of context.document
                Ok(ActionResult::Success)
            }
            // ... other cases
        }
    }
}
```

## Common Pitfalls and Solutions

### Pitfall 1: Forgetting `#[async_trait]` on implementations
**Error**: `expected fn pointer, found opaque type`
**Solution**: Add `#[async_trait]` to both trait definition AND all implementations

### Pitfall 2: Mixing owned and borrowed data in context
**Error**: `cannot move out of borrowed content` 
**Solution**: Use owned data (`DocumentId` instead of `&Document`)

### Pitfall 3: Complex trait bounds in generics
**Error**: `trait bound not satisfied`
**Solution**: Use concrete types or simplify bounds

### Pitfall 4: Lifetime issues with closures
**Error**: `closure may outlive the current function`
**Solution**: Convert closures to regular functions or use `move` closures

## Testing Strategy

### Unit Test Template
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use tokio_test;

    #[tokio::test]
    async fn test_action_execution() {
        // Arrange
        let mut mock_executor = MockActionExecutor::new();
        mock_executor
            .expect_execute_action()
            .with(eq(WorkflowAction::SetState(DocumentState::InReview)), always())
            .times(1)
            .returning(|_, _| Ok(ActionResult::Success));

        let context = ActionContext::new(
            WorkflowInstanceId::new(),
            DocumentId::new(),
            Uuid::new_v4(),
            HashMap::new(),
            NodeId::new("start"),
        );

        // Act
        let result = mock_executor.execute_action(
            &WorkflowAction::SetState(DocumentState::InReview),
            &context,
        ).await;

        // Assert
        assert!(result.is_ok());
    }
}
```

### Integration Test Template
```rust
#[tokio::test]
async fn test_document_edit_triggers_workflow() {
    // Arrange
    let workflow_service = create_test_workflow_service().await;
    let document_id = DocumentId::new();
    
    // Act
    let edit_event = DocumentEditedDirect {
        document_id: document_id.clone(),
        // ... other fields
    };
    
    workflow_service.handle_document_edited(edit_event).await.unwrap();
    
    // Assert
    let instances = workflow_service.get_active_instances_for_document(document_id).await.unwrap();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].workflow_id.as_str(), "document_approval");
}
```

## Monitoring and Debugging

### Add Tracing
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
async fn execute_action(&self, action: &WorkflowAction, context: &ActionContext) -> WorkflowResult<ActionResult> {
    info!("Executing action: {:?} for document: {}", action, context.document_id);
    
    let result = match action {
        WorkflowAction::SetState(state) => {
            info!("Changing document state to: {:?}", state);
            // implementation
        }
        // ...
    };
    
    match &result {
        Ok(_) => info!("Action executed successfully"),
        Err(e) => error!("Action execution failed: {}", e),
    }
    
    result
}
```

This checklist provides a clear, actionable path forward for fixing the workflow module compilation issues while improving the overall architecture.