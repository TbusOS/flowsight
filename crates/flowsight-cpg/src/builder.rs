//! CPG builder — extracts variable definitions and uses from tree-sitter AST
//!
//! Walks function body to find:
//! - Assignments (x = expr)
//! - Declarations with initializers (int x = expr)
//! - Variable uses in expressions, conditions, arguments, returns

use crate::dataflow::compute_reaching_definitions;
use crate::types::*;
use flowsight_cfg::{CfgBuilder, ErrorPathDetector};
use tree_sitter::{Node, Parser};

/// Builds Code Property Graph from C source code
pub struct CpgBuilder {
    next_id: NodeId,
}

impl CpgBuilder {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    fn alloc_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Build CPG for a single function
    pub fn build_function_cpg(
        &mut self,
        source: &str,
        function_name: &str,
    ) -> Result<CodePropertyGraph, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c::language())
            .map_err(|e| format!("Failed to set language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or("Failed to parse source")?;

        let root = tree.root_node();

        let func_node = self
            .find_function_node(root, source, function_name)
            .ok_or_else(|| format!("Function '{}' not found", function_name))?;

        self.build_from_function(func_node, source, function_name)
    }

    /// Build CPG for all functions in source
    pub fn build_all_cpgs(
        &mut self,
        source: &str,
    ) -> Result<Vec<CodePropertyGraph>, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c::language())
            .map_err(|e| format!("Failed to set language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or("Failed to parse source")?;

        let root = tree.root_node();
        let mut cpgs = Vec::new();

        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "function_definition" {
                if let Some(name) = self.get_function_name(child, source) {
                    if let Ok(cpg) = self.build_from_function(child, source, &name) {
                        cpgs.push(cpg);
                    }
                }
            }
        }

        Ok(cpgs)
    }

    fn build_from_function(
        &mut self,
        func_node: Node,
        source: &str,
        function_name: &str,
    ) -> Result<CodePropertyGraph, String> {
        self.next_id = 0;

        let mut cpg = CodePropertyGraph {
            function_name: function_name.to_string(),
            source_file: None,
            definitions: Vec::new(),
            uses: Vec::new(),
            def_use_chains: Vec::new(),
            reaching_defs: Vec::new(),
            data_edges: Vec::new(),
            parameters: Vec::new(),
            returns: Vec::new(),
            stats: CpgStats::default(),
        };

        // Extract parameters as initial definitions
        self.extract_parameters(func_node, source, &mut cpg);

        // Find function body
        let body = self.find_child_by_kind(func_node, "compound_statement");
        if let Some(body) = body {
            self.walk_body(body, source, &mut cpg, 0);
        }

        // Build CFG and compute reaching definitions
        let cfg_builder = CfgBuilder::new();
        if let Ok(mut cfg) = cfg_builder.build_function_cfg(source, function_name) {
            ErrorPathDetector::analyze(&mut cfg);
            // Assign block IDs to defs/uses based on line numbers
            self.assign_block_ids(&cfg, &mut cpg);
            // Compute reaching definitions using CFG
            compute_reaching_definitions(&cfg, &mut cpg);
        }

        // Compute statistics
        cpg.stats = CpgStats {
            total_definitions: cpg.definitions.len(),
            total_uses: cpg.uses.len(),
            total_data_edges: cpg.data_edges.len(),
            total_def_use_chains: cpg.def_use_chains.len(),
            variables_tracked: cpg.all_variables().len(),
            parameters: cpg.parameters.len(),
            returns: cpg.returns.len(),
        };

        Ok(cpg)
    }

    /// Extract function parameters as definitions
    fn extract_parameters(&mut self, func_node: Node, source: &str, cpg: &mut CodePropertyGraph) {
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            if child.kind() == "function_declarator" || child.kind() == "pointer_declarator" {
                self.extract_params_recursive(child, source, cpg);
            }
        }
    }

    fn extract_params_recursive(&mut self, node: Node, source: &str, cpg: &mut CodePropertyGraph) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "parameter_list" {
                let mut param_idx = 0;
                let mut param_cursor = child.walk();
                for param in child.children(&mut param_cursor) {
                    if param.kind() == "parameter_declaration" {
                        let (name, type_name) = self.extract_param_info(param, source);
                        if !name.is_empty() {
                            let def_id = self.alloc_id();
                            cpg.parameters.push(ParameterDef {
                                name: name.clone(),
                                type_name: type_name.clone(),
                                index: param_idx,
                                def_id,
                            });
                            cpg.definitions.push(VarDef {
                                id: def_id,
                                var_name: name,
                                line: param.start_position().row + 1,
                                block_id: 0, // Entry block
                                kind: DefKind::Parameter { index: param_idx },
                                rhs_text: None,
                                rhs_uses: vec![],
                                var_type: Some(type_name),
                            });
                            param_idx += 1;
                        }
                    }
                }
            } else if child.kind() == "function_declarator" || child.kind() == "pointer_declarator" {
                self.extract_params_recursive(child, source, cpg);
            }
        }
    }

    fn extract_param_info(&self, param: Node, source: &str) -> (String, String) {
        let mut name = String::new();
        let mut type_name = String::new();
        let mut cursor = param.walk();
        for child in param.children(&mut cursor) {
            match child.kind() {
                "primitive_type" | "type_identifier" | "sized_type_specifier" => {
                    type_name = self.node_text(child, source);
                }
                "struct_specifier" => {
                    let struct_name = self.find_child_text(child, "type_identifier", source);
                    type_name = format!("struct {}", struct_name);
                }
                "identifier" => {
                    name = self.node_text(child, source);
                }
                "pointer_declarator" => {
                    type_name = format!("{}*", type_name);
                    name = self.extract_identifier(child, source);
                }
                _ => {}
            }
        }
        (name, type_name)
    }

    /// Walk function body extracting definitions and uses
    fn walk_body(&mut self, node: Node, source: &str, cpg: &mut CodePropertyGraph, depth: usize) {
        if depth > 50 {
            return; // Prevent infinite recursion
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "declaration" => {
                    self.extract_declaration(child, source, cpg);
                }
                "expression_statement" => {
                    self.extract_expression_statement(child, source, cpg);
                }
                "return_statement" => {
                    self.extract_return(child, source, cpg);
                }
                "if_statement" => {
                    // Extract condition uses
                    if let Some(cond) = child.child_by_field_name("condition") {
                        self.extract_uses_from_expr(cond, source, cpg, UseKind::Condition);
                    }
                    // Recurse into branches (handle both compound_statement and single statement)
                    if let Some(cons) = child.child_by_field_name("consequence") {
                        self.walk_statement_or_block(cons, source, cpg, depth + 1);
                    }
                    if let Some(alt) = child.child_by_field_name("alternative") {
                        self.walk_statement_or_block(alt, source, cpg, depth + 1);
                    }
                }
                "while_statement" | "do_statement" => {
                    if let Some(cond) = self.find_child_by_kind(child, "parenthesized_expression") {
                        self.extract_uses_from_expr(cond, source, cpg, UseKind::Condition);
                    }
                    if let Some(body) = child.child_by_field_name("body") {
                        self.walk_body(body, source, cpg, depth + 1);
                    }
                }
                "for_statement" => {
                    // For loop may have init declaration
                    if let Some(init) = child.child_by_field_name("initializer") {
                        if init.kind() == "declaration" {
                            self.extract_declaration(init, source, cpg);
                        }
                    }
                    if let Some(body) = child.child_by_field_name("body") {
                        self.walk_body(body, source, cpg, depth + 1);
                    }
                }
                "switch_statement" => {
                    if let Some(cond) = self.find_child_by_kind(child, "parenthesized_expression") {
                        self.extract_uses_from_expr(cond, source, cpg, UseKind::Condition);
                    }
                    self.walk_body(child, source, cpg, depth + 1);
                }
                "compound_statement" | "case_statement" | "labeled_statement" | "else_clause" => {
                    self.walk_body(child, source, cpg, depth + 1);
                }
                _ => {}
            }
        }
    }

    /// Walk a single statement or compound_statement
    fn walk_statement_or_block(&mut self, node: Node, source: &str, cpg: &mut CodePropertyGraph, depth: usize) {
        match node.kind() {
            "compound_statement" => self.walk_body(node, source, cpg, depth),
            "return_statement" => self.extract_return(node, source, cpg),
            "expression_statement" => self.extract_expression_statement(node, source, cpg),
            "declaration" => self.extract_declaration(node, source, cpg),
            "if_statement" => {
                if let Some(cond) = node.child_by_field_name("condition") {
                    self.extract_uses_from_expr(cond, source, cpg, UseKind::Condition);
                }
                if let Some(cons) = node.child_by_field_name("consequence") {
                    self.walk_statement_or_block(cons, source, cpg, depth + 1);
                }
                if let Some(alt) = node.child_by_field_name("alternative") {
                    self.walk_statement_or_block(alt, source, cpg, depth + 1);
                }
            }
            "else_clause" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "else" {
                        self.walk_statement_or_block(child, source, cpg, depth);
                    }
                }
            }
            _ => self.walk_body(node, source, cpg, depth),
        }
    }

    /// Extract variable definition from declaration
    fn extract_declaration(&mut self, node: Node, source: &str, cpg: &mut CodePropertyGraph) {
        let line = node.start_position().row + 1;
        let mut type_name = String::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "primitive_type" | "type_identifier" | "sized_type_specifier" => {
                    type_name = self.node_text(child, source);
                }
                "struct_specifier" => {
                    let sname = self.find_child_text(child, "type_identifier", source);
                    type_name = format!("struct {}", sname);
                }
                "init_declarator" => {
                    let var_name = self.extract_declarator_name(child, source);
                    if !var_name.is_empty() {
                        // Find initializer expression
                        let rhs = child.child_by_field_name("value");
                        let rhs_text = rhs.map(|r| self.node_text(r, source));
                        let mut rhs_uses = Vec::new();
                        if let Some(rhs_node) = rhs {
                            self.collect_identifiers(rhs_node, source, &mut rhs_uses);
                        }

                        // Check if RHS is a function call
                        let kind = if let Some(rhs_node) = rhs {
                            if self.has_call_in(rhs_node) {
                                let callee = self.find_call_name(rhs_node, source);
                                DefKind::CallReturn {
                                    callee: callee.unwrap_or_else(|| "?".to_string()),
                                }
                            } else {
                                DefKind::Declaration
                            }
                        } else {
                            DefKind::Declaration
                        };

                        let def_id = self.alloc_id();
                        cpg.definitions.push(VarDef {
                            id: def_id,
                            var_name,
                            line,
                            block_id: 0,
                            kind,
                            rhs_text,
                            rhs_uses,
                            var_type: Some(type_name.clone()),
                        });

                        // Also extract uses from the RHS
                        if let Some(rhs_node) = rhs {
                            self.extract_uses_from_expr(rhs_node, source, cpg, UseKind::RhsExpression);
                        }
                    }
                }
                "identifier" => {
                    // Declaration without initializer: `int x;`
                    let var_name = self.node_text(child, source);
                    if !var_name.is_empty() {
                        let def_id = self.alloc_id();
                        cpg.definitions.push(VarDef {
                            id: def_id,
                            var_name,
                            line,
                            block_id: 0,
                            kind: DefKind::Declaration,
                            rhs_text: None,
                            rhs_uses: vec![],
                            var_type: Some(type_name.clone()),
                        });
                    }
                }
                "pointer_declarator" => {
                    let var_name = self.extract_identifier(child, source);
                    if !var_name.is_empty() {
                        let def_id = self.alloc_id();
                        cpg.definitions.push(VarDef {
                            id: def_id,
                            var_name,
                            line,
                            block_id: 0,
                            kind: DefKind::Declaration,
                            rhs_text: None,
                            rhs_uses: vec![],
                            var_type: Some(format!("{}*", type_name)),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    /// Extract definition from expression statement (assignments)
    fn extract_expression_statement(
        &mut self,
        node: Node,
        source: &str,
        cpg: &mut CodePropertyGraph,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "assignment_expression" {
                self.extract_assignment(child, source, cpg);
            } else if child.kind() == "update_expression" {
                // x++ or ++x
                let var = self.extract_identifier(child, source);
                if !var.is_empty() {
                    let line = child.start_position().row + 1;
                    let def_id = self.alloc_id();
                    cpg.definitions.push(VarDef {
                        id: def_id,
                        var_name: var.clone(),
                        line,
                        block_id: 0,
                        kind: DefKind::CompoundAssignment("++".to_string()),
                        rhs_text: None,
                        rhs_uses: vec![var.clone()],
                        var_type: None,
                    });
                    let use_id = self.alloc_id();
                    cpg.uses.push(VarUse {
                        id: use_id,
                        var_name: var,
                        line,
                        block_id: 0,
                        kind: UseKind::Expression,
                    });
                }
            } else if child.kind() == "call_expression" {
                // Standalone function call (not assigned to variable)
                self.extract_call_uses(child, source, cpg);
            }
        }
    }

    /// Extract assignment definition
    fn extract_assignment(&mut self, node: Node, source: &str, cpg: &mut CodePropertyGraph) {
        let line = node.start_position().row + 1;
        let lhs = node.child_by_field_name("left");
        let rhs = node.child_by_field_name("right");
        let op = node.child(1).map(|n| self.node_text(n, source));

        if let Some(lhs_node) = lhs {
            let lhs_text = self.node_text(lhs_node, source);

            // Determine if this is a field assignment or simple assignment
            let (var_name, kind) = if lhs_text.contains("->") {
                let parts: Vec<&str> = lhs_text.splitn(2, "->").collect();
                (
                    lhs_text.clone(),
                    DefKind::FieldAssignment {
                        base: parts[0].to_string(),
                        field: parts.get(1).unwrap_or(&"").to_string(),
                    },
                )
            } else if lhs_text.contains('.') {
                let parts: Vec<&str> = lhs_text.splitn(2, '.').collect();
                (
                    lhs_text.clone(),
                    DefKind::FieldAssignment {
                        base: parts[0].to_string(),
                        field: parts.get(1).unwrap_or(&"").to_string(),
                    },
                )
            } else if op.as_deref() != Some("=") {
                (
                    lhs_text.clone(),
                    DefKind::CompoundAssignment(op.unwrap_or_default()),
                )
            } else if rhs.map(|r| self.has_call_in(r)).unwrap_or(false) {
                let callee = rhs.and_then(|r| self.find_call_name(r, source));
                (
                    lhs_text.clone(),
                    DefKind::CallReturn {
                        callee: callee.unwrap_or_else(|| "?".to_string()),
                    },
                )
            } else {
                (lhs_text.clone(), DefKind::Assignment)
            };

            let rhs_text = rhs.map(|r| self.node_text(r, source));
            let mut rhs_uses = Vec::new();
            if let Some(rhs_node) = rhs {
                self.collect_identifiers(rhs_node, source, &mut rhs_uses);
            }

            let def_id = self.alloc_id();
            cpg.definitions.push(VarDef {
                id: def_id,
                var_name,
                line,
                block_id: 0,
                kind,
                rhs_text,
                rhs_uses,
                var_type: None,
            });

            // Extract uses from RHS
            if let Some(rhs_node) = rhs {
                self.extract_uses_from_expr(rhs_node, source, cpg, UseKind::RhsExpression);
            }
        }
    }

    /// Extract return statement info
    fn extract_return(&mut self, node: Node, source: &str, cpg: &mut CodePropertyGraph) {
        let line = node.start_position().row + 1;
        let text = self.node_text(node, source);

        let mut used_vars = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "return" && child.kind() != ";" {
                self.collect_identifiers(child, source, &mut used_vars);
                self.extract_uses_from_expr(child, source, cpg, UseKind::ReturnValue);
            }
        }

        let return_expr = text
            .trim()
            .strip_prefix("return")
            .map(|s| s.trim().trim_end_matches(';').trim())
            .unwrap_or("");
        let is_error = return_expr.starts_with('-')
            || return_expr.contains("ENOMEM") || return_expr.contains("EINVAL")
            || return_expr.contains("ENODEV") || return_expr.contains("EIO")
            || return_expr.contains("ENXIO") || return_expr.contains("EBUSY")
            || return_expr.contains("ESHUTDOWN") || return_expr.contains("ETIMEDOUT")
            || return_expr.contains("PTR_ERR") || return_expr.contains("ERR_PTR")
            || matches!(return_expr, "ret" | "err" | "rc" | "status");

        let expression = text
            .strip_prefix("return")
            .map(|s| s.trim().trim_end_matches(';').trim().to_string())
            .filter(|s| !s.is_empty());

        cpg.returns.push(ReturnInfo {
            line,
            used_vars,
            expression,
            is_error,
        });
    }

    /// Extract variable uses from an expression subtree
    fn extract_uses_from_expr(
        &mut self,
        node: Node,
        source: &str,
        cpg: &mut CodePropertyGraph,
        default_kind: UseKind,
    ) {
        match node.kind() {
            "identifier" => {
                let name = self.node_text(node, source);
                // Skip type names, keywords, macro names (heuristic: skip ALL_CAPS unless it's a known var)
                if !name.is_empty()
                    && !is_keyword(&name)
                    && !is_type_name(&name)
                {
                    let use_id = self.alloc_id();
                    cpg.uses.push(VarUse {
                        id: use_id,
                        var_name: name,
                        line: node.start_position().row + 1,
                        block_id: 0,
                        kind: default_kind.clone(),
                    });
                }
            }
            "call_expression" => {
                self.extract_call_uses(node, source, cpg);
            }
            "field_expression" => {
                // ptr->field or obj.field
                if let Some(base) = node.child_by_field_name("argument") {
                    let base_name = self.node_text(base, source);
                    let field = node
                        .child_by_field_name("field")
                        .map(|f| self.node_text(f, source))
                        .unwrap_or_default();
                    if !base_name.is_empty() && !is_keyword(&base_name) {
                        let use_id = self.alloc_id();
                        cpg.uses.push(VarUse {
                            id: use_id,
                            var_name: base_name,
                            line: node.start_position().row + 1,
                            block_id: 0,
                            kind: UseKind::FieldAccess { field },
                        });
                    }
                }
                return; // Don't recurse further
            }
            "pointer_expression" => {
                // *ptr or &var
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        let name = self.node_text(child, source);
                        if !name.is_empty() && !is_keyword(&name) {
                            let use_id = self.alloc_id();
                            cpg.uses.push(VarUse {
                                id: use_id,
                                var_name: name,
                                line: node.start_position().row + 1,
                                block_id: 0,
                                kind: UseKind::Dereference,
                            });
                        }
                    }
                }
                return;
            }
            _ => {}
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_uses_from_expr(child, source, cpg, default_kind.clone());
        }
    }

    /// Extract uses from function call arguments
    fn extract_call_uses(&mut self, call_node: Node, source: &str, cpg: &mut CodePropertyGraph) {
        let callee = self.find_call_name_direct(call_node, source).unwrap_or_default();

        if let Some(args) = self.find_child_by_kind(call_node, "argument_list") {
            let mut arg_idx = 0;
            let mut cursor = args.walk();
            for child in args.children(&mut cursor) {
                if child.kind() != "(" && child.kind() != ")" && child.kind() != "," {
                    self.extract_uses_from_expr(
                        child,
                        source,
                        cpg,
                        UseKind::Argument {
                            callee: callee.clone(),
                            arg_index: arg_idx,
                        },
                    );
                    arg_idx += 1;
                }
            }
        }
    }

    /// Assign CFG block IDs to definitions and uses based on line numbers
    fn assign_block_ids(&self, cfg: &flowsight_cfg::ControlFlowGraph, cpg: &mut CodePropertyGraph) {
        for def in &mut cpg.definitions {
            def.block_id = self.find_block_for_line(cfg, def.line);
        }
        for u in &mut cpg.uses {
            u.block_id = self.find_block_for_line(cfg, u.line);
        }
    }

    fn find_block_for_line(&self, cfg: &flowsight_cfg::ControlFlowGraph, line: usize) -> usize {
        for block in &cfg.blocks {
            if line >= block.line_range.0 && line <= block.line_range.1 {
                return block.id;
            }
        }
        0 // Default to entry block
    }

    // ========================================================================
    // Tree-sitter helpers
    // ========================================================================

    fn collect_identifiers(&self, node: Node, source: &str, out: &mut Vec<String>) {
        if node.kind() == "identifier" {
            let name = self.node_text(node, source);
            if !name.is_empty() && !is_keyword(&name) && !is_type_name(&name) {
                if !out.contains(&name) {
                    out.push(name);
                }
            }
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_identifiers(child, source, out);
        }
    }

    fn has_call_in(&self, node: Node) -> bool {
        if node.kind() == "call_expression" {
            return true;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if self.has_call_in(child) {
                return true;
            }
        }
        false
    }

    fn find_call_name(&self, node: Node, source: &str) -> Option<String> {
        if node.kind() == "call_expression" {
            return self.find_call_name_direct(node, source);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(name) = self.find_call_name(child, source) {
                return Some(name);
            }
        }
        None
    }

    fn find_call_name_direct(&self, call_node: Node, source: &str) -> Option<String> {
        let mut cursor = call_node.walk();
        for child in call_node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(self.node_text(child, source));
            }
        }
        None
    }

    fn find_function_node<'a>(&self, root: Node<'a>, source: &str, name: &str) -> Option<Node<'a>> {
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

    fn extract_declarator_name(&self, node: Node, source: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => return self.node_text(child, source),
                "pointer_declarator" => return self.extract_identifier(child, source),
                _ => {}
            }
        }
        String::new()
    }

    fn extract_identifier(&self, node: Node, source: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return self.node_text(child, source);
            }
            if child.kind() == "pointer_declarator" {
                return self.extract_identifier(child, source);
            }
        }
        String::new()
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

    fn find_child_text(&self, node: Node, kind: &str, source: &str) -> String {
        self.find_child_by_kind(node, kind)
            .map(|n| self.node_text(n, source))
            .unwrap_or_default()
    }

    fn node_text(&self, node: Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").trim().to_string()
    }
}

