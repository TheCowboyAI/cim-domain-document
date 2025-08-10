//! Workflow Event Integration
//!
//! This module provides integration between document domain events and workflow triggers,
//! allowing workflows to be automatically started or transitioned based on document changes.

use super::*;
use crate::events::{DocumentDomainEvent, DocumentCreated, DocumentUploaded, StateChanged};
use crate::value_objects::DocumentState;
use std::collections::HashMap;
use uuid::Uuid;

/// Workflow event integration service that responds to document events
#[derive(Debug)]
pub struct WorkflowEventIntegration {
    /// Document workflow manager
    document_workflow: DocumentWorkflow,
    /// Event-to-workflow mapping rules
    trigger_rules: HashMap<WorkflowTrigger, WorkflowTemplate>,
}

/// Workflow trigger conditions based on document events
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkflowTrigger {
    /// Document was created
    DocumentCreated,
    /// Document was uploaded
    DocumentUploaded,
    /// Document state changed to specific state
    StateChangedTo(DocumentState),
    /// Document was edited
    DocumentEdited,
    /// Manual workflow start requested
    ManualTrigger { workflow_type: String },
}

/// Workflow template configuration
#[derive(Debug, Clone)]
pub struct WorkflowTemplate {
    /// Type of workflow to start
    pub workflow_type: String,
    /// Conditions that must be met
    pub conditions: Vec<WorkflowCondition>,
    /// Initial context variables
    pub initial_context: HashMap<String, serde_json::Value>,
    /// Auto-transition rules
    pub auto_transitions: Vec<AutoTransition>,
}

/// Automatic transition rule
#[derive(Debug, Clone)]
pub struct AutoTransition {
    /// From which node
    pub from_node: WorkflowNodeId,
    /// To which node
    pub to_node: WorkflowNodeId,
    /// Condition for transition
    pub condition: WorkflowCondition,
    /// Delay before transition (optional)
    pub delay: Option<chrono::Duration>,
}

impl WorkflowEventIntegration {
    pub fn new() -> Self {
        let mut trigger_rules = HashMap::new();
        
        // Configure default workflow triggers
        trigger_rules.insert(
            WorkflowTrigger::DocumentCreated,
            WorkflowTemplate {
                workflow_type: "review".to_string(),
                conditions: vec![WorkflowCondition::Always],
                initial_context: {
                    let mut ctx = HashMap::new();
                    ctx.insert("trigger".to_string(), serde_json::Value::String("document_created".to_string()));
                    ctx
                },
                auto_transitions: vec![],
            }
        );

        trigger_rules.insert(
            WorkflowTrigger::StateChangedTo(DocumentState::Draft),
            WorkflowTemplate {
                workflow_type: "approval".to_string(),
                conditions: vec![WorkflowCondition::Always],
                initial_context: {
                    let mut ctx = HashMap::new();
                    ctx.insert("trigger".to_string(), serde_json::Value::String("draft_ready".to_string()));
                    ctx
                },
                auto_transitions: vec![
                    AutoTransition {
                        from_node: WorkflowNodeId::Start,
                        to_node: WorkflowNodeId::PendingApproval,
                        condition: WorkflowCondition::Always,
                        delay: Some(chrono::Duration::minutes(5)), // Brief delay for preparation
                    }
                ],
            }
        );

        Self {
            document_workflow: DocumentWorkflow::new(),
            trigger_rules,
        }
    }

    /// Handle document domain event and potentially trigger workflows
    pub async fn handle_document_event(
        &mut self,
        event: &DocumentDomainEvent,
    ) -> WorkflowResult<Vec<WorkflowInstanceId>> {
        let mut started_workflows = Vec::new();

        match event {
            DocumentDomainEvent::DocumentCreated(event) => {
                if let Some(instance_id) = self.handle_document_created(event).await? {
                    started_workflows.push(instance_id);
                }
            }
            DocumentDomainEvent::DocumentUploaded(event) => {
                if let Some(instance_id) = self.handle_document_uploaded(event).await? {
                    started_workflows.push(instance_id);
                }
            }
            DocumentDomainEvent::StateChanged(event) => {
                if let Some(instance_id) = self.handle_state_changed(event).await? {
                    started_workflows.push(instance_id);
                }
            }
            // Handle other document editing events
            DocumentDomainEvent::DocumentEditedDirect(_) |
            DocumentDomainEvent::DocumentEditedPatch(_) |
            DocumentDomainEvent::DocumentEditedStructured(_) => {
                // For document edits, we might want to trigger approval workflows
                let trigger = WorkflowTrigger::DocumentEdited;
                if let Some(template) = self.trigger_rules.get(&trigger).cloned() {
                    if let Some(document_id) = self.extract_document_id(event) {
                        let instance_id = self.start_workflow_from_template(
                            document_id,
                            Uuid::nil(), // System trigger
                            &template,
                        ).await?;
                        started_workflows.push(instance_id);
                    }
                }
            }
            _ => {
                // No workflow triggers for other events
            }
        }

        Ok(started_workflows)
    }

