//! AST types for parsed code

use flowsight_core::Location;
use serde::{Deserialize, Serialize};

/// AST node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstNode {
    Function(FunctionNode),
    Struct(StructNode),
    Enum(EnumNode),
    Typedef(TypedefNode),
    Variable(VariableNode),
}

/// Function AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionNode {
    pub name: String,
    pub return_type: TypeNode,
    pub parameters: Vec<ParameterNode>,
    pub body: Option<BlockNode>,
    pub attributes: Vec<String>,
    pub location: Location,
}

/// Struct AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructNode {
    pub name: String,
    pub fields: Vec<FieldNode>,
    pub location: Location,
}

/// Enum AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumNode {
    pub name: String,
    pub values: Vec<EnumValueNode>,
    pub location: Location,
}

/// Enum value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValueNode {
    pub name: String,
    pub value: Option<String>,
}

/// Typedef AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedefNode {
    pub alias: String,
    pub original: TypeNode,
    pub location: Location,
}

/// Variable AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableNode {
    pub name: String,
    pub type_node: TypeNode,
    pub initializer: Option<String>,
    pub is_static: bool,
    pub is_const: bool,
    pub location: Location,
}

/// Type AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeNode {
    pub name: String,
    pub is_pointer: bool,
    pub pointer_depth: u32,
    pub is_const: bool,
    pub is_array: bool,
    pub array_size: Option<String>,
}

/// Parameter AST node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterNode {
    pub name: String,
    pub type_node: TypeNode,
}

/// Field AST node (for structs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldNode {
    pub name: String,
    pub type_node: TypeNode,
    pub is_function_ptr: bool,
    pub func_signature: Option<String>,
}

/// Block (compound statement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockNode {
    pub statements: Vec<StatementNode>,
}

/// Statement types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatementNode {
    Expression(ExpressionNode),
    Return(Option<ExpressionNode>),
    If {
        condition: ExpressionNode,
        then_block: Box<BlockNode>,
        else_block: Option<Box<BlockNode>>,
    },
    While {
        condition: ExpressionNode,
        body: Box<BlockNode>,
    },
    For {
        init: Option<ExpressionNode>,
        condition: Option<ExpressionNode>,
        update: Option<ExpressionNode>,
        body: Box<BlockNode>,
    },
    Declaration(VariableNode),
}

/// Expression types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpressionNode {
    Identifier(String),
    Literal(String),
    Call {
        function: Box<ExpressionNode>,
        arguments: Vec<ExpressionNode>,
    },
    BinaryOp {
        op: String,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    },
    UnaryOp {
        op: String,
        operand: Box<ExpressionNode>,
    },
    MemberAccess {
        object: Box<ExpressionNode>,
        member: String,
        is_pointer: bool,
    },
    Assignment {
        target: Box<ExpressionNode>,
        value: Box<ExpressionNode>,
    },
}
