//! Function Pointer Resolution
//!
//! Resolves function pointer assignments from:
//! - Operations tables (struct xxx_ops)
//! - Direct variable assignments
//! - Callback registration patterns

use flowsight_core::FunctionDef;
use regex::Regex;
use std::collections::HashMap;

/// Function pointer resolver
pub struct FuncPtrResolver {
    /// Known ops table patterns
    ops_patterns: Vec<OpsTablePattern>,
}

/// Pattern for recognizing ops tables
struct OpsTablePattern {
    /// Struct type name pattern (e.g., "file_operations", "usb_driver")
    struct_pattern: Regex,
    /// Field to callback mapping
    field_mappings: HashMap<String, String>,
}

/// Resolved function pointer binding
#[derive(Debug, Clone)]
pub struct FuncPtrBinding {
    /// The ops table or variable name
    pub source: String,
    /// The field or context
    pub field: String,
    /// The resolved function name
    pub function: String,
    /// Confidence level
    pub confidence: Confidence,
}

/// Confidence level for resolution
#[derive(Debug, Clone, Copy)]
pub enum Confidence {
    High,   // Direct assignment in initializer
    Medium, // Type-based matching
    Low,    // Heuristic
}

impl FuncPtrResolver {
    /// Create a new function pointer resolver
    pub fn new() -> Self {
        Self {
            ops_patterns: Self::default_patterns(),
        }
    }

