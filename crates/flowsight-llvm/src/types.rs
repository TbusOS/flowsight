//! Type definitions for LLVM IR parsing
//!
//! This module provides types for LLVM IR parsing with frontend compatibility.
//! All types are serialized using serde for JSON serialization.

use flowsight_core::ExecutionContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// Frontend-compatible types (matching LlvmIrPanel interfaces)
// ============================================================

/// LLVM IR basic block - matches LlvmIrPanel.LlvmBasicBlock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmBasicBlock {
    /// Block name
    pub name: String,
    /// Instructions in this block
    #[serde(default)]
    pub instructions: Vec<LlvmInstruction>,
    /// Predecessor blocks
    #[serde(default)]
    pub predecessors: Vec<String>,
    /// Successor blocks
    #[serde(default)]
    pub successors: Vec<String>,
    /// Terminator instruction (ret, br, etc.)
    #[serde(default)]
    pub terminator: Option<LlvmInstruction>,
}

/// LLVM IR instruction - matches LlvmIrPanel.LlvmInstruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmInstruction {
    /// Opcode (add, call, ret, etc.)
    pub opcode: String,
    /// Destination register (if any)
    #[serde(default)]
    pub dest: Option<String>,
    /// Result type
    #[serde(default)]
    pub type_str: String,
    /// Operands
    #[serde(default)]
    pub operands: Vec<String>,
    /// Source location
    #[serde(default)]
    pub location: Option<LocationInfo>,
}

/// LLVM IR function - matches LlvmIrPanel.LlvmFunction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmFunction {
    /// Function name
    pub name: String,
    /// Return type
    pub return_type: String,
    /// Parameters
    #[serde(default)]
    pub parameters: Vec<LlvmParameter>,
    /// Basic blocks
    #[serde(default)]
    pub blocks: Vec<LlvmBasicBlock>,
    /// Whether this is a callback function
    #[serde(default)]
    pub is_callback: bool,
    /// Callback context
    #[serde(default)]
    pub callback_context: Option<String>,
}

/// LLVM IR parameter - matches LlvmIrPanel.LlvmParameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type string
    pub type_str: String,
}

/// LLVM IR parse result - matches LlvmIrPanel.LlvmIrParseResult
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlvmIrParseResult {
    /// Module name
    pub module_name: String,
    /// Functions mapping - function name -> LlvmFunction
    #[serde(default)]
    pub functions: HashMap<String, LlvmFunction>,
}

/// Page request for large IR data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmIrPageRequest {
    /// Function name to paginate
    pub function_name: String,
    /// Page number (0-indexed)
    pub page: u32,
    /// Page size (number of instructions per page)
    pub page_size: u32,
}

/// Page response for large IR data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmIrPageResponse {
    /// Function name
    pub function_name: String,
    /// Current page number
    pub page: u32,
    /// Page size
    pub page_size: u32,
    /// Total instructions
    pub total_instructions: u32,
    /// Total pages
    pub total_pages: u32,
    /// Whether has next page
    pub has_next: bool,
    /// Whether has previous page
    pub has_previous: bool,
    /// Instructions in this page
    #[serde(default)]
    pub instructions: Vec<LlvmInstruction>,
    /// Block name (if paginating by block)
    #[serde(default)]
    pub block_name: Option<String>,
}

// ============================================================
// Internal types (for parser use)
// ============================================================

/// Location information for instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrFunction {
    /// Function name
    pub name: String,
    /// Return type
    pub return_type: String,
    /// Parameters
    pub parameters: Vec<IrParameter>,
    /// Basic blocks
    pub blocks: Vec<IrBasicBlock>,
    /// Whether this is a callback function
    pub is_callback: bool,
    /// Callback context
    pub callback_context: Option<String>,
    /// Execution context
    pub execution_context: ExecutionContext,
    /// Source file
    pub source_file: Option<String>,
    /// Source line
    pub source_line: Option<u32>,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type string
    pub type_str: String,
}

