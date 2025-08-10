//! Document Workflow Example
//!
//! This example demonstrates:
//! - Creating a document using available commands
//! - Working with document value objects
//! - Basic document operations

use cim_domain_document::{
    commands::CreateDocument,
    handlers::DocumentCommandHandler,
    value_objects::{DocumentId, DocumentType},
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CIM Document Domain Example ===\n");

    // Initialize handlers
    let command_handler = DocumentCommandHandler::new();

    let document_id = DocumentId::new();
    let author_id = Uuid::new_v4();

    // Step 1: Create a new document
    println!("1. Creating document...");
    let create_command = CreateDocument {
        document_id: document_id.clone(),
        document_type: DocumentType::Proposal,
        title: "Project Proposal: AI-Powered Workflow System".to_string(),
        author_id,
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("department".to_string(), "Engineering".to_string());
            meta.insert("priority".to_string(), "High".to_string());
            meta
        },
    };

    match command_handler.handle(create_command).await {
        Ok(events) => {
            println!("   Document created successfully!");
            println!("   Generated {} events\n", events.len());
        }
        Err(e) => {
            println!("   Failed to create document: {:?}\n", e);
            return Ok(()); // Continue example even if this fails
        }
    }

    // Step 2: Show document value objects
    println!("2. Working with document value objects...");
    println!("   Document ID: {}", document_id);
    println!("   Document Type: {:?}", DocumentType::Proposal);
    println!("   Author ID: {}\n", author_id);

    // Step 3: Demonstrate different document types
    println!("3. Available document types:");
    let doc_types = vec![
        DocumentType::Text,
        DocumentType::Article,
        DocumentType::Report,
        DocumentType::Proposal,
        DocumentType::Contract,
        DocumentType::Pdf,
        DocumentType::Note,
        DocumentType::Other("Custom Type".to_string()),
    ];
    
    for doc_type in &doc_types {
        println!("   - {:?}", doc_type);
    }
    
    println!("\n=== Document Domain Example Complete ===");
    Ok(())
}