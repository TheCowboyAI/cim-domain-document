//! CID Chain Verification Service
//!
//! This module provides services for verifying the integrity of CID chains,
//! ensuring that document version histories are consistent and uncorrupted.

use std::collections::{HashMap, HashSet};
use cid::Cid;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

use crate::value_objects::{
    DocumentId, CidChain, ChainError
};
use crate::events::{
    ChainVerificationResult, ChainIssue, ChainIssueType, IssueSeverity
};

/// Trait for CID chain verification operations
#[async_trait]
pub trait CidChainVerificationService: Send + Sync {
    /// Verify the integrity of a complete CID chain
    async fn verify_chain(&self, chain: &CidChain) -> ChainVerificationResult;
    
    /// Verify a specific link in the chain
    async fn verify_link(&self, predecessor: &Cid, successor: &Cid) -> Result<bool, ChainError>;
    
    /// Check if a CID exists in storage
    async fn check_cid_exists(&self, cid: &Cid) -> Result<bool, ChainError>;
    
    /// Verify hash integrity of content at CID
    async fn verify_hash_integrity(&self, cid: &Cid) -> Result<bool, ChainError>;
    
    /// Get content size for a CID
    async fn get_content_size(&self, cid: &Cid) -> Result<u64, ChainError>;
}

/// Default implementation of CID chain verification
pub struct DefaultCidChainVerificationService {
    /// Mock storage for demonstration - in real implementation would connect to IPFS/storage
    mock_storage: HashMap<Cid, MockContent>,
}

/// Mock content representation for testing
#[derive(Debug, Clone)]
struct MockContent {
    data: Vec<u8>,
    created_at: DateTime<Utc>,
}

impl DefaultCidChainVerificationService {
    pub fn new() -> Self {
        Self {
            mock_storage: HashMap::new(),
        }
    }
    
    pub fn with_mock_content(mut self, cid: Cid, data: Vec<u8>) -> Self {
        self.mock_storage.insert(cid, MockContent {
            data,
            created_at: Utc::now(),
        });
        self
    }
}

