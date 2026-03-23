//! Analysis Quality Score (AQS)
//!
//! Single scalar metric (0.0–1.0) measuring analysis depth and accuracy.
//!
//! ## Dimensions
//!
//! | Dimension | Weight | What it measures |
//! |-----------|--------|-----------------|
//! | Direct call resolution | 0.30 | % of call sites successfully resolved |
//! | Indirect call identification | 0.20 | % of function pointers resolved |
//! | Knowledge base coverage | 0.20 | % of kernel APIs covered by KB |
//! | Error path coverage | 0.15 | % of error paths detected |
//! | Cross-file resolution | 0.15 | % of external symbols resolved |

use flowsight_analysis::AnalysisResult;
use flowsight_cfg::{CfgBuilder, Reachability};
use flowsight_core::CallType;
use flowsight_knowledge::KnowledgeBase;
use flowsight_parser::ParseResult;
use serde::{Deserialize, Serialize};

/// Weights for each AQS dimension
const W_DIRECT_CALL: f64 = 0.30;
const W_INDIRECT_CALL: f64 = 0.20;
const W_KB_COVERAGE: f64 = 0.20;
const W_ERROR_PATH: f64 = 0.15;
const W_CROSS_FILE: f64 = 0.15;

/// Analysis Quality Score — the single scalar metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisQualityScore {
    /// Final weighted score (0.0–1.0)
    pub score: f64,
    /// Per-dimension breakdown
    pub dimensions: AqsDimensions,
    /// Summary statistics
    pub stats: AqsStats,
}

/// Per-dimension scores (each 0.0–1.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AqsDimensions {
    /// Direct call resolution rate
    pub direct_call_resolution: f64,
    /// Indirect call (function pointer / callback) resolution rate
    pub indirect_call_resolution: f64,
    /// Knowledge base API coverage rate
    pub kb_coverage: f64,
    /// Error path detection coverage rate
    pub error_path_coverage: f64,
    /// Cross-file symbol resolution rate
    pub cross_file_resolution: f64,
}

/// Raw counts behind the scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AqsStats {
    /// Total function calls found
    pub total_calls: usize,
    /// Direct calls successfully resolved
    pub direct_calls_resolved: usize,
    /// Indirect calls found
    pub indirect_calls_total: usize,
    /// Indirect calls successfully resolved (High/Medium confidence)
    pub indirect_calls_resolved: usize,
    /// Kernel API calls found
    pub kernel_api_calls: usize,
    /// Kernel API calls covered by knowledge base
    pub kernel_api_covered: usize,
    /// Total functions with potential error paths
    pub functions_with_error_potential: usize,
    /// Error paths actually detected
    pub error_paths_detected: usize,
    /// External symbols referenced
    pub external_symbols_total: usize,
    /// External symbols resolved (found in current file)
    pub external_symbols_resolved: usize,
    /// Total functions analyzed
    pub functions_analyzed: usize,
}

impl AnalysisQualityScore {
    /// Compute AQS from analysis results
    ///
    /// Takes the raw analysis output and computes a single quality score.
    /// This is the primary entry point for quality measurement.
    pub fn compute(
        source: &str,
        parse_result: &ParseResult,
        analysis: &AnalysisResult,
        kb: &KnowledgeBase,
    ) -> Self {
        let stats = Self::collect_stats(source, parse_result, analysis, kb);
        let dimensions = Self::compute_dimensions(&stats);
        let score = Self::weighted_score(&dimensions);

        Self {
            score,
            dimensions,
            stats,
        }
    }

