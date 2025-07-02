//! Document template service

use crate::value_objects::{TemplateId, DocumentTemplate, TemplateVariable, VariableType};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use regex::Regex;

/// Template service for document generation
pub struct TemplateService {
    /// Template repository
    templates: HashMap<TemplateId, DocumentTemplate>,
}

impl TemplateService {
    /// Create new template service
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template
    pub fn register_template(&mut self, template: DocumentTemplate) -> Result<()> {
        self.templates.insert(template.id, template);
        Ok(())
    }

    /// Get template by ID
    pub fn get_template(&self, id: &TemplateId) -> Option<&DocumentTemplate> {
        self.templates.get(id)
    }

    /// Apply template with variables
    pub fn apply_template(
        &self,
        template_id: &TemplateId,
        variables: &HashMap<String, String>,
    ) -> Result<String> {
        let template = self.templates.get(template_id)
            .ok_or_else(|| anyhow!("Template not found"))?;

        // Validate required variables
        for var in &template.required_variables {
            if var.required && !variables.contains_key(&var.name) {
                if var.default_value.is_none() {
                    return Err(anyhow!("Required variable '{}' not provided", var.name));
                }
            }
        }

        // Apply variable substitution
        let content = template.content.clone();
        
        // Use regex to find all variable placeholders
        let re = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        
        let result = re.replace_all(&content, |caps: &regex::Captures| {
            let var_name = caps[1].trim();
            
            // Check provided variables first
            if let Some(value) = variables.get(var_name) {
                return value.clone();
            }
            
            // Check defaults
            if let Some(var_def) = template.required_variables.iter()
                .find(|v| v.name == var_name) {
                if let Some(default) = &var_def.default_value {
                    return default.clone();
                }
            }
            
            // Keep placeholder if not found
            caps[0].to_string()
        });

        Ok(result.to_string())
    }

    /// Validate variables against template
    pub fn validate_variables(
        &self,
        template_id: &TemplateId,
        variables: &HashMap<String, String>,
    ) -> Result<Vec<ValidationError>> {
        let template = self.templates.get(template_id)
            .ok_or_else(|| anyhow!("Template not found"))?;

        let mut errors = Vec::new();

        for var in &template.required_variables {
            let value = variables.get(&var.name)
                .or(var.default_value.as_ref());

            if let Some(val) = value {
                // Type validation
                match &var.var_type {
                    VariableType::Number => {
                        if val.parse::<f64>().is_err() {
                            errors.push(ValidationError {
                                variable: var.name.clone(),
                                error: "Value must be a number".to_string(),
                            });
                        }
                    }
                    VariableType::Boolean => {
                        if val.parse::<bool>().is_err() {
                            errors.push(ValidationError {
                                variable: var.name.clone(),
                                error: "Value must be true or false".to_string(),
                            });
                        }
                    }
                    VariableType::Date => {
                        if chrono::NaiveDate::parse_from_str(val, "%Y-%m-%d").is_err() {
                            errors.push(ValidationError {
                                variable: var.name.clone(),
                                error: "Value must be a valid date (YYYY-MM-DD)".to_string(),
                            });
                        }
                    }
                    VariableType::List(options) => {
                        if !options.contains(val) {
                            errors.push(ValidationError {
                                variable: var.name.clone(),
                                error: format!("Value must be one of: {}", options.join(", ")),
                            });
                        }
                    }
                    VariableType::Text => {
                        // Text is always valid
                    }
                }
            } else if var.required {
                errors.push(ValidationError {
                    variable: var.name.clone(),
                    error: "Required variable not provided".to_string(),
                });
            }
        }

        Ok(errors)
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<&DocumentTemplate> {
        self.templates.values().collect()
    }

    /// Search templates by category
    pub fn find_by_category(&self, category: &str) -> Vec<&DocumentTemplate> {
        self.templates.values()
            .filter(|t| t.category == category)
            .collect()
    }
}

/// Validation error
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub variable: String,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::DocumentVersion;

    #[test]
    fn test_apply_template() {
        let mut service = TemplateService::new();

        // Create test template
        let template = DocumentTemplate {
            id: TemplateId::new(),
            name: "Meeting Notes".to_string(),
            description: Some("Template for meeting notes".to_string()),
            content: "# Meeting Notes\n\nDate: {{date}}\nAttendees: {{attendees}}\n\n## Agenda\n{{agenda}}\n\n## Notes\n{{notes}}".to_string(),
            required_variables: vec![
                TemplateVariable {
                    name: "date".to_string(),
                    description: Some("Meeting date".to_string()),
                    var_type: VariableType::Date,
                    default_value: None,
                    required: true,
                },
                TemplateVariable {
                    name: "attendees".to_string(),
                    description: Some("List of attendees".to_string()),
                    var_type: VariableType::Text,
                    default_value: None,
                    required: true,
                },
                TemplateVariable {
                    name: "agenda".to_string(),
                    description: Some("Meeting agenda".to_string()),
                    var_type: VariableType::Text,
                    default_value: Some("TBD".to_string()),
                    required: false,
                },
                TemplateVariable {
                    name: "notes".to_string(),
                    description: Some("Meeting notes".to_string()),
                    var_type: VariableType::Text,
                    default_value: Some("".to_string()),
                    required: false,
                },
            ],
            category: "meetings".to_string(),
            version: DocumentVersion::new(1, 0, 0),
        };

        service.register_template(template.clone()).unwrap();

        // Apply template
        let mut variables = HashMap::new();
        variables.insert("date".to_string(), "2024-01-15".to_string());
        variables.insert("attendees".to_string(), "John, Jane, Bob".to_string());

        let result = service.apply_template(&template.id, &variables).unwrap();
        
        assert!(result.contains("Date: 2024-01-15"));
        assert!(result.contains("Attendees: John, Jane, Bob"));
        assert!(result.contains("## Agenda\nTBD")); // Default value
    }

    #[test]
    fn test_validate_variables() {
        let mut service = TemplateService::new();

        // Create template with typed variables
        let template = DocumentTemplate {
            id: TemplateId::new(),
            name: "Test Template".to_string(),
            description: None,
            content: "{{number}} {{date}}".to_string(),
            required_variables: vec![
                TemplateVariable {
                    name: "number".to_string(),
                    description: None,
                    var_type: VariableType::Number,
                    default_value: None,
                    required: true,
                },
                TemplateVariable {
                    name: "date".to_string(),
                    description: None,
                    var_type: VariableType::Date,
                    default_value: None,
                    required: true,
                },
            ],
            category: "test".to_string(),
            version: DocumentVersion::new(1, 0, 0),
        };

        service.register_template(template.clone()).unwrap();

        // Test invalid variables
        let mut variables = HashMap::new();
        variables.insert("number".to_string(), "not a number".to_string());
        variables.insert("date".to_string(), "invalid date".to_string());

        let errors = service.validate_variables(&template.id, &variables).unwrap();
        assert_eq!(errors.len(), 2);
    }
} 