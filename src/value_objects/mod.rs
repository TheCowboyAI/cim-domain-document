//! Document value objects

// Value objects are defined in the aggregate module
// This module can be used for additional value objects if needed

pub mod document_successor;

pub use document_successor::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Document identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub Uuid);

impl DocumentId {
    /// Create a new document ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

/// User identifier for actors in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    /// Create a new user ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for DocumentId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl<T> From<cim_domain::EntityId<T>> for DocumentId {
    fn from(entity_id: cim_domain::EntityId<T>) -> Self {
        let uuid: Uuid = entity_id.into();
        Self(uuid)
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Document metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub custom_attributes: HashMap<String, serde_json::Value>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub subcategories: Option<Vec<String>>,
    pub filename: Option<String>,
}

/// Document type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    Text,
    Image,
    Video,
    Audio,
    Pdf,
    Spreadsheet,
    Presentation,
    Archive,
    Note,
    Article,
    Proposal,
    Report,
    Contract,
    Other(String),
}

/// Document version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl DocumentVersion {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}

impl Default for DocumentVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl std::fmt::Display for DocumentVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Document revision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision(pub u32);

impl Revision {
    /// Create a new revision
    pub fn new(rev: u32) -> Self {
        Self(rev)
    }

    /// Get the next revision
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl Default for Revision {
    fn default() -> Self {
        Self(1)
    }
}

impl From<u32> for Revision {
    fn from(rev: u32) -> Self {
        Self(rev)
    }
}

/// Content block for structured documents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentBlock {
    /// Block ID
    pub id: String,
    /// Block type (e.g., "section", "paragraph", "image")
    pub block_type: String,
    /// Block title/heading
    pub title: Option<String>,
    /// Block content
    pub content: String,
    /// Metadata for the block
    pub metadata: HashMap<String, String>,
}

/// Access level for document sharing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Can view the document
    Read,
    /// Can view and comment
    Comment,
    /// Can view, comment, and edit
    Write,
    /// Full access including sharing
    Admin,
}

/// Document state in workflow
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentState {
    /// Initial draft state
    Draft,
    /// Under review
    InReview,
    /// Approved and published
    Approved,
    /// Rejected with feedback
    Rejected,
    /// Archived
    Archived,
}

/// Link type between documents
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    /// References another document
    References,
    /// Related to another document
    Related,
    /// Supersedes another document
    Supersedes,
    /// Derived from another document
    DerivedFrom,
    /// Part of a collection
    PartOf,
}

/// Document comment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID
    pub id: Uuid,
    /// Comment content
    pub content: String,
    /// Author ID
    pub author_id: Uuid,
    /// Optional reference to content block
    pub block_id: Option<String>,
    /// Parent comment for threads
    pub parent_id: Option<Uuid>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Whether the comment is resolved
    pub resolved: bool,
}

/// Version tag
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionTag {
    /// Tag name
    pub name: String,
    /// Tag description
    pub description: Option<String>,
    /// Version number at tag time
    pub version: DocumentVersion,
    /// Who created the tag
    pub tagged_by: Uuid,
    /// When the tag was created
    pub tagged_at: chrono::DateTime<chrono::Utc>,
}

/// Document collection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    /// Collection ID
    pub id: Uuid,
    /// Collection name
    pub name: String,
    /// Collection description
    pub description: Option<String>,
    /// Parent collection for hierarchy
    pub parent_id: Option<Uuid>,
    /// Collection metadata
    pub metadata: HashMap<String, String>,
}

/// Merge strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Three-way merge
    ThreeWay,
    /// Ours (keep target changes)
    Ours,
    /// Theirs (take source changes)
    Theirs,
    /// Manual (require manual resolution)
    Manual,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Automatically resolve conflicts
    Auto,
    /// Prefer target document changes
    PreferTarget,
    /// Prefer source document changes
    PreferSource,
    /// Mark conflicts for manual resolution
    Manual,
}

/// Entity extraction options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionOptions {
    /// Extract named entities (people, places, orgs)
    pub extract_entities: bool,
    /// Extract concepts and topics
    pub extract_concepts: bool,
    /// Extract keywords
    pub extract_keywords: bool,
    /// Minimum confidence threshold
    pub confidence_threshold: f32,
    /// Maximum entities to extract
    pub max_entities: Option<usize>,
}

impl Default for ExtractionOptions {
    fn default() -> Self {
        Self {
            extract_entities: true,
            extract_concepts: true,
            extract_keywords: true,
            confidence_threshold: 0.7,
            max_entities: Some(50),
        }
    }
}

