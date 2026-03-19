//! CFG builder — constructs control flow graph from tree-sitter AST
//!
//! Algorithm adapted from tree-climber (Joern CfgCreator):
//! 1. Walk function body AST
//! 2. Create basic blocks at control flow boundaries
//! 3. Connect blocks with typed edges
//! 4. Resolve goto labels
//! 5. Classify error paths

use crate::macro_semantics::{MacroSemantics, MacroTable};
use crate::types::*;
use tree_sitter::{Node, Parser};

/// Builds CFG from C source code using tree-sitter
pub struct CfgBuilder {
    macro_table: MacroTable,
}

/// Internal state during CFG construction
struct BuildState {
    cfg: ControlFlowGraph,
    /// Current block being populated
    current_block: BlockId,
    /// Pending goto targets: (label_name, source_block_id, source_line)
    pending_gotos: Vec<(String, BlockId, usize)>,
    /// Label → block mapping (filled as we encounter labels)
    label_blocks: Vec<(String, BlockId)>,
    /// Current reachability context (tracks if we're inside a branch)
    reachability_stack: Vec<Reachability>,
}

impl CfgBuilder {
    /// Create with default kernel macro table
    pub fn new() -> Self {
        Self {
            macro_table: MacroTable::kernel_defaults(),
        }
    }

    /// Create with custom macro table
    pub fn with_macro_table(macro_table: MacroTable) -> Self {
        Self { macro_table }
    }

