//! Result Classification System
//!
//! Classifies analysis results into three categories for 100% accuracy:
//! - Certain: Results that are 100% correct (direct calls, known frameworks)
//! - Possible: All possible values, guaranteed to include the actual value
//! - Unknown: Cannot determine, user should provide information
//!
//! **Core Principle**: Technology must be rigorous. Never guess!
//! Better to say "Unknown" than give wrong information.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Confidence level of an analysis result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Confidence {
    /// 100% certain - direct call, known pattern, explicit assignment
    Certain,
    /// All possible values listed - conditional assignment, array index, multiple targets
    Possible,
    /// Cannot determine - external library, complex indirect call
    Unknown,
}

impl Confidence {
    /// Get display symbol for UI
    pub fn symbol(&self) -> &'static str {
        match self {
            Confidence::Certain => "✓",
            Confidence::Possible => "?",
            Confidence::Unknown => "❓",
        }
    }

    /// Get CSS class for styling
    pub fn css_class(&self) -> &'static str {
        match self {
            Confidence::Certain => "result-certain",
            Confidence::Possible => "result-possible",
            Confidence::Unknown => "result-unknown",
        }
    }

    /// Get arrow style for visualization
    pub fn arrow_style(&self) -> &'static str {
        match self {
            Confidence::Certain => "solid",
            Confidence::Possible => "dashed",
            Confidence::Unknown => "dotted",
        }
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::Certain => write!(f, "Certain"),
            Confidence::Possible => write!(f, "Possible"),
            Confidence::Unknown => write!(f, "Unknown"),
        }
    }
}

/// A classified call target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedTarget {
    /// Target function name
    pub name: String,
    /// Confidence level
    pub confidence: Confidence,
    /// Reason for this classification
    pub reason: String,
}

/// A classified call edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedEdge {
    /// Caller function
    pub caller: String,
    /// Call site (line number or expression)
    pub call_site: String,
    /// Possible targets with confidence
    pub targets: Vec<ClassifiedTarget>,
    /// Overall confidence (lowest of all targets)
    pub overall_confidence: Confidence,
}

impl ClassifiedEdge {
    /// Create a certain edge with a single target
    pub fn certain(caller: &str, call_site: &str, target: &str, reason: &str) -> Self {
        Self {
            caller: caller.to_string(),
            call_site: call_site.to_string(),
            targets: vec![ClassifiedTarget {
                name: target.to_string(),
                confidence: Confidence::Certain,
                reason: reason.to_string(),
            }],
            overall_confidence: Confidence::Certain,
        }
    }

    /// Create an edge with multiple possible targets
    pub fn possible(caller: &str, call_site: &str, targets: Vec<(&str, &str)>) -> Self {
        let classified_targets: Vec<_> = targets.into_iter()
            .map(|(name, reason)| ClassifiedTarget {
                name: name.to_string(),
                confidence: Confidence::Possible,
                reason: reason.to_string(),
            })
            .collect();

        Self {
            caller: caller.to_string(),
            call_site: call_site.to_string(),
            targets: classified_targets,
            overall_confidence: Confidence::Possible,
        }
    }

    /// Create an unknown edge
    pub fn unknown(caller: &str, call_site: &str, reason: &str) -> Self {
        Self {
            caller: caller.to_string(),
            call_site: call_site.to_string(),
            targets: vec![ClassifiedTarget {
                name: "<unknown>".to_string(),
                confidence: Confidence::Unknown,
                reason: reason.to_string(),
            }],
            overall_confidence: Confidence::Unknown,
        }
    }
}

/// Classification reason for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClassificationReason {
    /// Direct function call: foo()
    DirectCall,
    /// Known async mechanism: INIT_WORK, timer_setup, etc.
    KnownAsyncMechanism { mechanism: String },
    /// Ops table assignment: .read = my_read
    OpsTableAssignment { table_type: String },
    /// Variable assignment: fp = handler
    VariableAssignment,
    /// Conditional assignment: if (x) fp = a; else fp = b;
    ConditionalAssignment,
    /// Array index: handlers[i]()
    ArrayIndex,
    /// External library with no source
    ExternalLibrary { library: String },
    /// Complex indirect call that cannot be resolved
    ComplexIndirectCall,
    /// User provided annotation
    UserAnnotation,
}

