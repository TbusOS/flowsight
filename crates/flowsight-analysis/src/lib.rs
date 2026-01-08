//! FlowSight Analysis Engine
//!
//! Provides advanced code analysis capabilities:
//! - Async mechanism tracking (work queues, timers, interrupts)
//! - Function pointer resolution
//! - Andersen-style pointer analysis
//! - Call graph construction
//! - Scenario-based symbolic execution
//! - Expression evaluation
//! - Data flow analysis

pub mod async_tracker;
pub mod callgraph;
pub mod constraint;
pub mod evaluator;
pub mod funcptr;
pub mod pointer;
pub mod propagation;
pub mod scenario;

use flowsight_core::{AsyncBinding, CallEdge, FlowNode, FunctionDef, Result};
use flowsight_parser::ParseResult;
use std::collections::HashMap;

/// Analysis result
#[derive(Debug, Default)]
pub struct AnalysisResult {
    /// Async bindings found
    pub async_bindings: Vec<AsyncBinding>,
    /// Call edges (including resolved function pointers)
    pub call_edges: Vec<CallEdge>,
    /// Entry points (callbacks, module init, etc.)
    pub entry_points: Vec<String>,
    /// Execution flow trees
    pub flow_trees: Vec<FlowNode>,
}

/// Main analyzer
pub struct Analyzer {
    async_tracker: async_tracker::AsyncTracker,
    funcptr_resolver: funcptr::FuncPtrResolver,
}

impl Analyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            async_tracker: async_tracker::AsyncTracker::new(),
            funcptr_resolver: funcptr::FuncPtrResolver::new(),
        }
    }

    /// Analyze parsed code
    pub fn analyze(
        &mut self,
        source: &str,
        parse_result: &mut ParseResult,
    ) -> Result<AnalysisResult> {
        let mut result = AnalysisResult::default();

        // Track async mechanisms
        result.async_bindings = self.async_tracker.analyze(source, &parse_result.functions);

        // Mark async handlers as callbacks
        for binding in &result.async_bindings {
            if let Some(func) = parse_result.functions.get_mut(&binding.handler) {
                func.is_callback = true;
                func.callback_context = Some(format!("async_{:?}", binding.mechanism));
            }
        }

        // Resolve function pointers from ops tables
        let ops_mappings = self
            .funcptr_resolver
            .analyze_ops_tables(source, &parse_result.functions);
        for (context, func_name) in &ops_mappings {
            if let Some(func) = parse_result.functions.get_mut(func_name) {
                func.is_callback = true;
                func.callback_context = Some(context.clone());
            }
        }

        // Find entry points
        result.entry_points = self.find_entry_points(source, &parse_result.functions);

        // Build call graph
        result.call_edges = callgraph::build_call_edges(parse_result, &result.async_bindings);

        // Build flow trees for entry points
        result.flow_trees =
            self.build_flow_trees(&result.entry_points, parse_result, &result.async_bindings);

        Ok(result)
    }

    fn find_entry_points(
        &self,
        source: &str,
        functions: &HashMap<String, FunctionDef>,
    ) -> Vec<String> {
        let mut entries = Vec::new();

        // module_init (always first)
        let init_re = regex::Regex::new(r"module_init\s*\(\s*(\w+)\s*\)").unwrap();
        if let Some(cap) = init_re.captures(source) {
            if let Some(name) = cap.get(1) {
                entries.push(name.as_str().to_string());
            }
        }

        // module_exit (always second)
        let exit_re = regex::Regex::new(r"module_exit\s*\(\s*(\w+)\s*\)").unwrap();
        if let Some(cap) = exit_re.captures(source) {
            if let Some(name) = cap.get(1) {
                entries.push(name.as_str().to_string());
            }
        }

        // All callback functions - collect and sort for stable order
        let mut callbacks: Vec<_> = functions
            .iter()
            .filter(|(name, func)| func.is_callback && !entries.contains(name))
            .map(|(name, func)| {
                // Sort by line number for consistent ordering
                let line = func.location.as_ref().map(|l| l.line).unwrap_or(u32::MAX);
                (name.clone(), line)
            })
            .collect();

        // Sort by line number (definition order in source file)
        callbacks.sort_by_key(|(_, line)| *line);

        for (name, _) in callbacks {
            entries.push(name);
        }

        entries
    }

    fn build_flow_trees(
        &self,
        entry_points: &[String],
        parse_result: &ParseResult,
        async_bindings: &[AsyncBinding],
    ) -> Vec<FlowNode> {
        entry_points
            .iter()
            .filter_map(|entry| {
                callgraph::build_flow_tree(
                    entry,
                    parse_result,
                    async_bindings,
                    &mut std::collections::HashSet::new(),
                    0,
                )
            })
            .collect()
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
