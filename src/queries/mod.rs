//! Document queries

use cim_domain::Query;
use serde::{Deserialize, Serialize};
use crate::value_objects::{DocumentId, DocumentState, DocumentType, ContentBlock, AccessLevel, DocumentVersion, LinkType, Comment};
use crate::events::DocumentDomainEvent;
use std::collections::HashMap;
use uuid::Uuid;

/// Query to get a document by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocument {
    /// Document ID to retrieve
    pub document_id: DocumentId,
    /// Include content blocks
    pub include_content: bool,
    /// Include metadata
    pub include_metadata: bool,
}

impl Query for GetDocument {}

/// Query to get document history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentHistory {
    /// Document ID
    pub document_id: DocumentId,
    /// Include content changes
    pub include_content_changes: bool,
}

impl Query for GetDocumentHistory {}

/// Query to search documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocuments {
    /// Text search query
    pub query: String,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Filter by MIME types
    pub mime_types: Vec<String>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

impl Query for SearchDocuments {}

/// Query to find similar documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindSimilarDocuments {
    /// Reference document ID
    pub document_id: DocumentId,
    /// Similarity threshold (0.0 to 1.0)
    pub threshold: f32,
    /// Maximum number of results
    pub limit: Option<usize>,
}

impl Query for FindSimilarDocuments {}

/// Query to get document comments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentComments {
    /// Document ID
    pub document_id: DocumentId,
    /// Include resolved comments
    pub include_resolved: bool,
    /// Filter by block ID
    pub block_id: Option<String>,
}

impl Query for GetDocumentComments {}

/// Query to get document versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentVersions {
    /// Document ID
    pub document_id: DocumentId,
    /// Include tag information
    pub include_tags: bool,
    /// Limit to specific version range
    pub from_version: Option<DocumentVersion>,
    pub to_version: Option<DocumentVersion>,
}

impl Query for GetDocumentVersions {}

/// Query to get linked documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLinkedDocuments {
    /// Document ID
    pub document_id: DocumentId,
    /// Filter by link type
    pub link_type: Option<LinkType>,
    /// Include bidirectional links
    pub bidirectional: bool,
}

impl Query for GetLinkedDocuments {}

/// Document view for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentView {
    pub document_id: DocumentId,
    pub title: String,
    pub document_type: DocumentType,
    pub state: DocumentState,
    pub author_id: Uuid,
    pub content_blocks: Vec<ContentBlock>,
    pub metadata: HashMap<String, String>,
    pub access_list: HashMap<Uuid, AccessLevel>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Document history view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHistoryView {
    pub document_id: DocumentId,
    pub events: Vec<DocumentDomainEvent>,
}

/// Comments view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsView {
    pub document_id: DocumentId,
    pub comments: Vec<Comment>,
    pub total_count: usize,
    pub unresolved_count: usize,
}

/// Versions view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionsView {
    pub document_id: DocumentId,
    pub current_version: DocumentVersion,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: DocumentVersion,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Uuid,
    pub change_summary: Option<String>,
    pub tags: Vec<String>,
}

/// Linked documents view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedDocumentsView {
    pub document_id: DocumentId,
    pub links: Vec<DocumentLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLink {
    pub target_id: DocumentId,
    pub link_type: LinkType,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Uuid,
}

/// Similar documents view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarDocumentsView {
    pub reference_id: DocumentId,
    pub similar_documents: Vec<SimilarDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarDocument {
    pub document_id: DocumentId,
    pub title: String,
    pub similarity_score: f32,
    pub common_tags: Vec<String>,
}

/// Document query handler
pub struct DocumentQueryHandler {
    // In a real implementation, this would have access to projections/read models
}