impl Default for DefaultCidChainVerificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CidChainVerificationService for DefaultCidChainVerificationService {
    async fn verify_chain(&self, chain: &CidChain) -> ChainVerificationResult {
        let start_time = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut links_verified = 0u32;
        
        // Verify root CID exists
        match self.check_cid_exists(&chain.root_cid).await {
            Ok(false) => {
                issues.push(ChainIssue {
                    issue_type: ChainIssueType::MissingContent,
                    position: 0,
                    description: format!("Root CID {} not found in storage", chain.root_cid),
                    severity: IssueSeverity::Critical,
                });
            }
            Err(e) => {
                issues.push(ChainIssue {
                    issue_type: ChainIssueType::MetadataInconsistent,
                    position: 0,
                    description: format!("Error verifying root CID: {}", e),
                    severity: IssueSeverity::High,
                });
            }
            Ok(true) => {
                // Verify hash integrity
                if let Err(e) = self.verify_hash_integrity(&chain.root_cid).await {
                    issues.push(ChainIssue {
                        issue_type: ChainIssueType::HashMismatch,
                        position: 0,
                        description: format!("Hash verification failed for root CID: {}", e),
                        severity: IssueSeverity::High,
                    });
                }
            }
        }
        
        // Track CIDs to detect duplicates
        let mut seen_cids = HashSet::new();
        seen_cids.insert(chain.root_cid.clone());
        
        // Verify each link in the chain
        let mut previous_cid = chain.root_cid.clone();
        for (index, link) in chain.chain.iter().enumerate() {
            let position = (index + 1) as u64;
            
            // Verify link consistency
            if link.predecessor_cid != previous_cid {
                issues.push(ChainIssue {
                    issue_type: ChainIssueType::BrokenLink,
                    position,
                    description: format!(
                        "Link predecessor mismatch: expected {}, got {}",
                        previous_cid, link.predecessor_cid
                    ),
                    severity: IssueSeverity::Critical,
                });
            }
            
            // Check for duplicate CIDs
            if seen_cids.contains(&link.successor_cid) {
                issues.push(ChainIssue {
                    issue_type: ChainIssueType::DuplicateContent,
                    position,
                    description: format!("Duplicate CID found: {}", link.successor_cid),
                    severity: IssueSeverity::Medium,
                });
            } else {
                seen_cids.insert(link.successor_cid.clone());
            }
            
            // Verify successor CID exists
            match self.check_cid_exists(&link.successor_cid).await {
                Ok(false) => {
                    issues.push(ChainIssue {
                        issue_type: ChainIssueType::MissingContent,
                        position,
                        description: format!("Successor CID {} not found in storage", link.successor_cid),
                        severity: IssueSeverity::Critical,
                    });
                }
                Err(e) => {
                    issues.push(ChainIssue {
                        issue_type: ChainIssueType::MetadataInconsistent,
                        position,
                        description: format!("Error verifying successor CID: {}", e),
                        severity: IssueSeverity::High,
                    });
                }
                Ok(true) => {
                    // Verify hash integrity
                    if let Err(e) = self.verify_hash_integrity(&link.successor_cid).await {
                        issues.push(ChainIssue {
                            issue_type: ChainIssueType::HashMismatch,
                            position,
                            description: format!("Hash verification failed for successor CID: {}", e),
                            severity: IssueSeverity::High,
                        });
                    }
                }
            }
            
            // Verify the link itself
            match self.verify_link(&link.predecessor_cid, &link.successor_cid).await {
                Ok(false) => {
                    issues.push(ChainIssue {
                        issue_type: ChainIssueType::BrokenLink,
                        position,
                        description: "Link verification failed".to_string(),
                        severity: IssueSeverity::High,
                    });
                }
                Err(e) => {
                    issues.push(ChainIssue {
                        issue_type: ChainIssueType::MetadataInconsistent,
                        position,
                        description: format!("Error verifying link: {}", e),
                        severity: IssueSeverity::Medium,
                    });
                }
                Ok(true) => {
                    links_verified += 1;
                }
            }
            
            previous_cid = link.successor_cid.clone();
        }
        
        // Verify head CID matches last successor
        if !chain.chain.is_empty() {
            let last_link = chain.chain.last().unwrap();
            if chain.head_cid != last_link.successor_cid {
                issues.push(ChainIssue {
                    issue_type: ChainIssueType::MetadataInconsistent,
                    position: chain.chain.len() as u64,
                    description: format!(
                        "Head CID mismatch: expected {}, got {}",
                        last_link.successor_cid, chain.head_cid
                    ),
                    severity: IssueSeverity::High,
                });
            }
        } else if chain.head_cid != chain.root_cid {
            issues.push(ChainIssue {
                issue_type: ChainIssueType::MetadataInconsistent,
                position: 0,
                description: format!(
                    "Empty chain head CID mismatch: expected {}, got {}",
                    chain.root_cid, chain.head_cid
                ),
                severity: IssueSeverity::High,
            });
        }
        
        // Verify chain length consistency
        let expected_length = chain.chain.len() as u64 + 1; // +1 for root
        if chain.chain_length != expected_length {
            issues.push(ChainIssue {
                issue_type: ChainIssueType::MetadataInconsistent,
                position: 0,
                description: format!(
                    "Chain length mismatch: expected {}, got {}",
                    expected_length, chain.chain_length
                ),
                severity: IssueSeverity::Medium,
            });
        }
        
        let verification_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Determine overall validity
        let is_valid = !issues.iter().any(|issue| {
            matches!(issue.severity, IssueSeverity::Critical | IssueSeverity::High)
        });
        
        ChainVerificationResult {
            is_valid,
            issues,
            verification_time_ms,
            links_verified,
        }
    }
    
    async fn verify_link(&self, predecessor: &Cid, successor: &Cid) -> Result<bool, ChainError> {
        // Check that both CIDs exist
        let predecessor_exists = self.check_cid_exists(predecessor).await?;
        let successor_exists = self.check_cid_exists(successor).await?;
        
        if !predecessor_exists {
            return Err(ChainError::CidNotFound { cid: predecessor.clone() });
        }
        
        if !successor_exists {
            return Err(ChainError::CidNotFound { cid: successor.clone() });
        }
        
        // In a real implementation, this would verify cryptographic relationships
        // For now, we just check that both exist
        Ok(true)
    }
    
    async fn check_cid_exists(&self, cid: &Cid) -> Result<bool, ChainError> {
        // In mock implementation, check our in-memory storage
        Ok(self.mock_storage.contains_key(cid))
    }
    
    async fn verify_hash_integrity(&self, cid: &Cid) -> Result<bool, ChainError> {
        // In mock implementation, assume hash integrity is valid if content exists
        match self.mock_storage.get(cid) {
            Some(_content) => {
                // In real implementation, would recompute hash and compare with CID
                Ok(true)
            }
            None => Err(ChainError::CidNotFound { cid: cid.clone() }),
        }
    }
    
    async fn get_content_size(&self, cid: &Cid) -> Result<u64, ChainError> {
        match self.mock_storage.get(cid) {
            Some(content) => Ok(content.data.len() as u64),
            None => Err(ChainError::CidNotFound { cid: cid.clone() }),
        }
    }
}

/// Helper service for performing chain repairs
pub struct CidChainRepairService {
    verification_service: Box<dyn CidChainVerificationService>,
}