/// Basic block in LLVM IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrBasicBlock {
    /// Block name
    pub name: String,
    /// Instructions in this block
    pub instructions: Vec<IrInstruction>,
    /// Terminator instruction (ret, br, etc.)
    pub terminator: Option<IrInstruction>,
    /// Predecessor blocks
    pub predecessors: Vec<String>,
    /// Successor blocks
    pub successors: Vec<String>,
}

/// LLVM IR instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrInstruction {
    /// Opcode (add, call, ret, etc.)
    pub opcode: String,
    /// Destination register (if any)
    pub dest: Option<String>,
    /// Result type
    pub type_str: String,
    /// Operands
    pub operands: Vec<String>,
    /// Source location
    pub location: Option<LocationInfo>,
}

/// Location information for instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    /// Source file
    pub file: String,
    /// Line number
    pub line: u32,
}

/// Function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrCall {
    /// Caller block
    pub caller_block: String,
    /// Caller function
    pub caller_function: String,
    /// Callee function name
    pub callee: String,
    /// Whether this is an indirect call
    pub is_indirect: bool,
    /// Call arguments
    pub arguments: Vec<String>,
}

/// Result of parsing LLVM IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrParseResult {
    /// All functions found
    pub functions: HashMap<String, IrFunction>,
    /// All calls found
    pub calls: Vec<IrCall>,
    /// All types found
    pub types: HashMap<String, IrType>,
    /// Module name
    pub module_name: String,
    /// Source files
    pub source_files: Vec<String>,
}

/// LLVM integer types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntegerType {
    I1,
    I8,
    I16,
    I32,
    I64,
    I128,
}

/// Floating point types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FloatType {
    Half,    // f16
    Float,   // f32
    Double,  // f64
    Fp128,   // f128
    X86Fp80, // x86_fp80
}

/// Pointer types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointerType {
    pub pointee: Box<IrType>,
    pub addr_space: u32,
}

/// Array types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayType {
    pub element: Box<IrType>,
    pub length: u64,
}

/// Structure types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructType {
    pub name: Option<String>,
    pub elements: Vec<IrType>,
    pub is_packed: bool,
}

/// Function types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionType {
    pub return_type: Box<IrType>,
    pub parameters: Vec<IrType>,
    pub is_var_arg: bool,
}

/// All possible LLVM types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrType {
    Integer(IntegerType),
    Float(FloatType),
    Pointer(PointerType),
    Array(ArrayType),
    Struct(StructType),
    Function(FunctionType),
    Void,
    Label,
    Metadata,
    Opaque,
}

impl IrType {
    /// Get the type as a string
    pub fn as_str(&self) -> &str {
        match self {
            IrType::Integer(t) => match t {
                IntegerType::I1 => "i1",
                IntegerType::I8 => "i8",
                IntegerType::I16 => "i16",
                IntegerType::I32 => "i32",
                IntegerType::I64 => "i64",
                IntegerType::I128 => "i128",
            },
            IrType::Float(t) => match t {
                FloatType::Half => "half",
                FloatType::Float => "float",
                FloatType::Double => "double",
                FloatType::Fp128 => "fp128",
                FloatType::X86Fp80 => "x86_fp80",
            },
            IrType::Pointer(_) => "ptr",
            IrType::Array(_) => "array",
            IrType::Struct(_) => "struct",
            IrType::Function(_) => "function",
            IrType::Void => "void",
            IrType::Label => "label",
            IrType::Metadata => "metadata",
            IrType::Opaque => "opaque",
        }
    }

