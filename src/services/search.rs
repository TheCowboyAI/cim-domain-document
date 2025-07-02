//! Document search service

use crate::value_objects::{DocumentId, SearchQuery, SearchField, SearchFilter, FilterOperator};
use crate::projections::DocumentFullView;
use anyhow::Result;
use std::collections::HashMap;

/// Document search service
pub struct DocumentSearchService {
    /// Search index (simplified for now)
    index: HashMap<DocumentId, DocumentIndex>,
}

/// Document index entry
#[derive(Debug, Clone)]
struct DocumentIndex {
    /// Document ID
    pub document_id: DocumentId,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Tags
    pub tags: Vec<String>,
    /// Author
    pub author: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl DocumentSearchService {
    /// Create new search service
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Index a document
    pub fn index_document(&mut self, document: &DocumentFullView) -> Result<()> {
        let index_entry = DocumentIndex {
            document_id: document.id.clone(),
            title: document.title.clone(),
            content: document.content.clone(),
            tags: document.tags.clone(),
            author: document.author.to_string(),
            metadata: document.metadata.clone(),
        };

        self.index.insert(document.id.clone(), index_entry);
        Ok(())
    }

    /// Search documents
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for (doc_id, index) in &self.index {
            if self.matches_query(index, query) {
                let score = self.calculate_score(index, &query.query);
                results.push(SearchResult {
                    document_id: doc_id.clone(),
                    title: index.title.clone(),
                    snippet: self.generate_snippet(&index.content, &query.query),
                    score,
                    highlights: self.find_highlights(&index.content, &query.query),
                });
            }
        }

        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply pagination
        let start = query.pagination.page * query.pagination.size;
        let _end = start + query.pagination.size;
        let paginated = results.into_iter()
            .skip(start)
            .take(query.pagination.size)
            .collect();

        Ok(paginated)
    }

    /// Check if document matches query
    fn matches_query(&self, index: &DocumentIndex, query: &SearchQuery) -> bool {
        // Check text match in specified fields
        let text_match = query.fields.iter().any(|field| {
            let text = match field {
                SearchField::Title => &index.title,
                SearchField::Content => &index.content,
                SearchField::Tags => &index.tags.join(" "),
                SearchField::Author => &index.author,
                SearchField::All => {
                    return self.contains_text(&index.title, &query.query) ||
                           self.contains_text(&index.content, &query.query) ||
                           self.contains_text(&index.tags.join(" "), &query.query) ||
                           self.contains_text(&index.author, &query.query);
                }
            };
            self.contains_text(text, &query.query)
        });

        if !text_match {
            return false;
        }

        // Apply filters
        query.filters.iter().all(|filter| {
            self.matches_filter(index, filter)
        })
    }

    /// Check if document matches filter
    fn matches_filter(&self, index: &DocumentIndex, filter: &SearchFilter) -> bool {
        let value = match filter.field.as_str() {
            "title" => Some(index.title.clone()),
            "author" => Some(index.author.clone()),
            "tags" => Some(index.tags.join(",")),
            _ => index.metadata.get(&filter.field).cloned(),
        };

        match (&filter.operator, value) {
            (FilterOperator::Equals, Some(v)) => v == filter.value,
            (FilterOperator::NotEquals, Some(v)) => v != filter.value,
            (FilterOperator::Contains, Some(v)) => v.contains(&filter.value),
            (FilterOperator::GreaterThan, Some(v)) => v > filter.value,
            (FilterOperator::LessThan, Some(v)) => v < filter.value,
            (FilterOperator::In, Some(v)) => {
                let values: Vec<&str> = filter.value.split(',').collect();
                values.contains(&v.as_str())
            }
            _ => false,
        }
    }

    /// Simple text search
    fn contains_text(&self, text: &str, query: &str) -> bool {
        text.to_lowercase().contains(&query.to_lowercase())
    }

    /// Calculate relevance score
    fn calculate_score(&self, index: &DocumentIndex, query: &str) -> f32 {
        let mut score = 0.0;
        let query_lower = query.to_lowercase();

        // Title match is weighted higher
        if index.title.to_lowercase().contains(&query_lower) {
            score += 10.0;
        }

        // Count content matches
        let content_matches = index.content.to_lowercase()
            .matches(&query_lower)
            .count() as f32;
        score += content_matches;

        // Tag matches
        for tag in &index.tags {
            if tag.to_lowercase().contains(&query_lower) {
                score += 5.0;
            }
        }

        score
    }

    /// Generate snippet with context
    fn generate_snippet(&self, content: &str, query: &str) -> String {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();

        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(50);
            let end = (pos + query.len() + 50).min(content.len());
            
            let mut snippet = String::new();
            if start > 0 {
                snippet.push_str("...");
            }
            snippet.push_str(&content[start..end]);
            if end < content.len() {
                snippet.push_str("...");
            }
            snippet
        } else {
            // Return first 100 chars if no match found
            let end = 100.min(content.len());
            format!("{}...", &content[..end])
        }
    }

    /// Find highlight positions
    fn find_highlights(&self, content: &str, query: &str) -> Vec<(usize, usize)> {
        let mut highlights = Vec::new();
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();

        let mut start = 0;
        while let Some(pos) = content_lower[start..].find(&query_lower) {
            let absolute_pos = start + pos;
            highlights.push((absolute_pos, absolute_pos + query.len()));
            start = absolute_pos + 1;
        }

        highlights
    }
}

/// Search result
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// Document ID
    pub document_id: DocumentId,
    /// Document title
    pub title: String,
    /// Content snippet
    pub snippet: String,
    /// Relevance score
    pub score: f32,
    /// Highlight positions
    pub highlights: Vec<(usize, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{DocumentVersion, DocumentType, SearchPagination, SearchSort, SortDirection};
    use uuid::Uuid;

    #[test]
    fn test_document_search() {
        let mut service = DocumentSearchService::new();

        // Create test document
        let doc = DocumentFullView {
            id: DocumentId::new(),
            title: "Test Document".to_string(),
            content: "This is a test document with some content about testing.".to_string(),
            version: DocumentVersion::new(1, 0, 0),
            doc_type: DocumentType::Report,
            tags: vec!["test".to_string(), "example".to_string()],
            author: Uuid::new_v4(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Index document
        service.index_document(&doc).unwrap();

        // Search for document
        let query = SearchQuery {
            query: "test".to_string(),
            fields: vec![SearchField::All],
            filters: vec![],
            sort: SearchSort {
                field: "score".to_string(),
                direction: SortDirection::Descending,
            },
            pagination: SearchPagination::default(),
        };

        let results = service.search(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, doc.id);
        assert!(results[0].score > 0.0);
    }
} 