    /// Build CFG for a single function in the source code
    pub fn build_function_cfg(
        &self,
        source: &str,
        function_name: &str,
    ) -> Result<ControlFlowGraph, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c::language())
            .map_err(|e| format!("Failed to set language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Failed to parse source".to_string())?;

        let root = tree.root_node();

        // Find the target function
        let func_node = self
            .find_function_node(root, source, function_name)
            .ok_or_else(|| format!("Function '{}' not found", function_name))?;

        self.build_cfg_from_function(func_node, source, function_name)
    }

    /// Build CFGs for all functions in the source code
    pub fn build_all_cfgs(&self, source: &str) -> Result<Vec<ControlFlowGraph>, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c::language())
            .map_err(|e| format!("Failed to set language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Failed to parse source".to_string())?;

        let root = tree.root_node();
        let mut cfgs = Vec::new();

        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "function_definition" {
                if let Some(name) = self.get_function_name(child, source) {
                    if let Ok(cfg) = self.build_cfg_from_function(child, source, &name) {
                        cfgs.push(cfg);
                    }
                }
            }
        }

        Ok(cfgs)
    }

    /// Build CFG from a function_definition AST node
    fn build_cfg_from_function(
        &self,
        func_node: Node,
        source: &str,
        function_name: &str,
    ) -> Result<ControlFlowGraph, String> {
        // Find the compound_statement (function body)
        let body = self
            .find_child_by_kind(func_node, "compound_statement")
            .ok_or_else(|| format!("No body found for function '{}'", function_name))?;

        let mut state = BuildState {
            cfg: ControlFlowGraph::new(function_name.to_string()),
            current_block: 0,
            pending_gotos: Vec::new(),
            label_blocks: Vec::new(),
            reachability_stack: vec![Reachability::Always],
        };

        // Set entry block line range
        let start_line = body.start_position().row + 1;
        if let Some(block) = state.cfg.block_mut(0) {
            block.line_range.0 = start_line;
        }

        // Walk the function body
        self.process_compound_statement(body, source, &mut state);

        // Create exit block if the last block doesn't end with return
        let last_block = state.current_block;
        let has_return = state
            .cfg
            .block(last_block)
            .map(|b| {
                b.statements
                    .iter()
                    .any(|s| matches!(s.kind, StatementKind::Return))
            })
            .unwrap_or(false);

        if !has_return {
            let exit_id = self.new_block(&mut state, BlockType::Exit);
            state.cfg.add_edge(last_block, exit_id, EdgeType::FallThrough);
            state.cfg.exits.push(exit_id);
        }

        // Resolve goto targets
        self.resolve_gotos(&mut state);

        // Store label mappings
        state.cfg.labels = state.label_blocks.clone();

        Ok(state.cfg)
    }

    /// Process a compound_statement (block of statements)
    fn process_compound_statement(&self, node: Node, source: &str, state: &mut BuildState) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "if_statement" => self.process_if(child, source, state),
                "while_statement" | "do_statement" => self.process_loop(child, source, state),
                "for_statement" => self.process_for(child, source, state),
                "switch_statement" => self.process_switch(child, source, state),
                "return_statement" => self.process_return(child, source, state),
                "goto_statement" => self.process_goto(child, source, state),
                "labeled_statement" => self.process_label(child, source, state),
                "expression_statement" => self.process_expression_statement(child, source, state),
                "declaration" => self.process_declaration(child, source, state),
                "compound_statement" => self.process_compound_statement(child, source, state),
                // Skip braces and other tokens
                "{" | "}" | ";" | "comment" | "preproc_ifdef" | "preproc_if" | "preproc_else"
                | "preproc_endif" | "preproc_include" | "preproc_def" | "preproc_call" => {}
                _ => {
                    // Other statement types — add as expression
                    self.add_statement(child, source, state, StatementKind::Expression);
                }
            }
        }
    }

    /// Process if/else statement
    fn process_if(&self, node: Node, source: &str, state: &mut BuildState) {
        let condition_node = self.find_child_by_kind(node, "parenthesized_expression");
        let condition_text = condition_node
            .map(|n| self.node_text(n, source))
            .unwrap_or_else(|| "?".to_string());

        // Remove outer parentheses for display
        let cond_display = condition_text
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&condition_text)
            .to_string();

        // Add condition as statement in current block
        self.add_statement(node, source, state, StatementKind::Condition);

        let pre_if_block = state.current_block;

        // Create merge block (where true/false paths rejoin)
        let merge_block = self.new_block(state, BlockType::Normal);

        // Process "consequence" (true branch)
        let true_block = self.new_block(state, BlockType::Normal);
        state.cfg.add_edge(
            pre_if_block,
            true_block,
            EdgeType::BranchTrue(cond_display.clone()),
        );
        state.current_block = true_block;

        // Push conditional reachability
        state
            .reachability_stack
            .push(Reachability::Conditional(cond_display.clone()));

        if let Some(consequence) = self.find_child_by_field(node, "consequence") {
            self.process_statement_or_block(consequence, source, state);
        }

        let true_end_block = state.current_block;
        // Connect true branch end to merge (if it doesn't end with return/goto)
        if !self.block_terminates(true_end_block, state) {
            state
                .cfg
                .add_edge(true_end_block, merge_block, EdgeType::FallThrough);
        }

        state.reachability_stack.pop();

        // Process "alternative" (else branch)
        if let Some(alternative) = self.find_child_by_field(node, "alternative") {
            let false_block = self.new_block(state, BlockType::Normal);
            state.cfg.add_edge(
                pre_if_block,
                false_block,
                EdgeType::BranchFalse(cond_display.clone()),
            );
            state.current_block = false_block;

            // Check if this is error path handling
            let is_error = self.is_error_condition(&cond_display);
            if is_error {
                state.reachability_stack.push(Reachability::ErrorPath);
            } else {
                state
                    .reachability_stack
                    .push(Reachability::Conditional(format!("!{}", cond_display)));
            }

            // else-if chain: the alternative might be another if_statement
            let inner = self.unwrap_else_clause(alternative);
            self.process_statement_or_block(inner, source, state);

            let false_end_block = state.current_block;
            if !self.block_terminates(false_end_block, state) {
                state
                    .cfg
                    .add_edge(false_end_block, merge_block, EdgeType::FallThrough);
            }

            state.reachability_stack.pop();
        } else {
            // No else branch — false path goes directly to merge
            state
                .cfg
                .add_edge(pre_if_block, merge_block, EdgeType::BranchFalse(cond_display));
        }

        state.current_block = merge_block;
    }

    /// Process while/do-while loop
    fn process_loop(&self, node: Node, source: &str, state: &mut BuildState) {
        let condition_node = self.find_child_by_kind(node, "parenthesized_expression");
        let condition_text = condition_node
            .map(|n| self.node_text(n, source))
            .unwrap_or_else(|| "?".to_string());

        let cond_display = condition_text
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&condition_text)
            .to_string();

        let pre_loop = state.current_block;

        // Loop header block (condition check)
        let header = self.new_block(state, BlockType::LoopHeader);
        state.cfg.add_edge(pre_loop, header, EdgeType::FallThrough);

        // Loop body block
        let body_block = self.new_block(state, BlockType::LoopBody);
        state.cfg.add_edge(
            header,
            body_block,
            EdgeType::BranchTrue(cond_display.clone()),
        );

        // Loop exit block
        let exit_block = self.new_block(state, BlockType::Normal);
        state
            .cfg
            .add_edge(header, exit_block, EdgeType::LoopExit);

        // Process body
        state.current_block = body_block;
        state
            .reachability_stack
            .push(Reachability::Conditional(format!("loop: {}", cond_display)));

        if let Some(body) = self.find_child_by_field(node, "body") {
            self.process_statement_or_block(body, source, state);
        }

        let body_end = state.current_block;
        if !self.block_terminates(body_end, state) {
            state.cfg.add_edge(body_end, header, EdgeType::LoopBack);
        }

        state.reachability_stack.pop();
        state.current_block = exit_block;
    }

    /// Process for loop
    fn process_for(&self, node: Node, source: &str, state: &mut BuildState) {
        // For loops are treated similarly to while loops
        // The initializer is in the current block, condition in loop header
        let pre_loop = state.current_block;

        let header = self.new_block(state, BlockType::LoopHeader);
        state.cfg.add_edge(pre_loop, header, EdgeType::FallThrough);

        let body_block = self.new_block(state, BlockType::LoopBody);
        state
            .cfg
            .add_edge(header, body_block, EdgeType::BranchTrue("for-cond".into()));

        let exit_block = self.new_block(state, BlockType::Normal);
        state
            .cfg
            .add_edge(header, exit_block, EdgeType::LoopExit);

        state.current_block = body_block;
        state
            .reachability_stack
            .push(Reachability::Conditional("for-loop".into()));

        if let Some(body) = self.find_child_by_field(node, "body") {
            self.process_statement_or_block(body, source, state);
        }

        let body_end = state.current_block;
        if !self.block_terminates(body_end, state) {
            state.cfg.add_edge(body_end, header, EdgeType::LoopBack);
        }

        state.reachability_stack.pop();
        state.current_block = exit_block;
    }

    /// Process switch statement
    fn process_switch(&self, node: Node, source: &str, state: &mut BuildState) {
        let dispatch_block = state.current_block;

        // Create merge block for after the switch
        let merge_block = self.new_block(state, BlockType::Normal);

        // Find the compound_statement body of the switch
        if let Some(body) = self.find_child_by_kind(node, "compound_statement") {
            let mut cursor = body.walk();
            let mut current_case_block: Option<BlockId> = None;

            for child in body.children(&mut cursor) {
                match child.kind() {
                    "case_statement" => {
                        // Get case value
                        let case_value = child
                            .child(1)
                            .map(|n| self.node_text(n, source))
                            .unwrap_or_else(|| "?".into());

                        let case_block = self.new_block(state, BlockType::Normal);
                        state.cfg.add_edge(
                            dispatch_block,
                            case_block,
                            EdgeType::SwitchCase(case_value),
                        );

                        // Fall-through from previous case (if no break)
                        if let Some(prev) = current_case_block {
                            if !self.block_terminates(prev, state) {
                                state
                                    .cfg
                                    .add_edge(prev, case_block, EdgeType::FallThrough);
                            }
                        }

                        state.current_block = case_block;
                        current_case_block = Some(case_block);

                        // Process statements within the case
                        let mut case_cursor = child.walk();
                        for case_child in child.children(&mut case_cursor) {
                            if case_child.kind() != "case" && case_child.kind() != ":" {
                                self.process_statement_or_block(case_child, source, state);
                                current_case_block = Some(state.current_block);
                            }
                        }
                    }
                    "default_statement" => {
                        let default_block = self.new_block(state, BlockType::Normal);
                        state
                            .cfg
                            .add_edge(dispatch_block, default_block, EdgeType::SwitchDefault);

                        if let Some(prev) = current_case_block {
                            if !self.block_terminates(prev, state) {
                                state
                                    .cfg
                                    .add_edge(prev, default_block, EdgeType::FallThrough);
                            }
                        }

                        state.current_block = default_block;
                        current_case_block = Some(default_block);
                    }
                    "break_statement" => {
                        if let Some(case) = current_case_block {
                            state.cfg.add_edge(case, merge_block, EdgeType::FallThrough);
                            current_case_block = None;
                        }
                    }
                    _ => {}
                }
            }

            // Last case without break falls through to merge
            if let Some(last_case) = current_case_block {
                if !self.block_terminates(last_case, state) {
                    state
                        .cfg
                        .add_edge(last_case, merge_block, EdgeType::FallThrough);
                }
            }
        }

        state.current_block = merge_block;
    }

    /// Process return statement
    fn process_return(&self, node: Node, source: &str, state: &mut BuildState) {
        let return_text = self.node_text(node, source);
        let line = node.start_position().row + 1;

        self.add_statement(node, source, state, StatementKind::Return);

        // Check if this is an error return
        let is_error_return = self.is_error_return_text(&return_text);

        let exit_id = self.new_block(
            state,
            if is_error_return {
                BlockType::ErrorHandler
            } else {
                BlockType::Exit
            },
        );

        state
            .cfg
            .add_edge(state.current_block, exit_id, EdgeType::Return);
        state.cfg.exits.push(exit_id);

        // If error return, record as error path
        if is_error_return {
            let error_code = self.extract_error_code(&return_text);
            state.cfg.error_paths.push(ErrorPath {
                check_expression: return_text,
                check_line: line,
                check_block: state.current_block,
                strategy: ErrorStrategy::EarlyReturn { error_code },
                cleanup_calls: vec![],
                label: None,
            });
        }

        // After return, start a new unreachable block (for code after return)
        state.current_block = self.new_block(state, BlockType::Normal);
    }

    /// Process goto statement
    fn process_goto(&self, node: Node, source: &str, state: &mut BuildState) {
        let label = self
            .find_child_by_kind(node, "statement_identifier")
            .map(|n| self.node_text(n, source))
            .unwrap_or_default();
        let line = node.start_position().row + 1;

        self.add_statement(
            node,
            source,
            state,
            StatementKind::Goto {
                label: label.clone(),
            },
        );

        // Record pending goto for later resolution
        state
            .pending_gotos
            .push((label.clone(), state.current_block, line));

        // After goto, start a new block (code after goto is typically unreachable
        // unless it's a label target)
        state.current_block = self.new_block(state, BlockType::Normal);
    }

    /// Process labeled_statement (goto target)
    fn process_label(&self, node: Node, source: &str, state: &mut BuildState) {
        // Get label name
        let label_name = self
            .find_child_by_kind(node, "statement_identifier")
            .map(|n| self.node_text(n, source))
            .unwrap_or_default();

        if label_name.is_empty() {
            return;
        }

        // Start a new block for this label
        let label_block = self.new_block(
            state,
            if is_error_label(&label_name) {
                BlockType::ErrorHandler
            } else {
                BlockType::Normal
            },
        );

        // Fall-through from previous block
        let prev = state.current_block;
        if !self.block_terminates(prev, state) {
            state
                .cfg
                .add_edge(prev, label_block, EdgeType::FallThrough);
        }

        // Set label on the block
        if let Some(block) = state.cfg.block_mut(label_block) {
            block.label = Some(label_name.clone());
            block.line_range.0 = node.start_position().row + 1;
        }

        // Record label → block mapping
        state.label_blocks.push((label_name, label_block));

        state.current_block = label_block;

        // Process the statement after the label
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // Skip the label identifier and colon
            if child.kind() != "statement_identifier" && child.kind() != ":" {
                self.process_statement_or_block(child, source, state);
            }
        }
    }

    /// Process expression statement (most common: function calls, assignments)
    fn process_expression_statement(&self, node: Node, source: &str, state: &mut BuildState) {
        let line = node.start_position().row + 1;
        let text = self.node_text(node, source);

        // Check for function calls in this expression
        self.extract_calls_from_node(node, source, state, line);

        // Classify the statement
        let kind = if text.contains('=') && !text.contains("==") {
            StatementKind::Assignment
        } else if self.has_call_expression(node) {
            StatementKind::Call
        } else {
            StatementKind::Expression
        };

        self.add_statement(node, source, state, kind);
    }

    /// Process declaration (variable declarations, sometimes with calls)
    fn process_declaration(&self, node: Node, source: &str, state: &mut BuildState) {
        let line = node.start_position().row + 1;

        // Check for calls in initializers (e.g., `int ret = kmalloc(...)`)
        self.extract_calls_from_node(node, source, state, line);

        self.add_statement(node, source, state, StatementKind::Assignment);
    }

    /// Process a node that could be either a compound_statement or a single statement
    fn process_statement_or_block(&self, node: Node, source: &str, state: &mut BuildState) {
        match node.kind() {
            "compound_statement" => self.process_compound_statement(node, source, state),
            "if_statement" => self.process_if(node, source, state),
            "while_statement" | "do_statement" => self.process_loop(node, source, state),
            "for_statement" => self.process_for(node, source, state),
            "switch_statement" => self.process_switch(node, source, state),
            "return_statement" => self.process_return(node, source, state),
            "goto_statement" => self.process_goto(node, source, state),
            "labeled_statement" => self.process_label(node, source, state),
            "expression_statement" => self.process_expression_statement(node, source, state),
            "declaration" => self.process_declaration(node, source, state),
            "break_statement" | "continue_statement" => {
                self.add_statement(node, source, state, StatementKind::Expression);
            }
            // else_clause wraps another statement
            "else_clause" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "else" {
                        self.process_statement_or_block(child, source, state);
                    }
                }
            }
            _ => {
                self.add_statement(node, source, state, StatementKind::Expression);
            }
        }
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    /// Extract call sites from an AST node recursively
    fn extract_calls_from_node(
        &self,
        node: Node,
        source: &str,
        state: &mut BuildState,
        line: usize,
    ) {
        if node.kind() == "call_expression" {
            if let Some(callee_name) = self.get_call_name(node, source) {
                let call_kind = self.classify_call(&callee_name);
                let reachability = self.current_reachability(state);

                let call_site = CallSite {
                    callee: callee_name,
                    line,
                    call_kind,
                    reachability,
                    block_id: state.current_block,
                };

                if let Some(block) = state.cfg.block_mut(state.current_block) {
                    block.calls.push(call_site);
                }
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_calls_from_node(child, source, state, line);
        }
    }

    /// Classify a call based on macro semantics table
    fn classify_call(&self, name: &str) -> CallKind {
        match self.macro_table.lookup(name) {
            Some(MacroSemantics::AsyncRegistration {
                mechanism,
                handler_arg_index: _,
            }) => CallKind::AsyncRegistration {
                handler_name: None,
                mechanism: mechanism.clone(),
            },
            Some(MacroSemantics::Iterator) => CallKind::IteratorMacro,
            Some(MacroSemantics::Declaration) => CallKind::DeclarationMacro,
            Some(MacroSemantics::ContextChange { new_context }) => CallKind::ContextChange {
                new_context: new_context.clone(),
            },
            Some(MacroSemantics::EntryPointRegistration { .. }) => CallKind::DeclarationMacro,
            Some(MacroSemantics::BranchHint) => CallKind::MacroCall,
            Some(MacroSemantics::MemoryBarrier) => CallKind::MacroCall,
            Some(MacroSemantics::TypeCast) => CallKind::MacroCall,
            Some(MacroSemantics::FunctionCall) | None => CallKind::Direct,
        }
    }

    /// Get current reachability from the stack
    fn current_reachability(&self, state: &BuildState) -> Reachability {
        state
            .reachability_stack
            .last()
            .cloned()
            .unwrap_or(Reachability::Always)
    }

    /// Add a statement to the current block
    fn add_statement(
        &self,
        node: Node,
        source: &str,
        state: &mut BuildState,
        kind: StatementKind,
    ) {
        let line = node.start_position().row + 1;
        let text = self.node_text(node, source);
        let truncated = if text.len() > 120 {
            format!("{}...", &text[..117])
        } else {
            text
        };

        let stmt = Statement {
            line,
            kind,
            text: truncated,
        };

        if let Some(block) = state.cfg.block_mut(state.current_block) {
            block.statements.push(stmt);
            // Update line range
            if block.line_range.0 == 0 {
                block.line_range.0 = line;
            }
            block.line_range.1 = line;
        }
    }

    /// Create a new block and return its ID
    fn new_block(&self, state: &mut BuildState, block_type: BlockType) -> BlockId {
        let id = state.cfg.add_block(block_type);
        id
    }

    /// Resolve pending goto jumps to their label targets
    fn resolve_gotos(&self, state: &mut BuildState) {
        let gotos = state.pending_gotos.clone();
        let labels = state.label_blocks.clone();

        for (label, from_block, line) in &gotos {
            if let Some((_, target_block)) = labels.iter().find(|(l, _)| l == label) {
                state
                    .cfg
                    .add_edge(*from_block, *target_block, EdgeType::Goto(label.clone()));

                // If goto target is an error handler, record error path
                if is_error_label(label) {
                    // Collect cleanup calls at the target
                    let cleanup_calls: Vec<String> = state
                        .cfg
                        .block(*target_block)
                        .map(|b| b.calls.iter().map(|c| c.callee.clone()).collect())
                        .unwrap_or_default();

                    state.cfg.error_paths.push(ErrorPath {
                        check_expression: format!("goto {}", label),
                        check_line: *line,
                        check_block: *from_block,
                        strategy: ErrorStrategy::GotoCleanup {
                            label: label.clone(),
                        },
                        cleanup_calls,
                        label: Some(label.clone()),
                    });
                }
            }
        }
    }

    /// Check if a block ends with a terminating statement (return or goto)
    fn block_terminates(&self, block_id: BlockId, state: &BuildState) -> bool {
        state
            .cfg
            .block(block_id)
            .map(|b| {
                b.statements.iter().any(|s| {
                    matches!(
                        s.kind,
                        StatementKind::Return | StatementKind::Goto { .. }
                    )
                })
            })
            .unwrap_or(false)
    }

    /// Check if a condition expression is an error check
    fn is_error_condition(&self, cond: &str) -> bool {
        // Common kernel error check patterns
        cond.contains("< 0")
            || cond.contains("!= 0")
            || cond.starts_with('!')
            || cond.contains("IS_ERR")
            || cond.contains("PTR_ERR")
            || cond.contains("== NULL")
            || cond.contains("== -")
            || cond.contains("err")
            || cond.contains("ret < 0")
            || cond.contains("rc < 0")
            || cond.contains("status < 0")
    }

    /// Check if a return statement text indicates an error return
    fn is_error_return_text(&self, text: &str) -> bool {
        text.contains("-E")
            || text.contains("PTR_ERR")
            || text.contains("ERR_PTR")
            || text.contains("ERR_CAST")
            || text.contains("-1")
            || (text.contains("return") && text.contains("ret") && !text.contains("return 0"))
            || (text.contains("return") && text.contains("err"))
            || (text.contains("return") && text.contains("rc"))
    }

    /// Extract error code from return statement
    fn extract_error_code(&self, text: &str) -> String {
        // Try to find -EXXXX pattern
        if let Some(pos) = text.find("-E") {
            let rest = &text[pos..];
            let end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                .unwrap_or(rest.len());
            return rest[..end].to_string();
        }
        "error".to_string()
    }

    /// Unwrap else clause to get the inner statement
    fn unwrap_else_clause<'a>(&self, node: Node<'a>) -> Node<'a> {
        if node.kind() == "else_clause" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() != "else" {
                    return child;
                }
            }
        }
        node
    }

    // ========================================================================
    // Tree-sitter node helpers
    // ========================================================================

    fn find_function_node<'a>(
        &self,
        root: Node<'a>,
        source: &str,
        name: &str,
    ) -> Option<Node<'a>> {
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "function_definition" {
                if let Some(func_name) = self.get_function_name(child, source) {
                    if func_name == name {
                        return Some(child);
                    }
                }
            }
        }
        None
    }

    fn get_function_name(&self, func_node: Node, source: &str) -> Option<String> {
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            match child.kind() {
                "function_declarator" | "pointer_declarator" => {
                    return self.extract_func_name_recursive(child, source);
                }
                _ => {}
            }
        }
        None
    }

    fn extract_func_name_recursive(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => return Some(self.node_text(child, source)),
                "function_declarator" | "pointer_declarator" => {
                    return self.extract_func_name_recursive(child, source);
                }
                _ => {}
            }
        }
        None
    }

    fn find_child_by_kind<'a>(&self, node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == kind {
                return Some(child);
            }
        }
        None
    }

    fn find_child_by_field<'a>(&self, node: Node<'a>, field: &str) -> Option<Node<'a>> {
        node.child_by_field_name(field)
    }

    fn get_call_name(&self, call_node: Node, source: &str) -> Option<String> {
        let mut cursor = call_node.walk();
        for child in call_node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(self.node_text(child, source));
            }
        }
        None
    }

    fn has_call_expression(&self, node: Node) -> bool {
        if node.kind() == "call_expression" {
            return true;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if self.has_call_expression(child) {
                return true;
            }
        }
        false
    }

    fn node_text(&self, node: Node, source: &str) -> String {
        node.utf8_text(source.as_bytes())
            .unwrap_or("")
            .trim()
            .to_string()
    }
}