impl Default for CpgBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if name is a C keyword
fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "if" | "else" | "while" | "for" | "do" | "switch" | "case" | "default"
            | "break" | "continue" | "return" | "goto" | "sizeof" | "typeof"
            | "void" | "int" | "char" | "long" | "short" | "unsigned" | "signed"
            | "float" | "double" | "const" | "static" | "extern" | "inline"
            | "volatile" | "struct" | "union" | "enum" | "typedef"
            | "NULL" | "true" | "false"
    )
}

/// Heuristic: skip ALL_CAPS names (likely macros/constants, not variables)
fn is_type_name(name: &str) -> bool {
    // Skip if all uppercase and > 2 chars (likely a macro)
    if name.len() > 2 && name == name.to_uppercase() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let source = r#"
int add(int a, int b) {
    int result = a + b;
    return result;
}
"#;
        let mut builder = CpgBuilder::new();
        let cpg = builder.build_function_cpg(source, "add").unwrap();

        // Parameters
        assert_eq!(cpg.parameters.len(), 2);
        assert_eq!(cpg.parameters[0].name, "a");
        assert_eq!(cpg.parameters[1].name, "b");

        // Definitions: params + result declaration
        assert!(cpg.definitions.len() >= 3);

        // Uses: a and b in RHS, result in return
        assert!(cpg.uses.len() >= 3);

        // Returns
        assert_eq!(cpg.returns.len(), 1);
        assert!(!cpg.returns[0].is_error);
    }

    #[test]
    fn test_kernel_probe_defs_uses() {
        let source = r#"
static int my_probe(struct platform_device *pdev) {
    struct my_dev *dev;
    int ret;

    dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    ret = clk_prepare_enable(dev->clk);
    if (ret < 0)
        return ret;

    platform_set_drvdata(pdev, dev);
    return 0;
}
"#;
        let mut builder = CpgBuilder::new();
        let cpg = builder.build_function_cpg(source, "my_probe").unwrap();

        // Should have parameter
        assert!(cpg.parameters.len() >= 1);
        assert_eq!(cpg.parameters[0].name, "pdev");

        // Should have definitions for dev and ret
        let dev_defs: Vec<_> = cpg.defs_of("dev").iter().map(|d| &d.kind).collect();
        assert!(!dev_defs.is_empty(), "Expected 'dev' definitions");

        let ret_defs: Vec<_> = cpg.defs_of("ret").iter().map(|d| &d.kind).collect();
        assert!(!ret_defs.is_empty(), "Expected 'ret' definitions");

        // ret should have a CallReturn def
        assert!(
            ret_defs.iter().any(|k| matches!(k, DefKind::CallReturn { .. })),
            "ret should be assigned from function call"
        );

        // Should have uses of pdev, dev, ret
        assert!(!cpg.uses_of("pdev").is_empty(), "pdev should be used");
        assert!(!cpg.uses_of("dev").is_empty(), "dev should be used");
        assert!(!cpg.uses_of("ret").is_empty(), "ret should be used");

        // Should have error returns
        assert!(
            cpg.returns.iter().any(|r| r.is_error),
            "Should have error returns"
        );

        // Stats
        assert!(cpg.stats.total_definitions > 0);
        assert!(cpg.stats.total_uses > 0);
        assert!(cpg.stats.variables_tracked > 0);
    }

    #[test]
    fn test_data_flow_chains() {
        let source = r#"
int compute(int x) {
    int y = x + 1;
    int z = y * 2;
    return z;
}
"#;
        let mut builder = CpgBuilder::new();
        let cpg = builder.build_function_cpg(source, "compute").unwrap();

        // x → y → z data flow chain
        assert!(cpg.stats.total_definitions >= 3); // x(param), y, z
        assert!(cpg.stats.total_uses >= 3); // x in y=x+1, y in z=y*2, z in return

        // Data edges should exist
        assert!(
            cpg.stats.total_data_edges > 0,
            "Expected data flow edges, stats: {}",
            cpg.stats
        );
    }

    #[test]
    fn test_field_assignment() {
        let source = r#"
void setup(struct device *dev) {
    dev->status = 1;
    dev->name = "test";
}
"#;
        let mut builder = CpgBuilder::new();
        let cpg = builder.build_function_cpg(source, "setup").unwrap();

        let field_defs: Vec<_> = cpg
            .definitions
            .iter()
            .filter(|d| matches!(d.kind, DefKind::FieldAssignment { .. }))
            .collect();
        assert!(
            field_defs.len() >= 2,
            "Expected 2 field assignments, got {}",
            field_defs.len()
        );
    }

    #[test]
    fn test_all_variables() {
        let source = r#"
int foo(int a) {
    int b = a;
    int c = b + 1;
    return c;
}
"#;
        let mut builder = CpgBuilder::new();
        let cpg = builder.build_function_cpg(source, "foo").unwrap();
        let vars = cpg.all_variables();
        assert!(vars.contains(&"a".to_string()));
        assert!(vars.contains(&"b".to_string()));
        assert!(vars.contains(&"c".to_string()));
    }
}