    fn default_patterns() -> Vec<OpsTablePattern> {
        vec![
            // file_operations
            OpsTablePattern {
                struct_pattern: Regex::new(r"struct\s+file_operations").unwrap(),
                field_mappings: [
                    ("open", "Called on open()"),
                    ("read", "Called on read()"),
                    ("write", "Called on write()"),
                    ("release", "Called on close()"),
                    ("unlocked_ioctl", "Called on ioctl()"),
                    ("mmap", "Called on mmap()"),
                    ("poll", "Called on poll()"),
                    ("llseek", "Called on lseek()"),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            },
            // usb_driver
            OpsTablePattern {
                struct_pattern: Regex::new(r"struct\s+usb_driver").unwrap(),
                field_mappings: [
                    ("probe", "Called when device matches"),
                    ("disconnect", "Called on device removal"),
                    ("suspend", "Called on system suspend"),
                    ("resume", "Called on system resume"),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            },
            // platform_driver
            OpsTablePattern {
                struct_pattern: Regex::new(r"struct\s+platform_driver").unwrap(),
                field_mappings: [
                    ("probe", "Called when device matches"),
                    ("remove", "Called on device removal"),
                    ("shutdown", "Called on system shutdown"),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            },
            // i2c_driver
            OpsTablePattern {
                struct_pattern: Regex::new(r"struct\s+i2c_driver").unwrap(),
                field_mappings: [
                    ("probe", "Called when device matches"),
                    ("remove", "Called on device removal"),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            },
            // net_device_ops
            OpsTablePattern {
                struct_pattern: Regex::new(r"struct\s+net_device_ops").unwrap(),
                field_mappings: [
                    ("ndo_open", "Called on interface up"),
                    ("ndo_stop", "Called on interface down"),
                    ("ndo_start_xmit", "Called to transmit packet"),
                    ("ndo_get_stats64", "Called to get statistics"),
                    ("ndo_set_mac_address", "Called to set MAC"),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            },
            // block_device_operations
            OpsTablePattern {
                struct_pattern: Regex::new(r"struct\s+block_device_operations").unwrap(),
                field_mappings: [
                    ("open", "Called on block device open"),
                    ("release", "Called on block device close"),
                    ("ioctl", "Called on block device ioctl"),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            },
        ]
    }

    /// Analyze source code for ops table assignments
    pub fn analyze_ops_tables(
        &self,
        source: &str,
        functions: &HashMap<String, FunctionDef>,
    ) -> Vec<(String, String)> {
        let mut mappings = Vec::new();

        // Pattern to match struct initializers like:
        // static struct file_operations my_fops = {
        //     .open = my_open,
        //     .read = my_read,
        // };
        let struct_init_re =
            Regex::new(r"(?s)static\s+(?:const\s+)?struct\s+(\w+)\s+(\w+)\s*=\s*\{([^}]+)\}")
                .unwrap();

        // Pattern to match field assignments
        let field_assign_re = Regex::new(r"\.(\w+)\s*=\s*(\w+)").unwrap();

        for caps in struct_init_re.captures_iter(source) {
            let struct_type = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let var_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let body = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            // Find matching ops pattern
            for pattern in &self.ops_patterns {
                if pattern
                    .struct_pattern
                    .is_match(&format!("struct {}", struct_type))
                {
                    // Extract field assignments
                    for field_caps in field_assign_re.captures_iter(body) {
                        let field = field_caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        let func_name = field_caps.get(2).map(|m| m.as_str()).unwrap_or("");

                        // Check if it's a known callback field and function exists
                        if pattern.field_mappings.contains_key(field)
                            && functions.contains_key(func_name)
                        {
                            let context = format!("{}.{}", var_name, field);
                            mappings.push((context, func_name.to_string()));
                        }
                    }
                }
            }
        }

        mappings
    }

    /// Analyze direct function pointer assignments
    pub fn analyze_assignments(
        &self,
        source: &str,
        functions: &HashMap<String, FunctionDef>,
    ) -> Vec<FuncPtrBinding> {
        let mut bindings = Vec::new();

        // Pattern: variable.field = function_name
        let assign_re = Regex::new(r"(\w+(?:\.\w+|->+\w+)*)\s*=\s*(\w+)\s*;").unwrap();

        for caps in assign_re.captures_iter(source) {
            let target = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let func_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            // Check if func_name is a known function
            if functions.contains_key(func_name) {
                // Look for patterns that suggest function pointer assignment
                if target.contains(".") || target.contains("->") {
                    let parts: Vec<&str> = target.split(|c| c == '.' || c == '>').collect();
                    if let Some(field) = parts.last() {
                        if Self::looks_like_callback_field(field) {
                            bindings.push(FuncPtrBinding {
                                source: parts[..parts.len() - 1].join("."),
                                field: field.to_string(),
                                function: func_name.to_string(),
                                confidence: Confidence::Medium,
                            });
                        }
                    }
                }
            }
        }

        bindings
    }

    /// Check if a field name looks like a callback
    fn looks_like_callback_field(field: &str) -> bool {
        let callback_patterns = [
            "callback", "handler", "func", "fn", "ops", "probe", "remove", "open", "close", "read",
            "write", "init", "exit", "start", "stop", "notify",
        ];

        let field_lower = field.to_lowercase();
        callback_patterns.iter().any(|p| field_lower.contains(p))
    }

    /// Get all resolved bindings
    pub fn resolve_all(
        &self,
        source: &str,
        functions: &HashMap<String, FunctionDef>,
    ) -> Vec<FuncPtrBinding> {
        let mut bindings = Vec::new();

        // Get ops table bindings (high confidence)
        for (context, func_name) in self.analyze_ops_tables(source, functions) {
            let parts: Vec<&str> = context.split('.').collect();
            bindings.push(FuncPtrBinding {
                source: parts.get(0).unwrap_or(&"").to_string(),
                field: parts.get(1).unwrap_or(&"").to_string(),
                function: func_name,
                confidence: Confidence::High,
            });
        }

        // Get direct assignment bindings
        bindings.extend(self.analyze_assignments(source, functions));

        bindings
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
    use flowsight_core::Location;

    fn make_func(name: &str) -> FunctionDef {
        FunctionDef {
            name: name.to_string(),
            return_type: "int".to_string(),
            params: vec![],
            location: Some(Location::new("test.c", 1, 0)),
            calls: vec![],
            called_by: vec![],
            is_callback: false,
            callback_context: None,
            attributes: vec![],
        }
    }

    #[test]
    fn test_ops_table_analysis() {
        let source = r#"
static int my_open(struct inode *i, struct file *f) { return 0; }
static ssize_t my_read(struct file *f, char __user *buf, size_t len, loff_t *off) { return 0; }

static const struct file_operations my_fops = {
    .owner = THIS_MODULE,
    .open = my_open,
    .read = my_read,
};
"#;

        let mut functions = HashMap::new();
        functions.insert("my_open".to_string(), make_func("my_open"));
        functions.insert("my_read".to_string(), make_func("my_read"));

        let resolver = FuncPtrResolver::new();
        let mappings = resolver.analyze_ops_tables(source, &functions);

        assert_eq!(mappings.len(), 2);
        assert!(mappings.iter().any(|(_, f)| f == "my_open"));
        assert!(mappings.iter().any(|(_, f)| f == "my_read"));
    }

    #[test]
    fn test_usb_driver_analysis() {
        let source = r#"
static int my_probe(struct usb_interface *intf, const struct usb_device_id *id) { return 0; }
static void my_disconnect(struct usb_interface *intf) {}

static struct usb_driver my_driver = {
    .name = "my_driver",
    .probe = my_probe,
    .disconnect = my_disconnect,
    .id_table = my_id_table,
};
"#;

        let mut functions = HashMap::new();
        functions.insert("my_probe".to_string(), make_func("my_probe"));
        functions.insert("my_disconnect".to_string(), make_func("my_disconnect"));

        let resolver = FuncPtrResolver::new();
        let mappings = resolver.analyze_ops_tables(source, &functions);

        assert_eq!(mappings.len(), 2);
    }
}
