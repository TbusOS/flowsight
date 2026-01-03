//! Function pointer resolver
//!
//! Resolves function pointers through:
//! - ops table analysis
//! - Variable assignment tracking
//! - Type signature matching

use flowsight_core::FunctionDef;
use regex::Regex;
use std::collections::HashMap;

/// Function pointer resolver
pub struct FuncPtrResolver {
    /// Known ops table patterns
    ops_patterns: Vec<OpsPattern>,
}

struct OpsPattern {
    struct_type: Regex,
    field_pattern: Regex,
}

impl FuncPtrResolver {
    /// Create a new resolver
    pub fn new() -> Self {
        Self {
            ops_patterns: Self::default_ops_patterns(),
        }
    }

    fn default_ops_patterns() -> Vec<OpsPattern> {
        vec![
            // Generic struct initialization: static struct xxx yyy = { .field = func, ... }
            OpsPattern {
                struct_type: Regex::new(r"(?:static\s+)?(?:const\s+)?struct\s+(\w+)\s+(\w+)\s*=\s*\{").unwrap(),
                field_pattern: Regex::new(r"\.(\w+)\s*=\s*(\w+)").unwrap(),
            },
        ]
    }

    /// Analyze ops tables in source code
    pub fn analyze_ops_tables(
        &self,
        source: &str,
        functions: &HashMap<String, FunctionDef>,
    ) -> HashMap<String, String> {
        let mut mappings = HashMap::new();

        // Find struct initializations
        let struct_init_re = Regex::new(
            r"(?s)(?:static\s+)?(?:const\s+)?struct\s+(\w+)\s+(\w+)\s*=\s*\{([^}]+)\}"
        ).unwrap();

        for caps in struct_init_re.captures_iter(source) {
            let struct_type = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let _var_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let init_content = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            // Extract field assignments
            let field_re = Regex::new(r"\.(\w+)\s*=\s*(\w+)").unwrap();
            for field_caps in field_re.captures_iter(init_content) {
                let field_name = field_caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let func_name = field_caps.get(2).map(|m| m.as_str()).unwrap_or("");

                // Check if it's a function
                if functions.contains_key(func_name) {
                    let context = format!("{}.{}", struct_type, field_name);
                    mappings.insert(context, func_name.to_string());
                }
            }
        }

        mappings
    }

    /// Resolve indirect call through function pointer
    pub fn resolve_indirect_call(
        &self,
        callee_expr: &str,
        functions: &HashMap<String, FunctionDef>,
        ops_mappings: &HashMap<String, String>,
    ) -> Option<String> {
        // Try to match against known ops mappings
        for (context, func_name) in ops_mappings {
            if callee_expr.contains(&context.replace(".", "->")) {
                return Some(func_name.clone());
            }
        }

        None
    }
}

impl Default for FuncPtrResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ops_table_analysis() {
        let resolver = FuncPtrResolver::new();
        let source = r#"
static int my_open(struct inode *inode, struct file *file) {
    return 0;
}

static ssize_t my_read(struct file *file, char __user *buf, size_t count, loff_t *pos) {
    return 0;
}

static struct file_operations my_fops = {
    .owner = THIS_MODULE,
    .open = my_open,
    .read = my_read,
};
"#;

        let mut functions = HashMap::new();
        functions.insert("my_open".to_string(), FunctionDef {
            name: "my_open".to_string(),
            return_type: "int".to_string(),
            params: vec![],
            location: None,
            calls: vec![],
            called_by: vec![],
            is_callback: false,
            callback_context: None,
            attributes: vec![],
        });
        functions.insert("my_read".to_string(), FunctionDef {
            name: "my_read".to_string(),
            return_type: "ssize_t".to_string(),
            params: vec![],
            location: None,
            calls: vec![],
            called_by: vec![],
            is_callback: false,
            callback_context: None,
            attributes: vec![],
        });

        let mappings = resolver.analyze_ops_tables(source, &functions);
        
        assert!(mappings.contains_key("file_operations.open"));
        assert_eq!(mappings.get("file_operations.open"), Some(&"my_open".to_string()));
        assert!(mappings.contains_key("file_operations.read"));
        assert_eq!(mappings.get("file_operations.read"), Some(&"my_read".to_string()));
    }
}

