//! Document aggregate for command processing

use super::{Document, DocumentMarker};
use crate::events::*;
use crate::value_objects::*;
use cim_domain::{DomainResult, DomainError, EntityId, AggregateRoot};
use cid::Cid;
use std::collections::HashSet;
use uuid::Uuid;

/// Wrapper aggregate for command processing
pub struct DocumentAggregate {
    document: Document,
}

impl DocumentAggregate {
    /// Create a new document aggregate
    pub fn new(id: Uuid) -> Self {
        let entity_id = EntityId::<DocumentMarker>::from_uuid(id);
        let info = super::DocumentInfoComponent {
            title: String::new(),
            description: None,
            mime_type: "application/octet-stream".to_string(),
            filename: None,
            size_bytes: 0,
            language: None,
        };
        
        // Create a placeholder CID for now
        let placeholder_cid = Cid::default();
        
        Self {
            document: Document::new(entity_id, info, placeholder_cid),
        }
    }
    
    /// Create from existing document
    pub fn from(document: Document) -> Self {
        Self {
            document,
        }
    }
    
    /// Upload a new document
    pub fn upload(
        &mut self,
        path: std::path::PathBuf,
        content_cid: Cid,
        metadata: DocumentMetadata,
        document_type: DocumentType,
        uploaded_by: String,
    ) -> DomainResult<Vec<DocumentUploaded>> {
        // Update document info
        let info = super::DocumentInfoComponent {
            title: metadata.title.clone(),
            description: metadata.description.clone(),
            mime_type: metadata.mime_type.clone().unwrap_or("application/octet-stream".to_string()),
            filename: Some(path.file_name().unwrap_or_default().to_string_lossy().to_string()),
            size_bytes: metadata.size_bytes.unwrap_or(0),
            language: metadata.language.clone(),
        };
        
        // Update content address
        let content_address = super::ContentAddressComponent {
            content_cid,
            metadata_cid: None,
            hash_algorithm: "sha2-256".to_string(),
            encoding: "raw".to_string(),
            is_chunked: false,
            chunk_cids: vec![],
        };
        
        // Add components
        self.document.add_component(info, &uploaded_by, Some("Document upload".to_string()))?;
        self.document.add_component(content_address, &uploaded_by, Some("Content address".to_string()))?;
        
        // Create classification
        let classification = super::ClassificationComponent {
            document_type: format!("{document_type:?}"),
            category: metadata.category.clone().unwrap_or_default(),
            subcategories: metadata.subcategories.clone().unwrap_or_default(),
            tags: metadata.tags.clone(),
            confidentiality: super::ConfidentialityLevel::Internal,
        };
        self.document.add_component(classification, &uploaded_by, Some("Initial classification".to_string()))?;
        
        // Create lifecycle
        let lifecycle = super::LifecycleComponent {
            status: super::DocumentStatus::Published,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            version_number: "1.0".to_string(),
            previous_version_cid: None,
            expires_at: None,
            retention_policy: None,
        };
        self.document.add_component(lifecycle, &uploaded_by, Some("Initial lifecycle".to_string()))?;
        
        // Create event
        let event = DocumentUploaded {
            document_id: self.document.id().into(),
            path,
            content_cid,
            metadata,
            document_type,
            uploaded_by: uploaded_by.clone(),
            uploaded_at: chrono::Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    /// Update document metadata
    pub fn update_metadata(
        &mut self,
        metadata: DocumentMetadata,
        updated_by: String,
    ) -> DomainResult<Vec<DocumentMetadataUpdated>> {
        // Get current info
        let current_info = self.document.get_component::<super::DocumentInfoComponent>()
            .ok_or_else(|| DomainError::generic("Document info not found"))?;
        
        // Create updated info
        let updated_info = super::DocumentInfoComponent {
            title: metadata.title.clone(),
            description: metadata.description.clone(),
            mime_type: metadata.mime_type.clone().unwrap_or(current_info.mime_type.clone()),
            filename: current_info.filename.clone(),
            size_bytes: metadata.size_bytes.unwrap_or(current_info.size_bytes),
            language: metadata.language.clone(),
        };
        
        // Update component
        self.document.remove_component::<super::DocumentInfoComponent>()?;
        self.document.add_component(updated_info, &updated_by, Some("Metadata update".to_string()))?;
        
        // Update lifecycle
        if let Some(lifecycle) = self.document.get_component::<super::LifecycleComponent>() {
            let mut updated_lifecycle = lifecycle.clone();
            updated_lifecycle.modified_at = chrono::Utc::now();
            self.document.remove_component::<super::LifecycleComponent>()?;
            self.document.add_component(updated_lifecycle, &updated_by, Some("Update timestamp".to_string()))?;
        }
        
        // Create event
        let event = DocumentMetadataUpdated {
            document_id: self.document.id().into(),
            metadata,
            updated_by: updated_by.clone(),
            updated_at: chrono::Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    /// Share document with users
    pub fn share(
        &mut self,
        shared_with: HashSet<String>,
        permissions: Vec<String>,
        shared_by: String,
    ) -> DomainResult<Vec<DocumentShared>> {
        // Get or create access control
        let access_control = if let Some(ac) = self.document.get_component::<super::AccessControlComponent>() {
            ac.clone()
        } else {
            super::AccessControlComponent {
                read_access: vec![],
                write_access: vec![],
                share_access: vec![],
                audit_access: false,
                encryption_key_id: None,
            }
        };
        
        // Update access control based on permissions
        let mut updated_ac = access_control;
        for user in &shared_with {
            let user_id = Uuid::parse_str(user).unwrap_or_else(|_| Uuid::new_v4());
            
            if permissions.contains(&"read".to_string()) && !updated_ac.read_access.contains(&user_id) {
                updated_ac.read_access.push(user_id);
            }
            if permissions.contains(&"write".to_string()) && !updated_ac.write_access.contains(&user_id) {
                updated_ac.write_access.push(user_id);
            }
            if permissions.contains(&"share".to_string()) && !updated_ac.share_access.contains(&user_id) {
                updated_ac.share_access.push(user_id);
            }
        }
        
        // Update component
        self.document.remove_component::<super::AccessControlComponent>().ok();
        self.document.add_component(updated_ac, &shared_by, Some("Share document".to_string()))?;
        
        // Create event
        let event = DocumentShared {
            document_id: self.document.id().into(),
            shared_with,
            permissions,
            shared_by: shared_by.clone(),
            shared_at: chrono::Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    /// Archive document
    pub fn archive(
        &mut self,
        reason: String,
        archived_by: String,
    ) -> DomainResult<Vec<DocumentArchived>> {
        // Update lifecycle status
        let lifecycle = self.document.get_component::<super::LifecycleComponent>()
            .ok_or_else(|| DomainError::generic("Lifecycle component not found"))?;
        
        let mut updated_lifecycle = lifecycle.clone();
        updated_lifecycle.status = super::DocumentStatus::Archived;
        updated_lifecycle.modified_at = chrono::Utc::now();
        
        self.document.remove_component::<super::LifecycleComponent>()?;
        self.document.add_component(updated_lifecycle, &archived_by, Some("Archive document".to_string()))?;
        
        // Create event
        let event = DocumentArchived {
            document_id: self.document.id().into(),
            reason,
            archived_by: archived_by.clone(),
            archived_at: chrono::Utc::now(),
        };
        
        Ok(vec![event])
    }
}

impl From<DocumentAggregate> for Document {
    fn from(aggregate: DocumentAggregate) -> Self {
        aggregate.document
    }
} 