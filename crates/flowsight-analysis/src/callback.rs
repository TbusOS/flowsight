//! Generic Callback Pattern Recognition
//!
//! Identifies callback patterns without relying on specific frameworks:
//! - Pattern 1: Simple callback (obj->callback = handler; obj->callback())
//! - Pattern 2: Register function (register_xxx(handler))
//! - Pattern 3: Event loop (while(1) handlers[event]())

use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Parser as TSParser};

/// A callback binding: where a function is assigned to a callback slot
#[derive(Debug, Clone)]
pub struct CallbackBinding {
    /// The target (e.g., "dev->callback", "handlers[0]")
    pub target: String,
    /// The handler function name
    pub handler: String,
    /// Line number of the binding
    pub line: u32,
}

/// A callback invocation: where a callback is called
#[derive(Debug, Clone)]
pub struct CallbackInvocation {
    /// The callback expression (e.g., "dev->callback", "handlers[i]")
    pub expr: String,
    /// Line number of the invocation
    pub line: u32,
}

/// A registration call: register_xxx(handler)
#[derive(Debug, Clone)]
pub struct RegistrationCall {
    /// The register function name
    pub register_func: String,
    /// The handler function name
    pub handler: String,
    /// Line number
    pub line: u32,
}

/// An event loop pattern: while(1) { handlers[event](); }
#[derive(Debug, Clone)]
pub struct EventLoopPattern {
    /// The dispatch expression (e.g., "handlers[event]")
    pub dispatch_expr: String,
    /// Line number of the loop
    pub line: u32,
}

/// Result of callback pattern analysis
#[derive(Debug, Default)]
pub struct CallbackAnalysis {
    /// All callback bindings found
    pub bindings: Vec<CallbackBinding>,
    /// All callback invocations found
    pub invocations: Vec<CallbackInvocation>,
    /// All registration calls found
    pub registrations: Vec<RegistrationCall>,
    /// Event loop patterns found
    pub event_loops: Vec<EventLoopPattern>,
    /// Resolved mappings: invocation expr -> possible handlers
    pub resolved: HashMap<String, HashSet<String>>,
}

/// Generic callback pattern analyzer
pub struct CallbackAnalyzer {
    /// Known function names
    functions: HashSet<String>,
}

impl CallbackAnalyzer {
    pub fn new() -> Self {
        Self {
            functions: HashSet::new(),
        }
    }

    pub fn set_functions(&mut self, funcs: impl IntoIterator<Item = String>) {
        self.functions = funcs.into_iter().collect();
    }

    /// Analyze source code for callback patterns
    pub fn analyze(&self, source: &str) -> CallbackAnalysis {
        let mut result = CallbackAnalysis::default();

        let mut parser = TSParser::new();
        parser
            .set_language(&tree_sitter_c::language())
            .expect("Failed to load C grammar");

        if let Some(tree) = parser.parse(source, None) {
            self.collect_bindings(tree.root_node(), source, &mut result);
            self.collect_invocations(tree.root_node(), source, &mut result);
            self.collect_registrations(tree.root_node(), source, &mut result);
            self.collect_event_loops(tree.root_node(), source, &mut result);
        }

        // Resolve bindings to invocations
        self.resolve(&mut result);

        result
    }