impl DocumentQueryHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle<Q: Query + 'static>(&self, query: &Q) -> Result<Box<dyn std::any::Any>, Box<dyn std::error::Error>> {
        // This is a mock implementation for the example
        // In reality, this would query projections/read models
        
        if let Some(_get_doc) = (query as &dyn std::any::Any).downcast_ref::<GetDocument>() {
            // Mock document view
            let doc = DocumentView {
                document_id: DocumentId::new(),
                title: "Mock Document".to_string(),
                document_type: DocumentType::Proposal,
                state: DocumentState::Draft,
                author_id: Uuid::new_v4(),
                content_blocks: vec![],
                metadata: HashMap::new(),
                access_list: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            Ok(Box::new(doc))
        } else if let Some(_history) = (query as &dyn std::any::Any).downcast_ref::<GetDocumentHistory>() {
            // Mock history
            let history = DocumentHistoryView {
                document_id: DocumentId::new(),
                events: vec![],
            };
            Ok(Box::new(history))
        } else {
            Err("Unknown query type".into())
        }
    }
}

impl Default for DocumentQueryHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    // Test helper functions
    fn create_test_document_id() -> DocumentId {
        DocumentId::new()
    }

    #[test]
    fn test_get_document_query_creation() {
        // US-015: Test GetDocument query creation and validation
        let query = GetDocument {
            document_id: create_test_document_id(),
            include_content: true,
            include_metadata: false,
        };

        assert_eq!(query.include_content, true);
        assert_eq!(query.include_metadata, false);
    }

    #[test]
    fn test_get_document_query_serialization() {
        // US-015: Test GetDocument query serialization/deserialization
        let query = GetDocument {
            document_id: create_test_document_id(),
            include_content: true,
            include_metadata: true,
        };

        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: GetDocument = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.include_content, query.include_content);
        assert_eq!(deserialized.include_metadata, query.include_metadata);
    }

    #[test]
    fn test_get_document_history_query() {
        // US-015: Test GetDocumentHistory query creation
        let query = GetDocumentHistory {
            document_id: create_test_document_id(),
            include_content_changes: true,
        };

        assert_eq!(query.include_content_changes, true);

        // Test serialization
        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: GetDocumentHistory = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.include_content_changes, query.include_content_changes);
    }

    #[test]
    fn test_search_documents_query() {
        // US-015: Test SearchDocuments query creation
        let query = SearchDocuments {
            query: "test search".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            mime_types: vec!["text/plain".to_string()],
            limit: Some(10),
        };

        assert_eq!(query.query, "test search");
        assert_eq!(query.tags.len(), 2);
        assert_eq!(query.mime_types.len(), 1);
        assert_eq!(query.limit, Some(10));

        // Test serialization
        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: SearchDocuments = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.query, query.query);
        assert_eq!(deserialized.tags, query.tags);
        assert_eq!(deserialized.limit, query.limit);
    }

    #[test]
    fn test_search_documents_empty_query() {
        // US-017: Test SearchDocuments with empty parameters
        let query = SearchDocuments {
            query: "".to_string(),
            tags: vec![],
            mime_types: vec![],
            limit: None,
        };

        assert_eq!(query.query, "");
        assert!(query.tags.is_empty());
        assert!(query.mime_types.is_empty());
        assert_eq!(query.limit, None);
    }

    #[test]
    fn test_find_similar_documents_query() {
        // US-015: Test FindSimilarDocuments query
        let query = FindSimilarDocuments {
            document_id: create_test_document_id(),
            threshold: 0.8,
            limit: Some(5),
        };

        assert_eq!(query.threshold, 0.8);
        assert_eq!(query.limit, Some(5));

        // Test serialization
        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: FindSimilarDocuments = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.threshold, query.threshold);
        assert_eq!(deserialized.limit, query.limit);
    }

    #[test]
    fn test_find_similar_documents_boundary_values() {
        // US-017: Test FindSimilarDocuments with boundary threshold values
        let query_min = FindSimilarDocuments {
            document_id: create_test_document_id(),
            threshold: 0.0,
            limit: None,
        };

        let query_max = FindSimilarDocuments {
            document_id: create_test_document_id(),
            threshold: 1.0,
            limit: Some(1000),
        };

        assert_eq!(query_min.threshold, 0.0);
        assert_eq!(query_max.threshold, 1.0);
        assert_eq!(query_min.limit, None);
        assert_eq!(query_max.limit, Some(1000));
    }

    #[test]
    fn test_get_document_comments_query() {
        // US-016: Test GetDocumentComments query
        let query = GetDocumentComments {
            document_id: create_test_document_id(),
            include_resolved: false,
            block_id: Some("block_123".to_string()),
        };

        assert_eq!(query.include_resolved, false);
        assert_eq!(query.block_id, Some("block_123".to_string()));

        // Test serialization
        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: GetDocumentComments = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.include_resolved, query.include_resolved);
        assert_eq!(deserialized.block_id, query.block_id);
    }

    #[test]
    fn test_get_document_versions_query() {
        // US-016: Test GetDocumentVersions query
        let query = GetDocumentVersions {
            document_id: create_test_document_id(),
            include_tags: true,
            from_version: Some(DocumentVersion { major: 1, minor: 0, patch: 0 }),
            to_version: Some(DocumentVersion { major: 2, minor: 0, patch: 0 }),
        };

        assert_eq!(query.include_tags, true);
        assert!(query.from_version.is_some());
        assert!(query.to_version.is_some());

        // Test serialization
        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: GetDocumentVersions = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.include_tags, query.include_tags);
        assert_eq!(deserialized.from_version, query.from_version);
        assert_eq!(deserialized.to_version, query.to_version);
    }

    #[test]
    fn test_get_linked_documents_query() {
        // US-016: Test GetLinkedDocuments query
        let query = GetLinkedDocuments {
            document_id: create_test_document_id(),
            link_type: Some(LinkType::References),
            bidirectional: true,
        };

        assert_eq!(query.link_type, Some(LinkType::References));
        assert_eq!(query.bidirectional, true);

        // Test serialization
        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: GetLinkedDocuments = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.link_type, query.link_type);
        assert_eq!(deserialized.bidirectional, query.bidirectional);
    }

    #[test]
    fn test_document_view_creation() {
        // US-016: Test DocumentView creation
        let view = DocumentView {
            document_id: create_test_document_id(),
            title: "Test Document".to_string(),
            document_type: DocumentType::Text,
            state: DocumentState::Draft,
            author_id: Uuid::new_v4(),
            content_blocks: vec![],
            metadata: HashMap::new(),
            access_list: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(view.title, "Test Document");
        assert_eq!(view.document_type, DocumentType::Text);
        assert_eq!(view.state, DocumentState::Draft);
        assert!(view.content_blocks.is_empty());
        assert!(view.metadata.is_empty());
        assert!(view.access_list.is_empty());
    }

    #[test]
    fn test_document_view_serialization() {
        // US-016: Test DocumentView serialization
        let view = DocumentView {
            document_id: create_test_document_id(),
            title: "Serialization Test".to_string(),
            document_type: DocumentType::Report,
            state: DocumentState::Approved,
            author_id: Uuid::new_v4(),
            content_blocks: vec![],
            metadata: HashMap::new(),
            access_list: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&view).unwrap();
        let deserialized: DocumentView = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.title, view.title);
        assert_eq!(deserialized.document_type, view.document_type);
        assert_eq!(deserialized.state, view.state);
    }

    #[test]
    fn test_document_history_view() {
        // US-016: Test DocumentHistoryView creation
        let history = DocumentHistoryView {
            document_id: create_test_document_id(),
            events: vec![],
        };

        assert!(history.events.is_empty());

        // Test serialization
        let serialized = serde_json::to_string(&history).unwrap();
        let deserialized: DocumentHistoryView = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.events.len(), history.events.len());
    }

    #[test]
    fn test_comments_view() {
        // US-016: Test CommentsView creation
        let comments = CommentsView {
            document_id: create_test_document_id(),
            comments: vec![],
            total_count: 5,
            unresolved_count: 2,
        };

        assert_eq!(comments.total_count, 5);
        assert_eq!(comments.unresolved_count, 2);
        assert!(comments.comments.is_empty());

        // Test serialization
        let serialized = serde_json::to_string(&comments).unwrap();
        let deserialized: CommentsView = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.total_count, comments.total_count);
        assert_eq!(deserialized.unresolved_count, comments.unresolved_count);
    }

    #[test]
    fn test_versions_view() {
        // US-016: Test VersionsView creation
        let versions = VersionsView {
            document_id: create_test_document_id(),
            current_version: DocumentVersion { major: 1, minor: 2, patch: 3 },
            versions: vec![],
        };

        assert_eq!(versions.current_version.major, 1);
        assert_eq!(versions.current_version.minor, 2);
        assert_eq!(versions.current_version.patch, 3);
        assert!(versions.versions.is_empty());
    }

    #[test]
    fn test_version_info() {
        // US-016: Test VersionInfo creation
        let version_info = VersionInfo {
            version: DocumentVersion { major: 1, minor: 0, patch: 0 },
            created_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
            change_summary: Some("Initial version".to_string()),
            tags: vec!["v1.0".to_string()],
        };

        assert_eq!(version_info.version.major, 1);
        assert_eq!(version_info.change_summary, Some("Initial version".to_string()));
        assert_eq!(version_info.tags.len(), 1);
        assert_eq!(version_info.tags[0], "v1.0");
    }

    #[test]
    fn test_linked_documents_view() {
        // US-016: Test LinkedDocumentsView creation
        let linked_docs = LinkedDocumentsView {
            document_id: create_test_document_id(),
            links: vec![],
        };

        assert!(linked_docs.links.is_empty());

        // Test serialization
        let serialized = serde_json::to_string(&linked_docs).unwrap();
        let deserialized: LinkedDocumentsView = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.links.len(), linked_docs.links.len());
    }

    #[test]
    fn test_document_link() {
        // US-016: Test DocumentLink creation
        let link = DocumentLink {
            target_id: create_test_document_id(),
            link_type: LinkType::References,
            description: Some("Test link".to_string()),
            created_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        };

        assert_eq!(link.link_type, LinkType::References);
        assert_eq!(link.description, Some("Test link".to_string()));
    }

    #[test]
    fn test_similar_documents_view() {
        // US-016: Test SimilarDocumentsView creation
        let similar_docs = SimilarDocumentsView {
            reference_id: create_test_document_id(),
            similar_documents: vec![],
        };

        assert!(similar_docs.similar_documents.is_empty());

        // Test serialization
        let serialized = serde_json::to_string(&similar_docs).unwrap();
        let deserialized: SimilarDocumentsView = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.similar_documents.len(), similar_docs.similar_documents.len());
    }

    #[test]
    fn test_similar_document() {
        // US-016: Test SimilarDocument creation
        let similar_doc = SimilarDocument {
            document_id: create_test_document_id(),
            title: "Similar Document".to_string(),
            similarity_score: 0.75,
            common_tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        assert_eq!(similar_doc.title, "Similar Document");
        assert_eq!(similar_doc.similarity_score, 0.75);
        assert_eq!(similar_doc.common_tags.len(), 2);
    }

    #[tokio::test]
    async fn test_document_query_handler_creation() {
        // US-015: Test DocumentQueryHandler creation
        let handler = DocumentQueryHandler::new();
        
        // Verify handler can be created
        assert!(std::ptr::addr_of!(handler) != std::ptr::null());
    }

    #[tokio::test]
    async fn test_document_query_handler_default() {
        // US-015: Test DocumentQueryHandler default implementation
        let handler = DocumentQueryHandler::default();
        
        // Verify default works
        assert!(std::ptr::addr_of!(handler) != std::ptr::null());
    }

    #[tokio::test]
    async fn test_handle_get_document_query() {
        // US-015: Test handling GetDocument query
        let handler = DocumentQueryHandler::new();
        let query = GetDocument {
            document_id: create_test_document_id(),
            include_content: true,
            include_metadata: true,
        };

        let result = handler.handle(&query).await;
        
        // Verify mock implementation returns a result
        assert!(result.is_ok());
        
        // Try to downcast the result to DocumentView
        let boxed_result = result.unwrap();
        let doc_view = boxed_result.downcast::<DocumentView>().ok();
        assert!(doc_view.is_some());
        
        let view = doc_view.unwrap();
        assert_eq!(view.title, "Mock Document");
        assert_eq!(view.document_type, DocumentType::Proposal);
        assert_eq!(view.state, DocumentState::Draft);
    }

    #[tokio::test]
    async fn test_handle_get_document_history_query() {
        // US-015: Test handling GetDocumentHistory query
        let handler = DocumentQueryHandler::new();
        let query = GetDocumentHistory {
            document_id: create_test_document_id(),
            include_content_changes: true,
        };

        let result = handler.handle(&query).await;
        
        // Verify mock implementation returns a result
        assert!(result.is_ok());
        
        // Try to downcast the result to DocumentHistoryView
        let boxed_result = result.unwrap();
        let history_view = boxed_result.downcast::<DocumentHistoryView>().ok();
        assert!(history_view.is_some());
        
        let view = history_view.unwrap();
        assert!(view.events.is_empty()); // Mock returns empty events
    }

    #[tokio::test]
    async fn test_handle_unsupported_query() {
        // US-017: Test handling unsupported query type
        let handler = DocumentQueryHandler::new();
        let query = SearchDocuments {
            query: "test".to_string(),
            tags: vec![],
            mime_types: vec![],
            limit: None,
        };

        let result = handler.handle(&query).await;
        
        // Should return error for unsupported query types in mock implementation
        assert!(result.is_err());
    }

    #[test]
    fn test_all_query_types_implement_query_trait() {
        // US-015: Test that all query types implement the Query trait
        
        // This is primarily a compilation test to ensure all queries implement Query
        let _get_doc: Box<dyn Query> = Box::new(GetDocument {
            document_id: create_test_document_id(),
            include_content: true,
            include_metadata: true,
        });

        let _history: Box<dyn Query> = Box::new(GetDocumentHistory {
            document_id: create_test_document_id(),
            include_content_changes: true,
        });

        let _search: Box<dyn Query> = Box::new(SearchDocuments {
            query: "test".to_string(),
            tags: vec![],
            mime_types: vec![],
            limit: None,
        });

        let _similar: Box<dyn Query> = Box::new(FindSimilarDocuments {
            document_id: create_test_document_id(),
            threshold: 0.5,
            limit: Some(10),
        });

        let _comments: Box<dyn Query> = Box::new(GetDocumentComments {
            document_id: create_test_document_id(),
            include_resolved: true,
            block_id: None,
        });

        let _versions: Box<dyn Query> = Box::new(GetDocumentVersions {
            document_id: create_test_document_id(),
            include_tags: false,
            from_version: None,
            to_version: None,
        });

        let _linked: Box<dyn Query> = Box::new(GetLinkedDocuments {
            document_id: create_test_document_id(),
            link_type: None,
            bidirectional: false,
        });

        // If this compiles, all queries implement Query trait
        assert!(true);
    }

    #[test]
    fn test_edge_case_empty_values() {
        // US-017: Test query creation with edge case empty values
        let empty_search = SearchDocuments {
            query: "".to_string(),
            tags: vec![],
            mime_types: vec![],
            limit: Some(0), // Edge case: zero limit
        };

        assert_eq!(empty_search.limit, Some(0));
        
        let minimal_comments = GetDocumentComments {
            document_id: create_test_document_id(),
            include_resolved: false,
            block_id: None,
        };

        assert_eq!(minimal_comments.block_id, None);
        
        let no_versions = GetDocumentVersions {
            document_id: create_test_document_id(),
            include_tags: false,
            from_version: None,
            to_version: None,
        };

        assert!(no_versions.from_version.is_none());
        assert!(no_versions.to_version.is_none());
    }
}
