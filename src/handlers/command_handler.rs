//! Document command handler

use cim_core_domain::command::{CommandHandler, CommandEnvelope};
use cim_core_domain::repository::AggregateRepository;
use crate::Document;

pub struct DocumentCommandHandler<R: AggregateRepository<Document>> {
    repository: R,
}

impl<R: AggregateRepository<Document>> DocumentCommandHandler<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

// Command handler implementations will be added by complete script
