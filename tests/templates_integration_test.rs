//! Integration tests for document template functionality

use cim_domain_document::commands::{CreateDocument, ApplyTemplate};
use cim_domain_document::value_objects::*;
use cim_domain_document::services::TemplateService;
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn test_template_workflow() {
    // Create a template service
    let mut template_service = TemplateService::new();

    // Register a report template
    let template = DocumentTemplate {
        id: TemplateId::new(),
        name: "Quarterly Report".to_string(),
        description: Some("Template for quarterly business reports".to_string()),
        content: r#"# {{company}} Quarterly Report - {{quarter}} {{year}}

## Executive Summary
{{executive_summary}}

## Financial Overview
- Revenue: {{revenue}}
- Profit: {{profit}}
- Growth: {{growth}}%

## Key Achievements
{{achievements}}

## Challenges
{{challenges}}

## Outlook
{{outlook}}

---
*Generated on {{date}}*"#.to_string(),
        required_variables: vec![
            TemplateVariable {
                name: "company".to_string(),
                description: Some("Company name".to_string()),
                var_type: VariableType::Text,
                default_value: None,
                required: true,
            },
            TemplateVariable {
                name: "quarter".to_string(),
                description: Some("Quarter (Q1-Q4)".to_string()),
                var_type: VariableType::List(vec![
                    "Q1".to_string(),
                    "Q2".to_string(),
                    "Q3".to_string(),
                    "Q4".to_string(),
                ]),
                default_value: None,
                required: true,
            },
            TemplateVariable {
                name: "year".to_string(),
                description: Some("Year".to_string()),
                var_type: VariableType::Number,
                default_value: None,
                required: true,
            },
            TemplateVariable {
                name: "date".to_string(),
                description: Some("Report date".to_string()),
                var_type: VariableType::Date,
                default_value: None,
                required: true,
            },
            TemplateVariable {
                name: "executive_summary".to_string(),
                description: Some("Executive summary".to_string()),
                var_type: VariableType::Text,
                default_value: Some("To be provided".to_string()),
                required: false,
            },
            TemplateVariable {
                name: "revenue".to_string(),
                description: Some("Revenue amount".to_string()),
                var_type: VariableType::Text,
                default_value: None,
                required: true,
            },
            TemplateVariable {
                name: "profit".to_string(),
                description: Some("Profit amount".to_string()),
                var_type: VariableType::Text,
                default_value: None,
                required: true,
            },
            TemplateVariable {
                name: "growth".to_string(),
                description: Some("Growth percentage".to_string()),
                var_type: VariableType::Number,
                default_value: None,
                required: true,
            },
            TemplateVariable {
                name: "achievements".to_string(),
                description: Some("Key achievements".to_string()),
                var_type: VariableType::Text,
                default_value: Some("- Achievement 1\n- Achievement 2".to_string()),
                required: false,
            },
            TemplateVariable {
                name: "challenges".to_string(),
                description: Some("Challenges faced".to_string()),
                var_type: VariableType::Text,
                default_value: Some("- Challenge 1\n- Challenge 2".to_string()),
                required: false,
            },
            TemplateVariable {
                name: "outlook".to_string(),
                description: Some("Future outlook".to_string()),
                var_type: VariableType::Text,
                default_value: Some("Positive outlook for next quarter".to_string()),
                required: false,
            },
        ],
        category: "reports".to_string(),
        version: DocumentVersion::new(1, 0, 0),
    };

    template_service.register_template(template.clone()).unwrap();

    // Apply the template with variables
    let mut variables = HashMap::new();
    variables.insert("company".to_string(), "Acme Corp".to_string());
    variables.insert("quarter".to_string(), "Q4".to_string());
    variables.insert("year".to_string(), "2024".to_string());
    variables.insert("date".to_string(), "2024-12-31".to_string());
    variables.insert("revenue".to_string(), "$5.2M".to_string());
    variables.insert("profit".to_string(), "$1.3M".to_string());
    variables.insert("growth".to_string(), "23".to_string());
    variables.insert("achievements".to_string(), "- Launched new product line\n- Expanded to 3 new markets\n- Achieved 98% customer satisfaction".to_string());

    let result = template_service.apply_template(&template.id, &variables).unwrap();

    // Verify the result
    assert!(result.contains("# Acme Corp Quarterly Report - Q4 2024"));
    assert!(result.contains("Revenue: $5.2M"));
    assert!(result.contains("Growth: 23%"));
    assert!(result.contains("Launched new product line"));
    assert!(result.contains("Generated on 2024-12-31"));
}