impl ClassificationReason {
    /// Get default confidence for this reason
    pub fn default_confidence(&self) -> Confidence {
        match self {
            ClassificationReason::DirectCall => Confidence::Certain,
            ClassificationReason::KnownAsyncMechanism { .. } => Confidence::Certain,
            ClassificationReason::OpsTableAssignment { .. } => Confidence::Certain,
            ClassificationReason::VariableAssignment => Confidence::Certain,
            ClassificationReason::ConditionalAssignment => Confidence::Possible,
            ClassificationReason::ArrayIndex => Confidence::Possible,
            ClassificationReason::ExternalLibrary { .. } => Confidence::Unknown,
            ClassificationReason::ComplexIndirectCall => Confidence::Unknown,
            ClassificationReason::UserAnnotation => Confidence::Certain,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            ClassificationReason::DirectCall => "Direct function call".to_string(),
            ClassificationReason::KnownAsyncMechanism { mechanism } => {
                format!("Known async mechanism: {}", mechanism)
            }
            ClassificationReason::OpsTableAssignment { table_type } => {
                format!("Ops table assignment: {}", table_type)
            }
            ClassificationReason::VariableAssignment => "Variable assignment".to_string(),
            ClassificationReason::ConditionalAssignment => "Conditional assignment".to_string(),
            ClassificationReason::ArrayIndex => "Array index access".to_string(),
            ClassificationReason::ExternalLibrary { library } => {
                format!("External library: {}", library)
            }
            ClassificationReason::ComplexIndirectCall => "Complex indirect call".to_string(),
            ClassificationReason::UserAnnotation => "User annotation".to_string(),
        }
    }
}

/// Result classifier for call graph analysis
pub struct ResultClassifier {
    /// Known async mechanisms
    known_mechanisms: HashSet<String>,
    /// Known ops table types
    known_ops_types: HashSet<String>,
    /// User-provided annotations
    annotations: Vec<UserAnnotation>,
}

/// User annotation for unknown calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnnotation {
    /// Call site identifier
    pub call_site: String,
    /// User-specified targets
    pub targets: Vec<String>,
    /// Note from user
    pub note: Option<String>,
}

impl ResultClassifier {
    pub fn new() -> Self {
        let mut known_mechanisms = HashSet::new();
        // Linux kernel async mechanisms
        known_mechanisms.insert("INIT_WORK".to_string());
        known_mechanisms.insert("INIT_DELAYED_WORK".to_string());
        known_mechanisms.insert("timer_setup".to_string());
        known_mechanisms.insert("setup_timer".to_string());
        known_mechanisms.insert("tasklet_init".to_string());
        known_mechanisms.insert("request_irq".to_string());
        known_mechanisms.insert("request_threaded_irq".to_string());
        known_mechanisms.insert("kthread_create".to_string());

        let mut known_ops_types = HashSet::new();
        known_ops_types.insert("file_operations".to_string());
        known_ops_types.insert("usb_driver".to_string());
        known_ops_types.insert("i2c_driver".to_string());
        known_ops_types.insert("platform_driver".to_string());
        known_ops_types.insert("pci_driver".to_string());

        Self {
            known_mechanisms,
            known_ops_types,
            annotations: Vec::new(),
        }
    }

    /// Add user annotation
    pub fn add_annotation(&mut self, annotation: UserAnnotation) {
        self.annotations.push(annotation);
    }

    /// Classify a direct function call
    pub fn classify_direct_call(&self, caller: &str, target: &str, line: u32) -> ClassifiedEdge {
        ClassifiedEdge::certain(
            caller,
            &format!("L{}", line),
            target,
            "Direct function call",
        )
    }

