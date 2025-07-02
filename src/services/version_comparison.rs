//! Version comparison service

use crate::value_objects::{DocumentId, DocumentVersion};
use crate::projections::DocumentFullView;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Version comparison service
pub struct VersionComparisonService;

impl VersionComparisonService {
    /// Compare two document versions
    pub fn compare_versions(
        version_a: &DocumentFullView,
        version_b: &DocumentFullView,
        options: &ComparisonOptions,
    ) -> Result<ComparisonResult> {
        if version_a.id != version_b.id {
            return Err(anyhow!("Cannot compare versions of different documents"));
        }

        let mut result = ComparisonResult {
            document_id: version_a.id.clone(),
            version_a: version_a.version.clone(),
            version_b: version_b.version.clone(),
            content_changes: Vec::new(),
            metadata_changes: HashMap::new(),
            statistics: ComparisonStatistics::default(),
        };

        // Compare content
        let content_diff = Self::compare_content(&version_a.content, &version_b.content, options)?;
        result.content_changes = content_diff.changes;
        result.statistics = content_diff.statistics;

        // Compare metadata if requested
        if options.include_metadata {
            result.metadata_changes = Self::compare_metadata(&version_a.metadata, &version_b.metadata);
        }

        Ok(result)
    }

    /// Compare content blocks
    fn compare_content(
        content_a: &str,
        content_b: &str,
        options: &ComparisonOptions,
    ) -> Result<ContentDiff> {
        let lines_a: Vec<&str> = content_a.lines().collect();
        let lines_b: Vec<&str> = content_b.lines().collect();

        let changes = match options.algorithm {
            DiffAlgorithm::Myers => Self::myers_diff(&lines_a, &lines_b),
            DiffAlgorithm::Patience => Self::patience_diff(&lines_a, &lines_b),
            DiffAlgorithm::Histogram => Self::histogram_diff(&lines_a, &lines_b),
        };

        let statistics = Self::calculate_statistics(&changes);

        Ok(ContentDiff {
            changes,
            statistics,
        })
    }

    /// Myers diff algorithm (simplified)
    fn myers_diff(lines_a: &[&str], lines_b: &[&str]) -> Vec<Change> {
        // Use a simple LCS-based approach for now
        
        // Build LCS table
        let m = lines_a.len();
        let n = lines_b.len();
        let mut lcs = vec![vec![0; n + 1]; m + 1];
        
        for i in 1..=m {
            for j in 1..=n {
                if lines_a[i - 1] == lines_b[j - 1] {
                    lcs[i][j] = lcs[i - 1][j - 1] + 1;
                } else {
                    lcs[i][j] = lcs[i - 1][j].max(lcs[i][j - 1]);
                }
            }
        }
        
        // Backtrack to find changes
        let mut i = m;
        let mut j = n;
        let mut result = Vec::new();
        
        while i > 0 || j > 0 {
            if i > 0 && j > 0 && lines_a[i - 1] == lines_b[j - 1] {
                result.push(Change::Equal {
                    line: lines_a[i - 1].to_string(),
                    line_number_a: i,
                    line_number_b: j,
                });
                i -= 1;
                j -= 1;
            } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
                result.push(Change::Added {
                    line: lines_b[j - 1].to_string(),
                    line_number: j,
                });
                j -= 1;
            } else if i > 0 {
                result.push(Change::Deleted {
                    line: lines_a[i - 1].to_string(),
                    line_number: i,
                });
                i -= 1;
            }
        }
        
        // Reverse to get correct order
        result.reverse();
        result
    }

    /// Patience diff algorithm (placeholder)
    fn patience_diff(lines_a: &[&str], lines_b: &[&str]) -> Vec<Change> {
        // For now, fall back to Myers
        Self::myers_diff(lines_a, lines_b)
    }

    /// Histogram diff algorithm (placeholder)
    fn histogram_diff(lines_a: &[&str], lines_b: &[&str]) -> Vec<Change> {
        // For now, fall back to Myers
        Self::myers_diff(lines_a, lines_b)
    }

    /// Compare metadata
    fn compare_metadata(
        metadata_a: &HashMap<String, String>,
        metadata_b: &HashMap<String, String>,
    ) -> HashMap<String, MetadataChange> {
        let mut changes = HashMap::new();

        // Check for added/modified keys
        for (key, value_b) in metadata_b {
            match metadata_a.get(key) {
                Some(value_a) if value_a != value_b => {
                    changes.insert(key.clone(), MetadataChange::Modified {
                        old_value: value_a.clone(),
                        new_value: value_b.clone(),
                    });
                }
                None => {
                    changes.insert(key.clone(), MetadataChange::Added {
                        value: value_b.clone(),
                    });
                }
                _ => {} // No change
            }
        }

        // Check for removed keys
        for key in metadata_a.keys() {
            if !metadata_b.contains_key(key) {
                changes.insert(key.clone(), MetadataChange::Removed {
                    old_value: metadata_a[key].clone(),
                });
            }
        }

        changes
    }

    /// Calculate statistics
    fn calculate_statistics(changes: &[Change]) -> ComparisonStatistics {
        let mut stats = ComparisonStatistics::default();

        for change in changes {
            match change {
                Change::Added { .. } => stats.lines_added += 1,
                Change::Deleted { .. } => stats.lines_deleted += 1,
                Change::Equal { .. } => stats.lines_unchanged += 1,
            }
        }

        stats.total_changes = stats.lines_added + stats.lines_deleted;
        
        let total_lines = stats.lines_added + stats.lines_deleted + stats.lines_unchanged;
        if total_lines > 0 {
            stats.similarity_ratio = stats.lines_unchanged as f32 / total_lines as f32;
        }

        stats
    }
}