/// Summary length options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryLength {
    /// Brief summary (1-2 sentences)
    Brief,
    /// Standard summary (1 paragraph)
    Standard,
    /// Detailed summary (multiple paragraphs)
    Detailed,
    /// Custom length in words
    Custom(usize),
}

/// Extracted entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// Entity text
    pub text: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Confidence score
    pub confidence: f32,
    /// Start position in document
    pub start_offset: usize,
    /// End position in document
    pub end_offset: usize,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Entity type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    /// Person name
    Person,
    /// Organization
    Organization,
    /// Location
    Location,
    /// Date/Time
    DateTime,
    /// Concept
    Concept,
    /// Keyword
    Keyword,
    /// Custom type
    Custom(String),
}

/// Document summary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummary {
    /// Summary text
    pub text: String,
    /// Key points extracted
    pub key_points: Vec<String>,
    /// Summary length type
    pub length: SummaryLength,
    /// Language of summary
    pub language: String,
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Quality score
    pub quality_score: Option<f32>,
}

/// Merge conflict
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MergeConflict {
    /// Conflict ID
    pub id: Uuid,
    /// Block or section with conflict
    pub block_id: String,
    /// Target document content
    pub target_content: String,
    /// Source document content
    pub source_content: String,
    /// Base content (common ancestor)
    pub base_content: Option<String>,
    /// Conflict type
    pub conflict_type: ConflictType,
}

/// Conflict type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Content modification conflict
    ContentModified,
    /// Block deleted in one version
    BlockDeleted,
    /// Block added in both versions
    BlockAdded,
    /// Metadata conflict
    MetadataConflict,
}

/// Template ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateId(Uuid);

impl Default for TemplateId {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

/// Document template
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTemplate {
    /// Template ID
    pub id: TemplateId,
    /// Template name
    pub name: String,
    /// Template description
    pub description: Option<String>,
    /// Template content with variable placeholders
    pub content: String,
    /// Required variables
    pub required_variables: Vec<TemplateVariable>,
    /// Template category
    pub category: String,
    /// Template version
    pub version: DocumentVersion,
}

/// Template variable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name
    pub name: String,
    /// Variable description
    pub description: Option<String>,
    /// Variable type
    pub var_type: VariableType,
    /// Default value
    pub default_value: Option<String>,
    /// Is required
    pub required: bool,
}

/// Variable type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    /// Plain text
    Text,
    /// Number
    Number,
    /// Date
    Date,
    /// Boolean
    Boolean,
    /// List of values
    List(Vec<String>),
}

/// Import format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportFormat {
    /// Markdown format
    Markdown,
    /// Plain text
    PlainText,
    /// HTML
    Html,
    /// PDF
    Pdf,
    /// Microsoft Word
    Word,
    /// JSON
    Json,
    /// Custom format
    Custom(String),
}

/// Export format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Markdown format
    Markdown,
    /// Plain text
    PlainText,
    /// HTML
    Html,
    /// PDF
    Pdf,
    /// Microsoft Word
    Word,
    /// JSON
    Json,
    /// Custom format
    Custom(String),
}

/// Import options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportOptions {
    /// Extract metadata
    pub extract_metadata: bool,
    /// Preserve formatting
    pub preserve_formatting: bool,
    /// Convert images
    pub convert_images: bool,
    /// Character encoding
    pub encoding: String,
    /// Custom options
    pub custom_options: HashMap<String, String>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            extract_metadata: true,
            preserve_formatting: true,
            convert_images: true,
            encoding: "UTF-8".to_string(),
            custom_options: HashMap::new(),
        }
    }
}

/// Export options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Include metadata
    pub include_metadata: bool,
    /// Include version history
    pub include_history: bool,
    /// Include comments
    pub include_comments: bool,
    /// Watermark text
    pub watermark: Option<String>,
    /// Custom options
    pub custom_options: HashMap<String, String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_metadata: true,
            include_history: false,
            include_comments: true,
            watermark: None,
            custom_options: HashMap::new(),
        }
    }
}

/// Search query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Query text
    pub query: String,
    /// Search fields
    pub fields: Vec<SearchField>,
    /// Filters
    pub filters: Vec<SearchFilter>,
    /// Sort order
    pub sort: SearchSort,
    /// Pagination
    pub pagination: SearchPagination,
}

