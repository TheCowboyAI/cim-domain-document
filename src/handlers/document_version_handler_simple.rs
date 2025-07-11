//! Simplified document version management handler
//!
//! This handler manages document version history with a simplified approach.

use crate::{
    events::*,
    value_objects::*,
};
use cim_domain::DomainResult;
use cid::Cid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Handler for document version operations
pub struct DocumentVersionHandler {
    /// Version history storage
    version_history: Arc<RwLock<HashMap<uuid::Uuid, Vec<VersionHistoryEntry>>>>,
}

/// Version history entry
#[derive(Debug, Clone)]
pub struct VersionHistoryEntry {
    pub version_number: String,
    pub content_cid: Cid,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub change_summary: Option<String>,
    pub tags: Vec<String>,
}

impl DocumentVersionHandler {
    /// Create a new document version handler
    pub fn new() -> Self {
        Self {
            version_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Record a new version
    pub async fn record_version(
        &self,
        document_id: uuid::Uuid,
        version_number: String,
        content_cid: Cid,
        change_summary: String,
        created_by: String,
    ) -> DomainResult<DocumentVersionCreated> {
        // Add to version history
        let history_entry = VersionHistoryEntry {
            version_number: version_number.clone(),
            content_cid,
            created_at: Utc::now(),
            created_by: created_by.clone(),
            change_summary: Some(change_summary.clone()),
            tags: vec![],
        };
        
        let mut history = self.version_history.write().await;
        let entries = history.entry(document_id).or_insert_with(Vec::new);
        
        // Get previous version
        let previous_version = entries.last()
            .map(|e| e.version_number.clone())
            .unwrap_or_else(|| "0.0".to_string());
        
        entries.push(history_entry);
        
        // Create event
        let event = DocumentVersionCreated {
            document_id: DocumentId::from(document_id),
            version_number,
            content_cid,
            previous_version,
            change_summary,
            created_by,
            created_at: Utc::now(),
        };
        
        Ok(event)
    }
    
    /// Tag a specific version
    pub async fn tag_version(
        &self,
        document_id: uuid::Uuid,
        version_number: String,
        tag: String,
    ) -> DomainResult<()> {
        let mut history = self.version_history.write().await;
        
        if let Some(versions) = history.get_mut(&document_id) {
            if let Some(version) = versions.iter_mut().find(|v| v.version_number == version_number) {
                if !version.tags.contains(&tag) {
                    version.tags.push(tag);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get version history for a document
    pub async fn get_version_history(&self, document_id: uuid::Uuid) -> DomainResult<Vec<VersionHistoryEntry>> {
        let history = self.version_history.read().await;
        Ok(history.get(&document_id).cloned().unwrap_or_default())
    }
    
    /// Calculate next version number
    pub fn calculate_next_version(&self, current_version: &str, version_type: &VersionType) -> String {
        let parts: Vec<&str> = current_version.split('.').collect();
        let major = parts.get(0).unwrap_or(&"1").parse::<u32>().unwrap_or(1);
        let minor = parts.get(1).unwrap_or(&"0").parse::<u32>().unwrap_or(0);
        
        match version_type {
            VersionType::Major => format!("{}.0", major + 1),
            VersionType::Minor => format!("{}.{}", major, minor + 1),
            VersionType::Patch => {
                let patch = parts.get(2).unwrap_or(&"0").parse::<u32>().unwrap_or(0);
                format!("{}.{}.{}", major, minor, patch + 1)
            }
        }
    }
}

impl Default for DocumentVersionHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Version type for version number calculation
#[derive(Debug, Clone, PartialEq)]
pub enum VersionType {
    Major,
    Minor,
    Patch,
}