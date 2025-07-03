//! Document value objects

// Value objects are defined in the aggregate module
// This module can be used for additional value objects if needed

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Document identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub custom: HashMap<String, String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub subcategories: Option<Vec<String>>,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
