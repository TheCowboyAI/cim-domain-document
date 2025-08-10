# Document Domain Definition

## Core Definition

A **Document** in the CIM (Composable Information Machine) domain is an **abstract type** that identifies a **structure meant for reading**. Documents are fundamentally **data containers** with **processing requirements**.

## Key Characteristics

### 1. Abstract Type for Reading Structures
- Documents represent structured information designed for human or machine consumption
- The structure defines how the content should be interpreted and rendered
- Document types determine processing requirements and capabilities

### 2. Processing Requirements
- **All documents require some form of processing**
- **Text rendering** is the most rudimentary form of document processing
- Processing transforms the raw document data into a readable/usable format
- Different document types require different processing strategies

### 3. Associated Program Collections
Documents require an **associated collection of Programs**, generically abstracted as:
- **Input Processors**: Programs that accept this Document Type as input
- **Output Projectors**: Programs that can produce this Document Type as a projection (output)
- Programs define the document's capabilities and interaction patterns

### 4. Data with Optional Internal Programs
- Documents are primarily treated as **DATA**
- Documents **may contain internal programs** based on:
  - **Document Type** specifications
  - **Composed processor** capabilities
- Internal programs enable dynamic behavior within documents

### 5. Dual Storage Architecture
Documents follow a **dual storage pattern**:

#### Object Store (Immutable Content)
- **Immutable Document bits** are stored in the Object Store
- Content is addressed by **CID (Content Identifier)**
- Provides **content-addressed** immutable storage
- Enables **deduplication** and **verification**

#### Event Store (Mutable Metadata)
- **Document CIDs** are placed into the Event Store
- **Metadata, lifecycle, and relationships** are event-sourced
- Enables **audit trails** and **state reconstruction**
- **Cannot contain whole documents** due to size and immutability constraints

## Document Type System

### Abstract Document Types
```rust
pub enum DocumentType {
    // Text-based documents (rendering: text processing)
    Text,
    Article, 
    Report,
    Note,
    
    // Structured documents (rendering: layout processing)  
    Presentation,
    Spreadsheet,
    
    // Binary documents (rendering: specialized processing)
    Pdf,
    Image,
    Video,
    Audio,
    
    // Archive documents (rendering: extraction + processing)
    Archive,
    
    // Contract documents (rendering: legal processing)
    Contract,
    Proposal,
    
    // Extensible for domain-specific types
    Other(String),
}
```

### Processing Requirements by Type
- **Text/Article/Report/Note**: Text rendering, formatting, search indexing
- **Presentation**: Layout rendering, slide processing, media handling
- **Spreadsheet**: Formula processing, data visualization, calculation engines
- **PDF**: Page rendering, text extraction, annotation processing
- **Image/Video/Audio**: Media decoding, format conversion, metadata extraction
- **Archive**: Content extraction, nested document processing
- **Contract/Proposal**: Legal analysis, clause extraction, compliance checking

## Processing Architecture

### Input Processors
Programs that consume documents:
```rust
trait DocumentInputProcessor<T: DocumentType> {
    fn can_process(&self, document: &Document) -> bool;
    fn process(&self, document: &Document) -> ProcessingResult;
}
```

### Output Projectors  
Programs that produce documents:
```rust
trait DocumentOutputProjector<T: DocumentType> {
    fn can_project_to(&self, target_type: &DocumentType) -> bool;
    fn project(&self, source: &dyn Any) -> Document;
}
```

### Composed Processors
Combined processing pipelines:
```rust
struct ComposedProcessor {
    input_processors: Vec<Box<dyn DocumentInputProcessor<_>>>,
    output_projectors: Vec<Box<dyn DocumentOutputProjector<_>>>,
}
```

## Storage Contract

### Object Store Contract
```rust
// Documents stored as immutable content
struct DocumentContent {
    cid: Cid,                    // Content identifier
    content: Vec<u8>,           // Immutable document bits
    mime_type: String,          // Content type
    size: u64,                  // Content size
}
```

### Event Store Contract  
```rust
// Only CIDs and metadata in events
struct DocumentCreated {
    document_id: DocumentId,
    content_cid: Cid,           // Reference to Object Store
    document_type: DocumentType,
    metadata: DocumentMetadata,  // Mutable metadata
    created_at: DateTime<Utc>,
}
```

## Domain Invariants

### 1. Immutability Separation
- **Content** (Object Store) is immutable once stored
- **Metadata** (Event Store) is mutable and event-sourced

### 2. CID-based References
- All document content references use **CIDs**
- Enables **content verification** and **deduplication**

### 3. Processing Requirements
- Every document type MUST have associated processors
- Processors define document capabilities and interactions

### 4. Type Safety
- Document types constrain valid operations
- Processing compatibility checked at type level

## Implementation Implications

### 1. Aggregate Design
```rust
pub struct Document {
    // Identity and metadata (event-sourced)
    entity: Entity<DocumentMarker>,
    components: ComponentStorage,
    
    // Content reference (object store)  
    content_address: ContentAddressComponent,
}
```

### 2. Command/Event Separation
- **Commands** operate on metadata and relationships
- **Content operations** work with CIDs and Object Store
- **Events** contain CIDs, never raw content

### 3. Query Patterns
- **Metadata queries** use Event Store projections
- **Content access** requires Object Store resolution
- **Combined views** merge both storage layers

This definition establishes Documents as **first-class domain entities** within the **Composable Information Machine** architecture, with clear boundaries between **mutable metadata** (event-sourced) and **immutable content** (content-addressed), enabling rich processing ecosystems while maintaining data integrity and auditability.

The **Composable Information Machine** philosophy emphasizes the composability of information processing components, where Documents serve as the fundamental data abstraction that different processing components can compose together to create sophisticated information workflows.