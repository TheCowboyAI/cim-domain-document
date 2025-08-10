# 💻 Coding Activity Context

*Focus on implementation, TDD, and event-driven development*

## Activity Purpose
Coding involves implementing features using Test-Driven Development, event-sourcing patterns, and Rust best practices.

## Core Coding Principles

### Test-Driven Development (TDD)
```rust
// 1. Write failing test
#[test]
fn should_create_document_with_metadata() {
    let result = DocumentAggregate::create("test.md", content, metadata);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id().value(), "expected_id");
}

// 2. Write minimal code to pass
impl DocumentAggregate {
    pub fn create(name: &str, content: Content, metadata: Metadata) -> Result<Self> {
        // Minimal implementation
    }
}

// 3. Refactor while keeping tests green
```

### Event-Driven Implementation
```rust
// Events as first-class citizens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentEvent {
    DocumentCreated {
        id: DocumentId,
        content: Content,
        metadata: Metadata,
        timestamp: DateTime<Utc>,
    },
    ContentUpdated {
        id: DocumentId,
        new_content: Content,
        diff: ContentDiff,
        timestamp: DateTime<Utc>,
    },
}

// Aggregates apply events
impl DocumentAggregate {
    pub fn apply_event(&mut self, event: DocumentEvent) {
        match event {
            DocumentEvent::DocumentCreated { id, content, metadata, .. } => {
                self.id = id;
                self.content = content;
                self.metadata = metadata;
            }
            DocumentEvent::ContentUpdated { new_content, .. } => {
                self.content = new_content;
            }
        }
    }
}
```

### NATS Integration Patterns
```rust
// Service handlers for NATS subjects
async fn handle_document_create(msg: nats::Message) -> Result<()> {
    let cmd: CreateDocumentCommand = serde_json::from_slice(&msg.data)?;
    
    let events = document_service.handle_command(cmd).await?;
    
    for event in events {
        nats_client.publish("doc.events", &event).await?;
    }
    
    msg.respond("success").await?;
    Ok(())
}
```

## Implementation Patterns

### Aggregate Pattern
```rust
#[derive(Debug, Clone)]
pub struct DocumentAggregate {
    id: DocumentId,
    content: Content,
    metadata: Metadata,
    version: Version,
    uncommitted_events: Vec<DocumentEvent>,
}

impl DocumentAggregate {
    // Commands return events
    pub fn update_content(&mut self, new_content: Content) -> Result<Vec<DocumentEvent>> {
        let event = DocumentEvent::ContentUpdated {
            id: self.id.clone(),
            new_content: new_content.clone(),
            diff: self.content.diff(&new_content),
            timestamp: Utc::now(),
        };
        
        self.apply_event(event.clone());
        Ok(vec![event])
    }
}
```

### Repository Pattern
```rust
#[async_trait]
pub trait DocumentRepository {
    async fn save(&self, aggregate: &DocumentAggregate) -> Result<()>;
    async fn load(&self, id: &DocumentId) -> Result<DocumentAggregate>;
    async fn load_version(&self, id: &DocumentId, version: Version) -> Result<DocumentAggregate>;
}

// Event sourced implementation
pub struct EventSourcedDocumentRepository {
    event_store: Arc<dyn EventStore>,
}

impl DocumentRepository for EventSourcedDocumentRepository {
    async fn load(&self, id: &DocumentId) -> Result<DocumentAggregate> {
        let events = self.event_store.load_events(id).await?;
        let mut aggregate = DocumentAggregate::new();
        
        for event in events {
            aggregate.apply_event(event);
        }
        
        Ok(aggregate)
    }
}
```

### Command Handler Pattern
```rust
pub struct DocumentCommandHandler {
    repository: Arc<dyn DocumentRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl DocumentCommandHandler {
    pub async fn handle(&self, cmd: DocumentCommand) -> Result<()> {
        match cmd {
            DocumentCommand::CreateDocument { id, content, metadata } => {
                let mut aggregate = DocumentAggregate::create(id, content, metadata)?;
                self.repository.save(&aggregate).await?;
                
                // Publish events
                for event in aggregate.uncommitted_events() {
                    self.event_bus.publish(event).await?;
                }
                
                Ok(())
            }
        }
    }
}
```

## Coding Standards for Document Domain

### File Organization
```
src/
├── aggregate/           # Domain aggregates
│   └── document_aggregate.rs
├── commands/           # Command definitions
│   └── document_commands.rs  
├── events/            # Event definitions
│   └── document_events.rs
├── handlers/          # Command and event handlers
│   ├── command_handler.rs
│   └── event_handler.rs
├── projections/       # Read models
│   └── document_projection.rs
├── services/          # Domain services
│   └── content_intelligence.rs
└── value_objects/     # Value objects
    └── document_metadata.rs
```

### Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error("Document not found: {id}")]
    NotFound { id: String },
    
    #[error("Invalid document content: {reason}")]
    InvalidContent { reason: String },
    
    #[error("Version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}
```

### Testing Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    
    #[tokio::test]
    async fn test_document_creation_publishes_event() {
        // Arrange
        let (handler, event_capture) = create_test_handler().await;
        let cmd = CreateDocumentCommand::new("test.md", content(), metadata());
        
        // Act
        handler.handle(cmd).await.unwrap();
        
        // Assert
        let events = event_capture.captured_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DocumentEvent::DocumentCreated { .. }));
    }
    
    #[test]
    fn test_content_diff_calculation() {
        // Test value object behavior
        let old_content = Content::new("Hello World");
        let new_content = Content::new("Hello Rust");
        
        let diff = old_content.diff(&new_content);
        assert_eq!(diff.changes().len(), 1);
    }
}
```

## Code Quality Checklist

### Before Committing Code
- [ ] All tests pass: `cargo test`
- [ ] Code compiles: `cargo build`
- [ ] Linting passes: `cargo clippy`
- [ ] Formatting applied: `cargo fmt`
- [ ] New tests written for new functionality
- [ ] Events have correlation and causation IDs
- [ ] Error handling is comprehensive
- [ ] Documentation updated for public APIs

### Code Review Criteria
- [ ] Follows single responsibility principle
- [ ] Uses event-driven patterns correctly
- [ ] Implements proper error handling
- [ ] Has adequate test coverage
- [ ] Follows Rust idioms and best practices
- [ ] Integration with NATS is correct
- [ ] Performance implications considered

## Common Coding Tasks in Document Domain

### Adding New Document Event
1. Define event in `events/document_events.rs`
2. Add handling in `DocumentAggregate::apply_event`
3. Create command that produces the event
4. Add command handler logic
5. Write tests for event application
6. Update projections if needed

### Implementing New Query
1. Define query parameters and result types
2. Create or update projection handlers
3. Implement query handler with repository
4. Add NATS subject handler
5. Write integration tests
6. Update API documentation

### Adding Content Intelligence Feature
1. Define domain service interface
2. Implement service with business logic
3. Create events for intelligence results
4. Add integration points with aggregates
5. Write comprehensive tests
6. Add configuration and monitoring

---
*Coding context for systematic implementation using TDD and event-driven patterns*  
*Write tests first, then implement minimal code to make them pass*