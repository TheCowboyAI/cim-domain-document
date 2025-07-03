//! Integration tests for document version comparison

use cim_domain_document::events::*;
use cim_domain_document::projections::DocumentFullView;
use cim_domain_document::services::{ComparisonOptions, DiffAlgorithm, VersionComparisonService};
use cim_domain_document::value_objects::*;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_compare_versions_workflow() {
    let document_id = DocumentId::new();
    let author_id = Uuid::new_v4();

    // Create versions compared event
    let event = VersionsCompared {
        document_id: document_id.clone(),
        version_a: DocumentVersion::new(1, 0, 0),
        version_b: DocumentVersion::new(1, 1, 0),
        comparison_id: Uuid::new_v4(),
        compared_by: author_id,
        compared_at: chrono::Utc::now(),
    };

    // Verify event structure
    assert_eq!(event.version_a.major, 1);
    assert_eq!(event.version_b.minor, 1);
    assert_eq!(event.version_b.major, 1);
}

#[tokio::test]
async fn test_content_comparison() {
    // Create two versions of a document
    let doc_v1 = DocumentFullView {
        id: DocumentId::new(),
        title: "Technical Specification".to_string(),
        content: r#"# Technical Specification

## Overview
This document describes the system architecture.

## Components
- API Server
- Database
- Cache Layer

## Requirements
The system must handle 1000 requests per second."#
            .to_string(),
        version: DocumentVersion::new(1, 0, 0),
        doc_type: DocumentType::Report,
        tags: vec!["technical".to_string(), "architecture".to_string()],
        author: Uuid::new_v4(),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut doc_v2 = doc_v1.clone();
    doc_v2.version = DocumentVersion::new(1, 1, 0);
    doc_v2.content = r#"# Technical Specification

## Overview
This document describes the system architecture for the new platform.

## Components
- API Server
- Database
- Cache Layer
- Message Queue

## Requirements
The system must handle 5000 requests per second.

## Performance Metrics
Response time should be under 100ms."#
        .to_string();

    // Compare versions
    let result =
        VersionComparisonService::compare_versions(&doc_v1, &doc_v2, &ComparisonOptions::default())
            .unwrap();

    // Verify changes detected
    assert!(result.statistics.lines_added > 0);
    assert!(result.statistics.lines_deleted > 0);
    assert!(result.statistics.lines_unchanged > 0);
    assert!(result.statistics.similarity_ratio > 0.5); // Documents are still similar
}

#[tokio::test]
async fn test_metadata_comparison() {
    let document_id = DocumentId::new();

    // Create documents with different metadata
    let doc_v1 = DocumentFullView {
        id: document_id.clone(),
        title: "Project Plan".to_string(),
        content: "Project content".to_string(),
        version: DocumentVersion::new(1, 0, 0),
        doc_type: DocumentType::Report,
        tags: vec![],
        author: Uuid::new_v4(),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("status".to_string(), "draft".to_string());
            metadata.insert("priority".to_string(), "high".to_string());
            metadata.insert("department".to_string(), "engineering".to_string());
            metadata
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut doc_v2 = doc_v1.clone();
    doc_v2.version = DocumentVersion::new(1, 1, 0);
    doc_v2.metadata = {
        let mut metadata = HashMap::new();
        metadata.insert("status".to_string(), "approved".to_string()); // Modified
        metadata.insert("priority".to_string(), "high".to_string()); // Unchanged
        metadata.insert("reviewer".to_string(), "john.doe".to_string()); // Added
                                                                         // "department" removed
        metadata
    };

    // Compare with metadata
    let result = VersionComparisonService::compare_versions(
        &doc_v1,
        &doc_v2,
        &ComparisonOptions {
            include_metadata: true,
            include_formatting: false,
            algorithm: DiffAlgorithm::Myers,
        },
    )
    .unwrap();

    // Verify metadata changes
    assert_eq!(result.metadata_changes.len(), 3); // status modified, reviewer added, department removed
}

#[tokio::test]
async fn test_different_diff_algorithms() {
    let doc_v1 = DocumentFullView {
        id: DocumentId::new(),
        title: "Algorithm Test".to_string(),
        content: "Line 1\nLine 2\nLine 3\nLine 4\nLine 5".to_string(),
        version: DocumentVersion::new(1, 0, 0),
        doc_type: DocumentType::Note,
        tags: vec![],
        author: Uuid::new_v4(),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut doc_v2 = doc_v1.clone();
    doc_v2.version = DocumentVersion::new(1, 1, 0);
    doc_v2.content = "Line 1\nLine 2 modified\nLine 3\nNew Line\nLine 5".to_string();

    // Test Myers algorithm
    let myers_result = VersionComparisonService::compare_versions(
        &doc_v1,
        &doc_v2,
        &ComparisonOptions {
            include_metadata: false,
            include_formatting: false,
            algorithm: DiffAlgorithm::Myers,
        },
    )
    .unwrap();

    // Test Patience algorithm (currently falls back to Myers)
    let patience_result = VersionComparisonService::compare_versions(
        &doc_v1,
        &doc_v2,
        &ComparisonOptions {
            include_metadata: false,
            include_formatting: false,
            algorithm: DiffAlgorithm::Patience,
        },
    )
    .unwrap();

    // Test Histogram algorithm (currently falls back to Myers)
    let histogram_result = VersionComparisonService::compare_versions(
        &doc_v1,
        &doc_v2,
        &ComparisonOptions {
            include_metadata: false,
            include_formatting: false,
            algorithm: DiffAlgorithm::Histogram,
        },
    )
    .unwrap();

    // All should produce same results for now
    assert_eq!(
        myers_result.statistics.lines_added,
        patience_result.statistics.lines_added
    );
    assert_eq!(
        myers_result.statistics.lines_added,
        histogram_result.statistics.lines_added
    );
}

#[tokio::test]
async fn test_large_document_comparison() {
    // Create a large document
    let mut content_v1 = String::new();
    for i in 0..1000 {
        content_v1.push_str(&format!("This is line {i} of the document.\n"));
    }

    let mut content_v2 = content_v1.clone();
    // Modify some lines in the middle
    content_v2 = content_v2.replace("line 500", "modified line 500");
    content_v2 = content_v2.replace("line 750", "modified line 750");
    // Add some lines at the end
    for i in 1000..1010 {
        content_v2.push_str(&format!("This is new line {i} of the document.\n"));
    }

    let doc_v1 = DocumentFullView {
        id: DocumentId::new(),
        title: "Large Document".to_string(),
        content: content_v1,
        version: DocumentVersion::new(1, 0, 0),
        doc_type: DocumentType::Report,
        tags: vec![],
        author: Uuid::new_v4(),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut doc_v2 = doc_v1.clone();
    doc_v2.version = DocumentVersion::new(1, 1, 0);
    doc_v2.content = content_v2;

    // Compare large documents
    let result =
        VersionComparisonService::compare_versions(&doc_v1, &doc_v2, &ComparisonOptions::default())
            .unwrap();

    // Verify performance and accuracy
    assert_eq!(result.statistics.lines_added, 12); // 2 modified + 10 new
    assert_eq!(result.statistics.lines_deleted, 2); // 2 original lines replaced
    assert!(result.statistics.similarity_ratio > 0.95); // Most content unchanged
}
