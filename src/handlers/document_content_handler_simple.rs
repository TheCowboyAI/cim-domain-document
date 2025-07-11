//! Simplified document content management handler
//!
//! This handler manages document content operations with a simplified approach.

use crate::{
    aggregate::DocumentAggregate,
    Document,
    events::*,
    value_objects::*,
};
use cim_domain::{DomainResult, AggregateRoot};
use cid::Cid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handler for document content operations
pub struct DocumentContentHandler {
    /// In-memory document storage
    documents: Arc<RwLock<HashMap<uuid::Uuid, Document>>>,
    /// Cache of document metadata for quick access
    metadata_cache: Arc<RwLock<HashMap<uuid::Uuid, DocumentMetadata>>>,
    /// Content storage backend reference
    content_store: Arc<dyn ContentStore>,
}

/// Trait for content storage operations
#[async_trait::async_trait]
pub trait ContentStore: Send + Sync {
    /// Store content and return CID
    async fn store(&self, content: &[u8]) -> DomainResult<Cid>;
    
    /// Retrieve content by CID
    async fn retrieve(&self, cid: &Cid) -> DomainResult<Vec<u8>>;
    
    /// Check if content exists
    async fn exists(&self, cid: &Cid) -> DomainResult<bool>;
    
    /// Delete content (for cleanup)
    async fn delete(&self, cid: &Cid) -> DomainResult<()>;
}

impl DocumentContentHandler {
    /// Create a new document content handler
    pub fn new(content_store: Arc<dyn ContentStore>) -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            content_store,
        }
    }
    
    /// Upload document content
    pub async fn upload_document(
        &self,
        document_id: uuid::Uuid,
        content: Vec<u8>,
        metadata: DocumentMetadata,
        document_type: DocumentType,
        uploaded_by: String,
    ) -> DomainResult<DocumentUploaded> {
        // Store content and get CID
        let content_cid = self.content_store.store(&content).await?;
        
        // Create or update document aggregate
        let documents = self.documents.read().await;
        let mut aggregate = match documents.get(&document_id) {
            Some(doc) => DocumentAggregate::from(doc.clone()),
            None => DocumentAggregate::new(document_id),
        };
        drop(documents);
        
        // Create path from metadata
        let path = std::path::PathBuf::from(
            metadata.filename.as_ref().unwrap_or(&format!("document_{}", document_id))
        );
        
        // Process upload through aggregate's public method
        let events = aggregate.upload(
            path.clone(),
            content_cid,
            metadata.clone(),
            document_type,
            uploaded_by.clone(),
        )?;
        
        // Save aggregate
        let document = Document::from(aggregate);
        let mut documents = self.documents.write().await;
        documents.insert(document_id, document);
        
        // Update metadata cache
        let mut cache = self.metadata_cache.write().await;
        cache.insert(document_id, metadata);
        
        Ok(events.into_iter().next().unwrap())
    }
    
    /// Get document content
    pub async fn get_content(&self, document_id: uuid::Uuid) -> DomainResult<Vec<u8>> {
        let documents = self.documents.read().await;
        let document = documents.get(&document_id)
            .ok_or_else(|| cim_domain::DomainError::generic("Document not found"))?;
        
        let content_address = document.get_component::<crate::aggregate::ContentAddressComponent>()
            .ok_or_else(|| cim_domain::DomainError::generic("Document has no content"))?;
        
        self.content_store.retrieve(&content_address.content_cid).await
    }
    
    /// Check if document content exists
    pub async fn content_exists(&self, document_id: uuid::Uuid) -> DomainResult<bool> {
        let documents = self.documents.read().await;
        match documents.get(&document_id) {
            Some(document) => {
                if let Some(content_address) = document.get_component::<crate::aggregate::ContentAddressComponent>() {
                    self.content_store.exists(&content_address.content_cid).await
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }
    
    /// Get cached metadata
    pub async fn get_cached_metadata(&self, document_id: uuid::Uuid) -> Option<DocumentMetadata> {
        let cache = self.metadata_cache.read().await;
        cache.get(&document_id).cloned()
    }
    
    /// Store a document for testing
    pub async fn store_document(&self, document: Document) {
        let id = document.id().into();
        let mut documents = self.documents.write().await;
        documents.insert(id, document);
    }
}

/// In-memory content store for testing
pub struct InMemoryContentStore {
    storage: Arc<RwLock<HashMap<Cid, Vec<u8>>>>,
}

impl InMemoryContentStore {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl ContentStore for InMemoryContentStore {
    async fn store(&self, content: &[u8]) -> DomainResult<Cid> {
        // For now, create a simple CID - in production this would use proper hashing
        let cid = Cid::default();
        
        let mut storage = self.storage.write().await;
        storage.insert(cid, content.to_vec());
        
        Ok(cid)
    }
    
    async fn retrieve(&self, cid: &Cid) -> DomainResult<Vec<u8>> {
        let storage = self.storage.read().await;
        storage.get(cid)
            .cloned()
            .ok_or_else(|| cim_domain::DomainError::generic("Content not found"))
    }
    
    async fn exists(&self, cid: &Cid) -> DomainResult<bool> {
        let storage = self.storage.read().await;
        Ok(storage.contains_key(cid))
    }
    
    async fn delete(&self, cid: &Cid) -> DomainResult<()> {
        let mut storage = self.storage.write().await;
        storage.remove(cid);
        Ok(())
    }
}

impl Default for InMemoryContentStore {
    fn default() -> Self {
        Self::new()
    }
}