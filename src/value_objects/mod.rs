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
