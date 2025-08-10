//! Document Domain Subject Algebra
//!
//! This module defines the complete subject algebra for the Document domain following CIM principles.
//! All NATS communication within and across domain boundaries must use these subject patterns.
//!
//! Subject Structure:
//! - `domain.document.{aggregate}.{operation}.{entity_id}`
//! - `events.document.{aggregate}.{event_type}.{entity_id}`
//! - `commands.document.{aggregate}.{command_type}.{entity_id}`
//! - `queries.document.{view}.{query_type}.{entity_id?}`
//!
//! Extended Subject Patterns:
//! - `events.document.cid.{event_type}.{content_cid}` - Content-addressed events
//! - `events.document.user.{event_type}.{user_id}` - User-scoped events
//! - `events.document.user.{user_id}.{aggregate}.{event_type}.{entity_id}` - User + entity events
//! - `events.document.cid.{content_cid}.{aggregate}.{event_type}` - CID + aggregate events
//!
//! This algebra ensures:
//! - Perfect domain isolation through event boundaries
//! - Content-addressable subscriptions via CID tracking
//! - User-scoped event streams for personalized views
//! - Multi-level subscription granularity (global, user, document, CID)
//! - Hierarchical subject organization for efficient routing
//! - Semantic clarity for AI-driven understanding
//! - NATS wildcard support for subscription patterns

use serde::{Serialize, Deserialize};
use std::fmt;
use crate::value_objects::{DocumentId, TemplateId};
use uuid::Uuid;
use cid::Cid;

/// Root subject algebra for the Document domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentSubject {
    /// Subject namespace (domain, events, commands, queries)
    pub namespace: SubjectNamespace,
    /// Document domain identifier
    pub domain: DocumentDomain,
    /// Subject scope (standard, user-scoped, CID-scoped)
    pub scope: SubjectScope,
    /// Operation or event type
    pub operation: SubjectOperation,
    /// Entity identifier (optional for broadcasts)
    pub entity_id: Option<String>,
}

/// Subject scope for different subscription patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectScope {
    /// Standard aggregate-based scope
    Aggregate(DocumentAggregate),
    /// User-scoped events and operations
    User {
        user_id: String,
        aggregate: Option<DocumentAggregate>,
    },
    /// Content-addressed scope using CID
    Cid {
        content_cid: String,
        aggregate: Option<DocumentAggregate>,
    },
    /// Combined user + document scope
    UserDocument {
        user_id: String,
        document_id: String,
    },
    /// Combined CID + user scope
    CidUser {
        content_cid: String,
        user_id: String,
    },
}

impl DocumentSubject {
    /// Create a new document subject
    pub fn new(
        namespace: SubjectNamespace,
        scope: SubjectScope,
        operation: SubjectOperation,
        entity_id: Option<String>,
    ) -> Self {
        Self {
            namespace,
            domain: DocumentDomain::Document,
            scope,
            operation,
            entity_id,
        }
    }

