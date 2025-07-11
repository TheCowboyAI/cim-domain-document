//! Document metadata management handler
//!
//! This handler manages document metadata operations including classification, tagging, and search.

use crate::{
    aggregate::{DocumentAggregate, ClassificationComponent, DocumentInfoComponent},
    Document,
    events::*,
    value_objects::{DocumentId, DocumentMetadata, DocumentType},
};
use cim_domain::{DomainResult, DomainError};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handler for document metadata operations
pub struct DocumentMetadataHandler {
    /// In-memory document storage
    documents: Arc<RwLock<HashMap<uuid::Uuid, Document>>>,
    /// Tag index for fast tag-based queries
    tag_index: Arc<RwLock<HashMap<String, HashSet<uuid::Uuid>>>>,
    /// Category index
    category_index: Arc<RwLock<HashMap<String, HashSet<uuid::Uuid>>>>,
    /// Full-text search index (simplified)
    search_index: Arc<RwLock<HashMap<String, HashSet<uuid::Uuid>>>>,
}

/// Search criteria for documents
#[derive(Debug, Clone, Default)]
pub struct DocumentSearchCriteria {
    pub title_contains: Option<String>,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub document_types: Vec<String>,
    pub mime_types: Vec<String>,
    pub language: Option<String>,
    pub min_size_bytes: Option<u64>,
    pub max_size_bytes: Option<u64>,
}

/// Search result entry
#[derive(Debug, Clone)]
pub struct DocumentSearchResult {
    pub document_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub category: String,
    pub document_type: String,
    pub relevance_score: f32,
}

impl DocumentMetadataHandler {
    /// Create a new document metadata handler
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            category_index: Arc::new(RwLock::new(HashMap::new())),
            search_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Update document classification
    pub async fn update_classification(
        &self,
        document_id: uuid::Uuid,
        document_type: DocumentType,
        category: String,
        subcategories: Vec<String>,
        confidentiality: crate::aggregate::ConfidentialityLevel,
        updated_by: String,
    ) -> DomainResult<DocumentClassified> {
        let documents = self.documents.read().await;
        let document = documents.get(&document_id)
            .ok_or_else(|| DomainError::generic("Document not found"))?;
        let aggregate = DocumentAggregate::from(document.clone());
        
        // Get current tags to preserve them
        let current_tags = document.get_component::<ClassificationComponent>()
            .map(|c| c.tags.clone())
            .unwrap_or_default();
        
        // Create new classification
        let _classification = ClassificationComponent {
            document_type: format!("{:?}", document_type),
            category: category.clone(),
            subcategories: subcategories.clone(),
            tags: current_tags,
            confidentiality,
        };
        
        // Note: In a real implementation, we would update the classification through
        // proper aggregate methods. For now, we'll just update our local storage.
        
        // Update indexes
        self.update_category_index(document_id, &category).await?;
        
        // Save document
        let updated_document = Document::from(aggregate);
        drop(documents);
        let mut documents = self.documents.write().await;
        documents.insert(document_id, updated_document);
        
        // Create event
        let event = DocumentClassified {
            document_id: DocumentId::from(document_id),
            document_type,
            category,
            subcategories,
            classified_by: updated_by.clone(),
            classified_at: chrono::Utc::now(),
        };
        
        Ok(event)
    }
    
    /// Add tags to document
    pub async fn add_tags(
        &self,
        document_id: uuid::Uuid,
        tags: Vec<String>,
        added_by: String,
    ) -> DomainResult<DocumentTagged> {
        let documents = self.documents.read().await;
        let document = documents.get(&document_id)
            .ok_or_else(|| DomainError::generic("Document not found"))?;
        let aggregate = DocumentAggregate::from(document.clone());
        
        // Get current classification
        let mut classification = document.get_component::<ClassificationComponent>()
            .cloned()
            .unwrap_or_else(|| ClassificationComponent {
                document_type: "General".to_string(),
                category: "Uncategorized".to_string(),
                subcategories: vec![],
                tags: vec![],
                confidentiality: crate::aggregate::ConfidentialityLevel::Internal,
            });
        
        // Add new tags (avoid duplicates)
        for tag in &tags {
            if !classification.tags.contains(tag) {
                classification.tags.push(tag.clone());
            }
        }
        
        // Note: In a real implementation, we would update the classification through
        // proper aggregate methods.
        
        // Update tag index
        for tag in &tags {
            self.update_tag_index(document_id, tag).await?;
        }
        
        // Save document
        let updated_document = Document::from(aggregate);
        drop(documents);
        let mut documents = self.documents.write().await;
        documents.insert(document_id, updated_document);
        
        // Create event
        let event = DocumentTagged {
            document_id: DocumentId::from(document_id),
            tags: tags.clone(),
            all_tags: classification.tags,
            tagged_by: added_by.clone(),
            tagged_at: chrono::Utc::now(),
        };
        
        Ok(event)
    }
    