#[tokio::test]
async fn test_template_validation() {
    let mut template_service = TemplateService::new();

    // Create a template with typed variables
    let template = DocumentTemplate {
        id: TemplateId::new(),
        name: "Meeting Notes".to_string(),
        description: None,
        content: "Date: {{date}}, Duration: {{duration}} minutes".to_string(),
        required_variables: vec![
            TemplateVariable {
                name: "date".to_string(),
                description: None,
                var_type: VariableType::Date,
                default_value: None,
                required: true,
            },
            TemplateVariable {
                name: "duration".to_string(),
                description: None,
                var_type: VariableType::Number,
                default_value: None,
                required: true,
            },
        ],
        category: "meetings".to_string(),
        version: DocumentVersion::new(1, 0, 0),
    };

    template_service.register_template(template.clone()).unwrap();

    // Test with invalid variables
    let mut invalid_vars = HashMap::new();
    invalid_vars.insert("date".to_string(), "not-a-date".to_string());
    invalid_vars.insert("duration".to_string(), "not-a-number".to_string());

    let errors = template_service.validate_variables(&template.id, &invalid_vars).unwrap();
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().any(|e| e.variable == "date"));
    assert!(errors.iter().any(|e| e.variable == "duration"));

    // Test with valid variables
    let mut valid_vars = HashMap::new();
    valid_vars.insert("date".to_string(), "2024-01-15".to_string());
    valid_vars.insert("duration".to_string(), "60".to_string());

    let errors = template_service.validate_variables(&template.id, &valid_vars).unwrap();
    assert_eq!(errors.len(), 0);
}

#[tokio::test]
async fn test_template_categories() {
    let mut template_service = TemplateService::new();

    // Register templates in different categories
    let meeting_template = DocumentTemplate {
        id: TemplateId::new(),
        name: "Meeting Notes".to_string(),
        description: None,
        content: "Meeting notes template".to_string(),
        required_variables: vec![],
        category: "meetings".to_string(),
        version: DocumentVersion::new(1, 0, 0),
    };

    let report_template = DocumentTemplate {
        id: TemplateId::new(),
        name: "Status Report".to_string(),
        description: None,
        content: "Status report template".to_string(),
        required_variables: vec![],
        category: "reports".to_string(),
        version: DocumentVersion::new(1, 0, 0),
    };

    let proposal_template = DocumentTemplate {
        id: TemplateId::new(),
        name: "Project Proposal".to_string(),
        description: None,
        content: "Project proposal template".to_string(),
        required_variables: vec![],
        category: "proposals".to_string(),
        version: DocumentVersion::new(1, 0, 0),
    };

    template_service.register_template(meeting_template).unwrap();
    template_service.register_template(report_template).unwrap();
    template_service.register_template(proposal_template).unwrap();

    // Find by category
    let meeting_templates = template_service.find_by_category("meetings");
    assert_eq!(meeting_templates.len(), 1);
    assert_eq!(meeting_templates[0].name, "Meeting Notes");

    let report_templates = template_service.find_by_category("reports");
    assert_eq!(report_templates.len(), 1);
    assert_eq!(report_templates[0].name, "Status Report");

    // List all templates
    let all_templates = template_service.list_templates();
    assert_eq!(all_templates.len(), 3);
} 