    /// Create an event subject with standard aggregate scope
    pub fn event(
        aggregate: DocumentAggregate,
        event_type: EventType,
        entity_id: String,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::Aggregate(aggregate),
            SubjectOperation::Event(event_type),
            Some(entity_id),
        )
    }

    /// Create a command subject with standard aggregate scope
    pub fn command(
        aggregate: DocumentAggregate,
        command_type: CommandType,
        entity_id: String,
    ) -> Self {
        Self::new(
            SubjectNamespace::Commands,
            SubjectScope::Aggregate(aggregate),
            SubjectOperation::Command(command_type),
            Some(entity_id),
        )
    }

    /// Create a query subject with standard aggregate scope
    pub fn query(
        aggregate: DocumentAggregate,
        query_type: QueryType,
        entity_id: Option<String>,
    ) -> Self {
        Self::new(
            SubjectNamespace::Queries,
            SubjectScope::Aggregate(aggregate),
            SubjectOperation::Query(query_type),
            entity_id,
        )
    }

    /// Create a workflow subject
    pub fn workflow(
        workflow_operation: WorkflowOperation,
        entity_id: String,
    ) -> Self {
        Self::new(
            SubjectNamespace::Domain,
            SubjectScope::Aggregate(DocumentAggregate::Workflow),
            SubjectOperation::Workflow(workflow_operation),
            Some(entity_id),
        )
    }

    /// Create a CID-scoped event subject for content-addressed subscriptions
    pub fn cid_event(
        content_cid: &Cid,
        event_type: EventType,
        aggregate: Option<DocumentAggregate>,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::Cid {
                content_cid: content_cid.to_string(),
                aggregate,
            },
            SubjectOperation::Event(event_type),
            None,
        )
    }

    /// Create a user-scoped event subject
    pub fn user_event(
        user_id: &Uuid,
        event_type: EventType,
        aggregate: Option<DocumentAggregate>,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::User {
                user_id: user_id.to_string(),
                aggregate,
            },
            SubjectOperation::Event(event_type),
            None,
        )
    }

    /// Create a user + document scoped event subject
    pub fn user_document_event(
        user_id: &Uuid,
        document_id: &DocumentId,
        event_type: EventType,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::UserDocument {
                user_id: user_id.to_string(),
                document_id: document_id.to_string(),
            },
            SubjectOperation::Event(event_type),
            None,
        )
    }

    /// Create a CID + user scoped event subject
    pub fn cid_user_event(
        content_cid: &Cid,
        user_id: &Uuid,
        event_type: EventType,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::CidUser {
                content_cid: content_cid.to_string(),
                user_id: user_id.to_string(),
            },
            SubjectOperation::Event(event_type),
            None,
        )
    }

    /// Get subject for wildcard subscription
    pub fn wildcard_pattern(&self) -> String {
        let base_pattern = self.build_base_subject();
        match &self.entity_id {
            Some(_) => format!("{}.>", base_pattern),
            None => format!("{}.*", base_pattern),
        }
    }

    /// Convert to NATS subject string
    pub fn to_subject(&self) -> String {
        let base_subject = self.build_base_subject();
        match &self.entity_id {
            Some(id) => format!("{}.{}", base_subject, id),
            None => base_subject,
        }
    }

    /// Build the base subject without entity ID
    fn build_base_subject(&self) -> String {
        let namespace = self.namespace.as_str();
        let domain = self.domain.as_str();
        let operation = self.operation.as_str();

        match &self.scope {
            SubjectScope::Aggregate(aggregate) => {
                format!("{}.{}.{}.{}", namespace, domain, aggregate.as_str(), operation)
            }
            SubjectScope::User { user_id, aggregate } => {
                match aggregate {
                    Some(agg) => format!("{}.{}.user.{}.{}.{}", namespace, domain, user_id, agg.as_str(), operation),
                    None => format!("{}.{}.user.{}.{}", namespace, domain, user_id, operation),
                }
            }
            SubjectScope::Cid { content_cid, aggregate } => {
                match aggregate {
                    Some(agg) => format!("{}.{}.cid.{}.{}.{}", namespace, domain, content_cid, agg.as_str(), operation),
                    None => format!("{}.{}.cid.{}.{}", namespace, domain, content_cid, operation),
                }
            }
            SubjectScope::UserDocument { user_id, document_id } => {
                format!("{}.{}.user.{}.document.{}.{}", namespace, domain, user_id, document_id, operation)
            }
            SubjectScope::CidUser { content_cid, user_id } => {
                format!("{}.{}.cid.{}.user.{}.{}", namespace, domain, content_cid, user_id, operation)
            }
        }
    }
}

impl fmt::Display for DocumentSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_subject())
    }
}

/// Subject namespaces for different message types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectNamespace {
    /// Domain-internal operations
    Domain,
    /// Event publications (past tense)
    Events,
    /// Command requests (imperative)
    Commands,
    /// Query requests (interrogative)
    Queries,
    /// Cross-domain integration
    Integration,
}

impl SubjectNamespace {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Domain => "domain",
            Self::Events => "events",
            Self::Commands => "commands",
            Self::Queries => "queries",
            Self::Integration => "integration",
        }
    }
}

/// Document domain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentDomain {
    Document,
}

impl DocumentDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "document",
        }
    }
}

/// Aggregates within the Document domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentAggregate {
    /// Core document aggregate
    Document,
    /// Document version management
    Version,
    /// Document metadata
    Metadata,
    /// Document content
    Content,
    /// Document templates
    Template,
    /// Document collections
    Collection,
    /// Document workflow
    Workflow,
    /// Document search/indexing
    Search,
    /// Document classification
    Classification,
    /// Document relationships
    Relationship,
    /// Document comments
    Comment,
}