    /// Classify an async binding
    pub fn classify_async_binding(
        &self,
        caller: &str,
        mechanism: &str,
        handler: &str,
        line: u32,
    ) -> ClassifiedEdge {
        let confidence = if self.known_mechanisms.contains(mechanism) {
            Confidence::Certain
        } else {
            Confidence::Possible
        };

        ClassifiedEdge {
            caller: caller.to_string(),
            call_site: format!("L{}", line),
            targets: vec![ClassifiedTarget {
                name: handler.to_string(),
                confidence,
                reason: format!("Async binding via {}", mechanism),
            }],
            overall_confidence: confidence,
        }
    }

    /// Classify a function pointer call with resolved targets
    pub fn classify_funcptr_call(
        &self,
        caller: &str,
        expr: &str,
        targets: &[String],
        line: u32,
    ) -> ClassifiedEdge {
        if targets.is_empty() {
            return ClassifiedEdge::unknown(
                caller,
                &format!("L{}: {}", line, expr),
                "No targets resolved for function pointer",
            );
        }

        if targets.len() == 1 {
            return ClassifiedEdge::certain(
                caller,
                &format!("L{}: {}", line, expr),
                &targets[0],
                "Single resolved function pointer target",
            );
        }

        // Multiple targets - classified as Possible
        let target_list: Vec<_> = targets.iter()
            .map(|t| (t.as_str(), "Possible function pointer target"))
            .collect();

        ClassifiedEdge::possible(caller, &format!("L{}: {}", line, expr), target_list)
    }

    /// Classify an ops table callback
    pub fn classify_ops_callback(
        &self,
        ops_type: &str,
        field: &str,
        handler: &str,
    ) -> ClassifiedEdge {
        let confidence = if self.known_ops_types.contains(ops_type) {
            Confidence::Certain
        } else {
            Confidence::Possible
        };

        ClassifiedEdge {
            caller: format!("{}.{}", ops_type, field),
            call_site: format!("{}.{}", ops_type, field),
            targets: vec![ClassifiedTarget {
                name: handler.to_string(),
                confidence,
                reason: format!("Ops table callback: {}.{}", ops_type, field),
            }],
            overall_confidence: confidence,
        }
    }

    /// Check for user annotation and apply it
    pub fn apply_annotation(&self, call_site: &str) -> Option<ClassifiedEdge> {
        self.annotations.iter()
            .find(|a| a.call_site == call_site)
            .map(|a| {
                let targets: Vec<_> = a.targets.iter()
                    .map(|t| ClassifiedTarget {
                        name: t.clone(),
                        confidence: Confidence::Certain,
                        reason: a.note.clone().unwrap_or_else(|| "User annotation".to_string()),
                    })
                    .collect();

                ClassifiedEdge {
                    caller: String::new(), // Will be filled by caller
                    call_site: call_site.to_string(),
                    targets,
                    overall_confidence: Confidence::Certain,
                }
            })
    }

    /// Get summary statistics
    pub fn summarize(&self, edges: &[ClassifiedEdge]) -> ClassificationSummary {
        let mut certain_count = 0;
        let mut possible_count = 0;
        let mut unknown_count = 0;

        for edge in edges {
            match edge.overall_confidence {
                Confidence::Certain => certain_count += 1,
                Confidence::Possible => possible_count += 1,
                Confidence::Unknown => unknown_count += 1,
            }
        }

        let total = edges.len();
        ClassificationSummary {
            total,
            certain_count,
            possible_count,
            unknown_count,
            certain_percentage: if total > 0 { certain_count * 100 / total } else { 0 },
        }
    }
}

impl Default for ResultClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of classification results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationSummary {
    /// Total number of edges
    pub total: usize,
    /// Number of certain edges
    pub certain_count: usize,
    /// Number of possible edges
    pub possible_count: usize,
    /// Number of unknown edges
    pub unknown_count: usize,
    /// Percentage of certain results
    pub certain_percentage: usize,
}

