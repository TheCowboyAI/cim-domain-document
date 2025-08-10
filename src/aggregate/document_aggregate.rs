//! Document aggregate for command processing

use super::{Document, DocumentMarker};
use crate::events::*;
use crate::value_objects::*;
use crate::{
    DocumentInfoComponent, ContentAddressComponent, ClassificationComponent,
    LifecycleComponent, AccessControlComponent, DocumentStatus, ConfidentialityLevel
};
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
        let info = DocumentInfoComponent {
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
        let info = DocumentInfoComponent {
            title: metadata.title.clone(),
            description: metadata.description.clone(),
            mime_type: metadata.mime_type.clone().unwrap_or("application/octet-stream".to_string()),
            filename: Some(path.file_name().unwrap_or_default().to_string_lossy().to_string()),
            size_bytes: metadata.size_bytes.unwrap_or(0),
            language: metadata.language.clone(),
        };
        
        // Update content address
        let content_address = ContentAddressComponent {
            content_cid,
            metadata_cid: None,
            hash_algorithm: "sha2-256".to_string(),
            encoding: "raw".to_string(),
            is_chunked: false,
            chunk_cids: vec![],
        };
        
        // Update or add document info component
        self.document.remove_component::<DocumentInfoComponent>().ok(); // Remove existing if present
        self.document.add_component(info, &uploaded_by, Some("Document upload".to_string()))?;
        
        // Update or add content address component  
        self.document.remove_component::<ContentAddressComponent>().ok(); // Remove existing if present
        self.document.add_component(content_address, &uploaded_by, Some("Content address".to_string()))?;
        
        // Create classification
        let classification = ClassificationComponent {
            document_type: format!("{document_type:?}"),
            category: metadata.category.clone().unwrap_or_default(),
            subcategories: metadata.subcategories.clone().unwrap_or_default(),
            tags: metadata.tags.clone(),
            confidentiality: ConfidentialityLevel::Internal,
        };
        self.document.add_component(classification, &uploaded_by, Some("Initial classification".to_string()))?;
        
        // Create lifecycle
        let lifecycle = LifecycleComponent {
            status: DocumentStatus::Published,
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
        let current_info = self.document.get_component::<DocumentInfoComponent>()
            .ok_or_else(|| DomainError::generic("Document info not found"))?;
        
        // Create updated info
        let updated_info = DocumentInfoComponent {
            title: metadata.title.clone(),
            description: metadata.description.clone(),
            mime_type: metadata.mime_type.clone().unwrap_or(current_info.mime_type.clone()),
            filename: current_info.filename.clone(),
            size_bytes: metadata.size_bytes.unwrap_or(current_info.size_bytes),
            language: metadata.language.clone(),
        };
        
        // Update component
        self.document.remove_component::<DocumentInfoComponent>()?;
        self.document.add_component(updated_info, &updated_by, Some("Metadata update".to_string()))?;
        
        // Update lifecycle
        if let Some(lifecycle) = self.document.get_component::<LifecycleComponent>() {
            let mut updated_lifecycle = lifecycle.clone();
            updated_lifecycle.modified_at = chrono::Utc::now();
            self.document.remove_component::<LifecycleComponent>()?;
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
        let access_control = if let Some(ac) = self.document.get_component::<AccessControlComponent>() {
            ac.clone()
        } else {
            AccessControlComponent {
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
        self.document.remove_component::<AccessControlComponent>().ok();
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
        let lifecycle = self.document.get_component::<LifecycleComponent>()
            .ok_or_else(|| DomainError::generic("Lifecycle component not found"))?;
        
        let mut updated_lifecycle = lifecycle.clone();
        updated_lifecycle.status = DocumentStatus::Archived;
        updated_lifecycle.modified_at = chrono::Utc::now();
        
        self.document.remove_component::<LifecycleComponent>()?;
        self.document.add_component(updated_lifecycle, &archived_by, Some("Archive document".to_string()))?;
        
        // Parse archived_by as UUID
        let archived_by_uuid = Uuid::parse_str(&archived_by).unwrap_or_else(|_| Uuid::new_v4());
        
        // Create event
        let event = DocumentArchived {
            document_id: self.document.id().into(),
            reason,
            archived_by: archived_by_uuid,
            archived_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        Ok(vec![event])
    }
    
    /// Apply document successor to update CID chain
    pub fn apply_successor(&mut self, successor: crate::value_objects::DocumentSuccessor) -> DomainResult<()> {
        // Update content address with new CID
        if let Some(mut content_address) = self.document.get_component::<ContentAddressComponent>() {
            content_address.content_cid = successor.new_cid;
            self.document.remove_component::<ContentAddressComponent>().ok();
            self.document.add_component(content_address, &successor.edited_by.to_string(), Some("CID chain update".to_string()))?;
        }
        
        // Update lifecycle timestamp
        if let Some(mut lifecycle) = self.document.get_component::<LifecycleComponent>() {
            lifecycle.modified_at = chrono::Utc::now();
            self.document.remove_component::<LifecycleComponent>().ok();
            self.document.add_component(lifecycle, &successor.edited_by.to_string(), Some("Successor applied".to_string()))?;
        }
        
        Ok(())
    }
}

impl From<DocumentAggregate> for Document {
    fn from(aggregate: DocumentAggregate) -> Self {
        aggregate.document
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cid::Cid;
    use std::collections::HashMap;
    
    // Import all needed components
    use crate::{
        DocumentInfoComponent, ContentAddressComponent, ClassificationComponent,
        LifecycleComponent, AccessControlComponent, DocumentStatus, ConfidentialityLevel
    };

    // Test helper to create sample metadata
    fn create_test_metadata() -> DocumentMetadata {
        DocumentMetadata {
            title: "Test Document".to_string(),
            description: Some("A test document".to_string()),
            tags: vec!["test".to_string(), "sample".to_string()],
            custom_attributes: HashMap::new(),
            mime_type: Some("text/plain".to_string()),
            size_bytes: Some(1024),
            language: Some("en".to_string()),
            category: Some("test".to_string()),
            subcategories: Some(vec!["unit-test".to_string()]),
            filename: Some("test.txt".to_string()),
        }
    }

    // Test helper to create test CID
    fn create_test_cid() -> Cid {
        Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap()
    }

    #[test]
    fn test_new_document_aggregate_creation() {
        // US-001: Test document aggregate creation
        let id = Uuid::new_v4();
        let aggregate = DocumentAggregate::new(id);
        
        // Verify the aggregate was created with proper defaults
        let document = &aggregate.document;
        assert_eq!(document.version(), 0);
        
        // Verify initial document info component
        let info = document.get_component::<DocumentInfoComponent>().unwrap();
        assert_eq!(info.title, "");
        assert_eq!(info.mime_type, "application/octet-stream");
        assert_eq!(info.size_bytes, 0);
        assert!(info.description.is_none());
        assert!(info.filename.is_none());
        assert!(info.language.is_none());
    }

    #[test]
    fn test_from_existing_document() {
        // US-001: Test creating aggregate from existing document
        let entity_id = EntityId::<DocumentMarker>::new();
        let info = DocumentInfoComponent {
            title: "Existing Document".to_string(),
            description: Some("An existing document".to_string()),
            mime_type: "text/plain".to_string(),
            filename: Some("existing.txt".to_string()),
            size_bytes: 512,
            language: Some("en".to_string()),
        };
        let cid = create_test_cid();
        let document = Document::new(entity_id, info.clone(), cid);
        
        let aggregate = DocumentAggregate::from(document);
        
        // Verify the aggregate preserves the original document
        let doc_info = aggregate.document.get_component::<DocumentInfoComponent>().unwrap();
        assert_eq!(doc_info.title, "Existing Document");
        assert_eq!(doc_info.size_bytes, 512);
        assert_eq!(doc_info.filename, Some("existing.txt".to_string()));
    }

    #[test]
    fn test_document_upload() {
        // US-002: Test document upload functionality
        let mut aggregate = DocumentAggregate::new(Uuid::new_v4());
        let metadata = create_test_metadata();
        let content_cid = create_test_cid();
        let path = std::path::PathBuf::from("/test/document.txt");
        let uploaded_by = "user123".to_string();

        let result = aggregate.upload(
            path.clone(),
            content_cid,
            metadata.clone(),
            DocumentType::Text,
            uploaded_by.clone(),
        );

        // Verify upload succeeded
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        // Verify event content
        let event = &events[0];
        assert_eq!(event.document_id, aggregate.document.id().into());
        assert_eq!(event.path, path);
        assert_eq!(event.content_cid, content_cid);
        assert_eq!(event.uploaded_by, uploaded_by);
        assert_eq!(event.document_type, DocumentType::Text);

        // Verify document components were updated
        let doc_info = aggregate.document.get_component::<DocumentInfoComponent>().unwrap();
        assert_eq!(doc_info.title, "Test Document");
        assert_eq!(doc_info.size_bytes, 1024);
        assert_eq!(doc_info.language, Some("en".to_string()));

        let content_addr = aggregate.document.get_component::<ContentAddressComponent>().unwrap();
        assert_eq!(content_addr.content_cid, content_cid);
        assert_eq!(content_addr.hash_algorithm, "sha2-256");

        let classification = aggregate.document.get_component::<ClassificationComponent>().unwrap();
        assert_eq!(classification.document_type, "Text");
        assert_eq!(classification.tags, vec!["test".to_string(), "sample".to_string()]);

        let lifecycle = aggregate.document.get_component::<LifecycleComponent>().unwrap();
        assert_eq!(lifecycle.status, DocumentStatus::Published);
        assert_eq!(lifecycle.version_number, "1.0");
    }

    #[test]
    fn test_metadata_update() {
        // US-002: Test document metadata updates
        let mut aggregate = DocumentAggregate::new(Uuid::new_v4());
        
        // First upload a document
        let initial_metadata = create_test_metadata();
        let content_cid = create_test_cid();
        let path = std::path::PathBuf::from("/test/document.txt");
        aggregate.upload(path, content_cid, initial_metadata, DocumentType::Text, "user123".to_string()).unwrap();

        // Now update metadata
        let mut updated_metadata = create_test_metadata();
        updated_metadata.title = "Updated Test Document".to_string();
        updated_metadata.description = Some("Updated description".to_string());
        updated_metadata.size_bytes = Some(2048);

        let result = aggregate.update_metadata(updated_metadata.clone(), "user456".to_string());

        // Verify update succeeded
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        // Verify event content
        let event = &events[0];
        assert_eq!(event.updated_by, "user456");
        assert_eq!(event.metadata.title, "Updated Test Document");

        // Verify document info was updated
        let doc_info = aggregate.document.get_component::<DocumentInfoComponent>().unwrap();
        assert_eq!(doc_info.title, "Updated Test Document");
        assert_eq!(doc_info.description, Some("Updated description".to_string()));
        assert_eq!(doc_info.size_bytes, 2048);
    }

    #[test]
    fn test_metadata_update_without_initial_info() {
        // US-007: Test edge case - update metadata on document without info component
        let mut aggregate = DocumentAggregate::new(Uuid::new_v4());
        
        // Remove the initial document info component
        aggregate.document.remove_component::<DocumentInfoComponent>().ok();
        
        let metadata = create_test_metadata();
        let result = aggregate.update_metadata(metadata, "user123".to_string());

        // Verify it fails gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_document_sharing() {
        // US-004: Test document sharing functionality
        let mut aggregate = DocumentAggregate::new(Uuid::new_v4());
        
        // First upload a document
        let metadata = create_test_metadata();
        let content_cid = create_test_cid();
        let path = std::path::PathBuf::from("/test/document.txt");
        aggregate.upload(path, content_cid, metadata, DocumentType::Text, "owner123".to_string()).unwrap();

        // Share with users
        let mut shared_with = HashSet::new();
        shared_with.insert("user1".to_string());
        shared_with.insert("user2".to_string());
        let permissions = vec!["read".to_string(), "write".to_string()];

        let result = aggregate.share(shared_with.clone(), permissions.clone(), "owner123".to_string());

        // Verify sharing succeeded
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        // Verify event content
        let event = &events[0];
        assert_eq!(event.shared_with, shared_with);
        assert_eq!(event.permissions, permissions);
        assert_eq!(event.shared_by, "owner123");

        // Verify access control component was added
        let access_control = aggregate.document.get_component::<AccessControlComponent>().unwrap();
        assert_eq!(access_control.read_access.len(), 2); // Should have 2 users with read access
        assert_eq!(access_control.write_access.len(), 2); // Should have 2 users with write access
    }

    #[test]
    fn test_document_sharing_with_existing_access_control() {
        // US-004: Test sharing when access control already exists
        let mut aggregate = DocumentAggregate::new(Uuid::new_v4());
        
        // Upload document
        let metadata = create_test_metadata();
        let content_cid = create_test_cid();
        let path = std::path::PathBuf::from("/test/document.txt");
        aggregate.upload(path, content_cid, metadata, DocumentType::Text, "owner123".to_string()).unwrap();

        // First share
        let mut first_share = HashSet::new();
        first_share.insert("user1".to_string());
        aggregate.share(first_share, vec!["read".to_string()], "owner123".to_string()).unwrap();

        // Second share with additional users
        let mut second_share = HashSet::new();
        second_share.insert("user2".to_string());
        let result = aggregate.share(second_share, vec!["write".to_string()], "owner123".to_string());

        assert!(result.is_ok());

        // Verify access control has been updated
        let access_control = aggregate.document.get_component::<AccessControlComponent>().unwrap();
        assert!(!access_control.read_access.is_empty());
        assert!(!access_control.write_access.is_empty());
    }

    #[test]
    fn test_document_archiving() {
        // US-003: Test document archiving with state transition
        let mut aggregate = DocumentAggregate::new(Uuid::new_v4());
        
        // First upload a document
        let metadata = create_test_metadata();
        let content_cid = create_test_cid();
        let path = std::path::PathBuf::from("/test/document.txt");
        aggregate.upload(path, content_cid, metadata, DocumentType::Text, "user123".to_string()).unwrap();

        // Archive the document
        let reason = "End of project lifecycle".to_string();
        let archived_by = "admin456".to_string();

        let result = aggregate.archive(reason.clone(), archived_by.clone());

        // Verify archiving succeeded
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        // Verify event content
        let event = &events[0];
        assert_eq!(event.reason, reason);
        assert_eq!(event.document_id, aggregate.document.id().into());

        // Verify lifecycle status was updated
        let lifecycle = aggregate.document.get_component::<LifecycleComponent>().unwrap();
        assert_eq!(lifecycle.status, DocumentStatus::Archived);
    }

    #[test]
    fn test_archiving_without_lifecycle() {
        // US-007: Test edge case - archive document without lifecycle component
        let mut aggregate = DocumentAggregate::new(Uuid::new_v4());
        
        let result = aggregate.archive("Test reason".to_string(), "user123".to_string());

        // Should fail because lifecycle component doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_uuid_handling_in_sharing() {
        // US-007: Test edge case - invalid user ID in sharing
        let mut aggregate = DocumentAggregate::new(Uuid::new_v4());
        
        // Upload document first
        let metadata = create_test_metadata();
        let content_cid = create_test_cid();
        let path = std::path::PathBuf::from("/test/document.txt");
        aggregate.upload(path, content_cid, metadata, DocumentType::Text, "user123".to_string()).unwrap();

        // Share with invalid user ID (should generate new UUID)
        let mut shared_with = HashSet::new();
        shared_with.insert("invalid-uuid".to_string());
        let permissions = vec!["read".to_string()];

        let result = aggregate.share(shared_with, permissions, "owner123".to_string());

        // Should succeed by generating a new UUID
        assert!(result.is_ok());
        
        // Verify access control was still created
        let access_control = aggregate.document.get_component::<AccessControlComponent>().unwrap();
        assert_eq!(access_control.read_access.len(), 1);
    }

    #[test]
    fn test_aggregate_to_document_conversion() {
        // US-001: Test conversion from aggregate to document
        let id = Uuid::new_v4();
        let aggregate = DocumentAggregate::new(id);
        let original_version = aggregate.document.version();

        let document: Document = aggregate.into();
        
        // Verify the document maintains its state
        assert_eq!(document.version(), original_version);
        let info = document.get_component::<DocumentInfoComponent>().unwrap();
        assert_eq!(info.mime_type, "application/octet-stream");
    }

    #[test]
    fn test_upload_with_minimal_metadata() {
        // US-007: Test upload with minimal required metadata
        let mut aggregate = DocumentAggregate::new(Uuid::new_v4());
        
        let minimal_metadata = DocumentMetadata {
            title: "Minimal".to_string(),
            description: None,
            tags: vec![],
            custom_attributes: HashMap::new(),
            mime_type: None,
            size_bytes: None,
            language: None,
            category: None,
            subcategories: None,
            filename: None,
        };

        let result = aggregate.upload(
            std::path::PathBuf::from("/test/minimal.txt"),
            create_test_cid(),
            minimal_metadata,
            DocumentType::Text,
            "user123".to_string(),
        );

        // Should succeed with defaults
        assert!(result.is_ok());
        
        let doc_info = aggregate.document.get_component::<DocumentInfoComponent>().unwrap();
        assert_eq!(doc_info.title, "Minimal");
        assert_eq!(doc_info.mime_type, "application/octet-stream"); // Default fallback
        assert_eq!(doc_info.size_bytes, 0); // Default fallback
    }
} 