impl DocumentAggregate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Version => "version",
            Self::Metadata => "metadata",
            Self::Content => "content",
            Self::Template => "template",
            Self::Collection => "collection",
            Self::Workflow => "workflow",
            Self::Search => "search",
            Self::Classification => "classification",
            Self::Relationship => "relationship",
            Self::Comment => "comment",
        }
    }
}

/// Operations within subject algebra
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectOperation {
    /// Event notification
    Event(EventType),
    /// Command execution
    Command(CommandType),
    /// Query execution
    Query(QueryType),
    /// Workflow operation
    Workflow(WorkflowOperation),
}

impl SubjectOperation {
    pub fn as_str(&self) -> String {
        match self {
            Self::Event(event_type) => event_type.as_str().to_string(),
            Self::Command(command_type) => command_type.as_str().to_string(),
            Self::Query(query_type) => query_type.as_str().to_string(),
            Self::Workflow(workflow_op) => workflow_op.as_str().to_string(),
        }
    }
}

/// Event types in the Document domain (past tense - things that happened)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // Core document events
    Created,
    Uploaded,
    Updated,
    Deleted,
    Archived,
    Restored,
    
    // Content events
    ContentUpdated,
    ContentReplaced,
    
    // Version events
    VersionCreated,
    VersionTagged,
    VersionRestored,
    VersionRolledBack,
    VersionsCompared,
    
    // Metadata events
    MetadataUpdated,
    Classified,
    Tagged,
    
    // Relationship events
    Shared,
    Linked,
    Merged,
    Forked,
    
    // Collection events
    AddedToCollection,
    RemovedFromCollection,
    CollectionCreated,
    
    // Processing events
    EntitiesExtracted,
    SummaryGenerated,
    TemplateApplied,
    Imported,
    Exported,
    Transformed,
    
    // Comment events
    CommentAdded,
    CommentUpdated,
    CommentDeleted,
    CommentResolved,
    
    // Edit events
    EditAccessRequested,
    EditAccessGranted,
    EditSessionStarted,
    EditSessionCancelled,
    EditedDirect,
    EditedPatch,
    EditedStructured,
    EditsMerged,
    EditFailed,
    
    // Chain events
    SuccessorCreated,
    ChainVerified,
    ChainBroken,
    
    // Workflow events
    WorkflowStarted,
    WorkflowTransitioned,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowCancelled,
    
    // State events
    StateChanged,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Uploaded => "uploaded",
            Self::Updated => "updated",
            Self::Deleted => "deleted",
            Self::Archived => "archived",
            Self::Restored => "restored",
            Self::ContentUpdated => "content_updated",
            Self::ContentReplaced => "content_replaced",
            Self::VersionCreated => "version_created",
            Self::VersionTagged => "version_tagged",
            Self::VersionRestored => "version_restored",
            Self::VersionRolledBack => "version_rolled_back",
            Self::VersionsCompared => "versions_compared",
            Self::MetadataUpdated => "metadata_updated",
            Self::Classified => "classified",
            Self::Tagged => "tagged",
            Self::Shared => "shared",
            Self::Linked => "linked",
            Self::Merged => "merged",
            Self::Forked => "forked",
            Self::AddedToCollection => "added_to_collection",
            Self::RemovedFromCollection => "removed_from_collection",
            Self::CollectionCreated => "collection_created",
            Self::EntitiesExtracted => "entities_extracted",
            Self::SummaryGenerated => "summary_generated",
            Self::TemplateApplied => "template_applied",
            Self::Imported => "imported",
            Self::Exported => "exported",
            Self::Transformed => "transformed",
            Self::CommentAdded => "comment_added",
            Self::CommentUpdated => "comment_updated",
            Self::CommentDeleted => "comment_deleted",
            Self::CommentResolved => "comment_resolved",
            Self::EditAccessRequested => "edit_access_requested",
            Self::EditAccessGranted => "edit_access_granted",
            Self::EditSessionStarted => "edit_session_started",
            Self::EditSessionCancelled => "edit_session_cancelled",
            Self::EditedDirect => "edited_direct",
            Self::EditedPatch => "edited_patch",
            Self::EditedStructured => "edited_structured",
            Self::EditsMerged => "edits_merged",
            Self::EditFailed => "edit_failed",
            Self::SuccessorCreated => "successor_created",
            Self::ChainVerified => "chain_verified",
            Self::ChainBroken => "chain_broken",
            Self::WorkflowStarted => "workflow_started",
            Self::WorkflowTransitioned => "workflow_transitioned",
            Self::WorkflowCompleted => "workflow_completed",
            Self::WorkflowFailed => "workflow_failed",
            Self::WorkflowCancelled => "workflow_cancelled",
            Self::StateChanged => "state_changed",
        }
    }
}

