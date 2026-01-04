//! Tree-sitter based parser for C language
//!
//! Provides fast incremental parsing using tree-sitter.

use flowsight_core::{FunctionDef, Location, Parameter, Result, StructDef, StructField};
use std::collections::HashMap;
use tree_sitter::{Parser as TSParser, Node, Tree};
use tracing::debug;

use crate::ParseResult;

/// Tree-sitter based parser
pub struct TreeSitterParser {
    parser: TSParser,
}

impl TreeSitterParser {
    /// Create a new Tree-sitter parser for C
    pub fn new() -> Self {
        let mut parser = TSParser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .expect("Failed to load C grammar");
        Self { parser }
    }

    /// Parse source code and extract information
    pub fn parse_source(&mut self, source: &str, filename: &str) -> Result<ParseResult> {
        let tree = self
            .parser
            .parse(source, None)
            .ok_or_else(|| flowsight_core::Error::Parse("Failed to parse source".into()))?;

        let mut result = ParseResult::default();
        self.extract_definitions(&tree, source, filename, &mut result);
        Ok(result)
    }

    /// Parse with incremental update support
    pub fn parse_incremental(
        &mut self,
        source: &str,
        old_tree: Option<&Tree>,
        filename: &str,
    ) -> Result<(ParseResult, Tree)> {
        let tree = self
            .parser
            .parse(source, old_tree)
            .ok_or_else(|| flowsight_core::Error::Parse("Failed to parse source".into()))?;

        let mut result = ParseResult::default();
        self.extract_definitions(&tree, source, filename, &mut result);
        Ok((result, tree))
    }

    fn extract_definitions(
        &self,
        tree: &Tree,
        source: &str,
        filename: &str,
        result: &mut ParseResult,
    ) {
        let root = tree.root_node();
        self.visit_node(root, source, filename, result);
    }