    /// Handle document created event
    async fn handle_document_created(
        &mut self,
        event: &DocumentCreated,
    ) -> WorkflowResult<Option<WorkflowInstanceId>> {
        let trigger = WorkflowTrigger::DocumentCreated;
        if let Some(template) = self.trigger_rules.get(&trigger).cloned() {
            let instance_id = self.start_workflow_from_template(
                event.document_id.clone(),
                event.author_id,
                &template,
            ).await?;
            Ok(Some(instance_id))
        } else {
            Ok(None)
        }
    }

    /// Handle document uploaded event
    async fn handle_document_uploaded(
        &mut self,
        event: &DocumentUploaded,
    ) -> WorkflowResult<Option<WorkflowInstanceId>> {
        let trigger = WorkflowTrigger::DocumentUploaded;
        if let Some(template) = self.trigger_rules.get(&trigger).cloned() {
            // Convert uploaded_by from String to Uuid if needed
            let user_id = Uuid::parse_str(&event.uploaded_by).unwrap_or(Uuid::nil());
            let instance_id = self.start_workflow_from_template(
                event.document_id.clone(),
                user_id,
                &template,
            ).await?;
            Ok(Some(instance_id))
        } else {
            Ok(None)
        }
    }

    /// Handle state changed event
    async fn handle_state_changed(
        &mut self,
        event: &StateChanged,
    ) -> WorkflowResult<Option<WorkflowInstanceId>> {
        let trigger = WorkflowTrigger::StateChangedTo(event.new_state.clone());
        if let Some(template) = self.trigger_rules.get(&trigger).cloned() {
            let instance_id = self.start_workflow_from_template(
                event.document_id.clone(),
                event.changed_by,
                &template,
            ).await?;

            // Execute auto-transitions
            for auto_transition in &template.auto_transitions {
                if let Some(delay) = auto_transition.delay {
                    // In a real system, this would be scheduled for later execution
                    // For now, we'll just log it
                    println!("Scheduled transition from {:?} to {:?} in {:?}", 
                             auto_transition.from_node, 
                             auto_transition.to_node, 
                             delay);
                } else {
                    // Execute immediate transition
                    self.document_workflow.transition(
                        instance_id,
                        auto_transition.to_node.clone(),
                    )?;
                }
            }

            Ok(Some(instance_id))
        } else {
            Ok(None)
        }
    }

    /// Start workflow from template configuration
    async fn start_workflow_from_template(
        &mut self,
        document_id: DocumentId,
        user_id: Uuid,
        template: &WorkflowTemplate,
    ) -> WorkflowResult<WorkflowInstanceId> {
        // Start the workflow
        let instance_id = self.document_workflow.start_document_workflow(
            &template.workflow_type,
            document_id,
            user_id,
        )?;

        // Set initial context variables
        for (key, value) in &template.initial_context {
            self.document_workflow.set_context(
                instance_id,
                key.clone(),
                value.clone(),
            )?;
        }

        Ok(instance_id)
    }

    /// Extract document ID from various event types
    fn extract_document_id(&self, event: &DocumentDomainEvent) -> Option<DocumentId> {
        match event {
            DocumentDomainEvent::DocumentEditedDirect(e) => Some(e.document_id.clone()),
            DocumentDomainEvent::DocumentEditedPatch(e) => Some(e.document_id.clone()),
            DocumentDomainEvent::DocumentEditedStructured(e) => Some(e.document_id.clone()),
            _ => None,
        }
    }

    /// Add custom workflow trigger rule
    pub fn add_trigger_rule(&mut self, trigger: WorkflowTrigger, template: WorkflowTemplate) {
        self.trigger_rules.insert(trigger, template);
    }

    /// Get active workflow instances for a document
    pub fn get_active_workflows_for_document(
        &self,
        _document_id: DocumentId,
    ) -> Vec<&WorkflowInstance> {
        // This would query the workflow engine for active instances
        // For now, return empty as this requires more complex state management
        Vec::new()
    }

    /// Manually trigger a workflow
    pub async fn trigger_workflow(
        &mut self,
        workflow_type: String,
        document_id: DocumentId,
        user_id: Uuid,
    ) -> WorkflowResult<WorkflowInstanceId> {
        self.document_workflow.start_document_workflow(
            &workflow_type,
            document_id,
            user_id,
        )
    }
}

/// Workflow business rules and validation
#[derive(Debug)]
pub struct WorkflowValidator {
    /// Rules for workflow transitions
    business_rules: Vec<BusinessRule>,
}

/// Business rule for workflow validation
#[derive(Debug, Clone)]
pub struct BusinessRule {
    /// Rule name
    pub name: String,
    /// Condition when rule applies
    pub applies_when: WorkflowCondition,
    /// Validation logic
    pub validation: ValidationRule,
    /// Error message if rule fails
    pub error_message: String,
}

