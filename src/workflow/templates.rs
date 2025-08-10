//! Workflow Template Definitions
//!
//! This module provides pre-built workflow templates for common document
//! management scenarios like approval workflows, review processes, and
//! compliance workflows.

use super::*;
use crate::value_objects::DocumentState;

/// Create a standard document approval workflow
pub fn create_standard_approval_workflow(created_by: Uuid) -> WorkflowDefinition {
    let mut workflow = WorkflowDefinition::new(
        "Standard Document Approval".to_string(),
        "Standard approval workflow for business documents requiring review and approval".to_string(),
        created_by,
    );
    
    workflow.category = "Approval".to_string();
    workflow.tags = vec![
        "approval".to_string(),
        "review".to_string(),
        "standard".to_string(),
    ];
    
    // Define workflow variables
    workflow.variables.insert("approval_threshold".to_string(), WorkflowVariable {
        name: "approval_threshold".to_string(),
        var_type: VariableType::Number,
        default_value: Some(serde_json::json!(1)),
        required: false,
        description: Some("Number of approvals required".to_string()),
    });
    
    workflow.variables.insert("review_deadline_days".to_string(), WorkflowVariable {
        name: "review_deadline_days".to_string(),
        var_type: VariableType::Number,
        default_value: Some(serde_json::json!(3)),
        required: false,
        description: Some("Days allowed for review".to_string()),
    });
    
    // Start Node
    let start_node = WorkflowNode::Start(StartNode {
        id: NodeId::new("start"),
        name: "Workflow Start".to_string(),
        actions: vec![
            WorkflowAction::SetState(DocumentState::Draft),
            WorkflowAction::LogEvent {
                level: LogLevel::Info,
                message: "Document approval workflow started".to_string(),
                metadata: HashMap::new(),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Draft Task Node
    let draft_task = WorkflowNode::Task(TaskNode {
        id: NodeId::new("draft"),
        name: "Draft Creation".to_string(),
        task_type: TaskType::Manual,
        assignees: vec![], // Dynamic assignment based on document owner
        duration_sla: Some(Duration::days(7)),
        guards: vec![
            Guard::RequireRole("author".to_string()),
        ],
        actions: vec![
            WorkflowAction::SetState(DocumentState::Draft),
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![], // Will be populated by workflow engine
                message: "Please complete the document draft".to_string(),
                template: Some("draft_reminder".to_string()),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Submit for Review Task
    let submit_task = WorkflowNode::Task(TaskNode {
        id: NodeId::new("submit"),
        name: "Submit for Review".to_string(),
        task_type: TaskType::Manual,
        assignees: vec![],
        duration_sla: Some(Duration::days(1)),
        guards: vec![
            Guard::RequireRole("author".to_string()),
            Guard::DocumentSizeLimit { min: Some(1), max: None },
        ],
        actions: vec![
            WorkflowAction::SetState(DocumentState::InReview),
            WorkflowAction::AssignTask {
                user_id: Uuid::nil(), // Will be determined dynamically
                task_description: "Review submitted document".to_string(),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Review Task Node
    let review_task = WorkflowNode::Task(TaskNode {
        id: NodeId::new("review"),
        name: "Document Review".to_string(),
        task_type: TaskType::Review,
        assignees: vec![], // Dynamic assignment based on document type/department
        duration_sla: Some(Duration::days(3)),
        guards: vec![
            Guard::RequireRole("reviewer".to_string()),
        ],
        actions: vec![
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "Document review required".to_string(),
                template: Some("review_required".to_string()),
            },
            WorkflowAction::SetDeadline {
                node_id: NodeId::new("review"),
                deadline: Utc::now() + Duration::days(3),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Approval Decision Node
    let approval_decision = WorkflowNode::Decision(DecisionNode {
        id: NodeId::new("approval_decision"),
        name: "Approval Decision".to_string(),
        conditions: vec![
            ("approve".to_string(), Condition::expression("review.decision == 'approve'".to_string())),
            ("reject".to_string(), Condition::expression("review.decision == 'reject'".to_string())),
            ("revise".to_string(), Condition::expression("review.decision == 'revise'".to_string())),
        ],
        actions: vec![
            WorkflowAction::LogEvent {
                level: LogLevel::Info,
                message: "Review decision made".to_string(),
                metadata: HashMap::new(),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Approved Task
    let approved_task = WorkflowNode::Task(TaskNode {
        id: NodeId::new("approved"),
        name: "Document Approved".to_string(),
        task_type: TaskType::Automatic,
        assignees: vec![],
        duration_sla: None,
        guards: vec![],
        actions: vec![
            WorkflowAction::SetState(DocumentState::Approved),
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "Document has been approved".to_string(),
                template: Some("document_approved".to_string()),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Revision Required Task
    let revision_task = WorkflowNode::Task(TaskNode {
        id: NodeId::new("revision"),
        name: "Revision Required".to_string(),
        task_type: TaskType::Manual,
        assignees: vec![],
        duration_sla: Some(Duration::days(5)),
        guards: vec![
            Guard::RequireRole("author".to_string()),
        ],
        actions: vec![
            WorkflowAction::SetState(DocumentState::Draft),
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "Document requires revision based on review feedback".to_string(),
                template: Some("revision_required".to_string()),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Rejected End Node
    let rejected_end = WorkflowNode::End(EndNode {
        id: NodeId::new("rejected"),
        name: "Document Rejected".to_string(),
        actions: vec![
            WorkflowAction::SetState(DocumentState::Rejected),
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "Document has been rejected".to_string(),
                template: Some("document_rejected".to_string()),
            },
            WorkflowAction::LogEvent {
                level: LogLevel::Warn,
                message: "Document approval workflow ended with rejection".to_string(),
                metadata: HashMap::new(),
            },
        ],
        completion_status: CompletionStatus::Error,
        metadata: HashMap::new(),
    });
    
    // Approved End Node
    let approved_end = WorkflowNode::End(EndNode {
        id: NodeId::new("end_approved"),
        name: "Workflow Complete - Approved".to_string(),
        actions: vec![
            WorkflowAction::LogEvent {
                level: LogLevel::Info,
                message: "Document approval workflow completed successfully".to_string(),
                metadata: HashMap::new(),
            },
        ],
        completion_status: CompletionStatus::Success,
        metadata: HashMap::new(),
    });
    
    // Add all nodes to graph
    workflow.graph.add_node(NodeId::new("start"), start_node);
    workflow.graph.add_node(NodeId::new("draft"), draft_task);
    workflow.graph.add_node(NodeId::new("submit"), submit_task);
    workflow.graph.add_node(NodeId::new("review"), review_task);
    workflow.graph.add_node(NodeId::new("approval_decision"), approval_decision);
    workflow.graph.add_node(NodeId::new("approved"), approved_task);
    workflow.graph.add_node(NodeId::new("revision"), revision_task);
    workflow.graph.add_node(NodeId::new("rejected"), rejected_end);
    workflow.graph.add_node(NodeId::new("end_approved"), approved_end);
    
    // Add edges
    workflow.graph.add_edge(
        EdgeId::new("start_to_draft"),
        NodeId::new("start"),
        NodeId::new("draft"),
        None,
    );
    
    workflow.graph.add_edge(
        EdgeId::new("draft_to_submit"),
        NodeId::new("draft"),
        NodeId::new("submit"),
        Some(Condition::expression("document.content_ready == true".to_string())),
    );
    
    workflow.graph.add_edge(
        EdgeId::new("submit_to_review"),
        NodeId::new("submit"),
        NodeId::new("review"),
        None,
    );
    
    workflow.graph.add_edge(
        EdgeId::new("review_to_decision"),
        NodeId::new("review"),
        NodeId::new("approval_decision"),
        Some(Condition::expression("review.completed == true".to_string())),
    );
    
    workflow.graph.add_edge(
        EdgeId::new("decision_to_approved"),
        NodeId::new("approval_decision"),
        NodeId::new("approved"),
        Some(Condition::expression("review.decision == 'approve'".to_string())),
    );
    
    workflow.graph.add_edge(
        EdgeId::new("decision_to_revision"),
        NodeId::new("approval_decision"),
        NodeId::new("revision"),
        Some(Condition::expression("review.decision == 'revise'".to_string())),
    );
    
    workflow.graph.add_edge(
        EdgeId::new("decision_to_rejected"),
        NodeId::new("approval_decision"),
        NodeId::new("rejected"),
        Some(Condition::expression("review.decision == 'reject'".to_string())),
    );
    
    workflow.graph.add_edge(
        EdgeId::new("approved_to_end"),
        NodeId::new("approved"),
        NodeId::new("end_approved"),
        None,
    );
    
    workflow.graph.add_edge(
        EdgeId::new("revision_to_submit"),
        NodeId::new("revision"),
        NodeId::new("submit"),
        Some(Condition::expression("revision.completed == true".to_string())),
    );
    
    workflow
}

/// Create a contract review workflow with legal and finance approval
pub fn create_contract_review_workflow(created_by: Uuid) -> WorkflowDefinition {
    let mut workflow = WorkflowDefinition::new(
        "Contract Review Workflow".to_string(),
        "Multi-stakeholder contract review with legal and finance approval".to_string(),
        created_by,
    );
    
    workflow.category = "Contract".to_string();
    workflow.tags = vec![
        "contract".to_string(),
        "legal".to_string(),
        "finance".to_string(),
        "multi-approval".to_string(),
    ];
    
    // Start Node
    let start_node = WorkflowNode::Start(StartNode {
        id: NodeId::new("start"),
        name: "Contract Review Start".to_string(),
        actions: vec![
            WorkflowAction::SetState(DocumentState::Draft),
            WorkflowAction::LogEvent {
                level: LogLevel::Info,
                message: "Contract review workflow started".to_string(),
                metadata: HashMap::new(),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Parallel Review Node
    let parallel_review = WorkflowNode::Parallel(ParallelNode {
        id: NodeId::new("parallel_review"),
        name: "Parallel Review".to_string(),
        min_branches: 2, // Both legal and finance must complete
        actions: vec![
            WorkflowAction::SetState(DocumentState::InReview),
        ],
        metadata: HashMap::new(),
    });
    
    // Legal Review Task
    let legal_review = WorkflowNode::Task(TaskNode {
        id: NodeId::new("legal_review"),
        name: "Legal Review".to_string(),
        task_type: TaskType::Review,
        assignees: vec![], // Assigned to legal team
        duration_sla: Some(Duration::days(5)),
        guards: vec![
            Guard::RequireRole("legal_reviewer".to_string()),
        ],
        actions: vec![
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "Contract requires legal review".to_string(),
                template: Some("legal_review_required".to_string()),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Finance Review Task
    let finance_review = WorkflowNode::Task(TaskNode {
        id: NodeId::new("finance_review"),
        name: "Finance Review".to_string(),
        task_type: TaskType::Review,
        assignees: vec![], // Assigned to finance team
        duration_sla: Some(Duration::days(3)),
        guards: vec![
            Guard::RequireRole("finance_reviewer".to_string()),
        ],
        actions: vec![
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "Contract requires finance review".to_string(),
                template: Some("finance_review_required".to_string()),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Join Reviews Node
    let join_reviews = WorkflowNode::Join(JoinNode {
        id: NodeId::new("join_reviews"),
        name: "Join Reviews".to_string(),
        join_strategy: JoinStrategy::WaitAll,
        actions: vec![],
        metadata: HashMap::new(),
    });
    
    // Final Decision Node
    let final_decision = WorkflowNode::Decision(DecisionNode {
        id: NodeId::new("final_decision"),
        name: "Final Decision".to_string(),
        conditions: vec![
            ("approve".to_string(), Condition::expression("legal_review.decision == 'approve' && finance_review.decision == 'approve'".to_string())),
            ("reject".to_string(), Condition::expression("legal_review.decision == 'reject' || finance_review.decision == 'reject'".to_string())),
        ],
        actions: vec![],
        metadata: HashMap::new(),
    });
    
    // Executive Approval (for high-value contracts)
    let executive_approval = WorkflowNode::Task(TaskNode {
        id: NodeId::new("executive_approval"),
        name: "Executive Approval".to_string(),
        task_type: TaskType::Review,
        assignees: vec![],
        duration_sla: Some(Duration::days(2)),
        guards: vec![
            Guard::RequireRole("executive".to_string()),
            Guard::DocumentSizeLimit { min: None, max: None }, // Could check contract value
        ],
        actions: vec![
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "High-value contract requires executive approval".to_string(),
                template: Some("executive_approval_required".to_string()),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Approved End Node
    let approved_end = WorkflowNode::End(EndNode {
        id: NodeId::new("approved"),
        name: "Contract Approved".to_string(),
        actions: vec![
            WorkflowAction::SetState(DocumentState::Approved),
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "Contract has been approved".to_string(),
                template: Some("contract_approved".to_string()),
            },
        ],
        completion_status: CompletionStatus::Success,
        metadata: HashMap::new(),
    });
    
    // Rejected End Node
    let rejected_end = WorkflowNode::End(EndNode {
        id: NodeId::new("rejected"),
        name: "Contract Rejected".to_string(),
        actions: vec![
            WorkflowAction::SetState(DocumentState::Rejected),
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "Contract has been rejected".to_string(),
                template: Some("contract_rejected".to_string()),
            },
        ],
        completion_status: CompletionStatus::Error,
        metadata: HashMap::new(),
    });
    
    // Add nodes to graph
    workflow.graph.add_node(NodeId::new("start"), start_node);
    workflow.graph.add_node(NodeId::new("parallel_review"), parallel_review);
    workflow.graph.add_node(NodeId::new("legal_review"), legal_review);
    workflow.graph.add_node(NodeId::new("finance_review"), finance_review);
    workflow.graph.add_node(NodeId::new("join_reviews"), join_reviews);
    workflow.graph.add_node(NodeId::new("final_decision"), final_decision);
    workflow.graph.add_node(NodeId::new("executive_approval"), executive_approval);
    workflow.graph.add_node(NodeId::new("approved"), approved_end);
    workflow.graph.add_node(NodeId::new("rejected"), rejected_end);
    
    // Add edges
    workflow.graph.add_edge(
        EdgeId::new("start_to_parallel"),
        NodeId::new("start"),
        NodeId::new("parallel_review"),
        None,
    );
    
    workflow.graph.add_edge(
        EdgeId::new("parallel_to_legal"),
        NodeId::new("parallel_review"),
        NodeId::new("legal_review"),
        None,
    );
    
    workflow.graph.add_edge(
        EdgeId::new("parallel_to_finance"),
        NodeId::new("parallel_review"),
        NodeId::new("finance_review"),
        None,
    );
    
    workflow.graph.add_edge(
        EdgeId::new("legal_to_join"),
        NodeId::new("legal_review"),
        NodeId::new("join_reviews"),
        None,
    );
    
    workflow.graph.add_edge(
        EdgeId::new("finance_to_join"),
        NodeId::new("finance_review"),
        NodeId::new("join_reviews"),
        None,
    );
    
    workflow.graph.add_edge(
        EdgeId::new("join_to_decision"),
        NodeId::new("join_reviews"),
        NodeId::new("final_decision"),
        None,
    );
    
    workflow.graph.add_edge(
        EdgeId::new("decision_to_executive"),
        NodeId::new("final_decision"),
        NodeId::new("executive_approval"),
        Some(Condition::expression("contract.value > 100000".to_string())),
    );
    
    workflow.graph.add_edge(
        EdgeId::new("decision_to_approved"),
        NodeId::new("final_decision"),
        NodeId::new("approved"),
        Some(Condition::expression("contract.value <= 100000 && legal_review.decision == 'approve' && finance_review.decision == 'approve'".to_string())),
    );
    
    workflow.graph.add_edge(
        EdgeId::new("executive_to_approved"),
        NodeId::new("executive_approval"),
        NodeId::new("approved"),
        Some(Condition::expression("executive_review.decision == 'approve'".to_string())),
    );
    
    workflow.graph.add_edge(
        EdgeId::new("decision_to_rejected"),
        NodeId::new("final_decision"),
        NodeId::new("rejected"),
        Some(Condition::expression("legal_review.decision == 'reject' || finance_review.decision == 'reject'".to_string())),
    );
    
    workflow.graph.add_edge(
        EdgeId::new("executive_to_rejected"),
        NodeId::new("executive_approval"),
        NodeId::new("rejected"),
        Some(Condition::expression("executive_review.decision == 'reject'".to_string())),
    );
    
    workflow
}

/// Create a compliance workflow with automated scanning
pub fn create_compliance_workflow(created_by: Uuid) -> WorkflowDefinition {
    let mut workflow = WorkflowDefinition::new(
        "Compliance Review Workflow".to_string(),
        "Automated compliance scanning with risk-based routing".to_string(),
        created_by,
    );
    
    workflow.category = "Compliance".to_string();
    workflow.tags = vec![
        "compliance".to_string(),
        "automated".to_string(),
        "risk-assessment".to_string(),
        "regulatory".to_string(),
    ];
    
    // Start with automated compliance scan
    let start_node = WorkflowNode::Start(StartNode {
        id: NodeId::new("start"),
        name: "Compliance Workflow Start".to_string(),
        actions: vec![
            WorkflowAction::LogEvent {
                level: LogLevel::Info,
                message: "Compliance workflow started".to_string(),
                metadata: HashMap::new(),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Automated Compliance Scan
    let compliance_scan = WorkflowNode::Task(TaskNode {
        id: NodeId::new("compliance_scan"),
        name: "Automated Compliance Scan".to_string(),
        task_type: TaskType::Automatic,
        assignees: vec![],
        duration_sla: Some(Duration::minutes(30)),
        guards: vec![],
        actions: vec![
            WorkflowAction::IntegrateWithSystem {
                system_name: "compliance_scanner".to_string(),
                action: "scan_document".to_string(),
                parameters: HashMap::new(),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // Risk Assessment Decision
    let risk_decision = WorkflowNode::Decision(DecisionNode {
        id: NodeId::new("risk_assessment"),
        name: "Risk Assessment".to_string(),
        conditions: vec![
            ("low_risk".to_string(), Condition::expression("compliance_scan.risk_score < 3".to_string())),
            ("medium_risk".to_string(), Condition::expression("compliance_scan.risk_score >= 3 && compliance_scan.risk_score < 7".to_string())),
            ("high_risk".to_string(), Condition::expression("compliance_scan.risk_score >= 7".to_string())),
        ],
        actions: vec![],
        metadata: HashMap::new(),
    });
    
    // Auto-Approval for Low Risk
    let auto_approved = WorkflowNode::End(EndNode {
        id: NodeId::new("auto_approved"),
        name: "Auto-Approved (Low Risk)".to_string(),
        actions: vec![
            WorkflowAction::SetState(DocumentState::Approved),
            WorkflowAction::LogEvent {
                level: LogLevel::Info,
                message: "Document auto-approved due to low risk score".to_string(),
                metadata: HashMap::new(),
            },
        ],
        completion_status: CompletionStatus::Success,
        metadata: HashMap::new(),
    });
    
    // Manual Review for Medium/High Risk
    let manual_review = WorkflowNode::Task(TaskNode {
        id: NodeId::new("manual_review"),
        name: "Manual Compliance Review".to_string(),
        task_type: TaskType::Review,
        assignees: vec![],
        duration_sla: Some(Duration::days(2)),
        guards: vec![
            Guard::RequireRole("compliance_officer".to_string()),
        ],
        actions: vec![
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "Document requires manual compliance review".to_string(),
                template: Some("compliance_review_required".to_string()),
            },
        ],
        metadata: HashMap::new(),
    });
    
    // High Risk Escalation
    let escalated_review = WorkflowNode::Task(TaskNode {
        id: NodeId::new("escalated_review"),
        name: "Escalated Compliance Review".to_string(),
        task_type: TaskType::Review,
        assignees: vec![],
        duration_sla: Some(Duration::days(1)),
        guards: vec![
            Guard::RequireRole("senior_compliance_officer".to_string()),
        ],
        actions: vec![
            WorkflowAction::SendNotification {
                notification_type: NotificationType::Email,
                recipients: vec![],
                message: "High-risk document requires immediate compliance review".to_string(),
                template: Some("urgent_compliance_review".to_string()),
            },
            WorkflowAction::EscalateToManager {
                user_id: Uuid::nil(), // Will be determined dynamically
                reason: "High-risk compliance issue detected".to_string(),
            },
        ],
        metadata: HashMap::new(),
    });
    
    workflow.graph.add_node(NodeId::new("start"), start_node);
    workflow.graph.add_node(NodeId::new("compliance_scan"), compliance_scan);
    workflow.graph.add_node(NodeId::new("risk_assessment"), risk_decision);
    workflow.graph.add_node(NodeId::new("auto_approved"), auto_approved);
    workflow.graph.add_node(NodeId::new("manual_review"), manual_review);
    workflow.graph.add_node(NodeId::new("escalated_review"), escalated_review);
    
    workflow
}

/// Get all available workflow templates
pub fn get_available_templates() -> Vec<(&'static str, &'static str, fn(Uuid) -> WorkflowDefinition)> {
    vec![
        (
            "standard_approval",
            "Standard Document Approval",
            create_standard_approval_workflow,
        ),
        (
            "contract_review",
            "Contract Review Workflow",
            create_contract_review_workflow,
        ),
        (
            "compliance_workflow",
            "Compliance Review Workflow",
            create_compliance_workflow,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_approval_workflow_creation() {
        let user_id = Uuid::new_v4();
        let workflow = create_standard_approval_workflow(user_id);
        
        assert_eq!(workflow.name, "Standard Document Approval");
        assert_eq!(workflow.created_by, user_id);
        assert_eq!(workflow.category, "Approval");
        assert!(workflow.tags.contains(&"approval".to_string()));
        
        // Verify workflow structure
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("start")));
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("draft")));
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("review")));
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("approval_decision")));
        
        // Verify workflow is valid
        assert!(workflow.validate().is_ok());
    }
    
    #[test]
    fn test_contract_review_workflow_creation() {
        let user_id = Uuid::new_v4();
        let workflow = create_contract_review_workflow(user_id);
        
        assert_eq!(workflow.name, "Contract Review Workflow");
        assert_eq!(workflow.category, "Contract");
        assert!(workflow.tags.contains(&"legal".to_string()));
        assert!(workflow.tags.contains(&"finance".to_string()));
        
        // Verify parallel structure
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("parallel_review")));
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("legal_review")));
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("finance_review")));
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("join_reviews")));
        
        // Verify workflow is valid
        assert!(workflow.validate().is_ok());
    }
    
    #[test]
    fn test_compliance_workflow_creation() {
        let user_id = Uuid::new_v4();
        let workflow = create_compliance_workflow(user_id);
        
        assert_eq!(workflow.name, "Compliance Review Workflow");
        assert_eq!(workflow.category, "Compliance");
        assert!(workflow.tags.contains(&"automated".to_string()));
        
        // Verify automated scan structure
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("compliance_scan")));
        assert!(workflow.graph.nodes.contains_key(&NodeId::new("risk_assessment")));
        
        // Check that scan task is automatic
        if let Some(WorkflowNode::Task(scan_task)) = workflow.graph.nodes.get(&NodeId::new("compliance_scan")) {
            assert_eq!(scan_task.task_type, TaskType::Automatic);
        }
    }
    
    #[test]
    fn test_available_templates_list() {
        let templates = get_available_templates();
        
        assert_eq!(templates.len(), 3);
        assert!(templates.iter().any(|(id, _, _)| *id == "standard_approval"));
        assert!(templates.iter().any(|(id, _, _)| *id == "contract_review"));
        assert!(templates.iter().any(|(id, _, _)| *id == "compliance_workflow"));
        
        // Test that all templates can be created
        let user_id = Uuid::new_v4();
        for (_, _, create_fn) in templates {
            let workflow = create_fn(user_id);
            assert!(workflow.validate().is_ok());
        }
    }
}