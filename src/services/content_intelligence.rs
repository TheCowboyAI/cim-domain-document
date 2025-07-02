//! Content intelligence services for document processing

use crate::value_objects::*;
use crate::events::Classification;
use cim_domain::DomainResult;
use std::collections::HashMap;

/// Service for extracting entities from document content
pub struct EntityExtractionService {
    // In a real implementation, this would use NLP models
}

impl EntityExtractionService {
    pub fn new() -> Self {
        Self {}
    }

    /// Extract entities from document content
    pub fn extract_entities(
        &self,
        content: &str,
        options: &ExtractionOptions,
    ) -> DomainResult<Vec<ExtractedEntity>> {
        let mut entities = Vec::new();

        // Simulate entity extraction with simple pattern matching
        if options.extract_entities {
            // Extract person names (simple heuristic)
            entities.extend(self.extract_person_names(content, options.confidence_threshold));
            
            // Extract organizations
            entities.extend(self.extract_organizations(content, options.confidence_threshold));
            
            // Extract locations
            entities.extend(self.extract_locations(content, options.confidence_threshold));
        }

        if options.extract_concepts {
            entities.extend(self.extract_concepts(content, options.confidence_threshold));
        }

        if options.extract_keywords {
            entities.extend(self.extract_keywords(content, options.confidence_threshold));
        }

        // Apply max entities limit
        if let Some(max) = options.max_entities {
            entities.truncate(max);
        }

        Ok(entities)
    }

