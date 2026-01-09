//! Function Pointer Type Analysis
//!
//! Analyzes C code to identify and track function pointer types:
//! - typedef: `typedef void (*callback_t)(int);`
//! - struct field: `struct ops { int (*open)(void); };`
//! - function param: `void register_cb(void (*cb)(void));`
//!
//! Builds a database of function pointer types and compatible functions.

use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Parser as TSParser};
use serde::{Deserialize, Serialize};

/// A function pointer type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncPtrType {
    /// Type name (e.g., "callback_t" for typedef, or "ops.open" for struct field)
    pub name: String,
    /// Return type
    pub return_type: String,
    /// Parameter types
    pub param_types: Vec<String>,
    /// Source location (file:line)
    pub location: Option<String>,
    /// How this type was defined
    pub definition_kind: FuncPtrDefKind,
}

/// How a function pointer type was defined
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FuncPtrDefKind {
    /// typedef void (*callback_t)(int);
    Typedef,
    /// struct ops { int (*open)(void); };
    StructField,
    /// void register_cb(void (*cb)(void));
    FunctionParam,
    /// void (*global_cb)(void);
    GlobalVar,
}

/// A function signature for compatibility checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Return type
    pub return_type: String,
    /// Parameter types
    pub param_types: Vec<String>,
}

/// Function pointer type database
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TypeDatabase {
    /// All function pointer types found
    pub func_ptr_types: HashMap<String, FuncPtrType>,
    /// All function signatures
    pub function_sigs: HashMap<String, FunctionSignature>,
    /// Compatibility map: type name -> set of compatible functions
    pub compatible_funcs: HashMap<String, HashSet<String>>,
}

impl TypeDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a function pointer type
    pub fn add_type(&mut self, fp_type: FuncPtrType) {
        self.func_ptr_types.insert(fp_type.name.clone(), fp_type);
    }

    /// Add a function signature
    pub fn add_function(&mut self, sig: FunctionSignature) {
        self.function_sigs.insert(sig.name.clone(), sig);
    }

    /// Check if a function is compatible with a function pointer type
    pub fn is_compatible(&self, func_name: &str, type_name: &str) -> bool {
        let fp_type = match self.func_ptr_types.get(type_name) {
            Some(t) => t,
            None => return false,
        };

        let func_sig = match self.function_sigs.get(func_name) {
            Some(s) => s,
            None => return false,
        };

        // Check return type compatibility
        if !self.types_compatible(&func_sig.return_type, &fp_type.return_type) {
            return false;
        }

        // Check parameter count
        if func_sig.param_types.len() != fp_type.param_types.len() {
            return false;
        }

        // Check each parameter type
        for (func_param, fp_param) in func_sig.param_types.iter().zip(&fp_type.param_types) {
            if !self.types_compatible(func_param, fp_param) {
                return false;
            }
        }

        true
    }

    /// Build compatibility map for all types and functions
    pub fn build_compatibility_map(&mut self) {
        self.compatible_funcs.clear();

        for type_name in self.func_ptr_types.keys() {
            let mut compatible = HashSet::new();
            for func_name in self.function_sigs.keys() {
                if self.is_compatible(func_name, type_name) {
                    compatible.insert(func_name.clone());
                }
            }
            if !compatible.is_empty() {
                self.compatible_funcs.insert(type_name.clone(), compatible);
            }
        }
    }

    /// Get functions compatible with a type
    pub fn get_compatible_functions(&self, type_name: &str) -> Option<&HashSet<String>> {
        self.compatible_funcs.get(type_name)
    }

    /// Check if two types are compatible (simplified comparison)
    fn types_compatible(&self, t1: &str, t2: &str) -> bool {
        let t1_norm = self.normalize_type(t1);
        let t2_norm = self.normalize_type(t2);

        // Exact match
        if t1_norm == t2_norm {
            return true;
        }

        // void* is compatible with any pointer
        if (t1_norm == "void*" && t2_norm.ends_with('*')) ||
           (t2_norm == "void*" && t1_norm.ends_with('*')) {
            return true;
        }

        // int/long/unsigned variations
        let int_types = ["int", "long", "unsigned", "unsigned int", "unsigned long", "size_t", "ssize_t"];
        if int_types.contains(&t1_norm.as_str()) && int_types.contains(&t2_norm.as_str()) {
            return true;
        }

        false
    }

    /// Normalize a type string
    fn normalize_type(&self, t: &str) -> String {
        t.trim()
            .replace("const ", "")
            .replace("volatile ", "")
            .replace("struct ", "")
            .replace("  ", " ")
            .trim()
            .to_string()
    }
}