    /// Collect raw statistics from analysis output
    fn collect_stats(
        source: &str,
        parse_result: &ParseResult,
        analysis: &AnalysisResult,
        kb: &KnowledgeBase,
    ) -> AqsStats {
        let functions_analyzed = parse_result.functions.len();

        // --- Call resolution stats ---
        let mut direct_calls_resolved = 0usize;
        let mut indirect_calls_total = 0usize;
        let mut indirect_calls_resolved = 0usize;
        let total_calls = analysis.call_edges.len();

        for edge in &analysis.call_edges {
            match &edge.call_type {
                CallType::Direct => {
                    direct_calls_resolved += 1;
                }
                CallType::Indirect { confidence } => {
                    indirect_calls_total += 1;
                    match confidence {
                        flowsight_core::Confidence::High
                        | flowsight_core::Confidence::Medium => {
                            indirect_calls_resolved += 1;
                        }
                        flowsight_core::Confidence::Low => {}
                    }
                }
                CallType::Async { .. } => {
                    // Async calls count as resolved indirect calls
                    indirect_calls_total += 1;
                    indirect_calls_resolved += 1;
                }
            }
        }

        // --- KB coverage stats ---
        // Collect all unique callee names that are NOT defined in the current file
        let defined_functions: std::collections::HashSet<&str> =
            parse_result.functions.keys().map(|s| s.as_str()).collect();

        let mut kernel_api_calls = 0usize;
        let mut kernel_api_covered = 0usize;
        let mut external_symbols: std::collections::HashSet<&str> =
            std::collections::HashSet::new();
        let mut external_resolved: std::collections::HashSet<&str> =
            std::collections::HashSet::new();

        for edge in &analysis.call_edges {
            let callee = edge.callee.as_str();
            if !defined_functions.contains(callee) {
                // External symbol (kernel API or cross-file)
                external_symbols.insert(callee);

                // Check if KB knows about it
                if kb.get_api(callee).is_some() {
                    kernel_api_calls += 1;
                    kernel_api_covered += 1;
                    external_resolved.insert(callee);
                } else if kb.identify_callback(callee, "").is_some() {
                    kernel_api_calls += 1;
                    kernel_api_covered += 1;
                    external_resolved.insert(callee);
                } else {
                    kernel_api_calls += 1;
                    // Not covered by KB
                }
            }
        }

        // --- Error path stats ---
        let mut functions_with_error_potential = 0usize;
        let mut error_paths_detected = 0usize;

        // Use CFG builder to detect error paths per function
        let cfg_builder = CfgBuilder::new();
        for (func_name, _func_def) in &parse_result.functions {
            if let Ok(cfg) = cfg_builder.build_function_cfg(source, func_name) {
                // Any function with a return statement could have error paths
                let has_error_potential = cfg.blocks.iter().any(|block| {
                    block.calls.iter().any(|call| {
                        // Functions that call APIs which can fail
                        if let Some(api) = kb.get_api(&call.callee) {
                            api.can_fail
                        } else {
                            false
                        }
                    })
                });

                if has_error_potential {
                    functions_with_error_potential += 1;
                }

                // Count detected error paths
                error_paths_detected += cfg.error_paths.len();

                // Also count error-path reachability annotations
                for block in &cfg.blocks {
                    for call in &block.calls {
                        if call.reachability == Reachability::ErrorPath {
                            // Already counted via error_paths
                        }
                    }
                }
            }
        }

        AqsStats {
            total_calls,
            direct_calls_resolved,
            indirect_calls_total,
            indirect_calls_resolved,
            kernel_api_calls,
            kernel_api_covered,
            functions_with_error_potential,
            error_paths_detected,
            external_symbols_total: external_symbols.len(),
            external_symbols_resolved: external_resolved.len(),
            functions_analyzed,
        }
    }

    /// Convert raw stats into dimension scores (0.0–1.0)
    fn compute_dimensions(stats: &AqsStats) -> AqsDimensions {
        let direct_call_resolution = if stats.total_calls > 0 {
            stats.direct_calls_resolved as f64 / stats.total_calls as f64
        } else {
            1.0 // No calls = nothing to resolve = perfect
        };

        let indirect_call_resolution = if stats.indirect_calls_total > 0 {
            stats.indirect_calls_resolved as f64 / stats.indirect_calls_total as f64
        } else {
            1.0 // No indirect calls = nothing to resolve
        };

        let kb_coverage = if stats.kernel_api_calls > 0 {
            stats.kernel_api_covered as f64 / stats.kernel_api_calls as f64
        } else {
            1.0 // No kernel API calls = fully covered
        };

        let error_path_coverage = if stats.functions_with_error_potential > 0 {
            // Ratio of detected error paths to functions that could have them
            // Cap at 1.0 since a function can have multiple error paths
            (stats.error_paths_detected as f64 / stats.functions_with_error_potential as f64)
                .min(1.0)
        } else {
            1.0 // No error-prone functions = nothing to detect
        };

        let cross_file_resolution = if stats.external_symbols_total > 0 {
            stats.external_symbols_resolved as f64 / stats.external_symbols_total as f64
        } else {
            1.0 // No external symbols = fully resolved
        };

        AqsDimensions {
            direct_call_resolution,
            indirect_call_resolution,
            kb_coverage,
            error_path_coverage,
            cross_file_resolution,
        }
    }

