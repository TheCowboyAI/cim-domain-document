//! Integration tests for content intelligence features

use cim_domain_document::{
    commands::*,
    value_objects::*,
    services::*,
};
use cim_domain::Command as DomainCommand;
use uuid::Uuid;

#[test]
fn test_entity_extraction_workflow() {
    let document_id = DocumentId::new();
    let user = Uuid::new_v4();
    
    let content = "John Smith, CEO of TechCorp Inc., announced a partnership with \
                   DataSoft LLC at their headquarters in Seattle. The new \
                   system will integrate advanced workflow capabilities.";
    
    // Create extraction command
    let extract_cmd = ExtractEntities {
        document_id: document_id.clone(),
        options: ExtractionOptions::default(),
        requested_by: user,
    };
    
    // Verify command setup
    let aggregate_id = extract_cmd.aggregate_id().unwrap();
    let uuid: Uuid = aggregate_id.into();
    assert_eq!(uuid, *document_id.as_uuid());
    
    // Test extraction service
    let service = EntityExtractionService::new();
    let entities = service.extract_entities(content, &extract_cmd.options).unwrap();
    
    // Verify entities found
    assert!(entities.len() > 0);
    
    // Check for person entities
    let person_entities: Vec<_> = entities.iter()
        .filter(|e| matches!(e.entity_type, EntityType::Person))
        .collect();
    assert!(person_entities.len() > 0);
    assert!(person_entities.iter().any(|e| e.text.contains("John Smith")));
    
    // Check for organization entities
    let org_entities: Vec<_> = entities.iter()
        .filter(|e| matches!(e.entity_type, EntityType::Organization))
        .collect();
    assert!(org_entities.len() >= 1);
    assert!(org_entities.iter().any(|e| e.text.contains("Inc.") || e.text.contains("LLC")));
    
    // Check for concepts
    let concept_entities: Vec<_> = entities.iter()
        .filter(|e| matches!(e.entity_type, EntityType::Concept))
        .collect();
    assert!(concept_entities.iter().any(|e| e.text == "workflow"));
    assert!(concept_entities.iter().any(|e| e.text == "system"));
}

#[test]
fn test_summarization_workflow() {
    let document_id = DocumentId::new();
    let user = Uuid::new_v4();
    
    let content = "This is an important document about our new product launch. \
                   The product features advanced AI capabilities that will revolutionize \
                   the industry. Our team has worked tirelessly to bring this innovation \
                   to market. The launch date is set for next quarter. We expect strong \
                   customer adoption based on early feedback. The pricing strategy has \
                   been carefully designed to maximize market penetration.";
    
    // Test different summary lengths
    let lengths = vec![
        SummaryLength::Brief,
        SummaryLength::Standard,
        SummaryLength::Detailed,
        SummaryLength::Custom(50),
    ];
    
    let service = SummarizationService::new();
    
    for length in lengths {
        let summary_cmd = GenerateSummary {
            document_id: document_id.clone(),
            length: length.clone(),
            language: Some("en".to_string()),
            requested_by: user,
        };
        
        let summary = service.generate_summary(content, &summary_cmd.length, "en").unwrap();
        
        // Verify summary properties
        assert!(!summary.text.is_empty());
        assert_eq!(summary.language, "en");
        assert!(summary.quality_score.is_some());
        
        // Check summary length varies by type
        match length {
            SummaryLength::Brief => assert!(summary.text.len() < 200),
            SummaryLength::Standard => assert!(summary.text.len() < 500),
            SummaryLength::Detailed => assert!(summary.text.len() < 1000),
            SummaryLength::Custom(_) => assert!(summary.text.len() > 0),
        }
        
        // Key points should be extracted
        assert!(!summary.key_points.is_empty());
        assert!(summary.key_points.len() <= 3);
    }
}

#[test]
fn test_document_classification_workflow() {
    let service = ClassificationService::new();
    
    // Test technical document
    let tech_content = "This software architecture document describes the implementation \
                        of our new API using microservices. The code is written in Rust \
                        with advanced programming patterns.";
    
    let tech_classifications = service.classify_document(tech_content, &DocumentType::Report).unwrap();
    assert!(!tech_classifications.is_empty());
    assert_eq!(tech_classifications[0].category, "Technical");
    assert!(tech_classifications[0].confidence > 0.5);
    assert!(tech_classifications[0].labels.contains(&"software".to_string()));
    
    // Test business document
    let business_content = "Q4 revenue exceeded expectations with strong sales growth. \
                            Customer acquisition costs decreased while profit margins \
                            improved across all market segments.";
    
    let business_classifications = service.classify_document(business_content, &DocumentType::Report).unwrap();
    assert!(!business_classifications.is_empty());
    assert_eq!(business_classifications[0].category, "Business");
    assert!(business_classifications[0].labels.contains(&"revenue".to_string()));
    
    // Test legal document
    let legal_content = "This agreement establishes the terms and conditions for the \
                         contract between the parties. Liability is limited as specified \
                         in clause 5.2 of this agreement.";
    
    let legal_classifications = service.classify_document(legal_content, &DocumentType::Contract).unwrap();
    assert!(!legal_classifications.is_empty());
    assert_eq!(legal_classifications[0].category, "Legal");
    assert!(legal_classifications[0].labels.contains(&"agreement".to_string()));
}

