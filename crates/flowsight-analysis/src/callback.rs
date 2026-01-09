//! Generic Callback Pattern Recognition
//!
//! Identifies callback patterns without relying on specific frameworks:
//! - Pattern 1: Simple callback (obj->callback = handler; obj->callback())
//! - Pattern 2: Register function (register_xxx(handler))
//! - Pattern 3: Event loop (while(1) handlers[event]())
//! - Pattern 4: Queue pattern (enqueue(work); work = dequeue(); work->func())
//! - Pattern 5: Signal/Slot pattern (connect(signal, slot); emit(signal))

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

/// A queue pattern: enqueue(work); work = dequeue(); work->func()
#[derive(Debug, Clone)]
pub struct QueuePattern {
    /// The enqueue function name
    pub enqueue_func: String,
    /// The dequeue function name
    pub dequeue_func: Option<String>,
    /// The work struct field that holds the callback
    pub callback_field: Option<String>,
    /// Line number
    pub line: u32,
}

/// A signal/slot pattern: connect(signal, handler); emit(signal)
#[derive(Debug, Clone)]
pub struct SignalSlotPattern {
    /// The connect function name
    pub connect_func: String,
    /// The signal identifier
    pub signal: String,
    /// The handler function name
    pub handler: String,
    /// The emit/trigger function (if found)
    pub emit_func: Option<String>,
    /// Line number
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
    /// Queue patterns found
    pub queue_patterns: Vec<QueuePattern>,
    /// Signal/slot patterns found
    pub signal_slots: Vec<SignalSlotPattern>,
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
            self.collect_queue_patterns(tree.root_node(), source, &mut result);
            self.collect_signal_slots(tree.root_node(), source, &mut result);
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

