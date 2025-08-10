//! Document import/export service

use crate::value_objects::{DocumentId, DocumentType, ImportOptions, ExportOptions, ImportFormat, ExportFormat}; // DocumentId used in tests
use crate::projections::DocumentFullView;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Import/Export service for documents
pub struct ImportExportService;

impl ImportExportService {
    /// Import document from external format
    pub fn import_document(
        content: &[u8],
        format: &ImportFormat,
        options: &ImportOptions,
    ) -> Result<ImportedDocument> {
        match format {
            ImportFormat::Markdown => Self::import_markdown(content, options),
            ImportFormat::PlainText => Self::import_plain_text(content, options),
            ImportFormat::Html => Self::import_html(content, options),
            ImportFormat::Json => Self::import_json(content, options),
            ImportFormat::Pdf => Err(anyhow!("PDF import not yet implemented")),
            ImportFormat::Word => Err(anyhow!("Word import not yet implemented")),
            ImportFormat::Custom(fmt) => Err(anyhow!("Custom format '{}' not supported", fmt)),
        }
    }

    /// Export document to external format
    pub fn export_document(
        document: &DocumentFullView,
        format: &ExportFormat,
        options: &ExportOptions,
    ) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Markdown => Self::export_markdown(document, options),
            ExportFormat::PlainText => Self::export_plain_text(document, options),
            ExportFormat::Html => Self::export_html(document, options),
            ExportFormat::Json => Self::export_json(document, options),
            ExportFormat::Pdf => Err(anyhow!("PDF export not yet implemented")),
            ExportFormat::Word => Err(anyhow!("Word export not yet implemented")),
            ExportFormat::Custom(fmt) => Err(anyhow!("Custom format '{}' not supported", fmt)),
        }
    }

    // Import implementations

    fn import_markdown(content: &[u8], _options: &ImportOptions) -> Result<ImportedDocument> {
        let text = String::from_utf8(content.to_vec())
            .map_err(|_| anyhow!("Invalid UTF-8 in markdown content"))?;

        let mut metadata = HashMap::new();
        let mut title = "Untitled".to_string();

        // Simple markdown parsing
        let lines: Vec<&str> = text.lines().collect();
        let mut content_start = 0;

        // Check for frontmatter
        if lines.first() == Some(&"---") {
            for (i, line) in lines.iter().enumerate().skip(1) {
                if *line == "---" {
                    content_start = i + 1;
                    break;
                }
                // Parse frontmatter
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim().to_string();
                    let value = value.trim().to_string();
                    if key == "title" {
                        title = value;
                    } else {
                        metadata.insert(key, value);
                    }
                }
            }
        }

        // Extract title from first heading if not in frontmatter
        if title == "Untitled" {
            for line in &lines[content_start..] {
                if line.starts_with("# ") {
                    title = line[2..].trim().to_string();
                    break;
                }
            }
        }

        // Build body
        let body = lines[content_start..].join("\n");

        Ok(ImportedDocument {
            title,
            content: body,
            doc_type: DocumentType::Report,
            metadata,
            tags: vec![],
        })
    }

    fn import_plain_text(content: &[u8], _options: &ImportOptions) -> Result<ImportedDocument> {
        let text = String::from_utf8(content.to_vec())
            .map_err(|_| anyhow!("Invalid UTF-8 in text content"))?;

        // Extract title from first line
        let lines: Vec<&str> = text.lines().collect();
        let title = lines.first()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        let body = if lines.len() > 1 {
            lines[1..].join("\n")
        } else {
            text.clone()
        };

        Ok(ImportedDocument {
            title,
            content: body,
            doc_type: DocumentType::Note,
            metadata: HashMap::new(),
            tags: vec![],
        })
    }

    fn import_html(content: &[u8], _options: &ImportOptions) -> Result<ImportedDocument> {
        let html = String::from_utf8(content.to_vec())
            .map_err(|_| anyhow!("Invalid UTF-8 in HTML content"))?;

        // Very basic HTML extraction (production would use proper parser)
        let title = if let Some(start) = html.find("<title>") {
            if let Some(end) = html.find("</title>") {
                html[start + 7..end].trim().to_string()
            } else {
                "Untitled".to_string()
            }
        } else {
            "Untitled".to_string()
        };

        // Strip HTML tags (basic implementation)
        let content = html
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<br />", "\n")
            .replace("<p>", "\n")
            .replace("</p>", "\n");

        // Remove remaining tags
        let re = regex::Regex::new(r"<[^>]+>").unwrap();
        let content = re.replace_all(&content, "").to_string();

        Ok(ImportedDocument {
            title,
            content,
            doc_type: DocumentType::Article,
            metadata: HashMap::new(),
            tags: vec![],
        })
    }

    fn import_json(content: &[u8], _options: &ImportOptions) -> Result<ImportedDocument> {
        let json: serde_json::Value = serde_json::from_slice(content)?;

        let title = json.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let content = json.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let doc_type = json.get("type")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "note" => Some(DocumentType::Note),
                "article" => Some(DocumentType::Article),
                "report" => Some(DocumentType::Report),
                _ => None,
            })
            .unwrap_or(DocumentType::Note);

        let tags = json.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let metadata = json.get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ImportedDocument {
            title,
            content,
            doc_type,
            metadata,
            tags,
        })
    }

    // Export implementations

    fn export_markdown(document: &DocumentFullView, options: &ExportOptions) -> Result<Vec<u8>> {
        let mut output = String::new();

        // Add frontmatter if metadata included
        if options.include_metadata {
            output.push_str("---\n");
            output.push_str(&format!("title: {}\n", document.title));
            output.push_str(&format!("version: {}\n", document.version));
            output.push_str(&format!("type: {:?}\n", document.doc_type));
            output.push_str(&format!("created: {}\n", document.created_at.format("%Y-%m-%d")));
            output.push_str(&format!("updated: {}\n", document.updated_at.format("%Y-%m-%d")));
            
            if !document.tags.is_empty() {
                output.push_str("tags:\n");
                for tag in &document.tags {
                    output.push_str(&format!("  - {tag}\n"));
                }
            }

            for (key, value) in &document.metadata {
                output.push_str(&format!("{key}: {value}\n"));
            }
            
            output.push_str("---\n\n");
        }

        // Add title
        output.push_str(&format!("# {}\n\n", document.title));

        // Add content
        output.push_str(&document.content);

        // Add watermark if specified
        if let Some(watermark) = &options.watermark {
            output.push_str(&format!("\n\n---\n*{watermark}*"));
        }

        Ok(output.into_bytes())
    }

    fn export_plain_text(document: &DocumentFullView, options: &ExportOptions) -> Result<Vec<u8>> {
        let mut output = String::new();

        output.push_str(&document.title);
        output.push('\n');
        output.push_str(&"=".repeat(document.title.len()));
        output.push_str("\n\n");

        if options.include_metadata {
            output.push_str(&format!("Version: {}\n", document.version));
            output.push_str(&format!("Created: {}\n", document.created_at.format("%Y-%m-%d")));
            output.push_str(&format!("Updated: {}\n\n", document.updated_at.format("%Y-%m-%d")));
        }

        output.push_str(&document.content);

        if let Some(watermark) = &options.watermark {
            output.push_str(&format!("\n\n{watermark}"));
        }

        Ok(output.into_bytes())
    }

    fn export_html(document: &DocumentFullView, options: &ExportOptions) -> Result<Vec<u8>> {
        let mut output = String::new();

        output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        output.push_str(&format!("  <title>{}</title>\n", html_escape(&document.title)));
        output.push_str("  <meta charset=\"UTF-8\">\n");
        
        if options.include_metadata {
            output.push_str(&format!("  <meta name=\"version\" content=\"{}\">\n", document.version));
            output.push_str(&format!("  <meta name=\"created\" content=\"{}\">\n", document.created_at.to_rfc3339()));
            for tag in &document.tags {
                output.push_str(&format!("  <meta name=\"keywords\" content=\"{}\">\n", html_escape(tag)));
            }
        }
        
        output.push_str("</head>\n<body>\n");
        output.push_str(&format!("  <h1>{}</h1>\n", html_escape(&document.title)));
        
        // Convert markdown-style content to basic HTML
        let html_content = document.content
            .replace("\n\n", "</p>\n  <p>")
            .replace("\n", "<br>\n");
        
        output.push_str("  <p>");
        output.push_str(&html_content);
        output.push_str("</p>\n");

        if let Some(watermark) = &options.watermark {
            output.push_str(&format!("  <hr>\n  <p><em>{}</em></p>\n", html_escape(watermark)));
        }

        output.push_str("</body>\n</html>");

        Ok(output.into_bytes())
    }

    fn export_json(document: &DocumentFullView, options: &ExportOptions) -> Result<Vec<u8>> {
        let mut json = serde_json::json!({
            "title": document.title,
            "content": document.content,
            "type": format!("{:?}", document.doc_type).to_lowercase(),
            "version": document.version.to_string(),
        });

        if options.include_metadata {
            json["created_at"] = serde_json::json!(document.created_at.to_rfc3339());
            json["updated_at"] = serde_json::json!(document.updated_at.to_rfc3339());
            json["author"] = serde_json::json!(document.author.to_string());
            json["tags"] = serde_json::json!(document.tags);
            json["metadata"] = serde_json::json!(document.metadata);
        }

        if let Some(watermark) = &options.watermark {
            json["watermark"] = serde_json::json!(watermark);
        }

        serde_json::to_vec_pretty(&json).map_err(Into::into)
    }
}