/// Type analyzer for C code
pub struct TypeAnalyzer {
    /// Collected types
    database: TypeDatabase,
}

impl TypeAnalyzer {
    pub fn new() -> Self {
        Self {
            database: TypeDatabase::new(),
        }
    }

    /// Analyze source code and extract types
    pub fn analyze(&mut self, source: &str) -> &TypeDatabase {
        let mut parser = TSParser::new();
        parser
            .set_language(&tree_sitter_c::language())
            .expect("Failed to load C grammar");

        if let Some(tree) = parser.parse(source, None) {
            self.collect_typedefs(tree.root_node(), source);
            self.collect_struct_fields(tree.root_node(), source);
            self.collect_function_params(tree.root_node(), source);
            self.collect_function_signatures(tree.root_node(), source);
        }

        self.database.build_compatibility_map();
        &self.database
    }

    /// Get the database
    pub fn database(&self) -> &TypeDatabase {
        &self.database
    }

    /// Take ownership of the database
    pub fn into_database(self) -> TypeDatabase {
        self.database
    }

    /// Collect typedef function pointers
    fn collect_typedefs(&mut self, node: Node, source: &str) {
        // Use text-based parsing for typedef function pointers
        if node.kind() == "type_definition" {
            let text = self.node_text(node, source);
            if text.contains("(*") {
                if let Some(fp_type) = self.parse_funcptr_typedef(&text, node.start_position().row as u32 + 1) {
                    self.database.add_type(fp_type);
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_typedefs(child, source);
        }
    }

    fn parse_funcptr_typedef(&self, text: &str, line: u32) -> Option<FuncPtrType> {
        // Pattern: typedef <return_type> (*<name>)(<params>);
        let re = regex::Regex::new(r"typedef\s+(\w+)\s*\(\s*\*\s*(\w+)\s*\)\s*\(([^)]*)\)").ok()?;

        if let Some(caps) = re.captures(text) {
            let return_type = caps.get(1)?.as_str().to_string();
            let name = caps.get(2)?.as_str().to_string();
            let params_str = caps.get(3)?.as_str();

            let param_types: Vec<String> = if params_str.trim().is_empty() || params_str.trim() == "void" {
                Vec::new()
            } else {
                params_str.split(',')
                    .map(|p| self.simplify_type(p.trim()))
                    .collect()
            };

            return Some(FuncPtrType {
                name,
                return_type,
                param_types,
                location: Some(format!("line:{}", line)),
                definition_kind: FuncPtrDefKind::Typedef,
            });
        }

        None
    }

    fn simplify_type(&self, param: &str) -> String {
        // Simplify type: "int arg" -> "int", "const void *ptr" -> "const void *"
        let param = param.trim();
        if param.is_empty() {
            return String::new();
        }

        // If ends with identifier (no * or &), remove it
        let parts: Vec<&str> = param.split_whitespace().collect();
        if parts.len() > 1 {
            let last = parts.last().unwrap();
            if !last.contains('*') && !last.contains('&') && last.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return parts[..parts.len()-1].join(" ");
            }
        }
        param.to_string()
    }

    fn extract_func_declarator(&self, node: Node, source: &str) -> Option<(String, Vec<String>)> {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();

        let mut name = None;
        let mut params = Vec::new();

        for child in children {
            match child.kind() {
                "identifier" | "type_identifier" => {
                    name = Some(self.node_text(child, source));
                }
                "parenthesized_declarator" | "pointer_declarator" => {
                    // Get name from nested declarator
                    name = self.extract_declarator_name(child, source);
                }
                "parameter_list" => {
                    params = self.extract_param_types(child, source);
                }
                _ => {}
            }
        }

        name.map(|n| (n, params))
    }

    fn extract_pointer_func_declarator(&self, node: Node, source: &str) -> Option<(String, Vec<String>)> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_declarator" {
                return self.extract_func_declarator(child, source);
            }
            if child.kind() == "pointer_declarator" {
                return self.extract_pointer_func_declarator(child, source);
            }
            if child.kind() == "parenthesized_declarator" {
                // Check inside parenthesized
                let mut inner_cursor = child.walk();
                for inner in child.children(&mut inner_cursor) {
                    if inner.kind() == "pointer_declarator" {
                        return self.extract_pointer_func_declarator(inner, source);
                    }
                }
            }
        }
        None
    }

    fn extract_declarator_name(&self, node: Node, source: &str) -> Option<String> {
        if node.kind() == "identifier" {
            return Some(self.node_text(node, source));
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(name) = self.extract_declarator_name(child, source) {
                return Some(name);
            }
        }
        None
    }

    fn extract_param_types(&self, node: Node, source: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                let param_type = self.extract_type_from_param(child, source);
                if !param_type.is_empty() {
                    params.push(param_type);
                }
            }
        }
        params
    }

    fn extract_type_from_param(&self, node: Node, source: &str) -> String {
        let mut type_parts = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "primitive_type" | "type_identifier" | "sized_type_specifier" => {
                    type_parts.push(self.node_text(child, source));
                }
                "struct_specifier" => {
                    type_parts.push(format!("struct {}", self.extract_struct_name(child, source).unwrap_or_default()));
                }
                "pointer_declarator" | "abstract_pointer_declarator" => {
                    type_parts.push("*".to_string());
                }
                _ => {}
            }
        }

        type_parts.join(" ").trim().to_string()
    }

    fn extract_struct_name(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Some(self.node_text(child, source));
            }
        }
        None
    }

    /// Collect function pointer fields from struct definitions
    fn collect_struct_fields(&mut self, node: Node, source: &str) {
        if node.kind() == "struct_specifier" {
            self.extract_struct_funcptr_fields(node, source);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_struct_fields(child, source);
        }
    }

    fn extract_struct_funcptr_fields(&mut self, node: Node, source: &str) {
        let struct_name = self.extract_struct_name(node, source).unwrap_or_else(|| "anonymous".to_string());

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration_list" {
                self.process_field_list(&struct_name, child, source);
            }
        }
    }

    fn process_field_list(&mut self, struct_name: &str, node: Node, source: &str) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration" {
                let text = self.node_text(child, source);
                // Check if this looks like a function pointer field
                if text.contains("(*") {
                    if let Some(fp_type) = self.parse_funcptr_field(struct_name, &text, child.start_position().row as u32 + 1) {
                        self.database.add_type(fp_type);
                    }
                }
            }
        }
    }

    fn parse_funcptr_field(&self, struct_name: &str, text: &str, line: u32) -> Option<FuncPtrType> {
        // Pattern: <return_type> (*<name>)(<params>);
        let re = regex::Regex::new(r"(\w+)\s*\(\s*\*\s*(\w+)\s*\)\s*\(([^)]*)\)").ok()?;

        if let Some(caps) = re.captures(text) {
            let return_type = caps.get(1)?.as_str().to_string();
            let field_name = caps.get(2)?.as_str().to_string();
            let params_str = caps.get(3)?.as_str();

            let param_types: Vec<String> = if params_str.trim().is_empty() || params_str.trim() == "void" {
                Vec::new()
            } else {
                params_str.split(',')
                    .map(|p| self.simplify_type(p.trim()))
                    .collect()
            };

            return Some(FuncPtrType {
                name: format!("{}.{}", struct_name, field_name),
                return_type,
                param_types,
                location: Some(format!("line:{}", line)),
                definition_kind: FuncPtrDefKind::StructField,
            });
        }

        None
    }

    /// Collect function pointer parameters
    fn collect_function_params(&mut self, node: Node, source: &str) {
        if node.kind() == "function_definition" || node.kind() == "declaration" {
            self.extract_funcptr_params(node, source);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_function_params(child, source);
        }
    }

    fn extract_funcptr_params(&mut self, node: Node, source: &str) {
        // Find function declarator and its parameter list
        let func_name = self.find_function_name(node, source);

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_declarator" {
                let mut inner_cursor = child.walk();
                for inner_child in child.children(&mut inner_cursor) {
                    if inner_child.kind() == "parameter_list" {
                        self.process_params_for_funcptrs(&func_name, inner_child, source);
                    }
                }
            }
        }
    }

    fn find_function_name(&self, node: Node, source: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_declarator" {
                if let Some((name, _)) = self.extract_func_declarator(child, source) {
                    return name;
                }
            }
        }
        "unknown".to_string()
    }

    fn process_params_for_funcptrs(&mut self, func_name: &str, node: Node, source: &str) {
        let mut cursor = node.walk();
        let mut param_index = 0;

        for child in node.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                if let Some(fp_type) = self.try_extract_param_funcptr(func_name, param_index, child, source) {
                    self.database.add_type(fp_type);
                }
                param_index += 1;
            }
        }
    }

    fn try_extract_param_funcptr(&self, func_name: &str, param_index: usize, node: Node, source: &str) -> Option<FuncPtrType> {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();

        let mut return_type = String::new();
        let mut param_name = None;
        let mut param_types = Vec::new();

        for child in &children {
            match child.kind() {
                "primitive_type" | "type_identifier" => {
                    return_type = self.node_text(*child, source);
                }
                "function_declarator" => {
                    if let Some((name, params)) = self.extract_func_declarator(*child, source) {
                        param_name = Some(name);
                        param_types = params;
                    }
                }
                "pointer_declarator" => {
                    if let Some((name, params)) = self.extract_pointer_func_declarator(*child, source) {
                        param_name = Some(name);
                        param_types = params;
                    }
                }
                _ => {}
            }
        }

        // Only add if this looks like a function pointer parameter
        if param_types.is_empty() {
            return None;
        }

        let name = param_name.unwrap_or_else(|| format!("param{}", param_index));

        Some(FuncPtrType {
            name: format!("{}::{}", func_name, name),
            return_type,
            param_types,
            location: Some(format!("line:{}", node.start_position().row + 1)),
            definition_kind: FuncPtrDefKind::FunctionParam,
        })
    }

    /// Collect all function signatures
    fn collect_function_signatures(&mut self, node: Node, source: &str) {
        if node.kind() == "function_definition" {
            if let Some(sig) = self.extract_function_signature(node, source) {
                self.database.add_function(sig);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_function_signatures(child, source);
        }
    }

    fn extract_function_signature(&self, node: Node, source: &str) -> Option<FunctionSignature> {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();

        let mut return_type = String::new();
        let mut func_name = None;
        let mut param_types = Vec::new();

        for child in &children {
            match child.kind() {
                "primitive_type" | "type_identifier" => {
                    if return_type.is_empty() {
                        return_type = self.node_text(*child, source);
                    }
                }
                "pointer_declarator" => {
                    // Return type is a pointer
                    if return_type.is_empty() {
                        return_type = "void*".to_string();
                    }
                }
                "function_declarator" => {
                    if let Some((name, params)) = self.extract_func_declarator(*child, source) {
                        func_name = Some(name);
                        param_types = params;
                    }
                }
                _ => {}
            }
        }

        let name = func_name?;

        Some(FunctionSignature {
            name,
            return_type,
            param_types,
        })
    }

    fn node_text(&self, node: Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }
}