    fn visit_node(&self, node: Node, source: &str, filename: &str, result: &mut ParseResult) {
        match node.kind() {
            "function_definition" => {
                if let Some(func) = self.extract_function(node, source, filename) {
                    debug!("Found function: {}", func.name);
                    result.functions.insert(func.name.clone(), func);
                }
            }
            "struct_specifier" => {
                if let Some(st) = self.extract_struct(node, source, filename) {
                    debug!("Found struct: {}", st.name);
                    result.structs.insert(st.name.clone(), st);
                }
            }
            _ => {}
        }

        // Recursively visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child, source, filename, result);
        }
    }

    fn extract_function(&self, node: Node, source: &str, filename: &str) -> Option<FunctionDef> {
        let mut name = String::new();
        let mut return_type = String::new();
        let mut params = Vec::new();
        let mut calls = Vec::new();
        let mut attributes = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "storage_class_specifier" | "type_qualifier" => {
                    let attr = self.node_text(child, source);
                    attributes.push(attr);
                }
                "primitive_type" | "type_identifier" | "sized_type_specifier" => {
                    return_type = self.node_text(child, source);
                }
                "pointer_declarator" | "function_declarator" => {
                    name = self.extract_function_name(child, source);
                    params = self.extract_parameters(child, source);
                }
                "compound_statement" => {
                    // Extract function calls from body
                    calls = self.extract_calls(child, source);
                }
                _ => {}
            }
        }

        if name.is_empty() {
            return None;
        }

        Some(FunctionDef {
            name,
            return_type,
            params,
            location: Some(Location::new(
                filename,
                node.start_position().row as u32 + 1,
                node.start_position().column as u32,
            )),
            calls,
            called_by: Vec::new(),
            is_callback: false,
            callback_context: None,
            attributes,
        })
    }

    fn extract_function_name(&self, node: Node, source: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    return self.node_text(child, source);
                }
                "pointer_declarator" | "function_declarator" => {
                    let result = self.extract_function_name(child, source);
                    if !result.is_empty() {
                        return result;
                    }
                }
                _ => {}
            }
        }
        String::new()
    }

    fn extract_parameters(&self, node: Node, source: &str) -> Vec<Parameter> {
        let mut params = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "parameter_list" {
                let mut param_cursor = child.walk();
                for param_node in child.children(&mut param_cursor) {
                    if param_node.kind() == "parameter_declaration" {
                        if let Some(param) = self.extract_parameter(param_node, source) {
                            params.push(param);
                        }
                    }
                }
            } else if child.kind() == "function_declarator" {
                return self.extract_parameters(child, source);
            }
        }

        params
    }

    fn extract_parameter(&self, node: Node, source: &str) -> Option<Parameter> {
        let mut name = String::new();
        let mut type_name = String::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "primitive_type" | "type_identifier" | "sized_type_specifier" => {
                    if type_name.is_empty() {
                        type_name = self.node_text(child, source);
                    }
                }
                "struct_specifier" => {
                    type_name = format!("struct {}", self.get_struct_name(child, source));
                }
                "pointer_declarator" => {
                    type_name = format!("{}*", type_name);
                    name = self.extract_identifier(child, source);
                }
                "identifier" => {
                    name = self.node_text(child, source);
                }
                _ => {}
            }
        }

        if name.is_empty() && !type_name.is_empty() {
            // Anonymous parameter
            Some(Parameter {
                name: String::new(),
                type_name,
            })
        } else if !name.is_empty() {
            Some(Parameter { name, type_name })
        } else {
            None
        }
    }

    fn extract_identifier(&self, node: Node, source: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return self.node_text(child, source);
            } else if child.kind() == "pointer_declarator" {
                return self.extract_identifier(child, source);
            }
        }
        String::new()
    }

    fn get_struct_name(&self, node: Node, source: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return self.node_text(child, source);
            }
        }
        String::new()
    }

    fn extract_calls(&self, node: Node, source: &str) -> Vec<String> {
        let mut calls = Vec::new();
        self.collect_calls(node, source, &mut calls);
        calls.sort();
        calls.dedup();
        calls
    }

    fn collect_calls(&self, node: Node, source: &str, calls: &mut Vec<String>) {
        if node.kind() == "call_expression" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    calls.push(self.node_text(child, source));
                    break;
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_calls(child, source, calls);
        }
    }

    fn extract_struct(&self, node: Node, source: &str, filename: &str) -> Option<StructDef> {
        let mut name = String::new();
        let mut fields = Vec::new();
        let mut referenced_structs = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "type_identifier" => {
                    name = self.node_text(child, source);
                }
                "field_declaration_list" => {
                    let (f, refs) = self.extract_fields(child, source);
                    fields = f;
                    referenced_structs = refs;
                }
                _ => {}
            }
        }

        if name.is_empty() {
            return None;
        }

        Some(StructDef {
            name,
            fields,
            location: Some(Location::new(
                filename,
                node.start_position().row as u32 + 1,
                node.start_position().column as u32,
            )),
            referenced_structs,
        })
    }

    fn extract_fields(&self, node: Node, source: &str) -> (Vec<StructField>, Vec<String>) {
        let mut fields = Vec::new();
        let mut refs = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration" {
                if let Some((field, maybe_ref)) = self.extract_field(child, source) {
                    fields.push(field);
                    if let Some(r) = maybe_ref {
                        refs.push(r);
                    }
                }
            }
        }

        refs.sort();
        refs.dedup();
        (fields, refs)
    }

    fn extract_field(&self, node: Node, source: &str) -> Option<(StructField, Option<String>)> {
        let mut name = String::new();
        let mut type_name = String::new();
        let mut is_pointer = false;
        let mut is_function_ptr = false;
        let mut func_ptr_signature: Option<String> = None;
        let mut array_size: Option<String> = None;
        let mut referenced_struct: Option<String> = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "primitive_type" | "type_identifier" | "sized_type_specifier" => {
                    type_name = self.node_text(child, source);
                }
                "struct_specifier" => {
                    let struct_name = self.get_struct_name(child, source);
                    type_name = format!("struct {}", struct_name);
                    referenced_struct = Some(struct_name);
                }
                "field_identifier" => {
                    name = self.node_text(child, source);
                }
                "pointer_declarator" => {
                    is_pointer = true;
                    name = self.extract_field_identifier(child, source);
                }
                "array_declarator" => {
                    let (arr_name, arr_size) = self.extract_array_info(child, source);
                    name = arr_name;
                    array_size = arr_size;
                }
                "function_declarator" => {
                    is_function_ptr = true;
                    is_pointer = true;
                    name = self.extract_function_name(child, source);
                    func_ptr_signature = Some(self.node_text(child, source));
                }
                _ => {}
            }
        }

        if name.is_empty() {
            return None;
        }

        Some((
            StructField {
                name,
                type_name,
                is_pointer,
                is_function_ptr,
                func_ptr_signature,
                array_size,
            },
            referenced_struct,
        ))
    }

    fn extract_field_identifier(&self, node: Node, source: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "field_identifier" | "identifier" => {
                    return self.node_text(child, source);
                }
                "pointer_declarator" => {
                    return self.extract_field_identifier(child, source);
                }
                _ => {}
            }
        }
        String::new()
    }

    fn extract_array_info(&self, node: Node, source: &str) -> (String, Option<String>) {
        let mut name = String::new();
        let mut size = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "field_identifier" | "identifier" => {
                    name = self.node_text(child, source);
                }
                "number_literal" | "identifier" => {
                    if name.is_empty() {
                        name = self.node_text(child, source);
                    } else {
                        size = Some(self.node_text(child, source));
                    }
                }
                _ => {}
            }
        }

        (name, size)
    }

    fn node_text(&self, node: Node, source: &str) -> String {
        node.utf8_text(source.as_bytes())
            .unwrap_or("")
            .to_string()
    }
}

