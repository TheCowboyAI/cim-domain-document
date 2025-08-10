# 🐛 Debugging Activity Context

*Focus on error analysis, troubleshooting, and issue resolution*

## Activity Purpose
Debugging involves identifying, analyzing, and resolving issues in code, tests, and system behavior through systematic investigation.

## Debugging Approach

### 1. Problem Identification
```rust
// Gather information about the problem
// - What was expected vs what actually happened?
// - When did the issue start occurring?
// - What are the exact error messages?
// - Can the issue be reproduced consistently?

// Example: Compilation error analysis
error[E0277]: the trait bound `DocumentId: Clone` is not satisfied
  --> src/aggregate/document_aggregate.rs:45:32
   |
45 |         self.events.push(event.clone());
   |                               ^^^^^ the trait `Clone` is not satisfied
```

### 2. Error Classification
```rust
// Categorize the type of error
#[derive(Debug)]
enum DebugContext {
    CompilationError {
        error_code: String,
        location: String,
        missing_trait: Option<String>,
    },
    TestFailure {
        test_name: String,
        assertion_failed: String,
        expected: String,
        actual: String,
    },
    RuntimeError {
        error_type: String,
        stack_trace: String,
        context: String,
    },
    LogicError {
        expected_behavior: String,
        actual_behavior: String,
        affected_component: String,
    },
}
```

### 3. Systematic Investigation
```bash
# Investigation checklist
1. Read the error message carefully
2. Check the exact line and file mentioned
3. Examine recent changes (git log --oneline -10)
4. Look for similar errors in the codebase
5. Check dependencies and imports
6. Verify test setup and fixtures
7. Add debugging output or logs
8. Use debugger or print statements
9. Isolate the minimal failing case
10. Research error patterns online
```

## Common Debugging Scenarios

### Compilation Errors
```rust
// Missing trait implementations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentId(Uuid);

// Incorrect import paths
use crate::events::DocumentEvent;  // ✓ Correct
use events::DocumentEvent;         // ✗ May fail

// Lifetime issues
impl<'a> DocumentRepository<'a> {
    pub fn find_by_id(&self, id: &DocumentId) -> Option<&Document> {
        // Ensure lifetimes are correct
    }
}

// Type mismatches
let doc_id: DocumentId = Uuid::new_v4().into();  // ✓ Correct conversion
let doc_id: DocumentId = "invalid".into();       // ✗ May fail
```

### Test Failures
```rust
// Debugging test failures
#[tokio::test]
async fn test_document_creation_debug() {
    // Add detailed logging
    env_logger::init();
    
    let content = Content::new("Test content");
    let metadata = Metadata::default();
    
    println!("Creating document with content: {:?}", content);
    
    let result = DocumentAggregate::create(content.clone(), metadata.clone());
    
    match result {
        Ok(doc) => {
            println!("Document created successfully: {:?}", doc);
            assert_eq!(doc.content(), &content);
        }
        Err(e) => {
            println!("Document creation failed: {:?}", e);
            println!("Content was: {:?}", content);
            println!("Metadata was: {:?}", metadata);
            panic!("Test failed with error: {}", e);
        }
    }
}

// Property test debugging
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]  // Reduce cases for debugging
    
    #[test]
    fn test_content_parsing_debug(content in "[a-zA-Z0-9 ]{1,100}") {
        println!("Testing with content: '{}'", content);
        let result = Content::parse(&content);
        prop_assert!(result.is_ok(), "Failed to parse: '{}'", content);
    }
}
```

### Runtime Errors
```rust
// Debugging async issues
#[tokio::test]
async fn debug_nats_communication() {
    let nc = nats::connect("nats://localhost:4222").await
        .map_err(|e| {
            eprintln!("Failed to connect to NATS: {}", e);
            eprintln!("Is NATS server running on localhost:4222?");
            e
        })?;
    
    let response = nc
        .request("doc.commands.create", b"test message")
        .await
        .map_err(|e| {
            eprintln!("NATS request failed: {}", e);
            eprintln!("Check if handler is registered for 'doc.commands.create'");
            e
        })?;
    
    println!("Received response: {:?}", String::from_utf8_lossy(&response.data));
}

// Debugging event sourcing issues
async fn debug_event_application() {
    let mut doc = DocumentAggregate::new();
    
    let events = vec![
        DocumentEvent::DocumentCreated { /* fields */ },
        DocumentEvent::ContentUpdated { /* fields */ },
    ];
    
    for (i, event) in events.iter().enumerate() {
        println!("Applying event {}: {:?}", i, event);
        
        let before_state = format!("{:?}", doc);
        doc.apply_event(event.clone());
        let after_state = format!("{:?}", doc);
        
        println!("State before: {}", before_state);
        println!("State after: {}", after_state);
        
        // Validate invariants after each event
        assert!(doc.is_valid(), "Aggregate invalid after event {}: {:?}", i, event);
    }
}
```