/// Command types in the Document domain (imperative - things to do)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandType {
    // Core document commands
    Create,
    Upload,
    Update,
    Delete,
    Archive,
    Restore,
    
    // Content commands
    UpdateContent,
    ReplaceContent,
    
    // Version commands
    CreateVersion,
    TagVersion,
    RestoreVersion,
    RollbackVersion,
    CompareVersions,
    
    // Metadata commands
    UpdateMetadata,
    Classify,
    Tag,
    
    // Relationship commands
    Share,
    Link,
    Merge,
    Fork,
    
    // Collection commands
    AddToCollection,
    RemoveFromCollection,
    CreateCollection,
    
    // Processing commands
    ExtractEntities,
    GenerateSummary,
    ApplyTemplate,
    Import,
    Export,
    Transform,
    
    // Comment commands
    AddComment,
    UpdateComment,
    DeleteComment,
    ResolveComment,
    
    // Edit commands
    RequestEditAccess,
    GrantEditAccess,
    StartEditSession,
    CancelEditSession,
    EditDirect,
    EditPatch,
    EditStructured,
    MergeEdits,
    
    // Chain commands
    CreateSuccessor,
    VerifyChain,
    
    // Workflow commands
    StartWorkflow,
    TransitionWorkflow,
    CompleteWorkflow,
    CancelWorkflow,
    
    // State commands
    ChangeState,
}

impl CommandType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Upload => "upload",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Archive => "archive",
            Self::Restore => "restore",
            Self::UpdateContent => "update_content",
            Self::ReplaceContent => "replace_content",
            Self::CreateVersion => "create_version",
            Self::TagVersion => "tag_version",
            Self::RestoreVersion => "restore_version",
            Self::RollbackVersion => "rollback_version",
            Self::CompareVersions => "compare_versions",
            Self::UpdateMetadata => "update_metadata",
            Self::Classify => "classify",
            Self::Tag => "tag",
            Self::Share => "share",
            Self::Link => "link",
            Self::Merge => "merge",
            Self::Fork => "fork",
            Self::AddToCollection => "add_to_collection",
            Self::RemoveFromCollection => "remove_from_collection",
            Self::CreateCollection => "create_collection",
            Self::ExtractEntities => "extract_entities",
            Self::GenerateSummary => "generate_summary",
            Self::ApplyTemplate => "apply_template",
            Self::Import => "import",
            Self::Export => "export",
            Self::Transform => "transform",
            Self::AddComment => "add_comment",
            Self::UpdateComment => "update_comment",
            Self::DeleteComment => "delete_comment",
            Self::ResolveComment => "resolve_comment",
            Self::RequestEditAccess => "request_edit_access",
            Self::GrantEditAccess => "grant_edit_access",
            Self::StartEditSession => "start_edit_session",
            Self::CancelEditSession => "cancel_edit_session",
            Self::EditDirect => "edit_direct",
            Self::EditPatch => "edit_patch",
            Self::EditStructured => "edit_structured",
            Self::MergeEdits => "merge_edits",
            Self::CreateSuccessor => "create_successor",
            Self::VerifyChain => "verify_chain",
            Self::StartWorkflow => "start_workflow",
            Self::TransitionWorkflow => "transition_workflow",
            Self::CompleteWorkflow => "complete_workflow",
            Self::CancelWorkflow => "cancel_workflow",
            Self::ChangeState => "change_state",
        }
    }
}

/// Query types in the Document domain (interrogative - things to ask)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryType {
    // Document queries
    Get,
    GetHistory,
    Search,
    List,
    
    // Version queries
    GetVersion,
    GetVersions,
    CompareVersions,
    
    // Metadata queries
    GetMetadata,
    GetClassification,
    GetTags,
    
    // Relationship queries
    GetRelationships,
    GetSharedWith,
    GetLinked,
    
    // Collection queries
    GetCollection,
    GetCollections,
    GetDocumentsInCollection,
    
    // Content queries
    GetContent,
    GetDiff,
    ExtractText,
    
    // Comment queries
    GetComments,
    GetComment,
    
    // Workflow queries
    GetWorkflowStatus,
    GetWorkflowHistory,
    GetActiveWorkflows,
    
    // Statistics queries
    GetStats,
    GetActivity,
    GetUsage,
}