impl std::fmt::Display for ClassificationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Analysis: {} edges ({} certain [{}%], {} possible, {} unknown)",
            self.total,
            self.certain_count,
            self.certain_percentage,
            self.possible_count,
            self.unknown_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_display() {
        assert_eq!(Confidence::Certain.symbol(), "✓");
        assert_eq!(Confidence::Possible.symbol(), "?");
        assert_eq!(Confidence::Unknown.symbol(), "❓");
    }

    #[test]
    fn test_classify_direct_call() {
        let classifier = ResultClassifier::new();
        let edge = classifier.classify_direct_call("main", "helper", 10);

        assert_eq!(edge.overall_confidence, Confidence::Certain);
        assert_eq!(edge.targets.len(), 1);
        assert_eq!(edge.targets[0].name, "helper");
    }

    #[test]
    fn test_classify_async_binding() {
        let classifier = ResultClassifier::new();

        // Known mechanism
        let edge = classifier.classify_async_binding("probe", "INIT_WORK", "work_handler", 20);
        assert_eq!(edge.overall_confidence, Confidence::Certain);

        // Unknown mechanism
        let edge = classifier.classify_async_binding("probe", "UNKNOWN_ASYNC", "handler", 20);
        assert_eq!(edge.overall_confidence, Confidence::Possible);
    }

    #[test]
    fn test_classify_funcptr_call() {
        let classifier = ResultClassifier::new();

        // No targets - unknown
        let edge = classifier.classify_funcptr_call("caller", "fp()", &[], 10);
        assert_eq!(edge.overall_confidence, Confidence::Unknown);

        // Single target - certain
        let edge = classifier.classify_funcptr_call("caller", "fp()", &["handler".to_string()], 10);
        assert_eq!(edge.overall_confidence, Confidence::Certain);

        // Multiple targets - possible
        let targets = vec!["handler1".to_string(), "handler2".to_string()];
        let edge = classifier.classify_funcptr_call("caller", "fp()", &targets, 10);
        assert_eq!(edge.overall_confidence, Confidence::Possible);
        assert_eq!(edge.targets.len(), 2);
    }

    #[test]
    fn test_classify_ops_callback() {
        let classifier = ResultClassifier::new();

        // Known ops type
        let edge = classifier.classify_ops_callback("file_operations", "read", "my_read");
        assert_eq!(edge.overall_confidence, Confidence::Certain);

        // Unknown ops type
        let edge = classifier.classify_ops_callback("custom_ops", "callback", "handler");
        assert_eq!(edge.overall_confidence, Confidence::Possible);
    }

    #[test]
    fn test_user_annotation() {
        let mut classifier = ResultClassifier::new();

        classifier.add_annotation(UserAnnotation {
            call_site: "L100: fp()".to_string(),
            targets: vec!["my_handler".to_string()],
            note: Some("Known from runtime trace".to_string()),
        });

        let result = classifier.apply_annotation("L100: fp()");
        assert!(result.is_some());

        let edge = result.unwrap();
        assert_eq!(edge.overall_confidence, Confidence::Certain);
        assert_eq!(edge.targets[0].name, "my_handler");
    }

    #[test]
    fn test_summary() {
        let classifier = ResultClassifier::new();
        let edges = vec![
            ClassifiedEdge::certain("a", "L1", "b", "test"),
            ClassifiedEdge::certain("a", "L2", "c", "test"),
            ClassifiedEdge::possible("a", "L3", vec![("d", "test"), ("e", "test")]),
            ClassifiedEdge::unknown("a", "L4", "test"),
        ];

        let summary = classifier.summarize(&edges);
        assert_eq!(summary.total, 4);
        assert_eq!(summary.certain_count, 2);
        assert_eq!(summary.possible_count, 1);
        assert_eq!(summary.unknown_count, 1);
        assert_eq!(summary.certain_percentage, 50);
    }
}