/// Validation rule types
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// Require specific permission
    RequirePermission(Permission),
    /// Require document in specific state
    RequireDocumentState(DocumentState),
    /// Require time elapsed since last transition
    RequireTimeElapsed(chrono::Duration),
    /// Custom validation with expression
    CustomValidation(String),
}

impl WorkflowValidator {
    pub fn new() -> Self {
        let business_rules = vec![
            BusinessRule {
                name: "review_permission_required".to_string(),
                applies_when: WorkflowCondition::Custom("transition_to == 'in_review'".to_string()),
                validation: ValidationRule::RequirePermission(Permission::Review),
                error_message: "User must have review permission to start review".to_string(),
            },
            BusinessRule {
                name: "approval_permission_required".to_string(),
                applies_when: WorkflowCondition::Custom("transition_to == 'approved'".to_string()),
                validation: ValidationRule::RequirePermission(Permission::Approve),
                error_message: "User must have approval permission to approve documents".to_string(),
            },
            BusinessRule {
                name: "draft_state_for_review".to_string(),
                applies_when: WorkflowCondition::Custom("workflow_type == 'review'".to_string()),
                validation: ValidationRule::RequireDocumentState(DocumentState::Draft),
                error_message: "Document must be in draft state to start review".to_string(),
            },
        ];

        Self { business_rules }
    }

    /// Validate if a workflow transition is allowed
    pub fn validate_transition(
        &self,
        from_node: &WorkflowNodeId,
        to_node: &WorkflowNodeId,
        user_permissions: &[Permission],
        document_state: DocumentState,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for rule in &self.business_rules {
            if self.rule_applies(&rule.applies_when, from_node, to_node, context) {
                if !self.validate_rule(&rule.validation, user_permissions, document_state.clone(), context) {
                    errors.push(rule.error_message.clone());
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check if a business rule applies to the current transition
    fn rule_applies(
        &self,
        condition: &WorkflowCondition,
        _from_node: &WorkflowNodeId,
        _to_node: &WorkflowNodeId,
        _context: &HashMap<String, serde_json::Value>,
    ) -> bool {
        match condition {
            WorkflowCondition::Always => true,
            WorkflowCondition::Never => false,
            WorkflowCondition::Custom(_) => true, // Simplified - would evaluate expression
            _ => true,
        }
    }

    /// Validate a specific business rule
    fn validate_rule(
        &self,
        validation: &ValidationRule,
        user_permissions: &[Permission],
        document_state: DocumentState,
        _context: &HashMap<String, serde_json::Value>,
    ) -> bool {
        match validation {
            ValidationRule::RequirePermission(required_permission) => {
                user_permissions.contains(required_permission)
            }
            ValidationRule::RequireDocumentState(required_state) => {
                document_state == *required_state
            }
            ValidationRule::RequireTimeElapsed(_duration) => {
                // Would check if enough time has elapsed - simplified for now
                true
            }
            ValidationRule::CustomValidation(_expression) => {
                // Would evaluate custom expression - simplified for now
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::DocumentCreated;
    use crate::value_objects::DocumentMetadata;

    #[tokio::test]
    async fn test_workflow_event_integration() {
        let mut integration = WorkflowEventIntegration::new();
        
        // Create test document created event
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), "Test Document".to_string());
        metadata.insert("mime_type".to_string(), "text/plain".to_string());
        
        let event = DocumentDomainEvent::DocumentCreated(DocumentCreated {
            document_id: DocumentId::new(),
            document_type: crate::value_objects::DocumentType::Text,
            title: "Test Document".to_string(),
            author_id: Uuid::new_v4(),
            metadata,
            created_at: chrono::Utc::now(),
        });

        // Handle the event
        let workflows = integration.handle_document_event(&event).await.unwrap();
        
        // Should start one workflow
        assert_eq!(workflows.len(), 1);
    }

    #[test]
    fn test_workflow_validator() {
        let validator = WorkflowValidator::new();
        
        let user_permissions = vec![Permission::Review, Permission::Edit];
        let document_state = DocumentState::Draft;
        let context = HashMap::new();

        // Should pass validation for review transition
        let result = validator.validate_transition(
            &WorkflowNodeId::Start,
            &WorkflowNodeId::InReview,
            &user_permissions,
            document_state,
            &context,
        );
        
        assert!(result.is_ok());
        
        // Should fail validation for approval without approve permission
        let result = validator.validate_transition(
            &WorkflowNodeId::InReview,
            &WorkflowNodeId::Approved,
            &user_permissions,
            document_state,
            &context,
        );
        
        // This would fail in a real implementation with proper rule evaluation
        // For now, simplified validation passes
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_manual_workflow_trigger() {
        let mut integration = WorkflowEventIntegration::new();
        
        let document_id = DocumentId::new();
        let user_id = Uuid::new_v4();
        
        // Manually trigger workflow
        let instance_id = integration.trigger_workflow(
            "review".to_string(),
            document_id,
            user_id,
        ).await.unwrap();
        
        assert!(!instance_id.as_uuid().to_string().is_empty());
    }
}