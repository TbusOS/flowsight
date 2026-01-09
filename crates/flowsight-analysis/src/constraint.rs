//! Constraint Collector
//!
//! Collects pointer constraints from C source code AST for Andersen analysis.

use std::collections::HashMap;
use tree_sitter::{Node, Parser as TSParser};

use super::pointer::{Constraint, Location};

/// Collects pointer constraints from C source code
pub struct ConstraintCollector {
    /// Collected constraints
    constraints: Vec<Constraint>,
    /// Known function names
    functions: HashMap<String, bool>,
    /// Current function being analyzed
    current_function: Option<String>,
    /// Array declarations with their element types (array_name -> is_function_pointer_array)
    arrays: HashMap<String, bool>,
}

impl ConstraintCollector {
    /// Create a new constraint collector
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            functions: HashMap::new(),
            current_function: None,
            arrays: HashMap::new(),
        }
    }

    /// Set known functions
    pub fn set_functions(&mut self, functions: impl IntoIterator<Item = String>) {
        self.functions = functions.into_iter().map(|f| (f, true)).collect();
    }

    /// Collect constraints from source code
    pub fn collect(&mut self, source: &str) -> Vec<Constraint> {
        self.constraints.clear();

        let mut parser = TSParser::new();
        parser
            .set_language(&tree_sitter_c::language())
            .expect("Failed to load C grammar");

        if let Some(tree) = parser.parse(source, None) {
            self.visit_node(tree.root_node(), source);
        }

        std::mem::take(&mut self.constraints)
    }

    /// Visit AST node and collect constraints
    fn visit_node(&mut self, node: Node, source: &str) {
        match node.kind() {
            "function_definition" => {
                self.current_function = self.get_function_name(node, source);
                self.visit_children(node, source);
                self.current_function = None;
            }
            "declaration" => {
                self.handle_declaration(node, source);
            }
            "init_declarator" => {
                self.handle_init_declarator(node, source);
            }
            "assignment_expression" => {
                self.handle_assignment(node, source);
            }
            "call_expression" => {
                self.handle_call(node, source);
            }
            "initializer_list" => {
                self.handle_initializer_list(node, source);
            }
            _ => {
                self.visit_children(node, source);
            }
        }
    }

    fn visit_children(&mut self, node: Node, source: &str) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child, source);
        }
    }

    /// Handle array and variable declarations
    fn handle_declaration(&mut self, node: Node, source: &str) {
        // Check for function pointer array declaration: void (*handlers[])(void) = {...}
        // Or simple array: handler_t handlers[] = {...}
        let text = self.node_text(node, source);

        // Extract array name and check if it's a function pointer array
        let mut array_name = None;
        let mut is_func_ptr_array = false;
        let mut init_list = None;

        // Parse for function pointer array pattern
        if text.contains("(*") && text.contains("[") {
            // Function pointer array: void (*handlers[N])(...)
            if let Some(start) = text.find("(*") {
                let after_paren = &text[start + 2..];
                if let Some(bracket) = after_paren.find('[') {
                    let name = after_paren[..bracket].trim();
                    if !name.is_empty() {
                        array_name = Some(name.to_string());
                        is_func_ptr_array = true;
                    }
                }
            }
        }

        // Look for array declarator pattern
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "init_declarator" {
                let mut inner_cursor = child.walk();
                for inner_child in child.children(&mut inner_cursor) {
                    if inner_child.kind() == "array_declarator" {
                        // Extract array name
                        if let Some(name) = self.extract_array_name(inner_child, source) {
                            array_name = Some(name);
                        }
                    } else if inner_child.kind() == "initializer_list" {
                        init_list = Some(inner_child);
                    }
                }
            }
        }

        // If we found an array with initializer list, extract function assignments
        if let (Some(name), Some(init)) = (array_name.clone(), init_list) {
            self.arrays.insert(name.clone(), is_func_ptr_array);
            self.handle_array_initializer(&name, init, source);
        } else if let Some(name) = array_name {
            self.arrays.insert(name, is_func_ptr_array);
        }

        // Continue visiting children for other declarations
        self.visit_children(node, source);
    }

    /// Extract array name from array_declarator node
    fn extract_array_name(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(self.node_text(child, source));
            }
            // Handle pointer declarator inside array declarator
            if child.kind() == "pointer_declarator" || child.kind() == "parenthesized_declarator" {
                if let Some(name) = self.extract_identifier(child, source) {
                    return Some(name);
                }
            }
        }
        None
    }

    /// Handle array initializer list: handlers[] = {func1, func2, func3}
    fn handle_array_initializer(&mut self, array_name: &str, init: Node, source: &str) {
        let mut cursor = init.walk();
        for child in init.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    let func_name = self.node_text(child, source);
                    if self.functions.contains_key(&func_name) {
                        self.constraints.push(Constraint::ArrayStore {
                            array: array_name.to_string(),
                            src: Location::func(&func_name),
                        });
                    }
                }
                "unary_expression" => {
                    // Handle &func in initializer
                    let text = self.node_text(child, source);
                    if text.starts_with('&') {
                        let func_name = text.trim_start_matches('&').trim();
                        if self.functions.contains_key(func_name) {
                            self.constraints.push(Constraint::ArrayStore {
                                array: array_name.to_string(),
                                src: Location::func(func_name),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Get function name from function_definition node
    fn get_function_name(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_declarator" || child.kind() == "pointer_declarator" {
                return self.extract_identifier(child, source);
            }
        }
        None
    }

    /// Extract identifier from a node
    fn extract_identifier(&self, node: Node, source: &str) -> Option<String> {
        if node.kind() == "identifier" {
            return Some(self.node_text(node, source));
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(id) = self.extract_identifier(child, source) {
                return Some(id);
            }
        }
        None
    }

    /// Handle variable initialization: type *p = expr;
    fn handle_init_declarator(&mut self, node: Node, source: &str) {
        let mut var_name = None;
        let mut init_value = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "pointer_declarator" => {
                    var_name = self.extract_identifier(child, source);
                }
                "identifier" => {
                    if var_name.is_none() {
                        var_name = Some(self.node_text(child, source));
                    } else if init_value.is_none() {
                        init_value = Some(child);
                    }
                }
                "unary_expression" | "call_expression" | "field_expression" => {
                    init_value = Some(child);
                }
                _ if child.kind().contains("expression") => {
                    init_value = Some(child);
                }
                _ => {}
            }
        }

        if let (Some(var), Some(init)) = (var_name, init_value) {
            self.collect_assignment_constraint(&var, init, source);
        }
    }

    /// Handle assignment: lhs = rhs;
    fn handle_assignment(&mut self, node: Node, source: &str) {
        let mut lhs = None;
        let mut rhs = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if lhs.is_none() && child.kind() != "=" {
                lhs = Some(child);
            } else if lhs.is_some() && child.kind() != "=" {
                rhs = Some(child);
            }
        }

        if let (Some(l), Some(r)) = (lhs, rhs) {
            // Check for array subscript assignment: arr[i] = func
            if l.kind() == "subscript_expression" {
                self.handle_array_assignment(l, r, source);
                return;
            }
            let lhs_text = self.node_text(l, source);
            self.collect_assignment_constraint(&lhs_text, r, source);
        }
    }

    /// Handle array assignment: arr[i] = func
    fn handle_array_assignment(&mut self, lhs: Node, rhs: Node, source: &str) {
        let rhs_text = self.node_text(rhs, source);

        // Extract array name from subscript expression
        let mut cursor = lhs.walk();
        for child in lhs.children(&mut cursor) {
            if child.kind() == "identifier" {
                let array_name = self.node_text(child, source);

                // Check if RHS is a function
                if self.functions.contains_key(&rhs_text) {
                    self.constraints.push(Constraint::ArrayStore {
                        array: array_name,
                        src: Location::func(&rhs_text),
                    });
                }
                return;
            }
        }
    }

    /// Collect constraint from an assignment
    fn collect_assignment_constraint(&mut self, lhs: &str, rhs: Node, source: &str) {
        let rhs_text = self.node_text(rhs, source);

        // Check for address-of: p = &x
        if rhs.kind() == "unary_expression" {
            let mut cursor = rhs.walk();
            let children: Vec<_> = rhs.children(&mut cursor).collect();
            if children.len() >= 2 && self.node_text(children[0], source) == "&" {
                let target = self.node_text(children[1], source);
                let target_loc = if self.functions.contains_key(&target) {
                    Location::func(&target)
                } else {
                    Location::var(&target)
                };

                self.constraints.push(Constraint::AddressOf {
                    pointer: self.parse_location(lhs),
                    target: target_loc,
                });
                return;
            }
        }

        // Check for function name (implicit address-of)
        if self.functions.contains_key(&rhs_text) {
            self.constraints.push(Constraint::AddressOf {
                pointer: self.parse_location(lhs),
                target: Location::func(&rhs_text),
            });
            return;
        }

        // Check for field access: p = obj->field or p = obj.field
        if rhs.kind() == "field_expression" {
            let (base, field) = self.parse_field_expression(rhs, source);
            if let Some(f) = field {
                self.constraints.push(Constraint::FieldLoad {
                    dest: self.parse_location(lhs),
                    base_ptr: Location::var(&base),
                    field: f,
                });
                return;
            }
        }

        // Check for dereference: p = *q
        if rhs.kind() == "pointer_expression" ||
           (rhs.kind() == "unary_expression" && rhs_text.starts_with('*')) {
            let inner = rhs_text.trim_start_matches('*').trim();
            self.constraints.push(Constraint::Load {
                dest: self.parse_location(lhs),
                src_ptr: Location::var(inner),
            });
            return;
        }

        // Check for store: *p = q
        if lhs.starts_with('*') {
            let ptr = lhs.trim_start_matches('*').trim();
            self.constraints.push(Constraint::Store {
                dest_ptr: Location::var(ptr),
                src: self.parse_location(&rhs_text),
            });
            return;
        }

        // Check for field store: obj->field = value
        if lhs.contains("->") || lhs.contains('.') {
            let (base, field) = self.parse_field_str(lhs);
            if let Some(f) = field {
                self.constraints.push(Constraint::FieldStore {
                    base_ptr: Location::var(&base),
                    field: f,
                    src: self.parse_location(&rhs_text),
                });
                return;
            }
        }

        // Simple copy: p = q
        self.constraints.push(Constraint::Copy {
            dest: self.parse_location(lhs),
            src: self.parse_location(&rhs_text),
        });
    }

    /// Handle function calls for callback registration patterns
    fn handle_call(&mut self, node: Node, source: &str) {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        if children.is_empty() {
            return;
        }

        let callee = children[0];

        // Handle array-based indirect calls: handlers[i]()
        if callee.kind() == "subscript_expression" {
            self.handle_array_call(callee, source);
            self.visit_children(node, source);
            return;
        }

        // Handle pointer-based indirect calls: (*func_ptr)()
        if callee.kind() == "parenthesized_expression" {
            // The actual function pointer variable is inside
            let inner = self.node_text(callee, source);
            let inner = inner.trim_start_matches('(').trim_end_matches(')');
            if inner.starts_with('*') {
                let _ptr_name = inner.trim_start_matches('*').trim();
                // Create an ArrayLoad-like constraint to track the call target
                // This is handled elsewhere, just note the indirect call
            }
        }

        let func_name = self.node_text(callee, source);

        // Handle common callback registration patterns
        match func_name.as_str() {
            // INIT_WORK(&work, handler)
            "INIT_WORK" | "INIT_DELAYED_WORK" => {
                if let Some(args) = self.get_call_args(node, source) {
                    if args.len() >= 2 {
                        let work = args[0].trim_start_matches('&');
                        let handler = &args[1];
                        if self.functions.contains_key(handler) {
                            self.constraints.push(Constraint::FieldStore {
                                base_ptr: Location::var(work),
                                field: "func".to_string(),
                                src: Location::func(handler),
                            });
                        }
                    }
                }
            }
            // timer_setup(&timer, handler, flags)
            "timer_setup" => {
                if let Some(args) = self.get_call_args(node, source) {
                    if args.len() >= 2 {
                        let timer = args[0].trim_start_matches('&');
                        let handler = &args[1];
                        if self.functions.contains_key(handler) {
                            self.constraints.push(Constraint::FieldStore {
                                base_ptr: Location::var(timer),
                                field: "function".to_string(),
                                src: Location::func(handler),
                            });
                        }
                    }
                }
            }
            // request_irq(irq, handler, ...)
            "request_irq" | "request_threaded_irq" => {
                if let Some(args) = self.get_call_args(node, source) {
                    if args.len() >= 2 {
                        let handler = &args[1];
                        if self.functions.contains_key(handler) {
                            self.constraints.push(Constraint::AddressOf {
                                pointer: Location::var(&format!("irq_{}", args[0])),
                                target: Location::func(handler),
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        self.visit_children(node, source);
    }

    /// Handle array-based indirect calls: handlers[i]()
    fn handle_array_call(&mut self, subscript: Node, source: &str) {
        // Extract array name from subscript expression
        let mut cursor = subscript.walk();
        for child in subscript.children(&mut cursor) {
            if child.kind() == "identifier" {
                let array_name = self.node_text(child, source);
                // Create a synthetic variable for the call target
                let call_target = format!("__call_from_{}", array_name);
                self.constraints.push(Constraint::ArrayLoad {
                    dest: Location::var(&call_target),
                    array: array_name.clone(),
                });
                return;
            }
        }
    }

    /// Handle struct initializer lists
    fn handle_initializer_list(&mut self, node: Node, source: &str) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "initializer_pair" {
                self.handle_initializer_pair(child, source);
            }
        }
    }

    /// Handle .field = value in initializer
    fn handle_initializer_pair(&mut self, node: Node, source: &str) {
        let mut field_name = None;
        let mut value = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "field_designator" => {
                    field_name = self.extract_identifier(child, source)
                        .or_else(|| {
                            let text = self.node_text(child, source);
                            Some(text.trim_start_matches('.').to_string())
                        });
                }
                "identifier" => {
                    if field_name.is_some() {
                        value = Some(self.node_text(child, source));
                    }
                }
                _ => {}
            }
        }

        if let (Some(field), Some(val)) = (field_name, value) {
            if self.functions.contains_key(&val) {
                // This is a function pointer assignment in struct initializer
                // We'll create a constraint when we know the struct variable name
                self.constraints.push(Constraint::AddressOf {
                    pointer: Location::field("__init__", &field),
                    target: Location::func(&val),
                });
            }
        }
    }

    /// Get call arguments as strings
    fn get_call_args(&self, node: Node, source: &str) -> Option<Vec<String>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "argument_list" {
                let mut args = Vec::new();
                let mut arg_cursor = child.walk();
                for arg in child.children(&mut arg_cursor) {
                    if arg.kind() != "(" && arg.kind() != ")" && arg.kind() != "," {
                        args.push(self.node_text(arg, source));
                    }
                }
                return Some(args);
            }
        }
        None
    }

    /// Parse a location from a string
    fn parse_location(&self, s: &str) -> Location {
        let s = s.trim();
        if s.contains("->") || s.contains('.') {
            let (base, field) = self.parse_field_str(s);
            if let Some(f) = field {
                return Location::field(&base, &f);
            }
        }
        Location::var(s)
    }

    /// Parse field expression node
    fn parse_field_expression(&self, node: Node, source: &str) -> (String, Option<String>) {
        let text = self.node_text(node, source);
        self.parse_field_str(&text)
    }

    /// Parse field access string like "obj->field" or "obj.field"
    fn parse_field_str(&self, s: &str) -> (String, Option<String>) {
        if let Some(pos) = s.find("->") {
            let base = s[..pos].trim().to_string();
            let field = s[pos + 2..].trim().to_string();
            return (base, Some(field));
        }
        if let Some(pos) = s.rfind('.') {
            let base = s[..pos].trim().to_string();
            let field = s[pos + 1..].trim().to_string();
            return (base, Some(field));
        }
        (s.to_string(), None)
    }

    /// Get text of a node
    fn node_text(&self, node: Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }
}