impl QueryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::GetHistory => "get_history",
            Self::Search => "search",
            Self::List => "list",
            Self::GetVersion => "get_version",
            Self::GetVersions => "get_versions",
            Self::CompareVersions => "compare_versions",
            Self::GetMetadata => "get_metadata",
            Self::GetClassification => "get_classification",
            Self::GetTags => "get_tags",
            Self::GetRelationships => "get_relationships",
            Self::GetSharedWith => "get_shared_with",
            Self::GetLinked => "get_linked",
            Self::GetCollection => "get_collection",
            Self::GetCollections => "get_collections",
            Self::GetDocumentsInCollection => "get_documents_in_collection",
            Self::GetContent => "get_content",
            Self::GetDiff => "get_diff",
            Self::ExtractText => "extract_text",
            Self::GetComments => "get_comments",
            Self::GetComment => "get_comment",
            Self::GetWorkflowStatus => "get_workflow_status",
            Self::GetWorkflowHistory => "get_workflow_history",
            Self::GetActiveWorkflows => "get_active_workflows",
            Self::GetStats => "get_stats",
            Self::GetActivity => "get_activity",
            Self::GetUsage => "get_usage",
        }
    }
}

/// Workflow operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowOperation {
    // Instance operations
    Start,
    Transition,
    Complete,
    Cancel,
    Pause,
    Resume,
    
    // Definition operations
    Register,
    Update,
    Delete,
    
    // Status operations
    GetStatus,
    GetHistory,
    ListActive,
    
    // Node operations
    EnterNode,
    ExitNode,
    ExecuteAction,
    EvaluateCondition,
}

impl WorkflowOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Transition => "transition",
            Self::Complete => "complete",
            Self::Cancel => "cancel",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Register => "register",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::GetStatus => "get_status",
            Self::GetHistory => "get_history",
            Self::ListActive => "list_active",
            Self::EnterNode => "enter_node",
            Self::ExitNode => "exit_node",
            Self::ExecuteAction => "execute_action",
            Self::EvaluateCondition => "evaluate_condition",
        }
    }
}

/// Predefined subject patterns for common operations
pub struct SubjectPatterns;

impl SubjectPatterns {
    /// All document events
    pub fn all_document_events() -> String {
        "events.document.>".to_string()
    }
    
    /// All workflow events
    pub fn all_workflow_events() -> String {
        "events.document.workflow.>".to_string()
    }
    
    /// All commands for a specific document
    pub fn document_commands(document_id: &DocumentId) -> String {
        format!("commands.document.document.*.{}", document_id.to_string())
    }
    
    /// All events for a specific document
    pub fn document_events(document_id: &DocumentId) -> String {
        format!("events.document.document.*.{}", document_id.to_string())
    }
    
    /// All workflow operations for a document
    pub fn document_workflow(document_id: &DocumentId) -> String {
        format!("domain.document.workflow.*.{}", document_id.to_string())
    }
    
    /// Content-related events
    pub fn content_events() -> String {
        "events.document.content.>".to_string()
    }
    
    /// Version-related events
    pub fn version_events() -> String {
        "events.document.version.>".to_string()
    }
    
    /// Search queries
    pub fn search_queries() -> String {
        "queries.document.search.*".to_string()
    }
    
    /// Integration subjects for cross-domain communication
    pub fn integration_events() -> String {
        "integration.document.>".to_string()
    }

    // ===== NEW CID-BASED PATTERNS =====
    
    /// All events for a specific content CID (critical for content-addressed subscriptions)
    pub fn cid_events(content_cid: &Cid) -> String {
        format!("events.document.cid.{}.>", content_cid.to_string())
    }
    
    /// Metadata events for a specific CID
    pub fn cid_metadata_events(content_cid: &Cid) -> String {
        format!("events.document.cid.{}.metadata.*", content_cid.to_string())
    }
    
    /// All events affecting a CID from any aggregate
    pub fn cid_all_aggregates(content_cid: &Cid) -> String {
        format!("events.document.cid.{}.*.>", content_cid.to_string())
    }

    // ===== NEW USER-BASED PATTERNS =====
    
    /// All events for a specific user
    pub fn user_events(user_id: &Uuid) -> String {
        format!("events.document.user.{}.>", user_id.to_string())
    }
    