    fn collect_bindings(&self, node: Node, source: &str, result: &mut CallbackAnalysis) {
        if node.kind() == "assignment_expression" {
            if let Some(binding) = self.try_extract_binding(node, source) {
                result.bindings.push(binding);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_bindings(child, source, result);
        }
    }

    fn collect_registrations(&self, node: Node, source: &str, result: &mut CallbackAnalysis) {
        if node.kind() == "call_expression" {
            if let Some(reg) = self.try_extract_registration(node, source) {
                result.registrations.push(reg);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_registrations(child, source, result);
        }
    }

    fn try_extract_registration(&self, node: Node, source: &str) -> Option<RegistrationCall> {
        let mut cursor = node.walk();
        let mut children: Vec<Node> = node.children(&mut cursor).collect();

        if children.is_empty() {
            return None;
        }

        let callee = children.remove(0);
        if callee.kind() != "identifier" {
            return None;
        }

        let func_name = self.node_text(callee, source);

        // Check if it looks like a registration function
        if !self.is_register_function(&func_name) {
            return None;
        }

        // Find function pointer argument
        for child in children {
            if child.kind() == "argument_list" {
                let mut arg_cursor = child.walk();
                for arg in child.children(&mut arg_cursor) {
                    if arg.kind() == "identifier" {
                        let arg_text = self.node_text(arg, source);
                        if self.functions.contains(&arg_text) {
                            return Some(RegistrationCall {
                                register_func: func_name,
                                handler: arg_text,
                                line: node.start_position().row as u32 + 1,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    fn is_register_function(&self, name: &str) -> bool {
        let patterns = [
            "register", "subscribe", "connect", "bind", "attach",
            "add_handler", "set_callback", "on_", "listen",
        ];
        let name_lower = name.to_lowercase();
        patterns.iter().any(|p| name_lower.contains(p))
    }

    fn collect_event_loops(&self, node: Node, source: &str, result: &mut CallbackAnalysis) {
        // Look for while statements
        if node.kind() == "while_statement" {
            if let Some(pattern) = self.try_extract_event_loop(node, source) {
                result.event_loops.push(pattern);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_event_loops(child, source, result);
        }
    }

    fn try_extract_event_loop(&self, node: Node, source: &str) -> Option<EventLoopPattern> {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();

        // Check for infinite loop condition: while(1) or while(true)
        let mut is_infinite = false;
        for child in &children {
            if child.kind() == "parenthesized_expression" {
                let cond = self.node_text(*child, source);
                if cond == "(1)" || cond == "(true)" {
                    is_infinite = true;
                    break;
                }
            }
        }

        if !is_infinite {
            return None;
        }

        // Look for array/table dispatch in the loop body
        for child in &children {
            if child.kind() == "compound_statement" {
                if let Some(dispatch) = self.find_dispatch_in_body(*child, source) {
                    return Some(EventLoopPattern {
                        dispatch_expr: dispatch,
                        line: node.start_position().row as u32 + 1,
                    });
                }
            }
        }

        None
    }

    fn find_dispatch_in_body(&self, node: Node, source: &str) -> Option<String> {
        if node.kind() == "call_expression" {
            let mut cursor = node.walk();
            let children: Vec<Node> = node.children(&mut cursor).collect();
            if let Some(callee) = children.first() {
                // Look for subscript expression as callee: handlers[event]()
                if callee.kind() == "subscript_expression" {
                    return Some(self.normalize_target(&self.node_text(*callee, source)));
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(dispatch) = self.find_dispatch_in_body(child, source) {
                return Some(dispatch);
            }
        }

        None
    }

    fn try_extract_binding(&self, node: Node, source: &str) -> Option<CallbackBinding> {
        let mut lhs = None;
        let mut rhs = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "=" {
                continue;
            }
            if lhs.is_none() {
                lhs = Some(child);
            } else {
                rhs = Some(child);
            }
        }

        let (lhs_node, rhs_node) = (lhs?, rhs?);
        let lhs_text = self.node_text(lhs_node, source);
        let rhs_text = self.node_text(rhs_node, source);

        // Check if RHS is a known function
        if !self.functions.contains(&rhs_text) {
            return None;
        }

        // Check if LHS looks like a callback target
        if !self.is_callback_target(&lhs_text) {
            return None;
        }

        Some(CallbackBinding {
            target: self.normalize_target(&lhs_text),
            handler: rhs_text,
            line: lhs_node.start_position().row as u32 + 1,
        })
    }

    fn collect_invocations(&self, node: Node, source: &str, result: &mut CallbackAnalysis) {
        if node.kind() == "call_expression" {
            if let Some(invocation) = self.try_extract_invocation(node, source) {
                result.invocations.push(invocation);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_invocations(child, source, result);
        }
    }

    fn try_extract_invocation(&self, node: Node, source: &str) -> Option<CallbackInvocation> {
        let mut cursor = node.walk();
        let callee = node.children(&mut cursor).next()?;

        // Skip direct function calls
        if callee.kind() == "identifier" {
            return None;
        }

        // Look for indirect calls: field_expression or subscript_expression
        if callee.kind() == "field_expression" || callee.kind() == "subscript_expression" {
            let expr = self.node_text(callee, source);
            return Some(CallbackInvocation {
                expr: self.normalize_target(&expr),
                line: callee.start_position().row as u32 + 1,
            });
        }

        None
    }

    fn is_callback_target(&self, target: &str) -> bool {
        // Field access or array subscript
        target.contains("->") || target.contains('.') || target.contains('[')
    }

    fn normalize_target(&self, target: &str) -> String {
        // Normalize for matching: remove specific indices, keep structure
        let normalized = target.replace(" ", "");
        // Replace specific array indices with generic marker
        let re = regex::Regex::new(r"\[\d+\]").unwrap();
        re.replace_all(&normalized, "[*]").to_string()
    }

    fn resolve(&self, result: &mut CallbackAnalysis) {
        for invocation in &result.invocations {
            let mut handlers = HashSet::new();

            for binding in &result.bindings {
                if self.targets_match(&binding.target, &invocation.expr) {
                    handlers.insert(binding.handler.clone());
                }
            }

            if !handlers.is_empty() {
                result.resolved.insert(invocation.expr.clone(), handlers);
            }
        }
    }

    fn targets_match(&self, binding: &str, invocation: &str) -> bool {
        // Exact match
        if binding == invocation {
            return true;
        }

        // For arrays: handlers[*] matches handlers[*] (both normalized)
        let binding_base = binding.split('[').next().unwrap_or(binding);
        let invocation_base = invocation.split('[').next().unwrap_or(invocation);

        // Both are array accesses on the same base
        if binding.contains('[') && invocation.contains('[') && binding_base == invocation_base {
            return true;
        }

        false
    }

    fn node_text(&self, node: Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }
}

impl Default for CallbackAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_callback_pattern() {
        let source = r#"
void my_handler(void) {}

void setup(struct device *dev) {
    dev->callback = my_handler;
}

void trigger(struct device *dev) {
    dev->callback();
}
"#;
        let mut analyzer = CallbackAnalyzer::new();
        analyzer.set_functions(vec!["my_handler".to_string()]);

        let result = analyzer.analyze(source);

        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].handler, "my_handler");

        assert_eq!(result.invocations.len(), 1);
        assert!(result.resolved.contains_key("dev->callback"));
    }

    #[test]
    fn test_array_callback_pattern() {
        let source = r#"
void handler1(void) {}
void handler2(void) {}

void setup(void) {
    handlers[0] = handler1;
    handlers[1] = handler2;
}

void dispatch(int i) {
    handlers[i]();
}
"#;
        let mut analyzer = CallbackAnalyzer::new();
        analyzer.set_functions(vec!["handler1".to_string(), "handler2".to_string()]);

        let result = analyzer.analyze(source);

        assert_eq!(result.bindings.len(), 2);
        assert_eq!(result.invocations.len(), 1);

        // handlers[i] should resolve to both handlers
        // The key is normalized to handlers[*]
        assert!(!result.resolved.is_empty(), "resolved should not be empty");
        let key = result.resolved.keys().next().unwrap();
        let resolved = result.resolved.get(key).unwrap();
        assert!(resolved.contains("handler1"));
        assert!(resolved.contains("handler2"));
    }

    #[test]
    fn test_register_function_pattern() {
        let source = r#"
void my_callback(int event) {}

void setup(void) {
    register_event_handler(my_callback);
}
"#;
        let mut analyzer = CallbackAnalyzer::new();
        analyzer.set_functions(vec!["my_callback".to_string()]);

        let result = analyzer.analyze(source);

        assert_eq!(result.registrations.len(), 1);
        assert_eq!(result.registrations[0].register_func, "register_event_handler");
        assert_eq!(result.registrations[0].handler, "my_callback");
    }

    #[test]
    fn test_event_loop_pattern() {
        let source = r#"
void handler1(void) {}
void handler2(void) {}

void setup(void) {
    handlers[0] = handler1;
    handlers[1] = handler2;
}

void event_loop(void) {
    while (1) {
        int event = get_event();
        handlers[event]();
    }
}
"#;
        let mut analyzer = CallbackAnalyzer::new();
        analyzer.set_functions(vec!["handler1".to_string(), "handler2".to_string()]);

        let result = analyzer.analyze(source);

        assert_eq!(result.event_loops.len(), 1);
        assert!(result.event_loops[0].dispatch_expr.contains("handlers"));
    }
}
