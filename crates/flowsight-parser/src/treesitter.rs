//! Tree-sitter based parser

use crate::{ParseResult, Parser};
use flowsight_core::{FunctionDef, StructDef, StructField, Parameter, Location, Result, Error};
use std::collections::HashMap;
use tree_sitter::{Language, Node, Tree};
use regex::Regex;

/// Tree-sitter based C parser
pub struct TreeSitterParser {
    language: Language,
}

impl TreeSitterParser {
    /// Create a new tree-sitter parser
    pub fn new() -> Self {
        Self {
            language: tree_sitter_c::LANGUAGE.into(),
        }
    }

    /// Parse source code and return the syntax tree
    fn parse_to_tree(&self, source: &str) -> Result<Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&self.language)
            .map_err(|e| Error::Parse(format!("Failed to set language: {}", e)))?;
        
        parser.parse(source, None)
            .ok_or_else(|| Error::Parse("Failed to parse source".into()))
    }

    /// Extract function definitions from the tree
    fn extract_functions(&self, root: Node, source: &[u8], filename: &str) -> HashMap<String, FunctionDef> {
        let mut functions = HashMap::new();
        self.visit_functions(root, source, filename, &mut functions);
        functions
    }

    fn visit_functions(&self, node: Node, source: &[u8], filename: &str, functions: &mut HashMap<String, FunctionDef>) {
        if node.kind() == "function_definition" {
            if let Some(func) = self.extract_function(node, source, filename) {
                functions.insert(func.name.clone(), func);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_functions(child, source, filename, functions);
        }
    }

    fn extract_function(&self, node: Node, source: &[u8], filename: &str) -> Option<FunctionDef> {
        // Get return type
        let return_type = node.child_by_field_name("type")
            .map(|n| self.node_text(n, source))
            .unwrap_or_default();

        // Get declarator (contains function name and parameters)
        let declarator = node.child_by_field_name("declarator")?;
        
        let (name, params) = self.extract_function_declarator(declarator, source)?;

        // Get function body
        let body = node.child_by_field_name("body")?;
        
        // Extract calls from body
        let calls = self.extract_calls(body, source);

        // Check for attributes (static, inline, etc.)
        let mut attributes = Vec::new();
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "storage_class_specifier" {
                    attributes.push(self.node_text(child, source));
                }
            }
        }

        Some(FunctionDef {
            name,
            return_type,
            params,
            location: Some(Location::with_range(
                filename.to_string(),
                node.start_position().row as u32 + 1,
                node.start_position().column as u32,
                node.end_position().row as u32 + 1,
                node.end_position().column as u32,
            )),
            calls,
            called_by: Vec::new(),
            is_callback: false,
            callback_context: None,
            attributes,
        })
    }

    fn extract_function_declarator(&self, node: Node, source: &[u8]) -> Option<(String, Vec<Parameter>)> {
        let mut name = None;
        let mut params = Vec::new();

        if node.kind() == "function_declarator" {
            // Get function name
            if let Some(decl) = node.child_by_field_name("declarator") {
                if decl.kind() == "identifier" {
                    name = Some(self.node_text(decl, source));
                } else if decl.kind() == "pointer_declarator" {
                    // Handle pointer return type
                    name = self.extract_identifier(decl, source);
                }
            }

            // Get parameters
            if let Some(params_node) = node.child_by_field_name("parameters") {
                params = self.extract_parameters(params_node, source);
            }
        } else if node.kind() == "pointer_declarator" {
            // Recurse into pointer declarator
            if let Some(inner) = node.child_by_field_name("declarator") {
                return self.extract_function_declarator(inner, source);
            }
        }

        name.map(|n| (n, params))
    }

    fn extract_identifier(&self, node: Node, source: &[u8]) -> Option<String> {
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

    fn extract_parameters(&self, node: Node, source: &[u8]) -> Vec<Parameter> {
        let mut params = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                let type_name = child.child_by_field_name("type")
                    .map(|n| self.node_text(n, source))
                    .unwrap_or_default();
                
                let name = child.child_by_field_name("declarator")
                    .and_then(|n| self.extract_identifier(n, source))
                    .unwrap_or_default();

                params.push(Parameter { name, type_name });
            }
        }

        params
    }

    fn extract_calls(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut calls = Vec::new();
        self.visit_calls(node, source, &mut calls);
        calls.sort();
        calls.dedup();
        calls
    }

    fn visit_calls(&self, node: Node, source: &[u8], calls: &mut Vec<String>) {
        if node.kind() == "call_expression" {
            if let Some(func) = node.child_by_field_name("function") {
                if func.kind() == "identifier" {
                    let name = self.node_text(func, source);
                    // Filter out keywords
                    if !["if", "while", "for", "switch", "sizeof", "typeof"].contains(&name.as_str()) {
                        calls.push(name);
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_calls(child, source, calls);
        }
    }

    /// Extract struct definitions
    fn extract_structs(&self, root: Node, source: &[u8], filename: &str) -> HashMap<String, StructDef> {
        let mut structs = HashMap::new();
        self.visit_structs(root, source, filename, &mut structs);
        structs
    }

    fn visit_structs(&self, node: Node, source: &[u8], filename: &str, structs: &mut HashMap<String, StructDef>) {
        if node.kind() == "struct_specifier" {
            if let Some(body) = node.child_by_field_name("body") {
                if let Some(s) = self.extract_struct(node, source, filename) {
                    structs.insert(s.name.clone(), s);
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_structs(child, source, filename, structs);
        }
    }

    fn extract_struct(&self, node: Node, source: &[u8], filename: &str) -> Option<StructDef> {
        let name = node.child_by_field_name("name")
            .map(|n| self.node_text(n, source))
            .unwrap_or_else(|| format!("anonymous_{}", node.start_position().row));

        let body = node.child_by_field_name("body")?;
        let fields = self.extract_fields(body, source);

        // Find referenced structs
        let mut referenced = Vec::new();
        let struct_regex = Regex::new(r"struct\s+(\w+)").ok()?;
        for field in &fields {
            for cap in struct_regex.captures_iter(&field.type_name) {
                if let Some(m) = cap.get(1) {
                    referenced.push(m.as_str().to_string());
                }
            }
        }
        referenced.sort();
        referenced.dedup();

        Some(StructDef {
            name,
            fields,
            location: Some(Location::with_range(
                filename.to_string(),
                node.start_position().row as u32 + 1,
                node.start_position().column as u32,
                node.end_position().row as u32 + 1,
                node.end_position().column as u32,
            )),
            referenced_structs: referenced,
        })
    }

    fn extract_fields(&self, node: Node, source: &[u8]) -> Vec<StructField> {
        let mut fields = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration" {
                if let Some(field) = self.extract_field(child, source) {
                    fields.push(field);
                }
            }
        }

        fields
    }

    fn extract_field(&self, node: Node, source: &[u8]) -> Option<StructField> {
        let type_name = node.child_by_field_name("type")
            .map(|n| self.node_text(n, source))
            .unwrap_or_default();

        let declarator = node.child_by_field_name("declarator")?;
        let full_text = self.node_text(node, source);
        
        // Check for function pointer
        let is_function_ptr = full_text.contains("(*)") || full_text.contains("( *)");
        let is_pointer = full_text.contains("*");

        let name = self.extract_field_name(declarator, source)?;

        Some(StructField {
            name,
            type_name,
            is_pointer,
            is_function_ptr,
            func_ptr_signature: if is_function_ptr { Some(full_text) } else { None },
            array_size: None,
        })
    }

    fn extract_field_name(&self, node: Node, source: &[u8]) -> Option<String> {
        match node.kind() {
            "field_identifier" => Some(self.node_text(node, source)),
            "pointer_declarator" | "array_declarator" => {
                node.child_by_field_name("declarator")
                    .and_then(|n| self.extract_field_name(n, source))
            }
            _ => None,
        }
    }

    fn node_text(&self, node: Node, source: &[u8]) -> String {
        node.utf8_text(source).unwrap_or("").to_string()
    }
}

impl Default for TreeSitterParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for TreeSitterParser {
    fn parse(&self, source: &str, filename: &str) -> Result<ParseResult> {
        let tree = self.parse_to_tree(source)?;
        let source_bytes = source.as_bytes();
        let root = tree.root_node();

        let functions = self.extract_functions(root, source_bytes, filename);
        let structs = self.extract_structs(root, source_bytes, filename);

        Ok(ParseResult {
            functions,
            structs,
            errors: Vec::new(),
        })
    }

    fn name(&self) -> &str {
        "tree-sitter"
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CODE: &str = r#"
#include <linux/module.h>

struct my_device {
    int status;
    char *name;
};

static int my_probe(struct my_device *dev) {
    dev->status = 1;
    printk("probe called\n");
    return 0;
}

static void my_remove(struct my_device *dev) {
    kfree(dev->name);
}
"#;

    #[test]
    fn test_parse_functions() {
        let parser = TreeSitterParser::new();
        let result = parser.parse(SAMPLE_CODE, "test.c").unwrap();
        
        assert!(result.functions.contains_key("my_probe"));
        assert!(result.functions.contains_key("my_remove"));
        
        let probe = &result.functions["my_probe"];
        assert_eq!(probe.name, "my_probe");
        assert!(probe.calls.contains(&"printk".to_string()));
    }

    #[test]
    fn test_parse_structs() {
        let parser = TreeSitterParser::new();
        let result = parser.parse(SAMPLE_CODE, "test.c").unwrap();
        
        assert!(result.structs.contains_key("my_device"));
        
        let device = &result.structs["my_device"];
        assert_eq!(device.fields.len(), 2);
    }
}