#[test]
fn test_merge_conflict_detection() {
    let target_id = DocumentId::new();
    let source_id = DocumentId::new();
    let user = Uuid::new_v4();
    
    // Test different merge strategies
    let strategies = vec![
        (MergeStrategy::ThreeWay, ConflictResolution::Auto),
        (MergeStrategy::Ours, ConflictResolution::PreferTarget),
        (MergeStrategy::Theirs, ConflictResolution::PreferSource),
        (MergeStrategy::Manual, ConflictResolution::Manual),
    ];
    
    for (strategy, resolution) in strategies {
        let merge_cmd = MergeDocuments {
            target_id: target_id.clone(),
            source_id: source_id.clone(),
            strategy: strategy.clone(),
            conflict_resolution: resolution.clone(),
            merged_by: user,
        };
        
        // Verify command properties
        assert_eq!(merge_cmd.strategy, strategy);
        assert_eq!(merge_cmd.conflict_resolution, resolution);
        
        // In a real scenario, we would:
        // 1. Load both documents
        // 2. Compare content blocks
        // 3. Detect conflicts
        // 4. Apply resolution strategy
        // 5. Generate merged document
    }
}

#[test]
fn test_version_rollback_workflow() {
    let document_id = DocumentId::new();
    let user = Uuid::new_v4();
    
    // Create a rollback command
    let rollback_cmd = RollbackVersion {
        document_id: document_id.clone(),
        target_version: DocumentVersion::new(1, 2, 0),
        reason: "Reverting breaking changes from v1.3.0".to_string(),
        rolled_back_by: user,
    };
    
    // Verify command properties
    assert_eq!(rollback_cmd.target_version.major, 1);
    assert_eq!(rollback_cmd.target_version.minor, 2);
    assert_eq!(rollback_cmd.target_version.patch, 0);
    assert!(rollback_cmd.reason.contains("breaking changes"));
}

#[test]
fn test_extraction_options_customization() {
    let service = EntityExtractionService::new();
    let content = "Jane Doe from Apple Inc. discussed machine learning concepts.";
    
    // Test with custom options - only extract entities, not concepts
    let custom_options = ExtractionOptions {
        extract_entities: true,
        extract_concepts: false,
        extract_keywords: false,
        confidence_threshold: 0.8,
        max_entities: Some(5),
    };
    
    let entities = service.extract_entities(content, &custom_options).unwrap();
    
    // Should only have person and organization entities
    assert!(entities.iter().all(|e| 
        matches!(e.entity_type, EntityType::Person | EntityType::Organization)
    ));
    
    // Should not have concepts
    assert!(!entities.iter().any(|e| matches!(e.entity_type, EntityType::Concept)));
    
    // All entities should meet confidence threshold
    assert!(entities.iter().all(|e| e.confidence >= 0.8));
}

#[test]
fn test_summary_quality_metrics() {
    let service = SummarizationService::new();
    let content = "The annual report shows significant growth. Revenue increased by 25%. \
                   Market share expanded. Customer satisfaction improved. New products launched. \
                   International expansion continued. Team size doubled. Innovation accelerated.";
    
    let summary = service.generate_summary(content, &SummaryLength::Standard, "en").unwrap();
    
    // Check quality score
    assert!(summary.quality_score.is_some());
    let quality = summary.quality_score.unwrap();
    assert!(quality > 0.0 && quality <= 1.0);
    
    // Verify key points extraction
    assert!(summary.key_points.len() > 0);
    assert!(summary.key_points[0].contains("annual report"));
}

#[test]
fn test_multi_category_classification() {
    let service = ClassificationService::new();
    
    // Document with multiple categories
    let mixed_content = "This technical analysis of our software system shows that \
                         API performance directly impacts revenue. The contract terms \
                         specify system uptime requirements.";
    
    let classifications = service.classify_document(mixed_content, &DocumentType::Report).unwrap();
    
    // Should detect multiple categories
    assert!(classifications.len() >= 2);
    
    // Should include both Technical and Business
    let categories: Vec<&str> = classifications.iter()
        .map(|c| c.category.as_str())
        .collect();
    assert!(categories.contains(&"Technical"));
    assert!(categories.contains(&"Business"));
    
    // Confidence should be ordered
    for i in 1..classifications.len() {
        assert!(classifications[i-1].confidence >= classifications[i].confidence);
    }
} 