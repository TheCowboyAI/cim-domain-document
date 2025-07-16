# CIM Domain: Document

## Overview

The Document domain manages content creation, versioning, collaboration, and intelligent document processing in the CIM system. It provides comprehensive document lifecycle management with support for various content types, collaborative editing, and semantic understanding.

## Key Features

- **Document Lifecycle Management**: Create, update, version, archive documents
- **Content Versioning**: Track all changes with full history
- **Collaborative Editing**: Multi-user document collaboration
- **Semantic Processing**: Extract meaning and relationships from content
- **Format Support**: Handle multiple document formats (Markdown, HTML, PDF, etc.)
- **Access Control**: Fine-grained permissions for document operations
- **Search & Discovery**: Full-text and semantic search capabilities

## Architecture

### Domain Structure
- **Aggregates**: `Document`, `DocumentVersion`
- **Value Objects**: `DocumentContent`, `DocumentMetadata`, `ContentType`, `AccessLevel`
- **Commands**: `CreateDocument`, `UpdateContent`, `PublishVersion`, `ShareDocument`
- **Events**: `DocumentCreated`, `ContentUpdated`, `VersionPublished`, `DocumentShared`
- **Queries**: `GetDocument`, `SearchDocuments`, `GetVersionHistory`, `GetSharedDocuments`

### Integration Points
- **Identity Domain**: Document ownership and access control
- **Workflow Domain**: Document approval workflows
- **Agent Domain**: AI-powered document processing
- **Graph Domain**: Visualize document relationships

## Usage Example

```rust
use cim_domain_document::{
    commands::{CreateDocument, UpdateContent},
    value_objects::{DocumentContent, ContentType},
};

// Create a new document
let create_doc = CreateDocument {
    id: DocumentId::new(),
    title: "Project Proposal".to_string(),
    content: DocumentContent {
        body: "# Project Proposal\n\n## Overview...".to_string(),
        content_type: ContentType::Markdown,
    },
    owner_id: user_id,
    metadata: Default::default(),
};

// Update document content
let update_content = UpdateContent {
    document_id,
    content: DocumentContent {
        body: updated_markdown,
        content_type: ContentType::Markdown,
    },
    change_summary: "Added budget section".to_string(),
};
```

## Testing

Run domain tests:
```bash
cargo test -p cim-domain-document
```

## Documentation

- [User Stories](doc/user-stories.md) - Business requirements and use cases
- [API Documentation](doc/api.md) - Technical API reference

## Contributing

See the main project [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines. 