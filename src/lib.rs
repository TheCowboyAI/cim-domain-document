//! Document domain module for CIM (Composable Information Machine)
//!
//! This module contains the document aggregate and related components for managing
//! business documents within the Composable Information Machine architecture.
//! 
//! A Document is an abstract type identifying a structure meant for reading,
//! with associated processing requirements and content-addressed storage using CIDs.

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;
pub mod services;
pub mod workflow;
pub mod nats;

// Re-export main types
pub use aggregate::{
    Document, DocumentMarker,
    DocumentInfoComponent, ContentAddressComponent, ClassificationComponent,
    OwnershipComponent, LifecycleComponent, AccessControlComponent,
    RelationshipsComponent, ProcessingComponent,
    ConfidentialityLevel, DocumentStatus, RelationType,
    DocumentRelation, ExternalReference, ThumbnailInfo,
    PublicDocumentView, SearchIndexProjection,
};

pub use commands::*;
pub use events::*;
pub use value_objects::*;
pub use services::*;
pub use handlers::{DocumentCommandHandler, DocumentEventHandler};
pub use projections::DocumentView;
pub use queries::{SearchDocuments, GetDocument, GetDocumentHistory, DocumentQueryHandler, DocumentView as DocumentQueryView, DocumentHistoryView};