    /// Collect queue patterns (enqueue/dequeue style)
    fn collect_queue_patterns(&self, node: Node, source: &str, result: &mut CallbackAnalysis) {
        if node.kind() == "call_expression" {
            if let Some(pattern) = self.try_extract_queue_pattern(node, source) {
                result.queue_patterns.push(pattern);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_queue_patterns(child, source, result);
        }
    }

    fn try_extract_queue_pattern(&self, node: Node, source: &str) -> Option<QueuePattern> {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();

        if children.is_empty() {
            return None;
        }

        let callee = children.first()?;
        if callee.kind() != "identifier" {
            return None;
        }

        let func_name = self.node_text(*callee, source);

        // Check if it's a queue operation function
        if !self.is_queue_function(&func_name) {
            return None;
        }

        // Try to identify the callback field from arguments
        let callback_field = self.find_callback_field_in_args(&children, source);

        Some(QueuePattern {
            enqueue_func: func_name,
            dequeue_func: None, // Would need cross-function analysis
            callback_field,
            line: node.start_position().row as u32 + 1,
        })
    }

    fn is_queue_function(&self, name: &str) -> bool {
        let patterns = [
            "queue_work", "schedule_work", "schedule_delayed_work",
            "enqueue", "push", "add_task", "submit",
            "kthread_queue_work", "queue_delayed_work",
            "tasklet_schedule", "tasklet_hi_schedule",
        ];
        let name_lower = name.to_lowercase();
        patterns.iter().any(|p| name_lower.contains(p))
    }

    fn find_callback_field_in_args(&self, children: &[Node], source: &str) -> Option<String> {
        for child in children {
            if child.kind() == "argument_list" {
                let mut cursor = child.walk();
                for arg in child.children(&mut cursor) {
                    let arg_text = self.node_text(arg, source);
                    // Look for work struct references like &dev->work
                    if arg_text.contains("->") && arg_text.contains("work") {
                        return Some(arg_text);
                    }
                }
            }
        }
        None
    }

    /// Collect signal/slot patterns (connect/emit style)
    fn collect_signal_slots(&self, node: Node, source: &str, result: &mut CallbackAnalysis) {
        if node.kind() == "call_expression" {
            if let Some(pattern) = self.try_extract_signal_slot(node, source) {
                result.signal_slots.push(pattern);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_signal_slots(child, source, result);
        }
    }

    fn try_extract_signal_slot(&self, node: Node, source: &str) -> Option<SignalSlotPattern> {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();

        if children.is_empty() {
            return None;
        }

        let callee = children.first()?;
        if callee.kind() != "identifier" {
            return None;
        }

        let func_name = self.node_text(*callee, source);

        // Check if it's a signal connect function
        if !self.is_signal_connect_function(&func_name) {
            return None;
        }

        // Extract signal and handler from arguments
        let (signal, handler) = self.extract_signal_handler_args(&children, source)?;

        Some(SignalSlotPattern {
            connect_func: func_name,
            signal,
            handler,
            emit_func: None, // Would need additional analysis
            line: node.start_position().row as u32 + 1,
        })
    }

    fn is_signal_connect_function(&self, name: &str) -> bool {
        let patterns = [
            "connect", "signal_connect", "g_signal_connect",
            "on", "bind", "subscribe", "attach_handler",
            "add_signal_handler", "notify_register",
        ];
        let name_lower = name.to_lowercase();
        patterns.iter().any(|p| name_lower.contains(p))
    }

    fn extract_signal_handler_args(&self, children: &[Node], source: &str) -> Option<(String, String)> {
        for child in children {
            if child.kind() == "argument_list" {
                let mut cursor = child.walk();
                let args: Vec<Node> = child.children(&mut cursor)
                    .filter(|n| n.kind() != "," && n.kind() != "(" && n.kind() != ")")
                    .collect();

                // Typically: connect(signal, handler) or connect(object, signal, handler)
                if args.len() >= 2 {
                    let signal = self.node_text(args[0], source);
                    let last_arg = self.node_text(*args.last()?, source);

                    // Check if last arg is a known function
                    if self.functions.contains(&last_arg) {
                        return Some((signal, last_arg));
                    }

                    // Try second-to-last for cases like connect(obj, signal, handler, data)
                    if args.len() >= 3 {
                        let second_last = self.node_text(args[args.len() - 2], source);
                        if self.functions.contains(&second_last) {
                            return Some((signal, second_last));
                        }
                    }
                }
            }
        }
        None
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

    #[test]
    fn test_queue_pattern() {
        let source = r#"
void work_handler(struct work_struct *work) {
    printk("work done\n");
}

void setup(struct my_device *dev) {
    INIT_WORK(&dev->work, work_handler);
    schedule_work(&dev->work);
}
"#;
        let mut analyzer = CallbackAnalyzer::new();
        analyzer.set_functions(vec!["work_handler".to_string()]);

        let result = analyzer.analyze(source);

        assert_eq!(result.queue_patterns.len(), 1);
        assert_eq!(result.queue_patterns[0].enqueue_func, "schedule_work");
        assert!(result.queue_patterns[0].callback_field.is_some());
    }

    #[test]
    fn test_queue_work_pattern() {
        let source = r#"
void my_work_fn(struct work_struct *work) {}

void trigger(void) {
    queue_work(my_wq, &my_device->work);
}
"#;
        let mut analyzer = CallbackAnalyzer::new();
        analyzer.set_functions(vec!["my_work_fn".to_string()]);

        let result = analyzer.analyze(source);

        assert_eq!(result.queue_patterns.len(), 1);
        assert_eq!(result.queue_patterns[0].enqueue_func, "queue_work");
    }

    #[test]
    fn test_signal_slot_pattern() {
        let source = r#"
void on_button_clicked(void *data) {
    printf("Button clicked!\n");
}

void setup(void) {
    g_signal_connect(button, "clicked", on_button_clicked, NULL);
}
"#;
        let mut analyzer = CallbackAnalyzer::new();
        analyzer.set_functions(vec!["on_button_clicked".to_string()]);

        let result = analyzer.analyze(source);

        assert_eq!(result.signal_slots.len(), 1);
        assert_eq!(result.signal_slots[0].connect_func, "g_signal_connect");
        assert_eq!(result.signal_slots[0].handler, "on_button_clicked");
    }

    #[test]
    fn test_signal_connect_pattern() {
        let source = r#"
void signal_handler(int sig) {}

void setup(void) {
    signal_connect(SIGINT, signal_handler);
}
"#;
        let mut analyzer = CallbackAnalyzer::new();
        analyzer.set_functions(vec!["signal_handler".to_string()]);

        let result = analyzer.analyze(source);

        assert_eq!(result.signal_slots.len(), 1);
        assert_eq!(result.signal_slots[0].handler, "signal_handler");
    }
}