    /// Parse a type from string
    pub fn parse_from_str(s: &str) -> Option<Self> {
        let s = s.trim();

        // Handle pointer types
        if s.ends_with("*") {
            let inner = &s[..s.len() - 1].trim();
            if let Some(inner_type) = Self::parse_from_str(inner) {
                return Some(IrType::Pointer(PointerType {
                    pointee: Box::new(inner_type),
                    addr_space: 0,
                }));
            }
        }

        // Handle vector types
        if s.starts_with("<") && s.ends_with(">") {
            let inner = &s[1..s.len() - 1];
            if let Some((len, inner_type)) = inner.split_once(" x ") {
                if let Ok(n) = len.parse::<u64>() {
                    if let Some(elem) = Self::parse_from_str(inner_type.trim()) {
                        return Some(IrType::Array(ArrayType {
                            element: Box::new(elem),
                            length: n,
                        }));
                    }
                }
            }
        }

        // Integer types
        if let Some(bits_str) = s.strip_prefix('i') {
            if let Ok(bits) = bits_str.parse::<u32>() {
                return Some(IrType::Integer(match bits {
                    1 => IntegerType::I1,
                    8 => IntegerType::I8,
                    16 => IntegerType::I16,
                    32 => IntegerType::I32,
                    64 => IntegerType::I64,
                    128 => IntegerType::I128,
                    _ => return None,
                }));
            }
        }

        // Float types
        match s {
            "float" => return Some(IrType::Float(FloatType::Float)),
            "double" => return Some(IrType::Float(FloatType::Double)),
            "half" => return Some(IrType::Float(FloatType::Half)),
            "fp128" => return Some(IrType::Float(FloatType::Fp128)),
            "x86_fp80" => return Some(IrType::Float(FloatType::X86Fp80)),
            "void" => return Some(IrType::Void),
            "label" => return Some(IrType::Label),
            "metadata" => return Some(IrType::Metadata),
            _ => {}
        }

        None
    }

    /// Check if type is a scalar
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            IrType::Integer(_) | IrType::Float(_)
        )
    }

    /// Check if type is an aggregate
    pub fn is_aggregate(&self) -> bool {
        matches!(
            self,
            IrType::Array(_) | IrType::Struct(_)
        )
    }
}

/// Memory layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLayout {
    pub pointer_size: usize,
    pub alignment: HashMap<String, usize>,
    pub size_of: HashMap<String, usize>,
}

impl MemoryLayout {
    /// Create default layout (64-bit)
    pub fn default_64bit() -> Self {
        Self {
            pointer_size: 8,
            alignment: [("i1", 1), ("i8", 1), ("i16", 2), ("i32", 4), ("i64", 8), ("float", 4), ("double", 8), ("ptr", 8)]
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
            size_of: [("i1", 1), ("i8", 1), ("i16", 2), ("i32", 4), ("i64", 8), ("float", 4), ("double", 8), ("ptr", 8)]
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
        }
    }

    /// Create 32-bit layout
    pub fn default_32bit() -> Self {
        Self {
            pointer_size: 4,
            alignment: [("i1", 1), ("i8", 1), ("i16", 2), ("i32", 4), ("i64", 8), ("float", 4), ("double", 8), ("ptr", 4)]
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
            size_of: [("i1", 1), ("i8", 1), ("i16", 2), ("i32", 4), ("i64", 8), ("float", 4), ("double", 8), ("ptr", 4)]
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer_types() {
        assert_eq!(IrType::parse_from_str("i32"), Some(IrType::Integer(IntegerType::I32)));
        assert_eq!(IrType::parse_from_str("i64"), Some(IrType::Integer(IntegerType::I64)));
        assert_eq!(IrType::parse_from_str("i1"), Some(IrType::Integer(IntegerType::I1)));
    }

    #[test]
    fn test_parse_pointer_type() {
        let ptr = IrType::parse_from_str("i32*");
        assert!(matches!(ptr, Some(IrType::Pointer(_))));
    }

    #[test]
    fn test_parse_vector_type() {
        let vec = IrType::parse_from_str("<4 x i32>");
        assert!(matches!(vec, Some(IrType::Array(_))));
    }

    #[test]
    fn test_type_as_str() {
        assert_eq!(IrType::Integer(IntegerType::I32).as_str(), "i32");
        assert_eq!(IrType::Float(FloatType::Double).as_str(), "double");
    }
}