    /// Document-related events for a specific user
    pub fn user_document_events(user_id: &Uuid) -> String {
        format!("events.document.user.{}.document.*", user_id.to_string())
    }
    
    /// Workflow events for a specific user
    pub fn user_workflow_events(user_id: &Uuid) -> String {
        format!("events.document.user.{}.workflow.*", user_id.to_string())
    }
    
    /// All events for a user + document combination
    pub fn user_document_activity(user_id: &Uuid, document_id: &DocumentId) -> String {
        format!("events.document.user.{}.document.{}.>", 
                user_id.to_string(), 
                document_id.to_string())
    }

    // ===== COMBINED PATTERNS =====
    
    /// Events where a user interacts with a specific CID
    pub fn cid_user_interactions(content_cid: &Cid, user_id: &Uuid) -> String {
        format!("events.document.cid.{}.user.{}.>", 
                content_cid.to_string(), 
                user_id.to_string())
    }
    
    /// All document activity (any user, any document, any aggregate)
    pub fn all_document_activity() -> String {
        "events.document.*.>".to_string()
    }
    
    /// All user activity across documents
    pub fn all_user_activity() -> String {
        "events.document.user.*.>".to_string()
    }
    
    /// All CID-based activity
    pub fn all_cid_activity() -> String {
        "events.document.cid.*.>".to_string()
    }

    // ===== SPECIALIZED PATTERNS =====
    
    /// Events when documents with specific CID are modified (metadata, sharing, etc.)
    pub fn cid_document_modifications(content_cid: &Cid) -> String {
        format!("events.document.cid.{}.{{metadata,document}}.{{updated,shared,tagged}}", 
                content_cid.to_string())
    }
    
    /// User permissions and access events
    pub fn user_access_events(user_id: &Uuid) -> String {
        format!("events.document.user.{}.*.{{shared,edit_access_granted,edit_access_requested}}", 
                user_id.to_string())
    }
}

/// Subject builder for programmatic subject construction
pub struct SubjectBuilder {
    namespace: Option<SubjectNamespace>,
    scope: Option<SubjectScope>,
    operation: Option<SubjectOperation>,
    entity_id: Option<String>,
}

impl SubjectBuilder {
    pub fn new() -> Self {
        Self {
            namespace: None,
            scope: None,
            operation: None,
            entity_id: None,
        }
    }
    
    pub fn namespace(mut self, namespace: SubjectNamespace) -> Self {
        self.namespace = Some(namespace);
        self
    }
    
    pub fn aggregate(mut self, aggregate: DocumentAggregate) -> Self {
        self.scope = Some(SubjectScope::Aggregate(aggregate));
        self
    }
    
    pub fn user_scope(mut self, user_id: &Uuid, aggregate: Option<DocumentAggregate>) -> Self {
        self.scope = Some(SubjectScope::User {
            user_id: user_id.to_string(),
            aggregate,
        });
        self
    }
    
    pub fn cid_scope(mut self, content_cid: &Cid, aggregate: Option<DocumentAggregate>) -> Self {
        self.scope = Some(SubjectScope::Cid {
            content_cid: content_cid.to_string(),
            aggregate,
        });
        self
    }
    
    pub fn user_document_scope(mut self, user_id: &Uuid, document_id: &DocumentId) -> Self {
        self.scope = Some(SubjectScope::UserDocument {
            user_id: user_id.to_string(),
            document_id: document_id.to_string(),
        });
        self
    }
    
    pub fn cid_user_scope(mut self, content_cid: &Cid, user_id: &Uuid) -> Self {
        self.scope = Some(SubjectScope::CidUser {
            content_cid: content_cid.to_string(),
            user_id: user_id.to_string(),
        });
        self
    }
    
    pub fn operation(mut self, operation: SubjectOperation) -> Self {
        self.operation = Some(operation);
        self
    }
    
    pub fn entity_id(mut self, entity_id: impl Into<String>) -> Self {
        self.entity_id = Some(entity_id.into());
        self
    }
    
    pub fn document_id(mut self, document_id: &DocumentId) -> Self {
        self.entity_id = Some(document_id.to_string());
        self
    }
    
    pub fn template_id(mut self, template_id: &TemplateId) -> Self {
        self.entity_id = Some(template_id.as_uuid().to_string());
        self
    }
    