    /// Remove tags from document
    pub async fn remove_tags(
        &self,
        document_id: uuid::Uuid,
        tags_to_remove: Vec<String>,
        _removed_by: String,
    ) -> DomainResult<()> {
        let documents = self.documents.read().await;
        let document = documents.get(&document_id)
            .ok_or_else(|| DomainError::generic("Document not found"))?;
        let aggregate = DocumentAggregate::from(document.clone());
        
        if let Some(mut classification) = document.get_component::<ClassificationComponent>().cloned() {
            // Remove specified tags
            classification.tags.retain(|tag| !tags_to_remove.contains(tag));
            
            // Note: In a real implementation, we would update the classification through
            // proper aggregate methods.
            
            // Update tag index
            for tag in &tags_to_remove {
                self.remove_from_tag_index(document_id, tag).await?;
            }
            
            // Save document
            let updated_document = Document::from(aggregate);
            drop(documents);
            let mut documents = self.documents.write().await;
            documents.insert(document_id, updated_document);
        }
        
        Ok(())
    }
    
    /// Search documents by criteria
    pub async fn search_documents(
        &self,
        criteria: DocumentSearchCriteria,
    ) -> DomainResult<Vec<DocumentSearchResult>> {
        let mut matching_documents = HashSet::new();
        let mut first_criteria = true;
        
        // Search by tags
        if !criteria.tags.is_empty() {
            let tag_index = self.tag_index.read().await;
            let mut tag_matches = HashSet::new();
            
            for tag in &criteria.tags {
                if let Some(doc_ids) = tag_index.get(tag) {
                    if first_criteria {
                        tag_matches.extend(doc_ids);
                    } else {
                        tag_matches = tag_matches.intersection(doc_ids).cloned().collect();
                    }
                }
            }
            
            if first_criteria {
                matching_documents = tag_matches;
                first_criteria = false;
            } else {
                matching_documents = matching_documents.intersection(&tag_matches).cloned().collect();
            }
        }
        
        // Search by categories
        if !criteria.categories.is_empty() {
            let category_index = self.category_index.read().await;
            let mut category_matches = HashSet::new();
            
            for category in &criteria.categories {
                if let Some(doc_ids) = category_index.get(category) {
                    category_matches.extend(doc_ids);
                }
            }
            
            if first_criteria {
                matching_documents = category_matches;
                first_criteria = false;
            } else {
                matching_documents = matching_documents.intersection(&category_matches).cloned().collect();
            }
        }
        
        // Search by title
        if let Some(title_query) = &criteria.title_contains {
            let search_index = self.search_index.read().await;
            let title_lower = title_query.to_lowercase();
            let mut title_matches = HashSet::new();
            
            for (term, doc_ids) in search_index.iter() {
                if term.contains(&title_lower) {
                    title_matches.extend(doc_ids);
                }
            }
            
            if first_criteria {
                matching_documents = title_matches;
            } else {
                matching_documents = matching_documents.intersection(&title_matches).cloned().collect();
            }
        }
        
        // Build results
        let mut results = Vec::new();
        
        let documents = self.documents.read().await;
        
        for doc_id in matching_documents {
            if let Some(document) = documents.get(&doc_id) {
                let info = document.get_component::<DocumentInfoComponent>();
                let classification = document.get_component::<ClassificationComponent>();
                
                if let (Some(info), Some(classification)) = (info, classification) {
                    // Apply additional filters
                    if let Some(lang) = &criteria.language {
                        if info.language.as_ref() != Some(lang) {
                            continue;
                        }
                    }
                    
                    if let Some(min_size) = criteria.min_size_bytes {
                        if info.size_bytes < min_size {
                            continue;
                        }
                    }
                    
                    if let Some(max_size) = criteria.max_size_bytes {
                        if info.size_bytes > max_size {
                            continue;
                        }
                    }
                    
                    if !criteria.document_types.is_empty() {
                        if !criteria.document_types.contains(&classification.document_type) {
                            continue;
                        }
                    }
                    
                    if !criteria.mime_types.is_empty() {
                        if !criteria.mime_types.contains(&info.mime_type) {
                            continue;
                        }
                    }
                    
                    // Calculate relevance score
                    let mut relevance_score = 0.0;
                    
                    // Tag matches
                    let tag_match_count = criteria.tags.iter()
                        .filter(|t| classification.tags.contains(t))
                        .count();
                    relevance_score += tag_match_count as f32 * 0.3;
                    
                    // Category match
                    if criteria.categories.contains(&classification.category) {
                        relevance_score += 0.2;
                    }
                    
                    // Title match
                    if let Some(title_query) = &criteria.title_contains {
                        if info.title.to_lowercase().contains(&title_query.to_lowercase()) {
                            relevance_score += 0.5;
                        }
                    }
                    
                    results.push(DocumentSearchResult {
                        document_id: doc_id,
                        title: info.title.clone(),
                        description: info.description.clone(),
                        tags: classification.tags.clone(),
                        category: classification.category.clone(),
                        document_type: classification.document_type.clone(),
                        relevance_score,
                    });
                }
            }
        }
        
        // Sort by relevance
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        Ok(results)
    }
    