impl Default for CfgBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a label name indicates an error handler
pub fn is_error_label(label: &str) -> bool {
    let error_prefixes = [
        "err_", "error_", "fail_", "out_", "cleanup_", "free_", "unwind_", "undo_", "bail_",
        "err", "out", "fail",
    ];
    let lower = label.to_lowercase();
    error_prefixes
        .iter()
        .any(|p| lower.starts_with(p) || lower == *p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_cfg(source: &str, func_name: &str) -> ControlFlowGraph {
        let builder = CfgBuilder::new();
        builder.build_function_cfg(source, func_name).unwrap()
    }

    #[test]
    fn test_simple_function() {
        let source = r#"
int simple(int x) {
    foo();
    bar();
    return 0;
}
"#;
        let cfg = build_cfg(source, "simple");
        assert_eq!(cfg.function_name, "simple");
        assert!(cfg.blocks.len() >= 2); // entry + exit
        assert!(!cfg.exits.is_empty());

        let calls = cfg.all_calls();
        let callee_names: Vec<&str> = calls.iter().map(|c| c.callee.as_str()).collect();
        assert!(callee_names.contains(&"foo"));
        assert!(callee_names.contains(&"bar"));
    }

    #[test]
    fn test_if_else_branches() {
        let source = r#"
int branching(int x) {
    if (x > 0) {
        foo();
    } else {
        bar();
    }
    baz();
    return 0;
}
"#;
        let cfg = build_cfg(source, "branching");

        // Should have: entry, true-branch, false-branch, merge, exit
        assert!(cfg.blocks.len() >= 4);

        let calls = cfg.all_calls();
        let callee_names: Vec<&str> = calls.iter().map(|c| c.callee.as_str()).collect();
        assert!(callee_names.contains(&"foo"));
        assert!(callee_names.contains(&"bar"));
        assert!(callee_names.contains(&"baz"));

        // foo and bar should be conditional
        let foo_call = calls.iter().find(|c| c.callee == "foo").unwrap();
        assert!(matches!(foo_call.reachability, Reachability::Conditional(_)));

        // baz should be always (on the merge path)
        let baz_call = calls.iter().find(|c| c.callee == "baz").unwrap();
        assert_eq!(baz_call.reachability, Reachability::Always);
    }

    #[test]
    fn test_goto_error_handling() {
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
        let cfg = build_cfg(source, "probe");

        // Should detect error paths
        assert!(
            !cfg.error_paths.is_empty(),
            "Expected error paths, got none"
        );

        // Should have error handler blocks
        let error_blocks: Vec<_> = cfg
            .blocks
            .iter()
            .filter(|b| b.block_type == BlockType::ErrorHandler)
            .collect();
        assert!(
            !error_blocks.is_empty(),
            "Expected error handler blocks"
        );

        // Error handler should have cleanup calls
        let cleanup_calls: Vec<&str> = cfg
            .blocks
            .iter()
            .filter(|b| b.block_type == BlockType::ErrorHandler)
            .flat_map(|b| b.calls.iter())
            .map(|c| c.callee.as_str())
            .collect();
        assert!(
            cleanup_calls.contains(&"undo_step2") || cleanup_calls.contains(&"undo_step1"),
            "Expected cleanup calls in error handlers, got: {:?}",
            cleanup_calls
        );
    }

    #[test]
    fn test_early_return_error() {
        let source = r#"
int alloc_thing(void) {
    void *ptr = kmalloc(sizeof(struct thing));
    if (!ptr)
        return -ENOMEM;
    init_thing(ptr);
    return 0;
}
"#;
        let cfg = build_cfg(source, "alloc_thing");

        // Should detect the error return
        assert!(
            !cfg.error_paths.is_empty(),
            "Expected error path for -ENOMEM return"
        );
    }

    #[test]
    fn test_while_loop() {
        let source = r#"
void looper(int n) {
    while (n > 0) {
        process(n);
        n--;
    }
    done();
}
"#;
        let cfg = build_cfg(source, "looper");

        // Should have loop header block
        let has_loop_header = cfg
            .blocks
            .iter()
            .any(|b| b.block_type == BlockType::LoopHeader);
        assert!(has_loop_header, "Expected loop header block");

        // Should have a loop-back edge
        let has_loopback = cfg
            .edges
            .iter()
            .any(|e| matches!(e.edge_type, EdgeType::LoopBack));
        assert!(has_loopback, "Expected loop-back edge");
    }

    #[test]
    fn test_macro_classification() {
        let source = r#"
void setup(struct device *dev) {
    INIT_WORK(&dev->work, my_handler);
    spin_lock(&dev->lock);
    DEFINE_MUTEX(my_mutex);
    list_for_each_entry(item, &list, node) {
        process(item);
    }
    spin_unlock(&dev->lock);
}
"#;
        let cfg = build_cfg(source, "setup");
        let calls = cfg.all_calls();

        // INIT_WORK should be classified as async registration
        let init_work = calls.iter().find(|c| c.callee == "INIT_WORK");
        assert!(init_work.is_some(), "Expected INIT_WORK call");
        assert!(
            matches!(
                init_work.unwrap().call_kind,
                CallKind::AsyncRegistration { .. }
            ),
            "INIT_WORK should be AsyncRegistration"
        );

        // spin_lock should be context change
        let spin_lock = calls.iter().find(|c| c.callee == "spin_lock");
        assert!(spin_lock.is_some(), "Expected spin_lock call");
        assert!(
            matches!(spin_lock.unwrap().call_kind, CallKind::ContextChange { .. }),
            "spin_lock should be ContextChange"
        );

        // DEFINE_MUTEX should be declaration
        let define_mutex = calls.iter().find(|c| c.callee == "DEFINE_MUTEX");
        assert!(define_mutex.is_some(), "Expected DEFINE_MUTEX call");
        assert!(
            matches!(define_mutex.unwrap().call_kind, CallKind::DeclarationMacro),
            "DEFINE_MUTEX should be DeclarationMacro"
        );
    }

    #[test]
    fn test_dot_output() {
        let source = r#"
int example(int x) {
    if (x > 0)
        return x;
    return 0;
}
"#;
        let cfg = build_cfg(source, "example");
        let dot = cfg.to_dot();

        assert!(dot.contains("digraph"));
        assert!(dot.contains("example"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn test_build_all_cfgs() {
        let source = r#"
int func_a(void) {
    return 0;
}

void func_b(void) {
    func_a();
}
"#;
        let builder = CfgBuilder::new();
        let cfgs = builder.build_all_cfgs(source).unwrap();

        assert_eq!(cfgs.len(), 2);
        let names: Vec<&str> = cfgs.iter().map(|c| c.function_name.as_str()).collect();
        assert!(names.contains(&"func_a"));
        assert!(names.contains(&"func_b"));
    }

    #[test]
    fn test_cfg_stats() {
        let source = r#"
int probe(struct device *dev) {
    int ret;
    ret = alloc();
    if (ret < 0)
        goto err;
    init();
    return 0;
err:
    cleanup();
    return ret;
}
"#;
        let cfg = build_cfg(source, "probe");
        let stats = cfg.stats();

        assert!(stats.block_count >= 3);
        assert!(stats.edge_count >= 2);
        assert!(stats.total_calls >= 2);
    }

    #[test]
    fn test_kernel_probe_full() {
        let source = r#"
static int fsl_udc_probe(struct platform_device *pdev)
{
    struct resource *res;
    struct fsl_udc *udc;
    int ret;

    udc = kzalloc(sizeof(*udc), GFP_KERNEL);
    if (!udc)
        return -ENOMEM;

    res = platform_get_resource(pdev, IORESOURCE_MEM, 0);
    if (!res) {
        ret = -ENXIO;
        goto err_free_udc;
    }

    udc->regs = devm_ioremap(&pdev->dev, res->start, resource_size(res));
    if (!udc->regs) {
        ret = -ENOMEM;
        goto err_free_udc;
    }

    INIT_WORK(&udc->charger_work, fsl_charger_work);
    spin_lock_init(&udc->lock);

    ret = usb_add_gadget_udc(&pdev->dev, &udc->gadget);
    if (ret < 0)
        goto err_free_irq;

    platform_set_drvdata(pdev, udc);
    return 0;

err_free_irq:
    free_irq(udc->irq, udc);
err_free_udc:
    kfree(udc);
    return ret;
}
"#;
        let cfg = build_cfg(source, "fsl_udc_probe");
        let stats = cfg.stats();

        // Basic structure checks
        assert!(stats.block_count >= 8, "Expected 8+ blocks for complex probe, got {}", stats.block_count);
        assert!(stats.error_path_count >= 2, "Expected 2+ error paths, got {}", stats.error_path_count);

        // Check macro classification
        let calls = cfg.all_calls();

        // INIT_WORK → async registration, not a function call
        let init_work = calls.iter().find(|c| c.callee == "INIT_WORK");
        assert!(init_work.is_some(), "Expected INIT_WORK");
        assert!(matches!(init_work.unwrap().call_kind, CallKind::AsyncRegistration { .. }));

        // kzalloc → always reachable (first call)
        let kzalloc = calls.iter().find(|c| c.callee == "kzalloc");
        assert!(kzalloc.is_some(), "Expected kzalloc");
        assert_eq!(kzalloc.unwrap().reachability, Reachability::Always);

        // platform_set_drvdata → always reachable (on normal path)
        let set_drvdata = calls.iter().find(|c| c.callee == "platform_set_drvdata");
        assert!(set_drvdata.is_some(), "Expected platform_set_drvdata");

        // Error handler blocks should exist
        let error_blocks: Vec<_> = cfg.blocks.iter()
            .filter(|b| b.block_type == BlockType::ErrorHandler)
            .collect();
        assert!(error_blocks.len() >= 2, "Expected 2+ error handler blocks, got {}", error_blocks.len());

        // free_irq and kfree should be on error paths
        let error_call_names: Vec<&str> = error_blocks.iter()
            .flat_map(|b| b.calls.iter())
            .map(|c| c.callee.as_str())
            .collect();
        assert!(error_call_names.contains(&"kfree"), "Expected kfree in error path, got {:?}", error_call_names);

        // DOT output should be valid
        let dot = cfg.to_dot();
        assert!(dot.contains("fsl_udc_probe"));
        assert!(dot.contains("err_free_irq") || dot.contains("err_free_udc"));
    }

    #[test]
    fn test_is_error_label() {
        assert!(is_error_label("err_free"));
        assert!(is_error_label("error_cleanup"));
        assert!(is_error_label("out_unlock"));
        assert!(is_error_label("fail_init"));
        assert!(is_error_label("cleanup_resources"));
        assert!(is_error_label("err"));
        assert!(!is_error_label("success"));
        assert!(!is_error_label("retry"));
        assert!(!is_error_label("next_step"));
    }
}