impl Default for TreeSitterParser {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::Parser for TreeSitterParser {
    fn parse(&self, source: &str, filename: &str) -> Result<ParseResult> {
        let mut parser = TreeSitterParser::new();
        parser.parse_source(source, filename)
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

    #[test]
    fn test_parse_simple_function() {
        let source = r#"
int my_function(int a, char *b) {
    printf("hello");
    return 0;
}
"#;
        let mut parser = TreeSitterParser::new();
        let result = parser.parse_source(source, "test.c").unwrap();

        assert_eq!(result.functions.len(), 1);
        let func = result.functions.get("my_function").unwrap();
        assert_eq!(func.name, "my_function");
        assert_eq!(func.return_type, "int");
        assert_eq!(func.params.len(), 2);
        assert!(func.calls.contains(&"printf".to_string()));
    }

    #[test]
    fn test_parse_struct() {
        let source = r#"
struct my_device {
    int status;
    char *name;
    struct work_struct work;
};
"#;
        let mut parser = TreeSitterParser::new();
        let result = parser.parse_source(source, "test.c").unwrap();

        assert_eq!(result.structs.len(), 1);
        let st = result.structs.get("my_device").unwrap();
        assert_eq!(st.name, "my_device");
        assert_eq!(st.fields.len(), 3);
    }

    #[test]
    fn test_parse_callback_function() {
        let source = r#"
static void my_work_handler(struct work_struct *work) {
    printk("work done\n");
}

static int my_probe(struct usb_interface *intf) {
    INIT_WORK(&dev->work, my_work_handler);
    schedule_work(&dev->work);
    return 0;
}
"#;
        let mut parser = TreeSitterParser::new();
        let result = parser.parse_source(source, "test.c").unwrap();

        assert_eq!(result.functions.len(), 2);
        assert!(result.functions.contains_key("my_work_handler"));
        assert!(result.functions.contains_key("my_probe"));
    }
}

