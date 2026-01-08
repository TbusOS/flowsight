//! Expression evaluator for scenario-based symbolic execution
//!
//! Supports C-style expressions: arithmetic, bitwise, logical, comparison operators.

use crate::scenario::SymbolicValue;
use std::collections::HashMap;

/// Evaluation result
#[derive(Debug, Clone)]
pub enum EvalResult {
    /// Concrete integer value
    Integer(i64),
    /// Boolean result
    Bool(bool),
    /// String value
    String(String),
    /// Pointer state
    Pointer { is_null: bool },
    /// Unknown/cannot evaluate
    Unknown,
}

impl EvalResult {
    pub fn is_truthy(&self) -> Option<bool> {
        match self {
            EvalResult::Integer(n) => Some(*n != 0),
            EvalResult::Bool(b) => Some(*b),
            EvalResult::Pointer { is_null } => Some(!is_null),
            EvalResult::Unknown => None,
            EvalResult::String(s) => Some(!s.is_empty()),
        }
    }

    pub fn to_i64(&self) -> Option<i64> {
        match self {
            EvalResult::Integer(n) => Some(*n),
            EvalResult::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }
}

impl From<&SymbolicValue> for EvalResult {
    fn from(val: &SymbolicValue) -> Self {
        match val {
            SymbolicValue::Integer(n) => EvalResult::Integer(*n),
            SymbolicValue::String(s) => EvalResult::String(s.clone()),
            SymbolicValue::Pointer { is_null, .. } => EvalResult::Pointer { is_null: *is_null },
            SymbolicValue::Range { min, max } => {
                // For ranges, use midpoint as representative value
                EvalResult::Integer((min + max) / 2)
            }
            SymbolicValue::Unknown { .. } => EvalResult::Unknown,
        }
    }
}

/// Expression evaluator
pub struct Evaluator {
    /// Variable bindings
    bindings: HashMap<String, SymbolicValue>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn with_bindings(bindings: HashMap<String, SymbolicValue>) -> Self {
        Self { bindings }
    }

    /// Set a variable binding
    pub fn set(&mut self, name: &str, value: SymbolicValue) {
        self.bindings.insert(name.to_string(), value);
    }

    /// Get a variable value
    pub fn get(&self, name: &str) -> Option<&SymbolicValue> {
        self.bindings.get(name)
    }

    /// Evaluate an expression string
    pub fn eval(&self, expr: &str) -> EvalResult {
        let expr = expr.trim();
        if expr.is_empty() {
            return EvalResult::Unknown;
        }

        // Try to parse and evaluate
        self.eval_expr(expr)
    }

    /// Evaluate a condition and return whether it's true/false/unknown
    pub fn eval_condition(&self, condition: &str) -> Option<bool> {
        self.eval(condition).is_truthy()
    }