/// Comparison options
#[derive(Debug, Clone)]
pub struct ComparisonOptions {
    pub include_metadata: bool,
    pub include_formatting: bool,
    pub algorithm: DiffAlgorithm,
}

impl Default for ComparisonOptions {
    fn default() -> Self {
        Self {
            include_metadata: true,
            include_formatting: false,
            algorithm: DiffAlgorithm::Myers,
        }
    }
}

/// Diff algorithm
#[derive(Debug, Clone)]
pub enum DiffAlgorithm {
    Myers,
    Patience,
    Histogram,
}

/// Comparison result
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub document_id: DocumentId,
    pub version_a: DocumentVersion,
    pub version_b: DocumentVersion,
    pub content_changes: Vec<Change>,
    pub metadata_changes: HashMap<String, MetadataChange>,
    pub statistics: ComparisonStatistics,
}

/// Content diff
#[derive(Debug, Clone)]
struct ContentDiff {
    changes: Vec<Change>,
    statistics: ComparisonStatistics,
}

/// Change type
#[derive(Debug, Clone)]
pub enum Change {
    Added {
        line: String,
        line_number: usize,
    },
    Deleted {
        line: String,
        line_number: usize,
    },
    Equal {
        line: String,
        line_number_a: usize,
        line_number_b: usize,
    },
}

/// Metadata change
#[derive(Debug, Clone)]
pub enum MetadataChange {
    Added {
        value: String,
    },
    Removed {
        old_value: String,
    },
    Modified {
        old_value: String,
        new_value: String,
    },
}

/// Comparison statistics
#[derive(Debug, Clone, Default)]
pub struct ComparisonStatistics {
    pub lines_added: usize,
    pub lines_deleted: usize,
    pub lines_unchanged: usize,
    pub total_changes: usize,
    pub similarity_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let doc_a = DocumentFullView {
            id: DocumentId::new(),
            title: "Test Document".to_string(),
            content: "Line 1\nLine 2\nLine 3".to_string(),
            version: DocumentVersion::new(1, 0, 0),
            doc_type: crate::value_objects::DocumentType::Note,
            tags: vec![],
            author: uuid::Uuid::new_v4(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut doc_b = doc_a.clone();
        doc_b.version = DocumentVersion::new(1, 1, 0);
        doc_b.content = "Line 1\nLine 2 modified\nLine 3\nLine 4".to_string();

        let result = VersionComparisonService::compare_versions(
            &doc_a,
            &doc_b,
            &ComparisonOptions::default(),
        ).unwrap();

        assert_eq!(result.statistics.lines_added, 2); // "Line 2 modified" and "Line 4"
        assert_eq!(result.statistics.lines_deleted, 1); // "Line 2"
        assert_eq!(result.statistics.lines_unchanged, 2); // "Line 1" and "Line 3"
    }

    #[test]
    fn test_metadata_comparison() {
        let mut metadata_a = HashMap::new();
        metadata_a.insert("key1".to_string(), "value1".to_string());
        metadata_a.insert("key2".to_string(), "value2".to_string());

        let mut metadata_b = HashMap::new();
        metadata_b.insert("key1".to_string(), "value1_modified".to_string());
        metadata_b.insert("key3".to_string(), "value3".to_string());

        let changes = VersionComparisonService::compare_metadata(&metadata_a, &metadata_b);

        assert_eq!(changes.len(), 3);
        assert!(matches!(changes.get("key1"), Some(MetadataChange::Modified { .. })));
        assert!(matches!(changes.get("key2"), Some(MetadataChange::Removed { .. })));
        assert!(matches!(changes.get("key3"), Some(MetadataChange::Added { .. })));
    }
} 