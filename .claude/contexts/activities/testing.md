# 🧪 Testing Activity Context

*Focus on verification, coverage, and quality assurance*

## Activity Purpose
Testing ensures code correctness, system reliability, and feature completeness through comprehensive test suites and verification strategies.

## Testing Pyramid for Document Domain

### Unit Tests (Foundation)
```rust
// Test individual components in isolation
#[cfg(test)]
mod document_aggregate_tests {
    use super::*;
    
    #[test]
    fn should_create_document_with_valid_content() {
        let content = Content::new("# Test Document\nContent here");
        let metadata = Metadata::builder().title("Test").build();
        
        let result = DocumentAggregate::create(content, metadata);
        
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.title(), "Test");
    }
    
    #[test]
    fn should_reject_empty_content() {
        let content = Content::new("");
        let metadata = Metadata::default();
        
        let result = DocumentAggregate::create(content, metadata);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DocumentError::InvalidContent { .. }));
    }
}
```

### Integration Tests (Middle)
```rust
// Test service integration and NATS communication
#[tokio::test]
async fn test_document_creation_workflow() {
    let test_harness = DocumentTestHarness::new().await;
    
    // Arrange
    let create_cmd = CreateDocumentCommand {
        content: "Test content".into(),
        metadata: test_metadata(),
    };
    
    // Act - send command via NATS
    let response = test_harness
        .nats_client
        .request("doc.commands.create", &create_cmd)
        .await
        .unwrap();
    
    // Assert - verify response and side effects
    assert_eq!(response.status, "success");
    
    // Verify event was published
    let events = test_harness.event_collector.collect_events().await;
    assert!(events.iter().any(|e| matches!(e, DocumentEvent::DocumentCreated { .. })));
    
    // Verify persistence
    let doc = test_harness.repository.load(&create_cmd.id).await.unwrap();
    assert_eq!(doc.content().value(), "Test content");
}
```

### End-to-End Tests (Top)
```rust
// Test complete user workflows
#[tokio::test]
async fn test_complete_document_lifecycle() {
    let system = TestSystem::start().await;
    
    // 1. Create document
    let doc_id = system.create_document("# My Document", metadata()).await?;
    
    // 2. Update content
    system.update_document_content(&doc_id, "# My Updated Document").await?;
    
    // 3. Create version
    let version = system.create_version(&doc_id, "v1.0").await?;
    
    // 4. Search for document
    let results = system.search("updated document").await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, doc_id);
    
    // 5. Archive document
    system.archive_document(&doc_id).await?;
    
    // Verify final state
    let doc = system.get_document(&doc_id).await?;
    assert!(doc.is_archived());
}
```

## Test Categories by Domain

### Aggregate Tests
```rust
// Test business logic and invariants
#[test]
fn test_version_ordering_invariant() {
    let mut doc = DocumentAggregate::new();
    
    doc.apply_event(DocumentEvent::VersionCreated { version: 2, .. });
    doc.apply_event(DocumentEvent::VersionCreated { version: 1, .. });
    
    // Versions should be ordered correctly despite event order
    assert_eq!(doc.versions().first().unwrap().number(), 1);
    assert_eq!(doc.versions().last().unwrap().number(), 2);
}

#[test]
fn test_content_change_creates_diff() {
    let mut doc = create_test_document("Original content");
    
    let events = doc.update_content("Modified content").unwrap();
    
    assert_eq!(events.len(), 1);
    if let DocumentEvent::ContentUpdated { diff, .. } = &events[0] {
        assert!(diff.has_changes());
        assert_eq!(diff.changed_lines(), 1);
    }
}
```

### Service Tests
```rust
// Test domain services and external integrations
#[tokio::test]
async fn test_content_intelligence_extraction() {
    let service = ContentIntelligenceService::new();
    let content = Content::new("# Important Document\nThis is about AI and machine learning.");
    
    let analysis = service.analyze_content(&content).await.unwrap();
    
    assert!(analysis.topics.contains(&"AI".to_string()));
    assert!(analysis.topics.contains(&"machine learning".to_string()));
    assert_eq!(analysis.document_type, DocumentType::Technical);
}

#[tokio::test]
async fn test_search_service_indexing() {
    let service = SearchService::new().await;
    let doc = create_test_document("Searchable content here");
    
    // Index document
    service.index_document(&doc).await.unwrap();
    
    // Search should find it
    let results = service.search("searchable content").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, doc.id());
}
```

### Event Sourcing Tests
```rust
// Test event application and sourcing
#[test]
fn test_aggregate_reconstruction_from_events() {
    let events = vec![
        DocumentEvent::DocumentCreated { id: doc_id(), content: original_content(), .. },
        DocumentEvent::ContentUpdated { new_content: updated_content(), .. },
        DocumentEvent::VersionCreated { version: 1, .. },
    ];
    
    let mut aggregate = DocumentAggregate::new();
    for event in events {
        aggregate.apply_event(event);
    }
    
    assert_eq!(aggregate.id(), doc_id());
    assert_eq!(aggregate.content(), updated_content());
    assert_eq!(aggregate.current_version().number(), 1);
}

#[tokio::test]
async fn test_event_store_consistency() {
    let event_store = InMemoryEventStore::new();
    let doc_id = DocumentId::new();
    
    let events = vec![
        DocumentEvent::DocumentCreated { id: doc_id.clone(), .. },
        DocumentEvent::ContentUpdated { id: doc_id.clone(), .. },
    ];
    
    event_store.save_events(&doc_id, &events).await.unwrap();
    let loaded = event_store.load_events(&doc_id).await.unwrap();
    
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].aggregate_id(), doc_id);
    assert_eq!(loaded[1].aggregate_id(), doc_id);
}
```

## Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_content_diff_symmetry(
        content1 in "[a-zA-Z0-9 \n]{1,1000}",
        content2 in "[a-zA-Z0-9 \n]{1,1000}"
    ) {
        let c1 = Content::new(&content1);
        let c2 = Content::new(&content2);
        
        let diff_12 = c1.diff(&c2);
        let diff_21 = c2.diff(&c1);
        
        // Diffs should be symmetric in change count
        prop_assert_eq!(diff_12.change_count(), diff_21.change_count());
    }
    
    #[test]
    fn test_version_numbering_consistency(
        versions in prop::collection::vec(1u64..100, 1..20)
    ) {
        let mut doc = DocumentAggregate::new();
        
        // Apply version events in random order
        for version in &versions {
            doc.apply_event(DocumentEvent::VersionCreated { 
                version: *version, 
                timestamp: Utc::now() 
            });
        }
        
        // Versions should still be in correct order
        let sorted_versions = doc.versions();
        for window in sorted_versions.windows(2) {
            prop_assert!(window[0].number() <= window[1].number());
        }
    }
}
```

## Test Fixtures and Helpers
```rust
// Test data builders
pub struct DocumentBuilder {
    id: Option<DocumentId>,
    content: Option<Content>,
    metadata: Option<Metadata>,
}

impl DocumentBuilder {
    pub fn new() -> Self {
        Self { id: None, content: None, metadata: None }
    }
    
    pub fn with_id(mut self, id: DocumentId) -> Self {
        self.id = Some(id);
        self
    }
    
    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(Content::new(content));
        self
    }
    
    pub fn build(self) -> DocumentAggregate {
        DocumentAggregate::create(
            self.id.unwrap_or_else(DocumentId::new),
            self.content.unwrap_or_else(|| Content::new("Default content")),
            self.metadata.unwrap_or_default(),
        ).unwrap()
    }
}

// Test harness for integration tests
pub struct DocumentTestHarness {
    pub nats_client: nats::Connection,
    pub event_store: Arc<dyn EventStore>,
    pub repository: Arc<dyn DocumentRepository>,
    pub event_collector: EventCollector,
}

impl DocumentTestHarness {
    pub async fn new() -> Self {
        let nats_client = nats::connect("nats://localhost:4222").await.unwrap();
        let event_store = Arc::new(InMemoryEventStore::new());
        let repository = Arc::new(EventSourcedDocumentRepository::new(event_store.clone()));
        let event_collector = EventCollector::new(&nats_client).await;
        
        Self { nats_client, event_store, repository, event_collector }
    }
}
```

## Performance Testing
```rust
#[tokio::test]
async fn test_bulk_document_creation_performance() {
    let harness = DocumentTestHarness::new().await;
    let document_count = 1000;
    
    let start = std::time::Instant::now();
    
    // Create documents in parallel
    let tasks: Vec<_> = (0..document_count)
        .map(|i| {
            let harness = &harness;
            async move {
                let content = format!("Document content {}", i);
                harness.create_document(&content).await
            }
        })
        .collect();
    
    futures::future::try_join_all(tasks).await.unwrap();
    
    let duration = start.elapsed();
    let docs_per_second = document_count as f64 / duration.as_secs_f64();
    
    // Should handle at least 100 docs/second
    assert!(docs_per_second > 100.0, "Too slow: {} docs/second", docs_per_second);
}
```

## Test Organization and Execution

### Test File Structure
```
tests/
├── unit/                   # Unit tests alongside source
│   └── document_aggregate_test.rs
├── integration/           # Cross-service integration tests
│   ├── document_workflow_test.rs
│   └── search_integration_test.rs  
└── e2e/                   # End-to-end system tests
    └── complete_lifecycle_test.rs
```

### Test Execution Commands
```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run integration tests
cargo test --test '*integration*'

# Run with coverage
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' \
LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' \
cargo test --lib

# Run performance tests
cargo test --release test_performance

# Run property tests with more cases
cargo test --release proptest -- --test-threads 1
```

## Quality Gates

### Before Merging Code
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] End-to-end tests pass
- [ ] Property tests pass with 1000+ cases
- [ ] Performance tests meet benchmarks
- [ ] Code coverage > 80%
- [ ] No flaky tests
- [ ] Tests run in CI pipeline

### Test Quality Criteria
- [ ] Tests are deterministic and repeatable
- [ ] Test names clearly describe what is being tested
- [ ] Tests follow Arrange-Act-Assert pattern
- [ ] Mock dependencies are used appropriately
- [ ] Tests cover both happy path and error cases
- [ ] Integration tests use realistic test data
- [ ] Performance tests have meaningful thresholds

---
*Testing context for comprehensive verification and quality assurance*  
*Write tests that give confidence in system correctness and reliability*