### Logic Errors
```rust
// Debugging business logic
impl DocumentAggregate {
    pub fn update_content(&mut self, new_content: Content) -> Result<Vec<DocumentEvent>, DocumentError> {
        // Add debug logging
        tracing::debug!("Updating content for document {}", self.id);
        tracing::debug!("Current content: {:?}", self.content);
        tracing::debug!("New content: {:?}", new_content);
        
        // Validate preconditions
        if self.is_archived() {
            tracing::warn!("Attempted to update archived document {}", self.id);
            return Err(DocumentError::DocumentArchived { id: self.id.clone() });
        }
        
        // Calculate diff
        let diff = self.content.diff(&new_content);
        tracing::debug!("Content diff: {:?}", diff);
        
        // Create event
        let event = DocumentEvent::ContentUpdated {
            id: self.id.clone(),
            new_content: new_content.clone(),
            diff,
            timestamp: Utc::now(),
        };
        
        // Apply event
        self.apply_event(event.clone());
        
        // Validate postconditions
        assert_eq!(self.content, new_content, "Content not updated correctly");
        
        tracing::debug!("Content update successful for document {}", self.id);
        Ok(vec![event])
    }
}
```

## Debugging Tools and Techniques

### Logging and Tracing
```rust
// Configure logging for debugging
use tracing::{debug, info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn handle_create_document(&self, cmd: CreateDocumentCommand) -> Result<DocumentId> {
    info!("Handling create document command");
    debug!("Command details: {:?}", cmd);
    
    // Implementation with detailed logging
    let result = self.create_document_internal(&cmd).await;
    
    match &result {
        Ok(id) => info!("Document created successfully with id: {}", id),
        Err(e) => error!("Failed to create document: {:?}", e),
    }
    
    result
}

// Use structured logging
tracing::info!(
    document_id = %doc_id,
    content_length = content.len(),
    metadata_fields = metadata.field_count(),
    "Document processing started"
);
```

### Debugger Integration
```rust
// Using rust-gdb or rust-lldb
// Set breakpoints in code
fn process_document(doc: &Document) {
    std::dbg!(&doc);  // Print debug info
    
    // Manual breakpoint for debugger
    if cfg!(debug_assertions) {
        std::process::Command::new("sleep").arg("0").status().unwrap();
    }
    
    // Continue processing
}
```

### Memory and Performance Debugging
```rust
// Memory usage debugging
#[cfg(debug_assertions)]
fn track_memory_usage() {
    use std::alloc::{GlobalAlloc, Layout, System};
    
    struct TrackingAllocator;
    
    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ptr = System.alloc(layout);
            println!("Allocated {} bytes at {:p}", layout.size(), ptr);
            ptr
        }
        
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            println!("Deallocated {} bytes at {:p}", layout.size(), ptr);
            System.dealloc(ptr, layout);
        }
    }
}

// Performance profiling
use std::time::Instant;

fn debug_performance<F, T>(name: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    
    println!("{} took {:?}", name, duration);
    
    if duration.as_millis() > 100 {
        println!("WARNING: {} is slow ({}ms)", name, duration.as_millis());
    }
    
    result
}
```

## Debugging Workflow

### Step-by-Step Process
```bash
# 1. Reproduce the issue
cargo test failing_test_name -- --nocapture

# 2. Add debug output
# Edit code to add println!, dbg!, or tracing statements

# 3. Rerun with debug info
RUST_LOG=debug cargo test failing_test_name -- --nocapture

# 4. Isolate the problem
# Create minimal failing test case

# 5. Check recent changes
git log --oneline -10
git diff HEAD~5

# 6. Check for similar issues
rg "similar error pattern" src/

# 7. Verify dependencies
cargo tree | grep problematic_crate

# 8. Test in isolation
# Create standalone reproduction case

# 9. Apply fix
# Make minimal change to fix issue

# 10. Verify fix
cargo test
cargo build
```

### Common Command Patterns
```bash
# Detailed compiler output
cargo build --verbose

# Run specific test with output
cargo test test_name -- --nocapture --test-threads=1

# Check for unused code
cargo +nightly udeps

# Analyze dependencies
cargo tree --duplicates
cargo audit

# Memory debugging (with Valgrind)
cargo valgrind test test_name

# Performance profiling
cargo flamegraph --test test_name

# Check formatting and linting
cargo fmt --check
cargo clippy -- -D warnings
```

## Issue Documentation

### Bug Report Template
```markdown
## Bug Description
Clear description of what went wrong

## Expected Behavior
What should have happened

## Actual Behavior  
What actually happened

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Environment
- Rust version: `rustc --version`
- OS: Linux/macOS/Windows
- Cargo version: `cargo --version`

## Error Output
```
Paste exact error messages here
```

## Investigation Results
- Suspected cause
- Attempted fixes
- Related issues
```

### Resolution Documentation
```markdown
## Root Cause
Explain what was actually wrong

## Fix Applied
Describe the specific changes made

## Testing
How the fix was verified

## Prevention
How to avoid this issue in the future

## Related Issues
Links to similar problems
```

---
*Debugging context for systematic problem analysis and resolution*  
*Document findings to prevent similar issues in the future*