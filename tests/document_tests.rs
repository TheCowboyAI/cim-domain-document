use cid::Cid;
use cim_domain::{AggregateRoot, EntityId};
use cim_domain_document::*;

#[test]
fn test_document_creation() {
    let document_id = EntityId::<DocumentMarker>::new();
    let info = DocumentInfoComponent {
        title: "Test Document".to_string(),
        description: Some("A test document".to_string()),
        mime_type: "text/plain".to_string(),
        filename: Some("test.txt".to_string()),
        size_bytes: 1024,
        language: Some("en".to_string()),
    };

    let content_cid =
        Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();

    let document = Document::new(document_id, info.clone(), content_cid);

    assert_eq!(document.id(), document_id);
    assert_eq!(document.version(), 0);
    assert_eq!(
        document
            .get_component::<DocumentInfoComponent>()
            .unwrap()
            .title,
        "Test Document"
    );
}
