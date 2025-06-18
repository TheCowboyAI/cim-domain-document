//! Document command handler

use cim_domain::{AggregateRepository, CommandEnvelope, CommandAcknowledgment, CommandStatus, DomainResult, CommandId, CorrelationId, DomainError};
use crate::{Document, commands::*, value_objects::{DocumentType, DocumentMetadata}, events::*};
use async_trait::async_trait;
use uuid::Uuid;
use crate::aggregate::DocumentAggregate;
use std::collections::HashMap;

/// Trait for handling document commands
#[async_trait]
pub trait DocumentCommandHandler: Send + Sync {
    /// Handle upload document command
    async fn handle_upload_document(&self, cmd: UploadDocument) -> DomainResult<Vec<DocumentDomainEvent>>;
    
    /// Handle update metadata command
    async fn handle_update_metadata(&self, cmd: UpdateDocumentMetadata) -> DomainResult<Vec<DocumentDomainEvent>>;
    
    /// Handle share document command
    async fn handle_share_document(&self, cmd: ShareDocument) -> DomainResult<Vec<DocumentDomainEvent>>;
    
    /// Handle archive document command
    async fn handle_archive_document(&self, cmd: ArchiveDocument) -> DomainResult<Vec<DocumentDomainEvent>>;
}

/// Implementation of document command handler
pub struct DocumentCommandHandlerImpl<R: AggregateRepository<Document>> {
    repository: R,
}

impl<R: AggregateRepository<Document>> DocumentCommandHandlerImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: AggregateRepository<Document> + Send + Sync> DocumentCommandHandler for DocumentCommandHandlerImpl<R> {
    async fn handle_upload_document(&self, cmd: UploadDocument) -> DomainResult<Vec<DocumentDomainEvent>> {
        // Create new document aggregate
        let document_id = cmd.document_id;
        let mut aggregate = DocumentAggregate::new(document_id);
        
        // Create metadata from the command
        let metadata = DocumentMetadata {
            title: cmd.info.title.clone(),
            description: cmd.info.description.clone(),
            tags: vec![],
            custom: std::collections::HashMap::new(),
            mime_type: Some(cmd.info.mime_type.clone()),
            size_bytes: Some(cmd.info.size_bytes as u64),
            language: cmd.info.language.clone(),
            category: None,
            subcategories: None,
        };
        
        // Process the upload command
        let events = aggregate.upload(
            std::path::PathBuf::from(cmd.info.filename.clone().unwrap_or_default()),
            cmd.content_cid,
            metadata,
            DocumentType::Other("Unknown".to_string()), // Default type
            cmd.uploaded_by.to_string(),
        )?;
        
        // Save aggregate
        self.repository.save(&aggregate.into())
            .map_err(|e| DomainError::InternalError(e))?;
        
        // Convert to domain events
        let domain_events = events.into_iter()
            .map(|e| DocumentDomainEvent::DocumentUploaded(e))
            .collect();
        
        Ok(domain_events)
    }
    
    async fn handle_update_metadata(&self, cmd: UpdateDocumentMetadata) -> DomainResult<Vec<DocumentDomainEvent>> {
        // Load existing aggregate
        let entity_id = cim_domain::EntityId::<crate::aggregate::DocumentMarker>::from_uuid(cmd.document_id);
        let document = self.repository.load(entity_id)
            .map_err(|e| DomainError::InternalError(e))?
            .ok_or_else(|| cim_domain::DomainError::EntityNotFound { 
                entity_type: "Document".to_string(),
                id: cmd.document_id.to_string()
            })?;
        let mut aggregate = DocumentAggregate::from(document);
        
        // Process the update command
        let events = aggregate.update_metadata(cmd.metadata, cmd.updated_by)?;
        
        // Save updated aggregate
        self.repository.save(&aggregate.into())
            .map_err(|e| DomainError::InternalError(e))?;
        
        // Convert to domain events
        let domain_events = events.into_iter()
            .map(|e| DocumentDomainEvent::DocumentMetadataUpdated(e))
            .collect();
        
        Ok(domain_events)
    }
    
    async fn handle_share_document(&self, cmd: ShareDocument) -> DomainResult<Vec<DocumentDomainEvent>> {
        // Load existing aggregate
        let entity_id = cim_domain::EntityId::<crate::aggregate::DocumentMarker>::from_uuid(cmd.document_id);
        let document = self.repository.load(entity_id)
            .map_err(|e| DomainError::InternalError(e))?
            .ok_or_else(|| cim_domain::DomainError::EntityNotFound { 
                entity_type: "Document".to_string(),
                id: cmd.document_id.to_string()
            })?;
        let mut aggregate = DocumentAggregate::from(document);
        
        // Process the share command
        let events = aggregate.share(cmd.shared_with, cmd.permissions, cmd.shared_by)?;
        
        // Save updated aggregate
        self.repository.save(&aggregate.into())
            .map_err(|e| DomainError::InternalError(e))?;
        
        // Convert to domain events
        let domain_events = events.into_iter()
            .map(|e| DocumentDomainEvent::DocumentShared(e))
            .collect();
        
        Ok(domain_events)
    }
    
    async fn handle_archive_document(&self, cmd: ArchiveDocument) -> DomainResult<Vec<DocumentDomainEvent>> {
        // Load existing aggregate
        let entity_id = cim_domain::EntityId::<crate::aggregate::DocumentMarker>::from_uuid(cmd.document_id);
        let document = self.repository.load(entity_id)
            .map_err(|e| DomainError::InternalError(e))?
            .ok_or_else(|| cim_domain::DomainError::EntityNotFound { 
                entity_type: "Document".to_string(),
                id: cmd.document_id.to_string()
            })?;
        let mut aggregate = DocumentAggregate::from(document);
        
        // Process the archive command
        let events = aggregate.archive(cmd.reason, cmd.archived_by.to_string())?;
        
        // Save updated aggregate
        self.repository.save(&aggregate.into())
            .map_err(|e| DomainError::InternalError(e))?;
        
        // Convert to domain events
        let domain_events = events.into_iter()
            .map(|e| DocumentDomainEvent::DocumentArchived(e))
            .collect();
        
        Ok(domain_events)
    }
}