    fn eval_expr(&self, expr: &str) -> EvalResult {
        let expr = expr.trim();

        // Handle parentheses
        if expr.starts_with('(') && expr.ends_with(')') {
            if let Some(inner) = self.extract_parens(expr) {
                return self.eval_expr(inner);
            }
        }

        // First, try to match as a complete variable (including member access like id->idVendor)
        // This must be done BEFORE operator parsing to avoid -> being parsed as >
        if self.is_valid_identifier(expr) {
            if let Some(val) = self.lookup_variable(expr) {
                return EvalResult::from(val);
            }
        }

        // Try logical OR (lowest precedence)
        if let Some(result) = self.try_binary_op(expr, "||", |a, b| {
            match (a.is_truthy(), b.is_truthy()) {
                (Some(true), _) | (_, Some(true)) => EvalResult::Bool(true),
                (Some(false), Some(false)) => EvalResult::Bool(false),
                _ => EvalResult::Unknown,
            }
        }) {
            return result;
        }

        // Try logical AND
        if let Some(result) = self.try_binary_op(expr, "&&", |a, b| {
            match (a.is_truthy(), b.is_truthy()) {
                (Some(false), _) | (_, Some(false)) => EvalResult::Bool(false),
                (Some(true), Some(true)) => EvalResult::Bool(true),
                _ => EvalResult::Unknown,
            }
        }) {
            return result;
        }

        // Try comparison operators (but not << or >>)
        for op in &["==", "!=", "<=", ">="] {
            if let Some(result) = self.try_comparison(expr, op) {
                return result;
            }
        }

        // Try shift operators BEFORE single < and >
        if let Some(result) = self.try_shift_op(expr, "<<", |a, b| a << b) {
            return result;
        }
        if let Some(result) = self.try_shift_op(expr, ">>", |a, b| a >> b) {
            return result;
        }

        // Now try single < and >
        for op in &["<", ">"] {
            if let Some(result) = self.try_comparison(expr, op) {
                return result;
            }
        }

        // Try bitwise OR (skip if it's ||)
        if let Some(result) = self.try_bitwise_op(expr, "|", |a, b| a | b) {
            return result;
        }

        // Try bitwise XOR
        if let Some(result) = self.try_binary_int_op(expr, "^", |a, b| a ^ b) {
            return result;
        }

        // Try bitwise AND (skip if it's &&)
        if let Some(result) = self.try_bitwise_op(expr, "&", |a, b| a & b) {
            return result;
        }

        // Try addition/subtraction
        if let Some(result) = self.try_binary_int_op_rtl(expr, "+", |a, b| a + b) {
            return result;
        }
        if let Some(result) = self.try_binary_int_op_rtl(expr, "-", |a, b| a - b) {
            return result;
        }

        // Try multiplication/division/modulo
        if let Some(result) = self.try_binary_int_op(expr, "*", |a, b| a * b) {
            return result;
        }
        if let Some(result) = self.try_binary_int_op(expr, "/", |a, b| {
            if b != 0 { a / b } else { 0 }
        }) {
            return result;
        }
        if let Some(result) = self.try_binary_int_op(expr, "%", |a, b| {
            if b != 0 { a % b } else { 0 }
        }) {
            return result;
        }

        // Try unary operators
        if expr.starts_with('!') {
            let inner = self.eval_expr(&expr[1..]);
            return match inner.is_truthy() {
                Some(b) => EvalResult::Bool(!b),
                None => EvalResult::Unknown,
            };
        }
        if expr.starts_with('~') {
            let inner = self.eval_expr(&expr[1..]);
            return match inner.to_i64() {
                Some(n) => EvalResult::Integer(!n),
                None => EvalResult::Unknown,
            };
        }
        if expr.starts_with('-') && !expr[1..].starts_with(|c: char| c.is_ascii_digit()) {
            let inner = self.eval_expr(&expr[1..]);
            return match inner.to_i64() {
                Some(n) => EvalResult::Integer(-n),
                None => EvalResult::Unknown,
            };
        }

        // Try to parse as literal or variable
        self.eval_atom(expr)
    }