    fn extract_person_names(&self, content: &str, threshold: f32) -> Vec<ExtractedEntity> {
        // Simplified: look for capitalized words that might be names
        let mut entities = Vec::new();
        let words: Vec<&str> = content.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            if word.chars().next().map_or(false, |c| c.is_uppercase()) && word.len() > 2 {
                // Check if next word is also capitalized (likely a full name)
                if i + 1 < words.len() && words[i + 1].chars().next().map_or(false, |c| c.is_uppercase()) {
                    let full_name = format!("{} {}", word, words[i + 1]);
                    entities.push(ExtractedEntity {
                        text: full_name,
                        entity_type: EntityType::Person,
                        confidence: 0.8,
                        start_offset: 0, // Would calculate actual offset
                        end_offset: 0,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        entities.into_iter().filter(|e| e.confidence >= threshold).collect()
    }

    fn extract_organizations(&self, content: &str, threshold: f32) -> Vec<ExtractedEntity> {
        // Look for common organization patterns
        let org_keywords = ["Inc.", "LLC", "Corp.", "Company", "Foundation", "Institute"];
        let mut entities = Vec::new();

        for keyword in &org_keywords {
            if let Some(pos) = content.find(keyword) {
                // Extract surrounding context
                let start = content[..pos].rfind(' ').unwrap_or(0);
                let end = pos + keyword.len();
                let org_name = content[start..end].trim();
                
                entities.push(ExtractedEntity {
                    text: org_name.to_string(),
                    entity_type: EntityType::Organization,
                    confidence: 0.85,
                    start_offset: start,
                    end_offset: end,
                    metadata: HashMap::new(),
                });
            }
        }

        entities.into_iter().filter(|e| e.confidence >= threshold).collect()
    }

    fn extract_locations(&self, _content: &str, _threshold: f32) -> Vec<ExtractedEntity> {
        // Would use NER model or gazetteer
        Vec::new()
    }

    fn extract_concepts(&self, content: &str, threshold: f32) -> Vec<ExtractedEntity> {
        // Extract domain-specific concepts
        let concepts = ["workflow", "document", "process", "system", "integration"];
        let mut entities = Vec::new();

        for concept in &concepts {
            if content.to_lowercase().contains(concept) {
                entities.push(ExtractedEntity {
                    text: concept.to_string(),
                    entity_type: EntityType::Concept,
                    confidence: 0.7,
                    start_offset: 0,
                    end_offset: 0,
                    metadata: HashMap::new(),
                });
            }
        }

        entities.into_iter().filter(|e| e.confidence >= threshold).collect()
    }

    fn extract_keywords(&self, content: &str, threshold: f32) -> Vec<ExtractedEntity> {
        // Simple keyword extraction based on word frequency
        let words: Vec<&str> = content.split_whitespace()
            .filter(|w| w.len() > 4)
            .collect();
        
        let mut word_count: HashMap<String, usize> = HashMap::new();
        for word in words {
            *word_count.entry(word.to_lowercase()).or_insert(0) += 1;
        }

        let mut keywords: Vec<_> = word_count.into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(word, count)| {
                ExtractedEntity {
                    text: word,
                    entity_type: EntityType::Keyword,
                    confidence: (count as f32 / 10.0).min(0.9),
                    start_offset: 0,
                    end_offset: 0,
                    metadata: HashMap::new(),
                }
            })
            .filter(|e| e.confidence >= threshold)
            .collect();

        keywords.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        keywords.truncate(10);
        keywords
    }
}

/// Service for generating document summaries
pub struct SummarizationService {
    // In a real implementation, this would use LLM or extractive summarization
}

impl SummarizationService {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate a summary of the document content
    pub fn generate_summary(
        &self,
        content: &str,
        length: &SummaryLength,
        language: &str,
    ) -> DomainResult<DocumentSummary> {
        let sentences: Vec<&str> = content.split(". ")
            .filter(|s| !s.is_empty())
            .collect();

        let (summary_text, _num_sentences) = match length {
            SummaryLength::Brief => {
                // Take first 1-2 sentences
                let text = sentences.iter()
                    .take(2)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(". ");
                (text, 2)
            }
            SummaryLength::Standard => {
                // Take first paragraph worth
                let text = sentences.iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(". ");
                (text, 5)
            }
            SummaryLength::Detailed => {
                // Take multiple paragraphs
                let text = sentences.iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(". ");
                (text, 10)
            }
            SummaryLength::Custom(words) => {
                // Approximate sentences based on word count
                let sentences_needed = words / 15; // Assume ~15 words per sentence
                let text = sentences.iter()
                    .take(sentences_needed)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(". ");
                (text, sentences_needed)
            }
        };

        // Extract key points (simplified)
        let key_points = sentences.iter()
            .take(3)
            .map(|s| s.to_string())
            .collect();

        Ok(DocumentSummary {
            text: summary_text,
            key_points,
            length: length.clone(),
            language: language.to_string(),
            generated_at: chrono::Utc::now(),
            quality_score: Some(0.75), // Would be calculated based on coverage, coherence, etc.
        })
    }
}

/// Service for document classification
pub struct ClassificationService {
    // In a real implementation, this would use ML models
}

impl ClassificationService {
    pub fn new() -> Self {
        Self {}
    }

    /// Classify document content into categories
    pub fn classify_document(
        &self,
        content: &str,
        _document_type: &DocumentType,
    ) -> DomainResult<Vec<Classification>> {
        let mut classifications = Vec::new();

        // Simple keyword-based classification
        let categories = [
            ("Technical", vec!["code", "programming", "software", "system", "api"]),
            ("Business", vec!["revenue", "profit", "market", "customer", "sales"]),
            ("Legal", vec!["contract", "agreement", "terms", "liability", "clause"]),
            ("Research", vec!["study", "analysis", "findings", "methodology", "results"]),
        ];

        for (category, keywords) in &categories {
            let mut score: f32 = 0.0;
            let mut matched_labels = Vec::new();

            for keyword in keywords {
                if content.to_lowercase().contains(keyword) {
                    score += 0.2;
                    matched_labels.push(keyword.to_string());
                }
            }

            if score > 0.0 {
                classifications.push(Classification {
                    category: category.to_string(),
                    confidence: score.min(0.95),
                    labels: matched_labels,
                });
            }
        }

        // Sort by confidence
        classifications.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(classifications)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_extraction() {
        let service = EntityExtractionService::new();
        let content = "John Smith from Microsoft Inc. discussed the new workflow system.";
        let options = ExtractionOptions::default();

        let entities = service.extract_entities(content, &options).unwrap();
        
        assert!(!entities.is_empty());
        assert!(entities.iter().any(|e| matches!(e.entity_type, EntityType::Person)));
        assert!(entities.iter().any(|e| matches!(e.entity_type, EntityType::Organization)));
    }

    #[test]
    fn test_summarization() {
        let service = SummarizationService::new();
        let content = "This is the first sentence. This is the second sentence. This is the third sentence. This is the fourth sentence.";
        
        let summary = service.generate_summary(content, &SummaryLength::Brief, "en").unwrap();
        
        assert!(summary.text.contains("first sentence"));
        assert_eq!(summary.language, "en");
        assert_eq!(summary.key_points.len(), 3);
    }

    #[test]
    fn test_classification() {
        let service = ClassificationService::new();
        let content = "This software system uses advanced programming techniques and APIs.";
        
        let classifications = service.classify_document(content, &DocumentType::Report).unwrap();
        
        assert!(!classifications.is_empty());
        assert_eq!(classifications[0].category, "Technical");
        assert!(classifications[0].confidence > 0.5);
    }
} 