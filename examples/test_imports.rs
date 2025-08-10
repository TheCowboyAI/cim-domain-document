/// Simple test to verify CID-first ingestion types are accessible

fn main() {
    println!("Testing imports...");
    
    // Test basic types
    let _user_id = cim_domain_document::UserId::new();
    let _document_id = cim_domain_document::DocumentId::new();
    
    println!("✅ Basic value objects accessible");
    
    // Test commands are accessible through glob export
    use cim_domain_document::*;
    
    let _cmd = IngestDocumentContent {
        content: b"test".to_vec(),
        suggested_filename: None,
        content_type_hint: None,
        target_partition: DocumentPartitions::staging(),
        enable_virus_scanning: true,
        enable_format_validation: true,
        enable_auto_promotion: false,
        uploaded_by: UserId::new(),
        correlation_id: None,
    };
    
    println!("✅ Ingestion command accessible");
    
    // Test services
    use cim_domain_document::ObjectStoreService;
    
    println!("✅ Object store service trait accessible");
    println!("🎉 All CID-first ingestion components are accessible!");
}