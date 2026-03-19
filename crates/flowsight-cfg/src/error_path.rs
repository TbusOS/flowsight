//! Error path detection for Linux kernel code
//!
//! Identifies and classifies error handling patterns:
//! - goto chain cleanup (goto err_xxx)
//! - Early return with error code (return -ENOMEM)
//! - IS_ERR / PTR_ERR patterns

use crate::types::*;

/// Detects and analyzes error handling paths in a CFG
pub struct ErrorPathDetector;

impl ErrorPathDetector {
    /// Analyze a CFG and enrich its error path information
    pub fn analyze(cfg: &mut ControlFlowGraph) {
        Self::detect_goto_chains(cfg);
        Self::propagate_error_reachability(cfg);
    }

    /// Detect goto-based cleanup chains
    ///
    /// Linux kernel pattern:
    /// ```c
    /// err_step3:
    ///     undo_step3();
    /// err_step2:
    ///     undo_step2();
    /// err_step1:
    ///     undo_step1();
    ///     return ret;
    /// ```
    fn detect_goto_chains(cfg: &mut ControlFlowGraph) {
        // Find sequences of error handler blocks connected by fall-through
        let error_handler_ids: Vec<BlockId> = cfg
            .blocks
            .iter()
            .filter(|b| b.block_type == BlockType::ErrorHandler)
            .map(|b| b.id)
            .collect();

        // For each error path that uses goto, collect ALL cleanup calls
        // by following the fall-through chain from the target label
        let mut updated_paths = Vec::new();

        for path in &cfg.error_paths {
            if let ErrorStrategy::GotoCleanup { label } = &path.strategy {
                // Find the target block
                if let Some((_, target_id)) = cfg.labels.iter().find(|(l, _)| l == label) {
                    let cleanup_calls = Self::collect_cleanup_chain(cfg, *target_id);
                    let error_code = Self::find_error_return_in_chain(cfg, *target_id);

                    updated_paths.push((
                        path.check_block,
                        ErrorPath {
                            check_expression: path.check_expression.clone(),
                            check_line: path.check_line,
                            check_block: path.check_block,
                            strategy: ErrorStrategy::CleanupAndReturn {
                                label: label.clone(),
                                error_code: error_code.unwrap_or_else(|| "error".to_string()),
                            },
                            cleanup_calls,
                            label: path.label.clone(),
                        },
                    ));
                }
            }
        }

        // Replace error paths with enriched versions
        for (check_block, new_path) in updated_paths {
            if let Some(existing) = cfg
                .error_paths
                .iter_mut()
                .find(|p| p.check_block == check_block)
            {
                *existing = new_path;
            }
        }

        // Mark error handler blocks' calls as error-path reachability
        for block_id in &error_handler_ids {
            if let Some(block) = cfg.block_mut(*block_id) {
                for call in &mut block.calls {
                    call.reachability = Reachability::ErrorPath;
                }
            }
        }
    }

    /// Collect all cleanup function calls by following fall-through from a block
    fn collect_cleanup_chain(cfg: &ControlFlowGraph, start_block: BlockId) -> Vec<String> {
        let mut calls = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut current = start_block;

        loop {
            if visited.contains(&current) {
                break;
            }
            visited.insert(current);

            if let Some(block) = cfg.block(current) {
                for call in &block.calls {
                    calls.push(call.callee.clone());
                }
            }

            // Follow fall-through edge
            let next = cfg
                .edges
                .iter()
                .find(|e| e.from == current && matches!(e.edge_type, EdgeType::FallThrough))
                .map(|e| e.to);

            match next {
                Some(n) => current = n,
                None => break,
            }
        }

        calls
    }

    /// Find the error return code in a cleanup chain
    fn find_error_return_in_chain(
        cfg: &ControlFlowGraph,
        start_block: BlockId,
    ) -> Option<String> {
        let mut visited = std::collections::HashSet::new();
        let mut current = start_block;

        loop {
            if visited.contains(&current) {
                break;
            }
            visited.insert(current);

            if let Some(block) = cfg.block(current) {
                for stmt in &block.statements {
                    if matches!(stmt.kind, StatementKind::Return) {
                        // Extract error code from return statement
                        if let Some(code) = Self::extract_error_code_from_return(&stmt.text) {
                            return Some(code);
                        }
                    }
                }
            }

            let next = cfg
                .edges
                .iter()
                .find(|e| e.from == current && matches!(e.edge_type, EdgeType::FallThrough))
                .map(|e| e.to);

            match next {
                Some(n) => current = n,
                None => break,
            }
        }

        None
    }

    /// Extract error code from a return statement text
    fn extract_error_code_from_return(text: &str) -> Option<String> {
        if let Some(pos) = text.find("-E") {
            let rest = &text[pos..];
            let end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                .unwrap_or(rest.len());
            return Some(rest[..end].to_string());
        }

        // return ret; return err; return rc;
        let trimmed = text.trim().trim_end_matches(';').trim();
        if let Some(rest) = trimmed.strip_prefix("return") {
            let var = rest.trim();
            if matches!(var, "ret" | "err" | "rc" | "status" | "result") {
                return Some(var.to_string());
            }
        }

        None
    }

    /// Propagate error reachability through the CFG
    ///
    /// If a block is only reachable from error conditions, mark all its calls
    /// as error-path calls.
    fn propagate_error_reachability(cfg: &mut ControlFlowGraph) {
        // Find blocks that are only reachable via error branches
        let error_only_blocks: Vec<BlockId> = cfg
            .blocks
            .iter()
            .filter(|b| b.block_type == BlockType::ErrorHandler)
            .map(|b| b.id)
            .collect();

        // Mark calls in error-only blocks
        for block_id in error_only_blocks {
            if let Some(block) = cfg.block_mut(block_id) {
                for call in &mut block.calls {
                    call.reachability = Reachability::ErrorPath;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::CfgBuilder;

    #[test]
    fn test_goto_chain_detection() {
        let source = r#"
int probe(struct device *dev) {
    int ret;

    ret = step1();
    if (ret < 0)
        goto err_step1;

    ret = step2();
    if (ret < 0)
        goto err_step2;

    return 0;

err_step2:
    undo_step2();
err_step1:
    undo_step1();
    return ret;
}
"#;
        let builder = CfgBuilder::new();
        let mut cfg = builder.build_function_cfg(source, "probe").unwrap();

        ErrorPathDetector::analyze(&mut cfg);

        // Should have error paths detected
        assert!(
            !cfg.error_paths.is_empty(),
            "Expected error paths after analysis"
        );

        // Error handler calls should be marked as error-path reachability
        let error_calls: Vec<&CallSite> = cfg.error_calls();
        let error_callee_names: Vec<&str> = error_calls.iter().map(|c| c.callee.as_str()).collect();
        assert!(
            error_callee_names.contains(&"undo_step1") || error_callee_names.contains(&"undo_step2"),
            "Expected cleanup calls in error paths, got: {:?}",
            error_callee_names
        );
    }
}