impl Default for ConstraintCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_init_work() {
        let source = r#"
void work_handler(struct work_struct *work) {}

void init(void) {
    INIT_WORK(&dev->work, work_handler);
}
"#;
        let mut collector = ConstraintCollector::new();
        collector.set_functions(vec!["work_handler".to_string()]);
        let constraints = collector.collect(source);

        assert!(constraints.iter().any(|c| matches!(c,
            Constraint::FieldStore { field, .. }
            if field == "func"
        )));
    }

    #[test]
    fn test_parse_field_str() {
        let collector = ConstraintCollector::new();

        let (base, field) = collector.parse_field_str("dev->callback");
        assert_eq!(base, "dev");
        assert_eq!(field, Some("callback".to_string()));

        let (base, field) = collector.parse_field_str("obj.handler");
        assert_eq!(base, "obj");
        assert_eq!(field, Some("handler".to_string()));
    }

    #[test]
    fn test_parse_location() {
        let collector = ConstraintCollector::new();

        let loc = collector.parse_location("fp");
        assert!(matches!(loc, Location::Variable(v) if v == "fp"));

        let loc = collector.parse_location("dev->callback");
        assert!(matches!(loc, Location::Field(b, f) if b == "dev" && f == "callback"));
    }

    #[test]
    fn test_array_assignment() {
        let source = r#"
void handler1(void) {}
void handler2(void) {}

void init(void) {
    handlers[0] = handler1;
    handlers[1] = handler2;
}
"#;
        let mut collector = ConstraintCollector::new();
        collector.set_functions(vec!["handler1".to_string(), "handler2".to_string()]);
        let constraints = collector.collect(source);

        let array_stores: Vec<_> = constraints.iter().filter(|c| {
            matches!(c, Constraint::ArrayStore { array, .. } if array == "handlers")
        }).collect();

        assert_eq!(array_stores.len(), 2);
    }

    #[test]
    fn test_array_initialization() {
        let source = r#"
void func1(void) {}
void func2(void) {}
void func3(void) {}

handler_t handlers[] = {func1, func2, func3};
"#;
        let mut collector = ConstraintCollector::new();
        collector.set_functions(vec![
            "func1".to_string(),
            "func2".to_string(),
            "func3".to_string(),
        ]);
        let constraints = collector.collect(source);

        let array_stores: Vec<_> = constraints.iter().filter(|c| {
            matches!(c, Constraint::ArrayStore { array, .. } if array == "handlers")
        }).collect();

        assert_eq!(array_stores.len(), 3);
    }

    #[test]
    fn test_array_call() {
        let source = r#"
void dispatch(int cmd) {
    handlers[cmd]();
}
"#;
        let mut collector = ConstraintCollector::new();
        let constraints = collector.collect(source);

        let array_loads: Vec<_> = constraints.iter().filter(|c| {
            matches!(c, Constraint::ArrayLoad { array, .. } if array == "handlers")
        }).collect();

        assert_eq!(array_loads.len(), 1);
    }

    #[test]
    fn test_array_full_flow() {
        // Test the complete flow: declaration, assignment, and call
        let source = r#"
void read_handler(void) {}
void write_handler(void) {}

void init(void) {
    ops[0] = read_handler;
    ops[1] = write_handler;
}

void dispatch(int cmd) {
    ops[cmd]();
}
"#;
        let mut collector = ConstraintCollector::new();
        collector.set_functions(vec![
            "read_handler".to_string(),
            "write_handler".to_string(),
        ]);
        let constraints = collector.collect(source);

        // Should have 2 ArrayStore for assignments
        let array_stores: Vec<_> = constraints.iter().filter(|c| {
            matches!(c, Constraint::ArrayStore { array, .. } if array == "ops")
        }).collect();
        assert_eq!(array_stores.len(), 2);

        // Should have 1 ArrayLoad for call
        let array_loads: Vec<_> = constraints.iter().filter(|c| {
            matches!(c, Constraint::ArrayLoad { array, .. } if array == "ops")
        }).collect();
        assert_eq!(array_loads.len(), 1);
    }
}