    /// Get documents by tag
    pub async fn get_documents_by_tag(&self, tag: &str) -> DomainResult<Vec<uuid::Uuid>> {
        let tag_index = self.tag_index.read().await;
        Ok(tag_index.get(tag).cloned().unwrap_or_default().into_iter().collect())
    }
    
    /// Get documents by category
    pub async fn get_documents_by_category(&self, category: &str) -> DomainResult<Vec<uuid::Uuid>> {
        let category_index = self.category_index.read().await;
        Ok(category_index.get(category).cloned().unwrap_or_default().into_iter().collect())
    }
    
    /// Update custom metadata attributes
    pub async fn update_custom_attributes(
        &self,
        document_id: uuid::Uuid,
        attributes: HashMap<String, serde_json::Value>,
        updated_by: String,
    ) -> DomainResult<()> {
        let documents = self.documents.read().await;
        let document = documents.get(&document_id)
            .ok_or_else(|| DomainError::generic("Document not found"))?;
        let mut aggregate = DocumentAggregate::from(document.clone());
        
        // Create metadata with custom attributes
        let metadata = DocumentMetadata {
            title: document.get_component::<DocumentInfoComponent>()
                .map(|i| i.title.clone())
                .unwrap_or_default(),
            description: document.get_component::<DocumentInfoComponent>()
                .and_then(|i| i.description.clone()),
            tags: document.get_component::<ClassificationComponent>()
                .map(|c| c.tags.clone())
                .unwrap_or_default(),
            category: document.get_component::<ClassificationComponent>()
                .map(|c| c.category.clone()),
            subcategories: document.get_component::<ClassificationComponent>()
                .map(|c| c.subcategories.clone()),
            filename: document.get_component::<DocumentInfoComponent>()
                .and_then(|i| i.filename.clone()),
            mime_type: document.get_component::<DocumentInfoComponent>()
                .map(|i| i.mime_type.clone()),
            size_bytes: document.get_component::<DocumentInfoComponent>()
                .map(|i| i.size_bytes),
            language: document.get_component::<DocumentInfoComponent>()
                .and_then(|i| i.language.clone()),
            custom_attributes: attributes,
        };
        
        // Update metadata
        aggregate.update_metadata(metadata, updated_by)?;
        
        // Save document
        let updated_document = Document::from(aggregate);
        drop(documents);
        let mut documents = self.documents.write().await;
        documents.insert(document_id, updated_document);
        
        Ok(())
    }
    
    /// Build search index for a document
    pub async fn index_document(&self, document_id: uuid::Uuid) -> DomainResult<()> {
        let documents = self.documents.read().await;
        let document = documents.get(&document_id)
            .ok_or_else(|| DomainError::generic("Document not found"))?;
        
        // Index by title
        if let Some(info) = document.get_component::<DocumentInfoComponent>() {
            let title_terms: Vec<String> = info.title
                .to_lowercase()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            
            let mut search_index = self.search_index.write().await;
            for term in title_terms {
                search_index.entry(term)
                    .or_insert_with(HashSet::new)
                    .insert(document_id);
            }
        }
        
        // Index by tags and category
        if let Some(classification) = document.get_component::<ClassificationComponent>() {
            for tag in &classification.tags {
                self.update_tag_index(document_id, tag).await?;
            }
            
            self.update_category_index(document_id, &classification.category).await?;
        }
        
        Ok(())
    }
    
    /// Update tag index
    async fn update_tag_index(&self, document_id: uuid::Uuid, tag: &str) -> DomainResult<()> {
        let mut tag_index = self.tag_index.write().await;
        tag_index.entry(tag.to_string())
            .or_insert_with(HashSet::new)
            .insert(document_id);
        Ok(())
    }
    
    /// Remove from tag index
    async fn remove_from_tag_index(&self, document_id: uuid::Uuid, tag: &str) -> DomainResult<()> {
        let mut tag_index = self.tag_index.write().await;
        if let Some(doc_ids) = tag_index.get_mut(tag) {
            doc_ids.remove(&document_id);
            if doc_ids.is_empty() {
                tag_index.remove(tag);
            }
        }
        Ok(())
    }
    
    /// Update category index
    async fn update_category_index(&self, document_id: uuid::Uuid, category: &str) -> DomainResult<()> {
        let mut category_index = self.category_index.write().await;
        category_index.entry(category.to_string())
            .or_insert_with(HashSet::new)
            .insert(document_id);
        Ok(())
    }
}

impl Default for DocumentMetadataHandler {
    fn default() -> Self {
        Self::new()
    }
}