/// Search field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchField {
    /// Title field
    Title,
    /// Content field
    Content,
    /// Tags field
    Tags,
    /// Author field
    Author,
    /// All fields
    All,
}

/// Search filter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Field to filter
    pub field: String,
    /// Filter operator
    pub operator: FilterOperator,
    /// Filter value
    pub value: String,
}

/// Filter operator
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    /// Equals
    Equals,
    /// Not equals
    NotEquals,
    /// Contains
    Contains,
    /// Greater than
    GreaterThan,
    /// Less than
    LessThan,
    /// In list
    In,
}

/// Search sort
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchSort {
    /// Sort field
    pub field: String,
    /// Sort direction
    pub direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    /// Ascending
    Ascending,
    /// Descending
    Descending,
}

/// Search pagination
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchPagination {
    /// Page number (0-based)
    pub page: usize,
    /// Page size
    pub size: usize,
}

impl Default for SearchPagination {
    fn default() -> Self {
        Self {
            page: 0,
            size: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json;
    use std::collections::HashMap;
    use uuid::Uuid;

    // Test helper functions
    fn create_test_uuid() -> Uuid {
        Uuid::new_v4()
    }

    fn create_test_datetime() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    // DocumentId tests
    #[test]
    fn test_document_id_creation() {
        // US-019: Test DocumentId creation and basic functionality
        let doc_id = DocumentId::new();
        
        assert!(doc_id.as_uuid() != &Uuid::nil());
        assert_eq!(format!("{}", doc_id), doc_id.0.to_string());
    }

    #[test]
    fn test_document_id_default() {
        // US-019: Test DocumentId default implementation
        let doc_id = DocumentId::default();
        
        assert!(doc_id.as_uuid() != &Uuid::nil());
    }

    #[test]
    fn test_document_id_from_uuid() {
        // US-019: Test DocumentId conversion from UUID
        let uuid = create_test_uuid();
        let doc_id = DocumentId::from(uuid);
        
        assert_eq!(doc_id.as_uuid(), &uuid);
    }

    #[test]
    fn test_document_id_serialization() {
        // US-019: Test DocumentId serialization/deserialization
        let doc_id = DocumentId::new();
        
        let serialized = serde_json::to_string(&doc_id).unwrap();
        let deserialized: DocumentId = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(doc_id, deserialized);
    }

    #[test]
    fn test_document_id_display() {
        // US-019: Test DocumentId display formatting
        let uuid = create_test_uuid();
        let doc_id = DocumentId::from(uuid);
        
        assert_eq!(format!("{}", doc_id), uuid.to_string());
    }

    // DocumentType tests
    #[test]
    fn test_document_type_variants() {
        // US-020: Test all DocumentType variants
        let types = vec![
            DocumentType::Text,
            DocumentType::Image,
            DocumentType::Video,
            DocumentType::Audio,
            DocumentType::Pdf,
            DocumentType::Spreadsheet,
            DocumentType::Presentation,
            DocumentType::Archive,
            DocumentType::Note,
            DocumentType::Article,
            DocumentType::Proposal,
            DocumentType::Report,
            DocumentType::Contract,
            DocumentType::Other("Custom".to_string()),
        ];
        
        for doc_type in types {
            let serialized = serde_json::to_string(&doc_type).unwrap();
            let deserialized: DocumentType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(doc_type, deserialized);
        }
    }

    #[test]
    fn test_document_type_custom() {
        // US-020: Test custom DocumentType variant
        let custom_type = DocumentType::Other("CustomType".to_string());
        
        if let DocumentType::Other(ref custom_name) = custom_type {
            assert_eq!(custom_name, "CustomType");
        } else {
            panic!("Expected Other variant");
        }
    }

    // DocumentVersion tests
    #[test]
    fn test_document_version_creation() {
        // US-020: Test DocumentVersion creation and display
        let version = DocumentVersion::new(1, 2, 3);
        
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(format!("{}", version), "1.2.3");
    }

    #[test]
    fn test_document_version_default() {
        // US-020: Test DocumentVersion default implementation
        let version = DocumentVersion::default();
        
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(format!("{}", version), "1.0.0");
    }

    #[test]
    fn test_document_version_serialization() {
        // US-020: Test DocumentVersion serialization
        let version = DocumentVersion::new(2, 5, 8);
        
        let serialized = serde_json::to_string(&version).unwrap();
        let deserialized: DocumentVersion = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(version, deserialized);
    }

    // Revision tests
    #[test]
    fn test_revision_creation() {
        // US-020: Test Revision creation and functionality
        let rev = Revision::new(5);
        
        assert_eq!(rev.0, 5);
        
        let next_rev = rev.next();
        assert_eq!(next_rev.0, 6);
    }

    #[test]
    fn test_revision_default() {
        // US-020: Test Revision default implementation
        let rev = Revision::default();
        
        assert_eq!(rev.0, 1);
    }

    #[test]
    fn test_revision_from_u32() {
        // US-020: Test Revision conversion from u32
        let rev = Revision::from(42u32);
        
        assert_eq!(rev.0, 42);
    }

    // ContentBlock tests
    #[test]
    fn test_content_block_creation() {
        // US-020: Test ContentBlock creation and structure
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());
        
        let block = ContentBlock {
            id: "block_123".to_string(),
            block_type: "paragraph".to_string(),
            title: Some("Block Title".to_string()),
            content: "Block content here".to_string(),
            metadata,
        };
        
        assert_eq!(block.id, "block_123");
        assert_eq!(block.block_type, "paragraph");
        assert_eq!(block.title, Some("Block Title".to_string()));
        assert_eq!(block.content, "Block content here");
        assert_eq!(block.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_content_block_serialization() {
        // US-020: Test ContentBlock serialization
        let block = ContentBlock {
            id: "test_block".to_string(),
            block_type: "section".to_string(),
            title: None,
            content: "Test content".to_string(),
            metadata: HashMap::new(),
        };
        
        let serialized = serde_json::to_string(&block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(block, deserialized);
    }

    // AccessLevel tests
    #[test]
    fn test_access_level_variants() {
        // US-020: Test AccessLevel enum variants
        let levels = vec![
            AccessLevel::Read,
            AccessLevel::Comment,
            AccessLevel::Write,
            AccessLevel::Admin,
        ];
        
        for level in levels {
            let serialized = serde_json::to_string(&level).unwrap();
            let deserialized: AccessLevel = serde_json::from_str(&serialized).unwrap();
            assert_eq!(level, deserialized);
        }
    }

    // DocumentState tests
    #[test]
    fn test_document_state_variants() {
        // US-020: Test DocumentState enum variants
        let states = vec![
            DocumentState::Draft,
            DocumentState::InReview,
            DocumentState::Approved,
            DocumentState::Rejected,
            DocumentState::Archived,
        ];
        
        for state in states {
            let serialized = serde_json::to_string(&state).unwrap();
            let deserialized: DocumentState = serde_json::from_str(&serialized).unwrap();
            assert_eq!(state, deserialized);
        }
    }

    // LinkType tests
    #[test]
    fn test_link_type_variants() {
        // US-020: Test LinkType enum variants
        let link_types = vec![
            LinkType::References,
            LinkType::Related,
            LinkType::Supersedes,
            LinkType::DerivedFrom,
            LinkType::PartOf,
        ];
        
        for link_type in link_types {
            let serialized = serde_json::to_string(&link_type).unwrap();
            let deserialized: LinkType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(link_type, deserialized);
        }
    }

    // Comment tests
    #[test]
    fn test_comment_creation() {
        // US-020: Test Comment structure and creation
        let comment = Comment {
            id: create_test_uuid(),
            content: "This is a comment".to_string(),
            author_id: create_test_uuid(),
            block_id: Some("block_123".to_string()),
            parent_id: None,
            created_at: create_test_datetime(),
            resolved: false,
        };
        
        assert_eq!(comment.content, "This is a comment");
        assert_eq!(comment.block_id, Some("block_123".to_string()));
        assert!(!comment.resolved);
    }

    #[test]
    fn test_comment_serialization() {
        // US-020: Test Comment serialization
        let comment = Comment {
            id: create_test_uuid(),
            content: "Test comment".to_string(),
            author_id: create_test_uuid(),
            block_id: None,
            parent_id: None,
            created_at: create_test_datetime(),
            resolved: true,
        };
        
        let serialized = serde_json::to_string(&comment).unwrap();
        let deserialized: Comment = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(comment, deserialized);
    }

    // TemplateId tests
    #[test]
    fn test_template_id_creation() {
        // US-021: Test TemplateId creation and functionality
        let template_id = TemplateId::new();
        
        assert!(template_id.as_uuid() != &Uuid::nil());
        
        let uuid = create_test_uuid();
        let template_id_from_uuid = TemplateId::from_uuid(uuid);
        assert_eq!(template_id_from_uuid.as_uuid(), &uuid);
    }

    #[test]
    fn test_template_id_default() {
        // US-021: Test TemplateId default implementation
        let template_id = TemplateId::default();
        
        assert!(template_id.as_uuid() != &Uuid::nil());
    }

    // DocumentTemplate tests
    #[test]
    fn test_document_template_creation() {
        // US-021: Test DocumentTemplate structure
        let template = DocumentTemplate {
            id: TemplateId::new(),
            name: "Test Template".to_string(),
            description: Some("A test template".to_string()),
            content: "Template content with {{variable}}".to_string(),
            required_variables: vec![],
            category: "test".to_string(),
            version: DocumentVersion::default(),
        };
        
        assert_eq!(template.name, "Test Template");
        assert_eq!(template.category, "test");
        assert!(template.content.contains("{{variable}}"));
    }

    // TemplateVariable tests
    #[test]
    fn test_template_variable_creation() {
        // US-021: Test TemplateVariable structure
        let variable = TemplateVariable {
            name: "test_var".to_string(),
            description: Some("A test variable".to_string()),
            var_type: VariableType::Text,
            default_value: Some("default".to_string()),
            required: true,
        };
        
        assert_eq!(variable.name, "test_var");
        assert_eq!(variable.var_type, VariableType::Text);
        assert!(variable.required);
    }

    // VariableType tests
    #[test]
    fn test_variable_type_variants() {
        // US-021: Test VariableType enum variants
        let types = vec![
            VariableType::Text,
            VariableType::Number,
            VariableType::Date,
            VariableType::Boolean,
            VariableType::List(vec!["option1".to_string(), "option2".to_string()]),
        ];
        
        for var_type in types {
            let serialized = serde_json::to_string(&var_type).unwrap();
            let deserialized: VariableType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(var_type, deserialized);
        }
    }

    // ImportFormat and ExportFormat tests
    #[test]
    fn test_import_format_variants() {
        // US-018: Test ImportFormat enum variants
        let formats = vec![
            ImportFormat::Markdown,
            ImportFormat::PlainText,
            ImportFormat::Html,
            ImportFormat::Pdf,
            ImportFormat::Word,
            ImportFormat::Json,
            ImportFormat::Custom("custom".to_string()),
        ];
        
        for format in formats {
            let serialized = serde_json::to_string(&format).unwrap();
            let deserialized: ImportFormat = serde_json::from_str(&serialized).unwrap();
            assert_eq!(format, deserialized);
        }
    }

    #[test]
    fn test_export_format_variants() {
        // US-018: Test ExportFormat enum variants
        let formats = vec![
            ExportFormat::Markdown,
            ExportFormat::PlainText,
            ExportFormat::Html,
            ExportFormat::Pdf,
            ExportFormat::Word,
            ExportFormat::Json,
            ExportFormat::Custom("custom".to_string()),
        ];
        
        for format in formats {
            let serialized = serde_json::to_string(&format).unwrap();
            let deserialized: ExportFormat = serde_json::from_str(&serialized).unwrap();
            assert_eq!(format, deserialized);
        }
    }

    // ImportOptions tests
    #[test]
    fn test_import_options_creation() {
        // US-018: Test ImportOptions structure and defaults
        let options = ImportOptions::default();
        
        assert!(options.extract_metadata);
        assert!(options.preserve_formatting);
        assert!(options.convert_images);
        assert_eq!(options.encoding, "UTF-8");
        assert!(options.custom_options.is_empty());
    }

    #[test]
    fn test_import_options_custom() {
        // US-018: Test ImportOptions with custom settings
        let mut custom_options = HashMap::new();
        custom_options.insert("custom_key".to_string(), "custom_value".to_string());
        
        let options = ImportOptions {
            extract_metadata: false,
            preserve_formatting: false,
            convert_images: false,
            encoding: "ISO-8859-1".to_string(),
            custom_options,
        };
        
        assert!(!options.extract_metadata);
        assert!(!options.preserve_formatting);
        assert!(!options.convert_images);
        assert_eq!(options.encoding, "ISO-8859-1");
        assert_eq!(options.custom_options.get("custom_key"), Some(&"custom_value".to_string()));
    }

    // ExportOptions tests
    #[test]
    fn test_export_options_creation() {
        // US-018: Test ExportOptions structure and defaults
        let options = ExportOptions::default();
        
        assert!(options.include_metadata);
        assert!(!options.include_history);
        assert!(options.include_comments);
        assert_eq!(options.watermark, None);
        assert!(options.custom_options.is_empty());
    }

    #[test]
    fn test_export_options_custom() {
        // US-018: Test ExportOptions with custom settings
        let mut custom_options = HashMap::new();
        custom_options.insert("quality".to_string(), "high".to_string());
        
        let options = ExportOptions {
            include_metadata: false,
            include_history: true,
            include_comments: false,
            watermark: Some("Confidential".to_string()),
            custom_options,
        };
        
        assert!(!options.include_metadata);
        assert!(options.include_history);
        assert!(!options.include_comments);
        assert_eq!(options.watermark, Some("Confidential".to_string()));
        assert_eq!(options.custom_options.get("quality"), Some(&"high".to_string()));
    }

    // MergeStrategy and ConflictResolution tests
    #[test]
    fn test_merge_strategy_variants() {
        // US-022: Test MergeStrategy enum variants
        let strategies = vec![
            MergeStrategy::ThreeWay,
            MergeStrategy::Ours,
            MergeStrategy::Theirs,
            MergeStrategy::Manual,
        ];
        
        for strategy in strategies {
            let serialized = serde_json::to_string(&strategy).unwrap();
            let deserialized: MergeStrategy = serde_json::from_str(&serialized).unwrap();
            assert_eq!(strategy, deserialized);
        }
    }

    #[test]
    fn test_conflict_resolution_variants() {
        // US-022: Test ConflictResolution enum variants
        let resolutions = vec![
            ConflictResolution::Auto,
            ConflictResolution::PreferTarget,
            ConflictResolution::PreferSource,
            ConflictResolution::Manual,
        ];
        
        for resolution in resolutions {
            let serialized = serde_json::to_string(&resolution).unwrap();
            let deserialized: ConflictResolution = serde_json::from_str(&serialized).unwrap();
            assert_eq!(resolution, deserialized);
        }
    }

    // ExtractionOptions tests
    #[test]
    fn test_extraction_options_default() {
        // US-023: Test ExtractionOptions default implementation
        let options = ExtractionOptions::default();
        
        assert!(options.extract_entities);
        assert!(options.extract_concepts);
        assert!(options.extract_keywords);
        assert_eq!(options.confidence_threshold, 0.7);
        assert_eq!(options.max_entities, Some(50));
    }

    #[test]
    fn test_extraction_options_custom() {
        // US-023: Test ExtractionOptions with custom settings
        let options = ExtractionOptions {
            extract_entities: false,
            extract_concepts: true,
            extract_keywords: false,
            confidence_threshold: 0.9,
            max_entities: None,
        };
        
        assert!(!options.extract_entities);
        assert!(options.extract_concepts);
        assert!(!options.extract_keywords);
        assert_eq!(options.confidence_threshold, 0.9);
        assert_eq!(options.max_entities, None);
    }

    // SummaryLength tests
    #[test]
    fn test_summary_length_variants() {
        // US-023: Test SummaryLength enum variants
        let lengths = vec![
            SummaryLength::Brief,
            SummaryLength::Standard,
            SummaryLength::Detailed,
            SummaryLength::Custom(100),
        ];
        
        for length in lengths {
            let serialized = serde_json::to_string(&length).unwrap();
            let deserialized: SummaryLength = serde_json::from_str(&serialized).unwrap();
            assert_eq!(length, deserialized);
        }
    }

    // ExtractedEntity tests
    #[test]
    fn test_extracted_entity_creation() {
        // US-023: Test ExtractedEntity structure
        let entity = ExtractedEntity {
            text: "John Doe".to_string(),
            entity_type: EntityType::Person,
            confidence: 0.95,
            start_offset: 0,
            end_offset: 8,
            metadata: HashMap::new(),
        };
        
        assert_eq!(entity.text, "John Doe");
        assert_eq!(entity.entity_type, EntityType::Person);
        assert_eq!(entity.confidence, 0.95);
        assert_eq!(entity.start_offset, 0);
        assert_eq!(entity.end_offset, 8);
    }

    // EntityType tests
    #[test]
    fn test_entity_type_variants() {
        // US-023: Test EntityType enum variants
        let types = vec![
            EntityType::Person,
            EntityType::Organization,
            EntityType::Location,
            EntityType::DateTime,
            EntityType::Concept,
            EntityType::Keyword,
            EntityType::Custom("CustomEntity".to_string()),
        ];
        
        for entity_type in types {
            let serialized = serde_json::to_string(&entity_type).unwrap();
            let deserialized: EntityType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(entity_type, deserialized);
        }
    }

    // DocumentSummary tests
    #[test]
    fn test_document_summary_creation() {
        // US-023: Test DocumentSummary structure
        let summary = DocumentSummary {
            text: "This is a summary".to_string(),
            key_points: vec!["Point 1".to_string(), "Point 2".to_string()],
            length: SummaryLength::Standard,
            language: "en".to_string(),
            generated_at: create_test_datetime(),
            quality_score: Some(0.85),
        };
        
        assert_eq!(summary.text, "This is a summary");
        assert_eq!(summary.key_points.len(), 2);
        assert_eq!(summary.length, SummaryLength::Standard);
        assert_eq!(summary.language, "en");
        assert_eq!(summary.quality_score, Some(0.85));
    }

    // MergeConflict tests
    #[test]
    fn test_merge_conflict_creation() {
        // US-022: Test MergeConflict structure
        let conflict = MergeConflict {
            id: create_test_uuid(),
            block_id: "block_123".to_string(),
            target_content: "Target content".to_string(),
            source_content: "Source content".to_string(),
            base_content: Some("Base content".to_string()),
            conflict_type: ConflictType::ContentModified,
        };
        
        assert_eq!(conflict.block_id, "block_123");
        assert_eq!(conflict.target_content, "Target content");
        assert_eq!(conflict.source_content, "Source content");
        assert_eq!(conflict.base_content, Some("Base content".to_string()));
        assert_eq!(conflict.conflict_type, ConflictType::ContentModified);
    }

    // ConflictType tests
    #[test]
    fn test_conflict_type_variants() {
        // US-022: Test ConflictType enum variants
        let types = vec![
            ConflictType::ContentModified,
            ConflictType::BlockDeleted,
            ConflictType::BlockAdded,
            ConflictType::MetadataConflict,
        ];
        
        for conflict_type in types {
            let serialized = serde_json::to_string(&conflict_type).unwrap();
            let deserialized: ConflictType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(conflict_type, deserialized);
        }
    }

    // Search-related tests
    #[test]
    fn test_search_query_creation() {
        // US-017: Test SearchQuery structure
        let query = SearchQuery {
            query: "test search".to_string(),
            fields: vec![SearchField::Title, SearchField::Content],
            filters: vec![],
            sort: SearchSort {
                field: "created_at".to_string(),
                direction: SortDirection::Descending,
            },
            pagination: SearchPagination::default(),
        };
        
        assert_eq!(query.query, "test search");
        assert_eq!(query.fields.len(), 2);
        assert_eq!(query.sort.field, "created_at");
        assert_eq!(query.sort.direction, SortDirection::Descending);
    }

    #[test]
    fn test_search_field_variants() {
        // US-017: Test SearchField enum variants
        let fields = vec![
            SearchField::Title,
            SearchField::Content,
            SearchField::Tags,
            SearchField::Author,
            SearchField::All,
        ];
        
        for field in fields {
            let serialized = serde_json::to_string(&field).unwrap();
            let deserialized: SearchField = serde_json::from_str(&serialized).unwrap();
            assert_eq!(field, deserialized);
        }
    }

    #[test]
    fn test_search_filter_creation() {
        // US-017: Test SearchFilter structure
        let filter = SearchFilter {
            field: "author".to_string(),
            operator: FilterOperator::Equals,
            value: "john@example.com".to_string(),
        };
        
        assert_eq!(filter.field, "author");
        assert_eq!(filter.operator, FilterOperator::Equals);
        assert_eq!(filter.value, "john@example.com");
    }

    #[test]
    fn test_filter_operator_variants() {
        // US-017: Test FilterOperator enum variants
        let operators = vec![
            FilterOperator::Equals,
            FilterOperator::NotEquals,
            FilterOperator::Contains,
            FilterOperator::GreaterThan,
            FilterOperator::LessThan,
            FilterOperator::In,
        ];
        
        for operator in operators {
            let serialized = serde_json::to_string(&operator).unwrap();
            let deserialized: FilterOperator = serde_json::from_str(&serialized).unwrap();
            assert_eq!(operator, deserialized);
        }
    }

    #[test]
    fn test_search_sort_creation() {
        // US-017: Test SearchSort structure
        let sort = SearchSort {
            field: "updated_at".to_string(),
            direction: SortDirection::Ascending,
        };
        
        assert_eq!(sort.field, "updated_at");
        assert_eq!(sort.direction, SortDirection::Ascending);
    }

    #[test]
    fn test_sort_direction_variants() {
        // US-017: Test SortDirection enum variants
        let directions = vec![
            SortDirection::Ascending,
            SortDirection::Descending,
        ];
        
        for direction in directions {
            let serialized = serde_json::to_string(&direction).unwrap();
            let deserialized: SortDirection = serde_json::from_str(&serialized).unwrap();
            assert_eq!(direction, deserialized);
        }
    }

    #[test]
    fn test_search_pagination_default() {
        // US-017: Test SearchPagination default implementation
        let pagination = SearchPagination::default();
        
        assert_eq!(pagination.page, 0);
        assert_eq!(pagination.size, 20);
    }

    #[test]
    fn test_search_pagination_custom() {
        // US-017: Test SearchPagination with custom values
        let pagination = SearchPagination {
            page: 5,
            size: 50,
        };
        
        assert_eq!(pagination.page, 5);
        assert_eq!(pagination.size, 50);
    }

    // Additional comprehensive tests for DocumentMetadata
    #[test]
    fn test_document_metadata_creation() {
        // US-020: Test DocumentMetadata structure
        let mut custom_attrs = HashMap::new();
        custom_attrs.insert("priority".to_string(), serde_json::Value::String("high".to_string()));
        
        let metadata = DocumentMetadata {
            title: "Test Document".to_string(),
            description: Some("A test document".to_string()),
            tags: vec!["test".to_string(), "document".to_string()],
            custom_attributes: custom_attrs,
            mime_type: Some("text/plain".to_string()),
            size_bytes: Some(1024),
            language: Some("en".to_string()),
            category: Some("testing".to_string()),
            subcategories: Some(vec!["unit-tests".to_string()]),
            filename: Some("test.txt".to_string()),
        };
        
        assert_eq!(metadata.title, "Test Document");
        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.size_bytes, Some(1024));
        assert_eq!(metadata.subcategories, Some(vec!["unit-tests".to_string()]));
    }

    // VersionTag tests
    #[test]
    fn test_version_tag_creation() {
        // US-021: Test VersionTag structure
        let tag = VersionTag {
            name: "v1.0.0".to_string(),
            description: Some("First release".to_string()),
            version: DocumentVersion::new(1, 0, 0),
            tagged_by: create_test_uuid(),
            tagged_at: create_test_datetime(),
        };
        
        assert_eq!(tag.name, "v1.0.0");
        assert_eq!(tag.description, Some("First release".to_string()));
        assert_eq!(tag.version, DocumentVersion::new(1, 0, 0));
    }

    // Collection tests
    #[test]
    fn test_collection_creation() {
        // US-021: Test Collection structure
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "folder".to_string());
        
        let collection = Collection {
            id: create_test_uuid(),
            name: "Test Collection".to_string(),
            description: Some("A test collection".to_string()),
            parent_id: None,
            metadata,
        };
        
        assert_eq!(collection.name, "Test Collection");
        assert_eq!(collection.parent_id, None);
        assert_eq!(collection.metadata.get("type"), Some(&"folder".to_string()));
    }

    // Edge cases and comprehensive serialization tests
    #[test]
    fn test_comprehensive_serialization() {
        // US-020: Test serialization of complex nested structures
        let metadata = DocumentMetadata {
            title: "Complex Document".to_string(),
            description: None,
            tags: vec![],
            custom_attributes: HashMap::new(),
            mime_type: None,
            size_bytes: None,
            language: None,
            category: None,
            subcategories: None,
            filename: None,
        };
        
        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: DocumentMetadata = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(metadata, deserialized);
    }

    #[test]
    fn test_edge_cases_empty_values() {
        // US-020: Test edge cases with empty/minimal values
        let content_block = ContentBlock {
            id: "".to_string(),
            block_type: "".to_string(),
            title: None,
            content: "".to_string(),
            metadata: HashMap::new(),
        };
        
        assert!(content_block.id.is_empty());
        assert!(content_block.content.is_empty());
        assert!(content_block.metadata.is_empty());
    }

    #[test]
    fn test_uuid_based_ids_uniqueness() {
        // US-019: Test that UUID-based IDs are unique
        let id1 = DocumentId::new();
        let id2 = DocumentId::new();
        let template_id1 = TemplateId::new();
        let template_id2 = TemplateId::new();
        
        assert_ne!(id1, id2);
        assert_ne!(template_id1, template_id2);
    }
}