    fn try_binary_op<F>(&self, expr: &str, op: &str, f: F) -> Option<EvalResult>
    where
        F: Fn(EvalResult, EvalResult) -> EvalResult,
    {
        // Find operator outside parentheses, from right to left for left-associativity
        let mut depth = 0;
        let bytes = expr.as_bytes();
        let op_bytes = op.as_bytes();

        for i in (0..expr.len().saturating_sub(op.len() - 1)).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                _ if depth == 0 && bytes[i..].starts_with(op_bytes) => {
                    // Make sure it's not part of a longer operator
                    if op == "|" && (i > 0 && bytes[i - 1] == b'|') {
                        continue;
                    }
                    if op == "&" && (i > 0 && bytes[i - 1] == b'&') {
                        continue;
                    }
                    if op == "|" && (i + 1 < bytes.len() && bytes[i + 1] == b'|') {
                        continue;
                    }
                    if op == "&" && (i + 1 < bytes.len() && bytes[i + 1] == b'&') {
                        continue;
                    }

                    let left = &expr[..i];
                    let right = &expr[i + op.len()..];
                    if !left.is_empty() && !right.is_empty() {
                        let left_val = self.eval_expr(left);
                        let right_val = self.eval_expr(right);
                        return Some(f(left_val, right_val));
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn try_binary_int_op<F>(&self, expr: &str, op: &str, f: F) -> Option<EvalResult>
    where
        F: Fn(i64, i64) -> i64,
    {
        self.try_binary_op(expr, op, |a, b| {
            match (a.to_i64(), b.to_i64()) {
                (Some(a), Some(b)) => EvalResult::Integer(f(a, b)),
                _ => EvalResult::Unknown,
            }
        })
    }

    /// Special handling for bitwise & and | to avoid confusion with && and ||
    fn try_bitwise_op<F>(&self, expr: &str, op: &str, f: F) -> Option<EvalResult>
    where
        F: Fn(i64, i64) -> i64,
    {
        let mut depth = 0;
        let bytes = expr.as_bytes();
        let op_byte = op.as_bytes()[0];

        for i in (0..expr.len()).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                c if depth == 0 && c == op_byte => {
                    // Skip if it's part of && or ||
                    if i > 0 && bytes[i - 1] == op_byte {
                        continue;
                    }
                    if i + 1 < bytes.len() && bytes[i + 1] == op_byte {
                        continue;
                    }

                    let left = &expr[..i];
                    let right = &expr[i + 1..];
                    if !left.is_empty() && !right.is_empty() {
                        let left_val = self.eval_expr(left);
                        let right_val = self.eval_expr(right);
                        match (left_val.to_i64(), right_val.to_i64()) {
                            (Some(a), Some(b)) => return Some(EvalResult::Integer(f(a, b))),
                            _ => return Some(EvalResult::Unknown),
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Special handling for shift operators << and >> to avoid confusion with < and >
    fn try_shift_op<F>(&self, expr: &str, op: &str, f: F) -> Option<EvalResult>
    where
        F: Fn(i64, i64) -> i64,
    {
        let mut depth = 0;
        let bytes = expr.as_bytes();
        let op_bytes = op.as_bytes();

        for i in (0..expr.len().saturating_sub(1)).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                _ if depth == 0 && bytes[i..].starts_with(op_bytes) => {
                    let left = &expr[..i];
                    let right = &expr[i + 2..];
                    if !left.is_empty() && !right.is_empty() {
                        let left_val = self.eval_expr(left);
                        let right_val = self.eval_expr(right);
                        match (left_val.to_i64(), right_val.to_i64()) {
                            (Some(a), Some(b)) => return Some(EvalResult::Integer(f(a, b))),
                            _ => return Some(EvalResult::Unknown),
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn try_binary_int_op_rtl<F>(&self, expr: &str, op: &str, f: F) -> Option<EvalResult>
    where
        F: Fn(i64, i64) -> i64,
    {
        // For + and -, scan from right to handle left-associativity correctly
        let mut depth = 0;
        let bytes = expr.as_bytes();

        for i in (1..expr.len()).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                b'+' | b'-' if depth == 0 && bytes[i] == op.as_bytes()[0] => {
                    // Skip if it's part of a number (e.g., 1e+5)
                    if i > 0 && (bytes[i - 1] == b'e' || bytes[i - 1] == b'E') {
                        continue;
                    }
                    let left = &expr[..i];
                    let right = &expr[i + 1..];
                    if !left.is_empty() && !right.is_empty() {
                        let left_val = self.eval_expr(left);
                        let right_val = self.eval_expr(right);
                        match (left_val.to_i64(), right_val.to_i64()) {
                            (Some(a), Some(b)) => return Some(EvalResult::Integer(f(a, b))),
                            _ => return Some(EvalResult::Unknown),
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn try_comparison(&self, expr: &str, op: &str) -> Option<EvalResult> {
        let mut depth = 0;
        let bytes = expr.as_bytes();
        let op_bytes = op.as_bytes();

        for i in (0..expr.len().saturating_sub(op.len() - 1)).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => depth -= 1,
                _ if depth == 0 && bytes[i..].starts_with(op_bytes) => {
                    // Skip if it's part of -> (member access)
                    if op == ">" && i > 0 && bytes[i - 1] == b'-' {
                        continue;
                    }
                    // Skip if < or > is part of << or >>
                    if op == "<" && i + 1 < bytes.len() && bytes[i + 1] == b'<' {
                        continue;
                    }
                    if op == ">" && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                        continue;
                    }
                    if op == "<" && i > 0 && bytes[i - 1] == b'<' {
                        continue;
                    }
                    if op == ">" && i > 0 && bytes[i - 1] == b'>' {
                        continue;
                    }

                    let left = &expr[..i];
                    let right = &expr[i + op.len()..];
                    if !left.is_empty() && !right.is_empty() {
                        let left_val = self.eval_expr(left);
                        let right_val = self.eval_expr(right);
                        return Some(self.compare_values(left_val, right_val, op));
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn compare_values(&self, a: EvalResult, b: EvalResult, op: &str) -> EvalResult {
        match (a.to_i64(), b.to_i64()) {
            (Some(a), Some(b)) => {
                let result = match op {
                    "==" => a == b,
                    "!=" => a != b,
                    "<" => a < b,
                    ">" => a > b,
                    "<=" => a <= b,
                    ">=" => a >= b,
                    _ => return EvalResult::Unknown,
                };
                EvalResult::Bool(result)
            }
            _ => {
                // Handle pointer comparisons
                match (&a, &b) {
                    (EvalResult::Pointer { is_null: a_null }, EvalResult::Pointer { is_null: b_null }) => {
                        match op {
                            "==" => EvalResult::Bool(a_null == b_null),
                            "!=" => EvalResult::Bool(a_null != b_null),
                            _ => EvalResult::Unknown,
                        }
                    }
                    // NULL pointer comparison with 0
                    (EvalResult::Pointer { is_null }, EvalResult::Integer(0))
                    | (EvalResult::Integer(0), EvalResult::Pointer { is_null }) => {
                        match op {
                            "==" => EvalResult::Bool(*is_null),
                            "!=" => EvalResult::Bool(!is_null),
                            _ => EvalResult::Unknown,
                        }
                    }
                    _ => EvalResult::Unknown,
                }
            }
        }
    }

    fn extract_parens<'a>(&self, expr: &'a str) -> Option<&'a str> {
        if !expr.starts_with('(') || !expr.ends_with(')') {
            return None;
        }
        let mut depth = 0;
        for (i, c) in expr.chars().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 && i < expr.len() - 1 {
                        return None; // Not a simple parenthesized expression
                    }
                }
                _ => {}
            }
        }
        Some(&expr[1..expr.len() - 1])
    }

    fn eval_atom(&self, expr: &str) -> EvalResult {
        let expr = expr.trim();

        // Try NULL/nullptr
        if expr.eq_ignore_ascii_case("null") || expr == "nullptr" || expr == "NULL" {
            return EvalResult::Pointer { is_null: true };
        }

        // Try true/false
        if expr == "true" {
            return EvalResult::Bool(true);
        }
        if expr == "false" {
            return EvalResult::Bool(false);
        }

        // Try hex number
        if expr.starts_with("0x") || expr.starts_with("0X") {
            if let Ok(n) = i64::from_str_radix(&expr[2..], 16) {
                return EvalResult::Integer(n);
            }
        }

        // Try binary number
        if expr.starts_with("0b") || expr.starts_with("0B") {
            if let Ok(n) = i64::from_str_radix(&expr[2..], 2) {
                return EvalResult::Integer(n);
            }
        }

        // Try decimal number
        if let Ok(n) = expr.parse::<i64>() {
            return EvalResult::Integer(n);
        }

        // Try string literal
        if expr.starts_with('"') && expr.ends_with('"') {
            return EvalResult::String(expr[1..expr.len() - 1].to_string());
        }

        // Try variable lookup
        // Handle member access: id->idVendor, dev.name, etc.
        if let Some(val) = self.lookup_variable(expr) {
            return EvalResult::from(val);
        }

        EvalResult::Unknown
    }

    fn lookup_variable(&self, path: &str) -> Option<&SymbolicValue> {
        // Direct lookup
        if let Some(val) = self.bindings.get(path) {
            return Some(val);
        }

        // Try normalized path (replace -> with .)
        let normalized = path.replace("->", ".");
        if let Some(val) = self.bindings.get(&normalized) {
            return Some(val);
        }

        // Try partial match for struct members
        for (key, val) in &self.bindings {
            let key_normalized = key.replace("->", ".");
            if key_normalized == normalized {
                return Some(val);
            }
            // Match suffix (e.g., "idVendor" matches "id->idVendor")
            if key_normalized.ends_with(&format!(".{}", path)) {
                return Some(val);
            }
        }

        None
    }

    /// Check if the expression looks like a valid identifier (variable name with optional member access)
    fn is_valid_identifier(&self, expr: &str) -> bool {
        if expr.is_empty() {
            return false;
        }

        // Check if it starts with a valid identifier character
        let first_char = expr.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' {
            return false;
        }

        // Check if it only contains valid identifier characters, '.', '-', '>'
        // This allows patterns like: x, foo, id->idVendor, dev.name, ptr->field->subfield
        let mut chars = expr.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' => continue,
                '-' => {
                    // Must be followed by '>' for member access
                    if chars.peek() == Some(&'>') {
                        chars.next(); // consume '>'
                        continue;
                    }
                    return false;
                }
                _ => return false,
            }
        }

        true
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let eval = Evaluator::new();
        assert_eq!(eval.eval("1 + 2").to_i64(), Some(3));
        assert_eq!(eval.eval("10 - 3").to_i64(), Some(7));
        assert_eq!(eval.eval("4 * 5").to_i64(), Some(20));
        assert_eq!(eval.eval("20 / 4").to_i64(), Some(5));
        assert_eq!(eval.eval("17 % 5").to_i64(), Some(2));
    }

    #[test]
    fn test_bitwise() {
        let eval = Evaluator::new();
        assert_eq!(eval.eval("0xFF & 0x0F").to_i64(), Some(0x0F));
        assert_eq!(eval.eval("0xF0 | 0x0F").to_i64(), Some(0xFF));
        assert_eq!(eval.eval("0xFF ^ 0x0F").to_i64(), Some(0xF0));
        assert_eq!(eval.eval("1 << 4").to_i64(), Some(16));
        assert_eq!(eval.eval("16 >> 2").to_i64(), Some(4));
    }

    #[test]
    fn test_comparison() {
        let eval = Evaluator::new();
        assert_eq!(eval.eval("5 == 5").is_truthy(), Some(true));
        assert_eq!(eval.eval("5 != 3").is_truthy(), Some(true));
        assert_eq!(eval.eval("3 < 5").is_truthy(), Some(true));
        assert_eq!(eval.eval("5 > 3").is_truthy(), Some(true));
        assert_eq!(eval.eval("5 <= 5").is_truthy(), Some(true));
        assert_eq!(eval.eval("5 >= 5").is_truthy(), Some(true));
    }

    #[test]
    fn test_logical() {
        let eval = Evaluator::new();
        assert_eq!(eval.eval("1 && 1").is_truthy(), Some(true));
        assert_eq!(eval.eval("1 && 0").is_truthy(), Some(false));
        assert_eq!(eval.eval("0 || 1").is_truthy(), Some(true));
        assert_eq!(eval.eval("0 || 0").is_truthy(), Some(false));
        assert_eq!(eval.eval("!0").is_truthy(), Some(true));
        assert_eq!(eval.eval("!1").is_truthy(), Some(false));
    }

    #[test]
    fn test_variables() {
        let mut eval = Evaluator::new();
        eval.set("x", SymbolicValue::Integer(10));
        eval.set("idVendor", SymbolicValue::Integer(0x1234));

        assert_eq!(eval.eval("x").to_i64(), Some(10));
        assert_eq!(eval.eval("x + 5").to_i64(), Some(15));
        assert_eq!(eval.eval("idVendor == 0x1234").is_truthy(), Some(true));
        assert_eq!(eval.eval("idVendor != 0x5678").is_truthy(), Some(true));
    }

    #[test]
    fn test_member_access() {
        let mut eval = Evaluator::new();
        eval.set("id->idVendor", SymbolicValue::Integer(0x1234));
        eval.set("dev.name", SymbolicValue::String("test".to_string()));

        // Direct lookup
        assert_eq!(eval.eval("id->idVendor").to_i64(), Some(0x1234));
        // Comparison with member access
        assert_eq!(eval.eval("id->idVendor == 0x1234").is_truthy(), Some(true));
    }

    #[test]
    fn test_pointer() {
        let mut eval = Evaluator::new();
        eval.set("ptr", SymbolicValue::Pointer { is_null: true, size: None });
        eval.set("valid_ptr", SymbolicValue::Pointer { is_null: false, size: None });

        assert_eq!(eval.eval("ptr == NULL").is_truthy(), Some(true));
        assert_eq!(eval.eval("valid_ptr != NULL").is_truthy(), Some(true));
        assert_eq!(eval.eval("!ptr").is_truthy(), Some(true));
        assert_eq!(eval.eval("valid_ptr").is_truthy(), Some(true));
    }

    #[test]
    fn test_complex_expr() {
        let mut eval = Evaluator::new();
        eval.set("a", SymbolicValue::Integer(5));
        eval.set("b", SymbolicValue::Integer(3));

        assert_eq!(eval.eval("(a + b) * 2").to_i64(), Some(16));
        assert_eq!(eval.eval("a > 0 && b > 0").is_truthy(), Some(true));
        assert_eq!(eval.eval("a == 5 || b == 10").is_truthy(), Some(true));
    }
}