    pub fn uuid(mut self, uuid: &Uuid) -> Self {
        self.entity_id = Some(uuid.to_string());
        self
    }
    
    pub fn build(self) -> Result<DocumentSubject, SubjectError> {
        let namespace = self.namespace.ok_or(SubjectError::MissingNamespace)?;
        let scope = self.scope.ok_or(SubjectError::MissingScope)?;
        let operation = self.operation.ok_or(SubjectError::MissingOperation)?;
        
        Ok(DocumentSubject::new(namespace, scope, operation, self.entity_id))
    }
}

impl Default for SubjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors in subject construction
#[derive(Debug, thiserror::Error)]
pub enum SubjectError {
    #[error("Missing namespace in subject")]
    MissingNamespace,
    
    #[error("Missing scope in subject")]
    MissingScope,
    
    #[error("Missing operation in subject")]
    MissingOperation,
    
    #[error("Invalid subject format: {0}")]
    InvalidFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::DocumentId;

    #[test]
    fn test_document_subject_creation() {
        let doc_id = DocumentId::new();
        let subject = DocumentSubject::event(
            DocumentAggregate::Document,
            EventType::Created,
            doc_id.as_str().to_string(),
        );
        
        assert_eq!(subject.namespace, SubjectNamespace::Events);
        assert!(matches!(subject.scope, SubjectScope::Aggregate(DocumentAggregate::Document)));
        assert!(matches!(subject.operation, SubjectOperation::Event(EventType::Created)));
        assert_eq!(subject.entity_id, Some(doc_id.as_str().to_string()));
    }
    