impl CidChainRepairService {
    pub fn new(verification_service: Box<dyn CidChainVerificationService>) -> Self {
        Self {
            verification_service,
        }
    }
    
    /// Attempt to repair a chain by removing problematic links
    pub async fn repair_chain(&self, chain: &mut CidChain) -> Result<Vec<ChainIssue>, ChainError> {
        let verification_result = self.verification_service.verify_chain(chain).await;
        let mut repaired_issues = Vec::new();
        
        if verification_result.is_valid {
            return Ok(repaired_issues);
        }
        
        // Handle critical issues first
        for issue in verification_result.issues {
            match issue.severity {
                IssueSeverity::Critical => {
                    match issue.issue_type {
                        ChainIssueType::BrokenLink => {
                            // Remove broken links
                            if issue.position > 0 && (issue.position as usize) <= chain.chain.len() {
                                chain.chain.remove(issue.position as usize - 1);
                                chain.chain_length -= 1;
                                repaired_issues.push(issue);
                            }
                        }
                        ChainIssueType::MissingContent => {
                            // Can't repair missing content automatically
                            // Would need external intervention
                        }
                        _ => {}
                    }
                }
                IssueSeverity::High | IssueSeverity::Medium | IssueSeverity::Low => {
                    // Handle non-critical issues
                    match issue.issue_type {
                        ChainIssueType::MetadataInconsistent => {
                            // Fix metadata inconsistencies
                            if issue.description.contains("Chain length mismatch") {
                                chain.chain_length = chain.chain.len() as u64 + 1;
                                repaired_issues.push(issue);
                            }
                        }
                        ChainIssueType::DuplicateContent => {
                            // Remove duplicate entries  
                            // This is complex and would need careful implementation
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Update head CID after repairs
        if let Some(last_link) = chain.chain.last() {
            chain.head_cid = last_link.successor_cid.clone();
        } else {
            chain.head_cid = chain.root_cid.clone();
        }
        
        chain.updated_at = Utc::now();
        
        Ok(repaired_issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{DocumentSuccessor, EditType, EditMetadata};
    
    fn create_test_cid(data: &str) -> Cid {
        // Simple test CID creation
        Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap()
    }
    
    async fn create_test_service_with_content() -> DefaultCidChainVerificationService {
        let root_cid = create_test_cid("root");
        let successor_cid = create_test_cid("successor");
        
        DefaultCidChainVerificationService::new()
            .with_mock_content(root_cid, b"root content".to_vec())
            .with_mock_content(successor_cid, b"successor content".to_vec())
    }
    
    #[tokio::test]
    async fn test_verify_valid_chain() {
        // US-024: Test verification of valid CID chain
        let service = create_test_service_with_content().await;
        let document_id = DocumentId::new();
        let root_cid = create_test_cid("root");
        
        let mut chain = CidChain::new(document_id, root_cid.clone());
        
        // Add a valid successor
        let successor_cid = create_test_cid("successor");
        let successor = DocumentSuccessor::new(
            chain.document_id.clone(),
            root_cid,
            successor_cid,
            EditType::DirectReplacement,
            uuid::Uuid::new_v4(),
        );
        
        chain.add_successor(successor).unwrap();
        
        let result = service.verify_chain(&chain).await;
        
        // Chain should be valid since both CIDs exist in mock storage
        assert!(result.is_valid);
        assert_eq!(result.links_verified, 1);
        assert!(result.verification_time_ms > 0);
    }
    
    #[tokio::test]
    async fn test_verify_chain_with_missing_content() {
        // US-024: Test verification with missing CID content
        let service = DefaultCidChainVerificationService::new();
        let document_id = DocumentId::new();
        let root_cid = create_test_cid("root");
        
        let chain = CidChain::new(document_id, root_cid);
        
        let result = service.verify_chain(&chain).await;
        
        // Chain should be invalid due to missing root content
        assert!(!result.is_valid);
        assert!(!result.issues.is_empty());
        
        let missing_content_issue = result.issues.iter()
            .find(|issue| matches!(issue.issue_type, ChainIssueType::MissingContent));
        assert!(missing_content_issue.is_some());
        assert!(matches!(
            missing_content_issue.unwrap().severity,
            IssueSeverity::Critical
        ));
    }
    
    #[tokio::test]
    async fn test_verify_chain_with_broken_link() {
        // US-024: Test verification with broken chain link
        let root_cid = create_test_cid("root");
        let wrong_cid = create_test_cid("wrong");
        let successor_cid = create_test_cid("successor");
        
        let service = DefaultCidChainVerificationService::new()
            .with_mock_content(root_cid.clone(), b"root content".to_vec())
            .with_mock_content(successor_cid.clone(), b"successor content".to_vec());
        
        let document_id = DocumentId::new();
        let mut chain = CidChain::new(document_id.clone(), root_cid);
        
        // Manually create a link with wrong predecessor
        use crate::value_objects::CidChainLink;
        let broken_link = CidChainLink {
            predecessor_cid: wrong_cid, // Wrong predecessor
            successor_cid,
            edit_type: EditType::DirectReplacement,
            created_at: Utc::now(),
            metadata_summary: None,
        };
        
        chain.chain.push(broken_link);
        chain.chain_length += 1;
        
        let result = service.verify_chain(&chain).await;
        
        // Chain should be invalid due to broken link
        assert!(!result.is_valid);
        
        let broken_link_issue = result.issues.iter()
            .find(|issue| matches!(issue.issue_type, ChainIssueType::BrokenLink));
        assert!(broken_link_issue.is_some());
        assert!(matches!(
            broken_link_issue.unwrap().severity,
            IssueSeverity::Critical
        ));
    }
    
    #[tokio::test]
    async fn test_check_cid_exists() {
        // US-024: Test CID existence check
        let cid = create_test_cid("test");
        let service = DefaultCidChainVerificationService::new()
            .with_mock_content(cid.clone(), b"test content".to_vec());
        
        // Existing CID should return true
        let exists = service.check_cid_exists(&cid).await.unwrap();
        assert!(exists);
        
        // Non-existing CID should return false
        let non_existing_cid = create_test_cid("nonexistent");
        let not_exists = service.check_cid_exists(&non_existing_cid).await.unwrap();
        assert!(!not_exists);
    }
    
    #[tokio::test]
    async fn test_verify_link() {
        // US-024: Test individual link verification
        let predecessor = create_test_cid("predecessor");
        let successor = create_test_cid("successor");
        
        let service = DefaultCidChainVerificationService::new()
            .with_mock_content(predecessor.clone(), b"predecessor content".to_vec())
            .with_mock_content(successor.clone(), b"successor content".to_vec());
        
        // Valid link should verify successfully
        let valid = service.verify_link(&predecessor, &successor).await.unwrap();
        assert!(valid);
        
        // Link with non-existing predecessor should fail
        let non_existing = create_test_cid("nonexistent");
        let result = service.verify_link(&non_existing, &successor).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ChainError::CidNotFound { .. }));
    }
    
    #[tokio::test]
    async fn test_get_content_size() {
        // US-024: Test content size retrieval
        let cid = create_test_cid("test");
        let content = b"test content with specific size";
        let expected_size = content.len() as u64;
        
        let service = DefaultCidChainVerificationService::new()
            .with_mock_content(cid.clone(), content.to_vec());
        
        let size = service.get_content_size(&cid).await.unwrap();
        assert_eq!(size, expected_size);
        
        // Non-existing CID should return error
        let non_existing = create_test_cid("nonexistent");
        let result = service.get_content_size(&non_existing).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_chain_repair_service() {
        // US-024: Test chain repair functionality
        let root_cid = create_test_cid("root");
        let verification_service = Box::new(
            DefaultCidChainVerificationService::new()
                .with_mock_content(root_cid.clone(), b"root content".to_vec())
        );
        
        let repair_service = CidChainRepairService::new(verification_service);
        let document_id = DocumentId::new();
        let mut chain = CidChain::new(document_id, root_cid);
        
        // Add metadata inconsistency
        chain.chain_length = 99; // Wrong length
        
        let repaired_issues = repair_service.repair_chain(&mut chain).await.unwrap();
        
        // Should have repaired the length issue
        assert!(!repaired_issues.is_empty());
        assert_eq!(chain.chain_length, 1); // Should be corrected to 1 (root only)
    }
    
    #[test]
    fn test_chain_issue_types() {
        // US-024: Test chain issue type variants
        let issues = vec![
            ChainIssueType::MissingContent,
            ChainIssueType::HashMismatch,
            ChainIssueType::BrokenLink,
            ChainIssueType::DuplicateContent,
            ChainIssueType::MetadataInconsistent,
        ];
        
        assert_eq!(issues.len(), 5);
        
        // Test serialization
        for issue_type in issues {
            let serialized = serde_json::to_string(&issue_type).unwrap();
            assert!(!serialized.is_empty());
        }
    }
    
    #[test]
    fn test_issue_severity_levels() {
        // US-024: Test issue severity ordering
        let severities = vec![
            IssueSeverity::Low,
            IssueSeverity::Medium,
            IssueSeverity::High,
            IssueSeverity::Critical,
        ];
        
        // Test that all severities are distinct
        for (i, severity) in severities.iter().enumerate() {
            for (j, other) in severities.iter().enumerate() {
                if i != j {
                    assert_ne!(severity, other);
                }
            }
        }
    }
}