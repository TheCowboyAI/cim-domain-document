//! Document import/export service

use crate::value_objects::{DocumentType, ImportOptions, ExportOptions, ImportFormat, ExportFormat};
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

    #[test]
    fn test_import_markdown() {
        let markdown = r#"---
title: Test Document
author: John Doe
tags:
  - test
  - example
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
        assert!(imported.content.contains("This is a test document"));
    }

    #[test]
    fn test_export_markdown() {
        let doc = DocumentFullView {
            id: DocumentId::new(),
            title: "Test Export".to_string(),
            content: "This is the content.".to_string(),
            version: DocumentVersion::new(1, 0, 0),
            doc_type: DocumentType::Note,
            tags: vec!["test".to_string()],
            author: Uuid::new_v4(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let exported = ImportExportService::export_document(
            &doc,
            &ExportFormat::Markdown,
            &ExportOptions::default(),
        ).unwrap();

        let result = String::from_utf8(exported).unwrap();
        assert!(result.contains("# Test Export"));
        assert!(result.contains("This is the content."));
        assert!(result.contains("tags:"));
    }
} 