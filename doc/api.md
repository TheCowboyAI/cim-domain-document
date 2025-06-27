# Document API Documentation

## Overview

The Document domain API provides commands, queries, and events for {domain purpose}.

## Commands

### CreateDocument

Creates a new document in the system.

```rust
use cim_domain_document::commands::CreateDocument;

let command = CreateDocument {
    id: DocumentId::new(),
    // ... fields
};
```

**Fields:**
- `id`: Unique identifier for the document
- `field1`: Description
- `field2`: Description

**Validation:**
- Field1 must be non-empty
- Field2 must be valid

**Events Emitted:**
- `DocumentCreated`

### UpdateDocument

Updates an existing document.

```rust
use cim_domain_document::commands::UpdateDocument;

let command = UpdateDocument {
    id: entity_id,
    // ... fields to update
};
```

**Fields:**
- `id`: Identifier of the document to update
- `field1`: New value (optional)

**Events Emitted:**
- `DocumentUpdated`

## Queries

### GetDocumentById

Retrieves a document by its identifier.

```rust
use cim_domain_document::queries::GetDocumentById;

let query = GetDocumentById {
    id: entity_id,
};
```

**Returns:** `Option<DocumentView>`

### List{Entities}

Lists all {entities} with optional filtering.

```rust
use cim_domain_document::queries::List{Entities};

let query = List{Entities} {
    filter: Some(Filter {
        // ... filter criteria
    }),
    pagination: Some(Pagination {
        page: 1,
        per_page: 20,
    }),
};
```

**Returns:** `Vec<DocumentView>`

## Events

### DocumentCreated

Emitted when a new document is created.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreated {
    pub id: DocumentId,
    pub timestamp: SystemTime,
    // ... other fields
}
```

### DocumentUpdated

Emitted when a document is updated.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUpdated {
    pub id: DocumentId,
    pub changes: Vec<FieldChange>,
    pub timestamp: SystemTime,
}
```

## Value Objects

### DocumentId

Unique identifier for {entities}.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(Uuid);

impl DocumentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### {ValueObject}

Represents {description}.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {ValueObject} {
    pub field1: String,
    pub field2: i32,
}
```

## Error Handling

The domain uses the following error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error("document not found: {id}")]
    NotFound { id: DocumentId },
    
    #[error("Invalid {field}: {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Operation not allowed: {reason}")]
    Forbidden { reason: String },
}
```

## Usage Examples

### Creating a New Document

```rust
use cim_domain_document::{
    commands::CreateDocument,
    handlers::handle_create_document,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = CreateDocument {
        id: DocumentId::new(),
        name: "Example".to_string(),
        // ... other fields
    };
    
    let events = handle_create_document(command).await?;
    
    for event in events {
        println!("Event emitted: {:?}", event);
    }
    
    Ok(())
}
```

### Querying {Entities}

```rust
use cim_domain_document::{
    queries::{List{Entities}, execute_query},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = List{Entities} {
        filter: None,
        pagination: Some(Pagination {
            page: 1,
            per_page: 10,
        }),
    };
    
    let results = execute_query(query).await?;
    
    for item in results {
        println!("{:?}", item);
    }
    
    Ok(())
}
```

## Integration with Other Domains

This domain integrates with:

- **{Other Domain}**: Description of integration
- **{Other Domain}**: Description of integration

## Performance Considerations

- Commands are processed asynchronously
- Queries use indexed projections for fast retrieval
- Events are published to NATS for distribution

## Security Considerations

- All commands require authentication
- Authorization is enforced at the aggregate level
- Sensitive data is encrypted in events 