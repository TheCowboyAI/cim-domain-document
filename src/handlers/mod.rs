//! Document handlers

mod command_handler;
mod event_handler;
mod document_content_handler_simple;
mod document_version_handler_simple;
mod document_metadata_handler;

pub use command_handler::{DocumentCommandHandler as DocumentCommandHandlerTrait, DocumentCommandHandlerImpl};
pub use event_handler::{DocumentEventHandler, DocumentEventHandlerImpl};
pub use document_content_handler_simple::*;
pub use document_version_handler_simple::*;
pub use document_metadata_handler::*;

use crate::events::*;

/// Simple command handler for examples
pub struct DocumentCommandHandler;

impl DocumentCommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(&self, _command: impl crate::commands::Command) -> Result<Vec<DocumentDomainEvent>, Box<dyn std::error::Error>> {
        // Mock implementation for examples
        Ok(vec![])
    }
}

impl Default for DocumentCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