/// Imported document structure
#[derive(Debug, Clone)]
pub struct ImportedDocument {
    pub title: String,
    pub content: String,
    pub doc_type: DocumentType,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::DocumentVersion;
    use uuid::Uuid;

    // Helper functions for test data
    fn create_test_document() -> DocumentFullView {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "testing".to_string());
        metadata.insert("priority".to_string(), "high".to_string());

        DocumentFullView {
            id: DocumentId::new(),
            title: "Test Document".to_string(),
            content: "This is the main content of the document.\n\nIt has multiple paragraphs.".to_string(),
            version: DocumentVersion::new(1, 2, 3),
            doc_type: DocumentType::Article,
            tags: vec!["test".to_string(), "sample".to_string()],
            author: Uuid::new_v4(),
            metadata,
            created_at: chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z").unwrap().with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339("2023-01-02T00:00:00Z").unwrap().with_timezone(&chrono::Utc),
        }
    }

    fn create_export_options(include_metadata: bool, watermark: Option<String>) -> ExportOptions {
        ExportOptions {
            include_metadata,
            include_history: false,
            include_comments: false,
            watermark,
            custom_options: HashMap::new(),
        }
    }

    // IMPORT TESTS

    #[test]
    fn test_import_markdown_with_frontmatter() {
        // US-018: Test markdown import with YAML frontmatter
        let markdown = r#"---
title: Test Document
author: John Doe
category: example
---

# Test Document

This is a test document with some content."#;

        let imported = ImportExportService::import_document(
            markdown.as_bytes(),
            &ImportFormat::Markdown,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "Test Document");
        assert_eq!(imported.metadata.get("author"), Some(&"John Doe".to_string()));
        assert_eq!(imported.metadata.get("category"), Some(&"example".to_string()));
        assert!(imported.content.contains("This is a test document"));
        assert_eq!(imported.doc_type, DocumentType::Report);
    }

    #[test]
    fn test_import_markdown_without_frontmatter() {
        // US-018: Test markdown import without frontmatter
        let markdown = r#"# Main Title

This is content without frontmatter.

## Section 2

More content here."#;

        let imported = ImportExportService::import_document(
            markdown.as_bytes(),
            &ImportFormat::Markdown,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "Main Title");
        assert!(imported.metadata.is_empty());
        assert!(imported.content.contains("This is content without frontmatter"));
        assert!(imported.content.contains("## Section 2"));
    }

    #[test]
    fn test_import_markdown_no_title_fallback() {
        // US-020: Test markdown import with no title (fallback to "Untitled")
        let markdown = r#"This is just content without any title or heading."#;

        let imported = ImportExportService::import_document(
            markdown.as_bytes(),
            &ImportFormat::Markdown,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "Untitled");
        assert_eq!(imported.content, "This is just content without any title or heading.");
    }

    #[test]
    fn test_import_markdown_invalid_utf8() {
        // US-020: Test markdown import with invalid UTF-8
        let invalid_bytes = vec![0xFF, 0xFE, 0xFD];

        let result = ImportExportService::import_document(
            &invalid_bytes,
            &ImportFormat::Markdown,
            &ImportOptions::default(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid UTF-8"));
    }

    #[test]
    fn test_import_plain_text_with_title() {
        // US-018: Test plain text import with title on first line
        let text = r#"Document Title
This is the body content.
With multiple lines of text."#;

        let imported = ImportExportService::import_document(
            text.as_bytes(),
            &ImportFormat::PlainText,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "Document Title");
        assert_eq!(imported.content, "This is the body content.\nWith multiple lines of text.");
        assert_eq!(imported.doc_type, DocumentType::Note);
        assert!(imported.metadata.is_empty());
        assert!(imported.tags.is_empty());
    }

    #[test]
    fn test_import_plain_text_single_line() {
        // US-020: Test plain text import with single line
        let text = "Single line content";

        let imported = ImportExportService::import_document(
            text.as_bytes(),
            &ImportFormat::PlainText,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "Single line content");
        assert_eq!(imported.content, "Single line content");
    }

    #[test]
    fn test_import_plain_text_empty() {
        // US-020: Test plain text import with empty content
        let text = "";

        let imported = ImportExportService::import_document(
            text.as_bytes(),
            &ImportFormat::PlainText,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "Untitled");
        assert_eq!(imported.content, "");
    }

    #[test]
    fn test_import_html_with_title() {
        // US-018: Test HTML import with title tag
        let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>HTML Document Title</title>
</head>
<body>
    <h1>Main Heading</h1>
    <p>This is a paragraph.</p>
    <br>
    <p>Another paragraph with <strong>bold text</strong>.</p>
</body>
</html>"#;

        let imported = ImportExportService::import_document(
            html.as_bytes(),
            &ImportFormat::Html,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "HTML Document Title");
        assert!(imported.content.contains("Main Heading"));
        assert!(imported.content.contains("This is a paragraph"));
        assert!(imported.content.contains("Another paragraph with bold text"));
        assert_eq!(imported.doc_type, DocumentType::Article);
    }

    #[test]
    fn test_import_html_no_title() {
        // US-020: Test HTML import without title tag
        let html = r#"<html><body><p>Content without title</p></body></html>"#;

        let imported = ImportExportService::import_document(
            html.as_bytes(),
            &ImportFormat::Html,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "Untitled");
        assert!(imported.content.contains("Content without title"));
    }

    #[test]
    fn test_import_html_malformed_title() {
        // US-020: Test HTML import with malformed title tag
        let html = r#"<html><head><title>Incomplete Title"#;

        let imported = ImportExportService::import_document(
            html.as_bytes(),
            &ImportFormat::Html,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "Untitled");
    }

    #[test]
    fn test_import_json_complete() {
        // US-018: Test JSON import with complete structure
        let json = r#"{
            "title": "JSON Document",
            "content": "This is JSON content",
            "type": "article",
            "tags": ["json", "test", "import"],
            "metadata": {
                "author": "Test Author",
                "priority": "high"
            }
        }"#;

        let imported = ImportExportService::import_document(
            json.as_bytes(),
            &ImportFormat::Json,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "JSON Document");
        assert_eq!(imported.content, "This is JSON content");
        assert_eq!(imported.doc_type, DocumentType::Article);
        assert_eq!(imported.tags, vec!["json", "test", "import"]);
        assert_eq!(imported.metadata.get("author"), Some(&"Test Author".to_string()));
        assert_eq!(imported.metadata.get("priority"), Some(&"high".to_string()));
    }

    #[test]
    fn test_import_json_minimal() {
        // US-020: Test JSON import with minimal structure
        let json = r#"{"content": "Just content"}"#;

        let imported = ImportExportService::import_document(
            json.as_bytes(),
            &ImportFormat::Json,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, "Untitled");
        assert_eq!(imported.content, "Just content");
        assert_eq!(imported.doc_type, DocumentType::Note);
        assert!(imported.tags.is_empty());
        assert!(imported.metadata.is_empty());
    }

    #[test]
    fn test_import_json_invalid() {
        // US-020: Test JSON import with invalid JSON
        let invalid_json = r#"{"invalid": json structure"#;

        let result = ImportExportService::import_document(
            invalid_json.as_bytes(),
            &ImportFormat::Json,
            &ImportOptions::default(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_import_json_type_variants() {
        // US-019: Test JSON import with different document types
        let test_cases = vec![
            ("note", DocumentType::Note),
            ("article", DocumentType::Article),
            ("report", DocumentType::Report),
            ("unknown", DocumentType::Note), // Falls back to Note
        ];

        for (type_str, expected_type) in test_cases {
            let json = format!(r#"{{"title": "Test", "type": "{}"}}"#, type_str);
            let imported = ImportExportService::import_document(
                json.as_bytes(),
                &ImportFormat::Json,
                &ImportOptions::default(),
            ).unwrap();
            assert_eq!(imported.doc_type, expected_type);
        }
    }

    #[test]
    fn test_import_unsupported_formats() {
        // US-020: Test import with unsupported formats
        let content = b"test content";

        let unsupported_formats = vec![
            ImportFormat::Pdf,
            ImportFormat::Word,
            ImportFormat::Custom("custom".to_string()),
        ];

        for format in unsupported_formats {
            let result = ImportExportService::import_document(
                content,
                &format,
                &ImportOptions::default(),
            );
            assert!(result.is_err());
        }
    }

    // EXPORT TESTS

    #[test]
    fn test_export_markdown_with_metadata() {
        // US-018: Test markdown export with metadata
        let doc = create_test_document();
        let options = create_export_options(true, None);

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Markdown, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(result.contains("---"));
        assert!(result.contains("title: Test Document"));
        assert!(result.contains("version: 1.2.3"));
        assert!(result.contains("type: Article"));
        assert!(result.contains("created: 2023-01-01"));
        assert!(result.contains("updated: 2023-01-02"));
        assert!(result.contains("tags:"));
        assert!(result.contains("- test"));
        assert!(result.contains("- sample"));
        assert!(result.contains("category: testing"));
        assert!(result.contains("priority: high"));
        assert!(result.contains("# Test Document"));
        assert!(result.contains("This is the main content"));
    }

    #[test]
    fn test_export_markdown_without_metadata() {
        // US-019: Test markdown export without metadata
        let doc = create_test_document();
        let options = create_export_options(false, None);

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Markdown, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(!result.contains("---"));
        assert!(!result.contains("version:"));
        assert!(result.contains("# Test Document"));
        assert!(result.contains("This is the main content"));
    }

    #[test]
    fn test_export_markdown_with_watermark() {
        // US-019: Test markdown export with watermark
        let doc = create_test_document();
        let options = create_export_options(false, Some("Confidential Document".to_string()));

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Markdown, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(result.contains("*Confidential Document*"));
    }

    #[test]
    fn test_export_plain_text_with_metadata() {
        // US-018: Test plain text export with metadata
        let doc = create_test_document();
        let options = create_export_options(true, None);

        let exported = ImportExportService::export_document(&doc, &ExportFormat::PlainText, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(result.contains("Test Document"));
        assert!(result.contains(&"=".repeat(13))); // Title underline
        assert!(result.contains("Version: 1.2.3"));
        assert!(result.contains("Created: 2023-01-01"));
        assert!(result.contains("Updated: 2023-01-02"));
        assert!(result.contains("This is the main content"));
    }

    #[test]
    fn test_export_plain_text_without_metadata() {
        // US-019: Test plain text export without metadata
        let doc = create_test_document();
        let options = create_export_options(false, None);

        let exported = ImportExportService::export_document(&doc, &ExportFormat::PlainText, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(result.contains("Test Document"));
        assert!(result.contains(&"=".repeat(13)));
        assert!(!result.contains("Version:"));
        assert!(!result.contains("Created:"));
        assert!(result.contains("This is the main content"));
    }

    #[test]
    fn test_export_plain_text_with_watermark() {
        // US-019: Test plain text export with watermark
        let doc = create_test_document();
        let options = create_export_options(false, Some("Internal Use Only".to_string()));

        let exported = ImportExportService::export_document(&doc, &ExportFormat::PlainText, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(result.contains("Internal Use Only"));
    }

    #[test]
    fn test_export_html_with_metadata() {
        // US-018: Test HTML export with metadata
        let doc = create_test_document();
        let options = create_export_options(true, None);

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Html, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>Test Document</title>"));
        assert!(result.contains("charset=\"UTF-8\""));
        assert!(result.contains("meta name=\"version\" content=\"1.2.3\""));
        assert!(result.contains("meta name=\"created\""));
        assert!(result.contains("meta name=\"keywords\" content=\"test\""));
        assert!(result.contains("meta name=\"keywords\" content=\"sample\""));
        assert!(result.contains("<h1>Test Document</h1>"));
        assert!(result.contains("This is the main content"));
        assert!(result.contains("</html>"));
    }

    #[test]
    fn test_export_html_without_metadata() {
        // US-019: Test HTML export without metadata
        let doc = create_test_document();
        let options = create_export_options(false, None);

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Html, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(result.contains("<title>Test Document</title>"));
        assert!(!result.contains("meta name=\"version\""));
        assert!(!result.contains("meta name=\"created\""));
        assert!(!result.contains("meta name=\"keywords\""));
    }

    #[test]
    fn test_export_html_with_watermark() {
        // US-019: Test HTML export with watermark
        let doc = create_test_document();
        let options = create_export_options(false, Some("Copyright 2023".to_string()));

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Html, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(result.contains("<hr>"));
        assert!(result.contains("<em>Copyright 2023</em>"));
    }

    #[test]
    fn test_export_html_content_conversion() {
        // US-019: Test HTML export content conversion (line breaks)
        let mut doc = create_test_document();
        doc.content = "Line 1\n\nLine 2\nLine 3".to_string();
        let options = create_export_options(false, None);

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Html, &options).unwrap();
        let result = String::from_utf8(exported).unwrap();

        assert!(result.contains("Line 1</p>"));
        assert!(result.contains("<p>Line 2<br>"));
        assert!(result.contains("Line 3</p>"));
    }

    #[test]
    fn test_export_json_with_metadata() {
        // US-018: Test JSON export with metadata
        let doc = create_test_document();
        let options = create_export_options(true, None);

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Json, &options).unwrap();
        let result: serde_json::Value = serde_json::from_slice(&exported).unwrap();

        assert_eq!(result["title"], "Test Document");
        assert_eq!(result["content"], "This is the main content of the document.\n\nIt has multiple paragraphs.");
        assert_eq!(result["type"], "article");
        assert_eq!(result["version"], "1.2.3");
        assert!(result["created_at"].is_string());
        assert!(result["updated_at"].is_string());
        assert!(result["author"].is_string());
        assert_eq!(result["tags"], serde_json::json!(["test", "sample"]));
        assert_eq!(result["metadata"]["category"], "testing");
        assert_eq!(result["metadata"]["priority"], "high");
    }

    #[test]
    fn test_export_json_without_metadata() {
        // US-019: Test JSON export without metadata
        let doc = create_test_document();
        let options = create_export_options(false, None);

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Json, &options).unwrap();
        let result: serde_json::Value = serde_json::from_slice(&exported).unwrap();

        assert_eq!(result["title"], "Test Document");
        assert_eq!(result["content"], "This is the main content of the document.\n\nIt has multiple paragraphs.");
        assert_eq!(result["type"], "article");
        assert_eq!(result["version"], "1.2.3");
        assert!(result.get("created_at").is_none());
        assert!(result.get("updated_at").is_none());
        assert!(result.get("author").is_none());
        assert!(result.get("tags").is_none());
        assert!(result.get("metadata").is_none());
    }

    #[test]
    fn test_export_json_with_watermark() {
        // US-019: Test JSON export with watermark
        let doc = create_test_document();
        let options = create_export_options(false, Some("Property of Company".to_string()));

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Json, &options).unwrap();
        let result: serde_json::Value = serde_json::from_slice(&exported).unwrap();

        assert_eq!(result["watermark"], "Property of Company");
    }

    #[test]
    fn test_export_unsupported_formats() {
        // US-020: Test export with unsupported formats
        let doc = create_test_document();
        let options = create_export_options(false, None);

        let unsupported_formats = vec![
            ExportFormat::Pdf,
            ExportFormat::Word,
            ExportFormat::Custom("custom".to_string()),
        ];

        for format in unsupported_formats {
            let result = ImportExportService::export_document(&doc, &format, &options);
            assert!(result.is_err());
        }
    }

    // HELPER FUNCTION TESTS

    #[test]
    fn test_html_escape() {
        // US-020: Test HTML escaping utility function
        let test_cases = vec![
            ("normal text", "normal text"),
            ("text & more", "text &amp; more"),
            ("<script>", "&lt;script&gt;"),
            ("\"quoted\"", "&quot;quoted&quot;"),
            ("'single'", "&#39;single&#39;"),
            ("<>&\"'", "&lt;&gt;&amp;&quot;&#39;"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(html_escape(input), expected);
        }
    }

    #[test]
    fn test_imported_document_structure() {
        // US-019: Test ImportedDocument structure creation
        let imported = ImportedDocument {
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            doc_type: DocumentType::Note,
            metadata: HashMap::new(),
            tags: vec!["tag1".to_string()],
        };

        assert_eq!(imported.title, "Test Title");
        assert_eq!(imported.content, "Test Content");
        assert_eq!(imported.doc_type, DocumentType::Note);
        assert!(imported.metadata.is_empty());
        assert_eq!(imported.tags, vec!["tag1"]);
    }

    #[test]
    fn test_round_trip_markdown() {
        // US-019: Test round-trip conversion (export -> import)
        let doc = create_test_document();
        let options = create_export_options(true, None);

        // Export to markdown
        let exported = ImportExportService::export_document(&doc, &ExportFormat::Markdown, &options).unwrap();

        // Import back
        let imported = ImportExportService::import_document(
            &exported,
            &ImportFormat::Markdown,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, doc.title);
        assert!(imported.content.contains(&doc.content));
    }

    #[test]
    fn test_round_trip_json() {
        // US-019: Test round-trip JSON conversion
        let doc = create_test_document();
        let options = create_export_options(true, None);

        // Export to JSON
        let exported = ImportExportService::export_document(&doc, &ExportFormat::Json, &options).unwrap();

        // Import back
        let imported = ImportExportService::import_document(
            &exported,
            &ImportFormat::Json,
            &ImportOptions::default(),
        ).unwrap();

        assert_eq!(imported.title, doc.title);
        assert_eq!(imported.content, doc.content);
        assert_eq!(imported.doc_type, doc.doc_type);
    }

    #[test]
    fn test_edge_cases_empty_document() {
        // US-020: Test export with empty/minimal document
        let doc = DocumentFullView {
            id: DocumentId::new(),
            title: "".to_string(),
            content: "".to_string(),
            version: DocumentVersion::new(0, 0, 1),
            doc_type: DocumentType::Note,
            tags: vec![],
            author: Uuid::new_v4(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let formats = vec![
            ExportFormat::Markdown,
            ExportFormat::PlainText,
            ExportFormat::Html,
            ExportFormat::Json,
        ];

        for format in formats {
            let result = ImportExportService::export_document(&doc, &format, &create_export_options(false, None));
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_large_content_handling() {
        // US-020: Test handling of large content
        let large_content = "x".repeat(10000);
        let mut doc = create_test_document();
        doc.content = large_content.clone();

        let exported = ImportExportService::export_document(&doc, &ExportFormat::Json, &create_export_options(false, None)).unwrap();
        let result: serde_json::Value = serde_json::from_slice(&exported).unwrap();

        assert_eq!(result["content"].as_str().unwrap(), &large_content);
    }

    #[test]
    fn test_special_characters_handling() {
        // US-020: Test handling of special characters in content
        let special_content = "Content with émojis 🚀 and spëcial chars: éñ中文";
        let mut doc = create_test_document();
        doc.title = special_content.to_string();
        doc.content = special_content.to_string();

        let formats = vec![
            ExportFormat::Markdown,
            ExportFormat::PlainText,
            ExportFormat::Html,
            ExportFormat::Json,
        ];

        for format in formats {
            let result = ImportExportService::export_document(&doc, &format, &create_export_options(false, None));
            assert!(result.is_ok());
        }
    }
} 