    /// Compute weighted score from dimensions
    fn weighted_score(dims: &AqsDimensions) -> f64 {
        let raw = dims.direct_call_resolution * W_DIRECT_CALL
            + dims.indirect_call_resolution * W_INDIRECT_CALL
            + dims.kb_coverage * W_KB_COVERAGE
            + dims.error_path_coverage * W_ERROR_PATH
            + dims.cross_file_resolution * W_CROSS_FILE;

        // Clamp to [0.0, 1.0]
        raw.clamp(0.0, 1.0)
    }
}

/// Aggregate multiple file AQS scores into a directory-level score
pub fn aggregate_scores(scores: &[AnalysisQualityScore]) -> AnalysisQualityScore {
    if scores.is_empty() {
        return AnalysisQualityScore {
            score: 0.0,
            dimensions: AqsDimensions {
                direct_call_resolution: 0.0,
                indirect_call_resolution: 0.0,
                kb_coverage: 0.0,
                error_path_coverage: 0.0,
                cross_file_resolution: 0.0,
            },
            stats: AqsStats {
                total_calls: 0,
                direct_calls_resolved: 0,
                indirect_calls_total: 0,
                indirect_calls_resolved: 0,
                kernel_api_calls: 0,
                kernel_api_covered: 0,
                functions_with_error_potential: 0,
                error_paths_detected: 0,
                external_symbols_total: 0,
                external_symbols_resolved: 0,
                functions_analyzed: 0,
            },
        };
    }

    // Sum all raw stats
    let stats = scores.iter().fold(
        AqsStats {
            total_calls: 0,
            direct_calls_resolved: 0,
            indirect_calls_total: 0,
            indirect_calls_resolved: 0,
            kernel_api_calls: 0,
            kernel_api_covered: 0,
            functions_with_error_potential: 0,
            error_paths_detected: 0,
            external_symbols_total: 0,
            external_symbols_resolved: 0,
            functions_analyzed: 0,
        },
        |mut acc, s| {
            acc.total_calls += s.stats.total_calls;
            acc.direct_calls_resolved += s.stats.direct_calls_resolved;
            acc.indirect_calls_total += s.stats.indirect_calls_total;
            acc.indirect_calls_resolved += s.stats.indirect_calls_resolved;
            acc.kernel_api_calls += s.stats.kernel_api_calls;
            acc.kernel_api_covered += s.stats.kernel_api_covered;
            acc.functions_with_error_potential += s.stats.functions_with_error_potential;
            acc.error_paths_detected += s.stats.error_paths_detected;
            acc.external_symbols_total += s.stats.external_symbols_total;
            acc.external_symbols_resolved += s.stats.external_symbols_resolved;
            acc.functions_analyzed += s.stats.functions_analyzed;
            acc
        },
    );

    // Recompute dimensions from aggregated stats
    let dimensions = AnalysisQualityScore::compute_dimensions(&stats);
    let score = AnalysisQualityScore::weighted_score(&dimensions);

    AnalysisQualityScore {
        score,
        dimensions,
        stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_score_all_perfect() {
        let dims = AqsDimensions {
            direct_call_resolution: 1.0,
            indirect_call_resolution: 1.0,
            kb_coverage: 1.0,
            error_path_coverage: 1.0,
            cross_file_resolution: 1.0,
        };
        let score = AnalysisQualityScore::weighted_score(&dims);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_weighted_score_all_zero() {
        let dims = AqsDimensions {
            direct_call_resolution: 0.0,
            indirect_call_resolution: 0.0,
            kb_coverage: 0.0,
            error_path_coverage: 0.0,
            cross_file_resolution: 0.0,
        };
        let score = AnalysisQualityScore::weighted_score(&dims);
        assert!(score.abs() < f64::EPSILON);
    }

    #[test]
    fn test_weighted_score_weights_sum_to_one() {
        let total = W_DIRECT_CALL + W_INDIRECT_CALL + W_KB_COVERAGE + W_ERROR_PATH + W_CROSS_FILE;
        assert!((total - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dimensions_empty_stats() {
        let stats = AqsStats {
            total_calls: 0,
            direct_calls_resolved: 0,
            indirect_calls_total: 0,
            indirect_calls_resolved: 0,
            kernel_api_calls: 0,
            kernel_api_covered: 0,
            functions_with_error_potential: 0,
            error_paths_detected: 0,
            external_symbols_total: 0,
            external_symbols_resolved: 0,
            functions_analyzed: 0,
        };
        let dims = AnalysisQualityScore::compute_dimensions(&stats);
        // All dimensions default to 1.0 when there's nothing to measure
        assert!((dims.direct_call_resolution - 1.0).abs() < f64::EPSILON);
        assert!((dims.indirect_call_resolution - 1.0).abs() < f64::EPSILON);
        assert!((dims.kb_coverage - 1.0).abs() < f64::EPSILON);
        assert!((dims.error_path_coverage - 1.0).abs() < f64::EPSILON);
        assert!((dims.cross_file_resolution - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dimensions_partial_resolution() {
        let stats = AqsStats {
            total_calls: 10,
            direct_calls_resolved: 7,
            indirect_calls_total: 4,
            indirect_calls_resolved: 2,
            kernel_api_calls: 5,
            kernel_api_covered: 3,
            functions_with_error_potential: 3,
            error_paths_detected: 2,
            external_symbols_total: 8,
            external_symbols_resolved: 4,
            functions_analyzed: 5,
        };
        let dims = AnalysisQualityScore::compute_dimensions(&stats);

        assert!((dims.direct_call_resolution - 0.7).abs() < f64::EPSILON);
        assert!((dims.indirect_call_resolution - 0.5).abs() < f64::EPSILON);
        assert!((dims.kb_coverage - 0.6).abs() < f64::EPSILON);
        assert!((dims.error_path_coverage - 2.0 / 3.0).abs() < f64::EPSILON);
        assert!((dims.cross_file_resolution - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_aggregate_empty() {
        let agg = aggregate_scores(&[]);
        assert!(agg.score.abs() < f64::EPSILON);
        assert_eq!(agg.stats.functions_analyzed, 0);
    }

    #[test]
    fn test_aggregate_sums_stats() {
        let s1 = AnalysisQualityScore {
            score: 0.5,
            dimensions: AqsDimensions {
                direct_call_resolution: 0.5,
                indirect_call_resolution: 0.5,
                kb_coverage: 0.5,
                error_path_coverage: 0.5,
                cross_file_resolution: 0.5,
            },
            stats: AqsStats {
                total_calls: 10,
                direct_calls_resolved: 5,
                indirect_calls_total: 4,
                indirect_calls_resolved: 2,
                kernel_api_calls: 6,
                kernel_api_covered: 3,
                functions_with_error_potential: 2,
                error_paths_detected: 1,
                external_symbols_total: 4,
                external_symbols_resolved: 2,
                functions_analyzed: 3,
            },
        };
        let s2 = AnalysisQualityScore {
            score: 0.8,
            dimensions: AqsDimensions {
                direct_call_resolution: 0.8,
                indirect_call_resolution: 0.8,
                kb_coverage: 0.8,
                error_path_coverage: 0.8,
                cross_file_resolution: 0.8,
            },
            stats: AqsStats {
                total_calls: 20,
                direct_calls_resolved: 16,
                indirect_calls_total: 6,
                indirect_calls_resolved: 5,
                kernel_api_calls: 10,
                kernel_api_covered: 8,
                functions_with_error_potential: 4,
                error_paths_detected: 3,
                external_symbols_total: 8,
                external_symbols_resolved: 6,
                functions_analyzed: 7,
            },
        };

        let agg = aggregate_scores(&[s1, s2]);
        assert_eq!(agg.stats.total_calls, 30);
        assert_eq!(agg.stats.direct_calls_resolved, 21);
        assert_eq!(agg.stats.functions_analyzed, 10);
        // Score is recomputed from aggregated stats, not averaged
        assert!(agg.score > 0.0 && agg.score <= 1.0);
    }

    #[test]
    fn test_real_kernel_file() {
        // Integration test with real kernel code
        let kernel_file =
            std::path::Path::new("/Users/sky/linux-kernel/linux/arch/arm/mach-imx/clk-imx6q.c");
        if !kernel_file.exists() {
            return; // Skip on CI
        }

        let source = std::fs::read_to_string(kernel_file).unwrap();
        let parser = flowsight_parser::get_parser();
        let filename = kernel_file.to_string_lossy();
        let mut parse_result = parser.parse(&source, &filename).unwrap();

        let mut analyzer = flowsight_analysis::Analyzer::new();
        let analysis = analyzer.analyze(&source, &mut parse_result).unwrap();
        let kb = flowsight_knowledge::KnowledgeBase::builtin();

        let aqs = AnalysisQualityScore::compute(&source, &parse_result, &analysis, &kb);

        // Sanity checks
        assert!(aqs.score >= 0.0 && aqs.score <= 1.0);
        assert!(aqs.stats.functions_analyzed > 0);
        assert!(aqs.stats.total_calls > 0);

        // clk-imx6q.c should have decent direct call resolution
        assert!(aqs.dimensions.direct_call_resolution > 0.3);
    }
}