impl Default for TypeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typedef_funcptr() {
        let source = r#"
typedef void (*callback_t)(int arg);
typedef int (*compare_fn)(const void *, const void *);
"#;
        let mut analyzer = TypeAnalyzer::new();
        analyzer.analyze(source);

        let db = analyzer.database();
        assert!(db.func_ptr_types.contains_key("callback_t"), "Should find callback_t typedef");
        assert!(db.func_ptr_types.contains_key("compare_fn"), "Should find compare_fn typedef");
    }

    #[test]
    fn test_struct_field_funcptr() {
        let source = r#"
struct file_operations {
    int (*open)(struct inode *, struct file *);
    ssize_t (*read)(struct file *, char *, size_t, loff_t *);
    int (*release)(struct inode *, struct file *);
};
"#;
        let mut analyzer = TypeAnalyzer::new();
        analyzer.analyze(source);

        let db = analyzer.database();
        assert!(db.func_ptr_types.contains_key("file_operations.open"), "Should find open field");
        assert!(db.func_ptr_types.contains_key("file_operations.read"), "Should find read field");
        assert!(db.func_ptr_types.contains_key("file_operations.release"), "Should find release field");
    }

    #[test]
    fn test_function_param_funcptr() {
        let source = r#"
void register_callback(void (*cb)(int event)) {
    callbacks[next++] = cb;
}

void set_handler(int (*handler)(void *data), void *ctx) {
    global_handler = handler;
}
"#;
        let mut analyzer = TypeAnalyzer::new();
        analyzer.analyze(source);

        let db = analyzer.database();

        // Should find function pointer parameters
        let has_cb_param = db.func_ptr_types.keys()
            .any(|k| k.contains("register_callback") && k.contains("cb"));
        let has_handler_param = db.func_ptr_types.keys()
            .any(|k| k.contains("set_handler") && k.contains("handler"));

        assert!(has_cb_param, "Should find cb parameter: {:?}", db.func_ptr_types.keys().collect::<Vec<_>>());
        assert!(has_handler_param, "Should find handler parameter: {:?}", db.func_ptr_types.keys().collect::<Vec<_>>());
    }

    #[test]
    fn test_function_signature() {
        let source = r#"
void my_callback(int event) {
    printf("Event: %d\n", event);
}

int my_compare(const void *a, const void *b) {
    return *(int*)a - *(int*)b;
}
"#;
        let mut analyzer = TypeAnalyzer::new();
        analyzer.analyze(source);

        let db = analyzer.database();
        assert!(db.function_sigs.contains_key("my_callback"), "Should find my_callback");
        assert!(db.function_sigs.contains_key("my_compare"), "Should find my_compare");

        let cb_sig = db.function_sigs.get("my_callback").unwrap();
        assert_eq!(cb_sig.return_type, "void");
        assert_eq!(cb_sig.param_types.len(), 1);
    }

    #[test]
    fn test_compatibility_check() {
        let source = r#"
typedef void (*callback_t)(int);

void handler1(int x) {}
void handler2(int y) {}
int wrong_return(int x) { return x; }
void wrong_params(void) {}
"#;
        let mut analyzer = TypeAnalyzer::new();
        analyzer.analyze(source);

        let db = analyzer.database();

        // Check compatibility
        let compatible = db.get_compatible_functions("callback_t");
        assert!(compatible.is_some(), "Should have compatible functions for callback_t");

        let funcs = compatible.unwrap();
        assert!(funcs.contains("handler1"), "handler1 should be compatible");
        assert!(funcs.contains("handler2"), "handler2 should be compatible");
        // wrong_return and wrong_params should NOT be compatible (different signature)
    }

    #[test]
    fn test_usb_driver_types() {
        let source = r#"
struct usb_driver {
    int (*probe)(struct usb_interface *intf, const struct usb_device_id *id);
    void (*disconnect)(struct usb_interface *intf);
};

static int my_probe(struct usb_interface *intf, const struct usb_device_id *id) {
    return 0;
}

static void my_disconnect(struct usb_interface *intf) {
}
"#;
        let mut analyzer = TypeAnalyzer::new();
        analyzer.analyze(source);

        let db = analyzer.database();

        // Should find struct field types
        assert!(db.func_ptr_types.contains_key("usb_driver.probe"), "Should find probe field");
        assert!(db.func_ptr_types.contains_key("usb_driver.disconnect"), "Should find disconnect field");

        // Should find function signatures
        assert!(db.function_sigs.contains_key("my_probe"), "Should find my_probe");
        assert!(db.function_sigs.contains_key("my_disconnect"), "Should find my_disconnect");
    }
}