    #[test]
    fn test_subject_to_string() {
        let doc_id = DocumentId::new();
        let subject = DocumentSubject::event(
            DocumentAggregate::Document,
            EventType::Created,
            doc_id.as_str().to_string(),
        );
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, format!("events.document.document.created.{}", doc_id.to_string()));
    }
    
    #[test]
    fn test_workflow_subject() {
        let doc_id = DocumentId::new();
        let subject = DocumentSubject::workflow(
            WorkflowOperation::Start,
            doc_id.as_str().to_string(),
        );
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, format!("domain.document.workflow.start.{}", doc_id.to_string()));
    }
    
    #[test]
    fn test_cid_subject() {
        use std::str::FromStr;
        let test_cid = Cid::from_str("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();
        let subject = DocumentSubject::cid_event(
            &test_cid,
            EventType::MetadataUpdated,
            Some(DocumentAggregate::Metadata),
        );
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, format!("events.document.cid.{}.metadata.metadata_updated", test_cid.to_string()));
    }
    
    #[test]
    fn test_user_subject() {
        let user_id = Uuid::new_v4();
        let subject = DocumentSubject::user_event(
            &user_id,
            EventType::Created,
            Some(DocumentAggregate::Document),
        );
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, format!("events.document.user.{}.document.created", user_id.to_string()));
    }
    
    #[test]
    fn test_user_document_subject() {
        let user_id = Uuid::new_v4();
        let doc_id = DocumentId::new();
        let subject = DocumentSubject::user_document_event(
            &user_id,
            &doc_id,
            EventType::Shared,
        );
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, format!("events.document.user.{}.document.{}.shared", 
                                        user_id.to_string(), 
                                        doc_id.to_string()));
    }
    
    #[test]
    fn test_subject_builder() {
        let doc_id = DocumentId::new();
        
        let subject = SubjectBuilder::new()
            .namespace(SubjectNamespace::Commands)
            .aggregate(DocumentAggregate::Content)
            .operation(SubjectOperation::Command(CommandType::Update))
            .document_id(&doc_id)
            .build()
            .unwrap();
        
        assert_eq!(subject.namespace, SubjectNamespace::Commands);
        assert!(matches!(subject.scope, SubjectScope::Aggregate(DocumentAggregate::Content)));
        assert!(matches!(subject.operation, SubjectOperation::Command(CommandType::Update)));
        assert_eq!(subject.entity_id, Some(doc_id.as_str().to_string()));
    }
    
    #[test]
    fn test_cid_subject_builder() {
        use std::str::FromStr;
        let test_cid = Cid::from_str("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();
        let user_id = Uuid::new_v4();
        
        let subject = SubjectBuilder::new()
            .namespace(SubjectNamespace::Events)
            .cid_user_scope(&test_cid, &user_id)
            .operation(SubjectOperation::Event(EventType::Tagged))
            .build()
            .unwrap();
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, format!("events.document.cid.{}.user.{}.tagged", 
                                        test_cid.to_string(), 
                                        user_id.to_string()));
    }
    
    #[test]
    fn test_wildcard_patterns() {
        let subject = DocumentSubject::event(
            DocumentAggregate::Document,
            EventType::Created,
            "doc123".to_string(),
        );
        
        let wildcard = subject.wildcard_pattern();
        assert_eq!(wildcard, "events.document.document.created.>");
    }
    
    #[test]
    fn test_predefined_patterns() {
        let doc_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        use std::str::FromStr;
        let test_cid = Cid::from_str("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();
        
        assert_eq!(SubjectPatterns::all_document_events(), "events.document.>");
        assert_eq!(SubjectPatterns::all_workflow_events(), "events.document.workflow.>");
        assert_eq!(
            SubjectPatterns::document_events(&doc_id), 
            format!("events.document.document.*.{}", doc_id.to_string())
        );
        assert_eq!(
            SubjectPatterns::document_workflow(&doc_id),
            format!("domain.document.workflow.*.{}", doc_id.to_string())
        );
        
        // Test new CID patterns
        assert_eq!(
            SubjectPatterns::cid_events(&test_cid),
            format!("events.document.cid.{}.>", test_cid.to_string())
        );
        
        // Test new user patterns
        assert_eq!(
            SubjectPatterns::user_events(&user_id),
            format!("events.document.user.{}.>", user_id.to_string())
        );
        
        // Test combined patterns
        assert_eq!(
            SubjectPatterns::user_document_activity(&user_id, &doc_id),
            format!("events.document.user.{}.document.{}.>", user_id.to_string(), doc_id.to_string())
        );
    }
    
    #[test]
    fn test_subject_builder_validation() {
        let result = SubjectBuilder::new()
            .aggregate(DocumentAggregate::Document)
            .build();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SubjectError::MissingNamespace));
        
        let result2 = SubjectBuilder::new()
            .namespace(SubjectNamespace::Events)
            .build();
        
        assert!(result2.is_err());
        assert!(matches!(result2.unwrap_err(), SubjectError::MissingScope));
    }
    
    #[test]
    fn test_subject_enum_serialization() {
        let namespace = SubjectNamespace::Events;
        let serialized = serde_json::to_string(&namespace).unwrap();
        let deserialized: SubjectNamespace = serde_json::from_str(&serialized).unwrap();
        assert_eq!(namespace, deserialized);
        
        let event_type = EventType::Created;
        let serialized = serde_json::to_string(&event_type).unwrap();
        let deserialized: EventType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event_type, deserialized);
    }
    
    #[test]
    fn test_all_event_types_have_string_representation() {
        let event_types = vec![
            EventType::Created, EventType::Uploaded, EventType::Updated,
            EventType::Deleted, EventType::Archived, EventType::Restored,
            EventType::WorkflowStarted, EventType::WorkflowCompleted,
            EventType::ChainVerified, EventType::EditedDirect,
        ];
        
        for event_type in event_types {
            let str_repr = event_type.as_str();
            assert!(!str_repr.is_empty());
            assert!(!str_repr.contains(' ')); // No spaces in subject components
            assert!(!str_repr.contains('.')); // No dots in subject components
        }
    }
    
    #[test]
    fn test_all_command_types_have_string_representation() {
        let command_types = vec![
            CommandType::Create, CommandType::Update, CommandType::Delete,
            CommandType::StartWorkflow, CommandType::EditDirect,
            CommandType::VerifyChain, CommandType::AddComment,
        ];
        
        for command_type in command_types {
            let str_repr = command_type.as_str();
            assert!(!str_repr.is_empty());
            assert!(!str_repr.contains(' ')); // No spaces in subject components
            assert!(!str_repr.contains('.')); // No dots in subject components
        }
    }
    
    #[test]
    fn test_subject_uniqueness() {
        let doc_id_1 = DocumentId::new();
        let doc_id_2 = DocumentId::new();
        
        let subject_1 = DocumentSubject::event(
            DocumentAggregate::Document,
            EventType::Created,
            doc_id_1.as_str().to_string(),
        );
        
        let subject_2 = DocumentSubject::event(
            DocumentAggregate::Document,
            EventType::Created,
            doc_id_2.as_str().to_string(),
        );
        
        assert_ne!(subject_1.to_subject(), subject_2.to_subject());
    }
}