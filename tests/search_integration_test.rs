//! Integration tests for document search functionality

use cim_domain_document::services::DocumentSearchService;
use cim_domain_document::projections::DocumentFullView;
use cim_domain_document::value_objects::*;
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn test_search_workflow() {
    let mut search_service = DocumentSearchService::new();

    // Create and index test documents
    let doc1 = DocumentFullView {
        id: DocumentId::new(),
        title: "Introduction to Rust Programming".to_string(),
        content: "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_string(),
        version: DocumentVersion::new(1, 0, 0),
        doc_type: DocumentType::Article,
        tags: vec!["rust".to_string(), "programming".to_string(), "tutorial".to_string()],
        author: Uuid::new_v4(),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let doc2 = DocumentFullView {
        id: DocumentId::new(),
        title: "Advanced Rust Patterns".to_string(),
        content: "This article explores advanced patterns in Rust including lifetime management, trait bounds, and async programming.".to_string(),
        version: DocumentVersion::new(1, 0, 0),
        doc_type: DocumentType::Article,
        tags: vec!["rust".to_string(), "advanced".to_string(), "patterns".to_string()],
        author: Uuid::new_v4(),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let doc3 = DocumentFullView {
        id: DocumentId::new(),
        title: "Python vs Rust Performance".to_string(),
        content: "A comprehensive comparison of Python and Rust performance characteristics in various scenarios.".to_string(),
        version: DocumentVersion::new(1, 0, 0),
        doc_type: DocumentType::Report,
        tags: vec!["python".to_string(), "rust".to_string(), "performance".to_string()],
        author: Uuid::new_v4(),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    search_service.index_document(&doc1).unwrap();
    search_service.index_document(&doc2).unwrap();
    search_service.index_document(&doc3).unwrap();

    // Test basic search
    let query = SearchQuery {
        query: "rust".to_string(),
        fields: vec![SearchField::All],
        filters: vec![],
        sort: SearchSort {
            field: "score".to_string(),
            direction: SortDirection::Descending,
        },
        pagination: SearchPagination::default(),
    };

    let results = search_service.search(&query).unwrap();
    assert_eq!(results.len(), 3); // All documents contain "rust"

    // Test title-specific search
    let title_query = SearchQuery {
        query: "Advanced".to_string(),
        fields: vec![SearchField::Title],
        filters: vec![],
        sort: SearchSort {
            field: "score".to_string(),
            direction: SortDirection::Descending,
        },
        pagination: SearchPagination::default(),
    };

    let title_results = search_service.search(&title_query).unwrap();
    assert_eq!(title_results.len(), 1);
    assert_eq!(title_results[0].title, "Advanced Rust Patterns");

    // Test with filters
    let filtered_query = SearchQuery {
        query: "programming".to_string(),
        fields: vec![SearchField::All],
        filters: vec![
            SearchFilter {
                field: "tags".to_string(),
                operator: FilterOperator::Contains,
                value: "rust".to_string(),
            }
        ],
        sort: SearchSort {
            field: "score".to_string(),
            direction: SortDirection::Descending,
        },
        pagination: SearchPagination::default(),
    };

    let filtered_results = search_service.search(&filtered_query).unwrap();
    assert_eq!(filtered_results.len(), 2); // Only Rust-tagged documents
}

#[tokio::test]
async fn test_search_pagination() {
    let mut search_service = DocumentSearchService::new();

    // Create and index 10 documents
    for i in 0..10 {
        let doc = DocumentFullView {
            id: DocumentId::new(),
            title: format!("Document {}", i),
            content: format!("This is the content of document number {}", i),
            version: DocumentVersion::new(1, 0, 0),
            doc_type: DocumentType::Note,
            tags: vec!["test".to_string()],
            author: Uuid::new_v4(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        search_service.index_document(&doc).unwrap();
    }

    // Test first page
    let page1_query = SearchQuery {
        query: "document".to_string(),
        fields: vec![SearchField::All],
        filters: vec![],
        sort: SearchSort {
            field: "score".to_string(),
            direction: SortDirection::Descending,
        },
        pagination: SearchPagination {
            page: 0,
            size: 5,
        },
    };

    let page1_results = search_service.search(&page1_query).unwrap();
    assert_eq!(page1_results.len(), 5);

    // Test second page
    let page2_query = SearchQuery {
        query: "document".to_string(),
        fields: vec![SearchField::All],
        filters: vec![],
        sort: SearchSort {
            field: "score".to_string(),
            direction: SortDirection::Descending,
        },
        pagination: SearchPagination {
            page: 1,
            size: 5,
        },
    };

    let page2_results = search_service.search(&page2_query).unwrap();
    assert_eq!(page2_results.len(), 5);

    // Ensure different results on different pages
    let page1_ids: Vec<_> = page1_results.iter().map(|r| r.document_id.clone()).collect();
    let page2_ids: Vec<_> = page2_results.iter().map(|r| r.document_id.clone()).collect();
    
    for id in &page1_ids {
        assert!(!page2_ids.contains(id));
    }
}

#[tokio::test]
async fn test_search_snippets_and_highlights() {
    let mut search_service = DocumentSearchService::new();

    let doc = DocumentFullView {
        id: DocumentId::new(),
        title: "Understanding Search Algorithms".to_string(),
        content: "Search algorithms are fundamental to computer science. Binary search is one of the most efficient search algorithms for sorted data. Linear search, while simpler, can be useful for unsorted data.".to_string(),
        version: DocumentVersion::new(1, 0, 0),
        doc_type: DocumentType::Article,
        tags: vec!["algorithms".to_string(), "search".to_string()],
        author: Uuid::new_v4(),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    search_service.index_document(&doc).unwrap();

    let query = SearchQuery {
        query: "search".to_string(),
        fields: vec![SearchField::Content],
        filters: vec![],
        sort: SearchSort {
            field: "score".to_string(),
            direction: SortDirection::Descending,
        },
        pagination: SearchPagination::default(),
    };

    let results = search_service.search(&query).unwrap();
    assert_eq!(results.len(), 1);

    let result = &results[0];
    
    // Check snippet generation (case-insensitive)
    assert!(result.snippet.to_lowercase().contains("search"));
    
    // Check highlights
    assert!(!result.highlights.is_empty());
    
    // Verify highlights are at correct positions
    for (start, end) in &result.highlights {
        let highlighted_text = &doc.content[*start..*end];
        assert_eq!(highlighted_text.to_lowercase(), "search");
    }
} 