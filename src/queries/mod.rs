//! Document queries

use cim_core_domain::query::Query;
use serde::{Deserialize, Serialize};

/// Query to search documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocuments {
    /// Text search query
    pub query: String,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Filter by MIME types
    pub mime_types: Vec<String>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

impl Query for SearchDocuments {}

/// Document query handler
pub struct DocumentQueryHandler;

// Query handler implementations will be added by complete script
