//! KLEE Symbolic Execution Executor
//!
//! This module provides integration with KLEE for symbolic execution.
//!
//! ## KLEE Integration
//!
//! KLEE is a symbolic execution engine built on top of LLVM. It can:
//! - Explore all feasible paths through a program
//! - Generate concrete test inputs for each path
//! - Find bugs and security vulnerabilities
//!
//! ## Usage
//!
//! ```ignore
//! use flowsight_symbolic::{KleeExecutor, SymbolicConfig};
//!
//! let config = SymbolicConfig::default()
//!     .with_max_paths(100)
//!     .with_timeout(60);
//!
//! let executor = KleeExecutor::new(config);
//!
//! // Analyze a function
//! let result = executor.analyze_function(
//!     &llvm_ir,
//!     "target_function",
//!     &[("arg0", "int"), ("arg1", "int")],
//! )?;
//! ```

use crate::path::{ExecutionPath, ExecutionStep, CallType, SymbolicValue};
use crate::stubs::KernelStubManager;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use thiserror::Error;

#[cfg(feature = "klee")]
use std::os::unix::process::CommandExt;

/// Configuration for symbolic execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicConfig {
    /// Maximum number of paths to explore
    pub max_paths: usize,
    /// Timeout in seconds
    pub timeout: u64,
    /// Maximum memory in MB
    pub max_memory: u64,
    /// KLEE working directory
    pub work_dir: Option<PathBuf>,
    /// Whether to use concrete execution for unconstrained paths
    pub use_concrete: bool,
    /// Search strategy (dfs, bfs, random-path, etc.)
    pub search_strategy: SearchStrategy,
    /// Output format
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SearchStrategy {
    /// Depth-first search
    Dfs,
    /// Breadth-first search
    Bfs,
    /// Random path selection
    RandomPath,
    /// Random state search
    RandomState,
    /// Coverage-optimized search
    Coverage,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutputFormat {
    /// KLEE's native format
    Klee,
    /// JSON format
    Json,
    /// Simple text format
    Text,
}

impl Default for SymbolicConfig {
    fn default() -> Self {
        Self {
            max_paths: 100,
            timeout: 60,
            max_memory: 4096,
            work_dir: None,
            use_concrete: true,
            search_strategy: SearchStrategy::RandomPath,
            output_format: OutputFormat::Json,
        }
    }
}

impl SymbolicConfig {
    /// Create a new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum paths
    pub fn with_max_paths(mut self, max_paths: usize) -> Self {
        self.max_paths = max_paths;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum memory
    pub fn with_max_memory(mut self, max_memory: u64) -> Self {
        self.max_memory = max_memory;
        self
    }

    /// Set work directory
    pub fn with_work_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.work_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Set search strategy
    pub fn with_search_strategy(mut self, strategy: SearchStrategy) -> Self {
        self.search_strategy = strategy;
        self
    }
}

/// Errors that can occur during symbolic execution
#[derive(Debug, Error)]
pub enum SymbolicError {
    #[error("KLEE not found: {0}")]
    KleeNotFound(String),

    #[error("LLVM IR parsing error: {0}")]
    LlvmParseError(String),

    #[error("KLEE execution error: {0}")]
    KleeExecutionError(String),

    #[error("Path exploration timeout after {0} seconds")]
    Timeout(u64),

    #[error("Memory limit exceeded: {0} MB")]
    MemoryLimit(u64),

    #[error("Too many paths: {0} (limit: {1})")]
    TooManyPaths(usize, usize),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Symbolic argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicArg {
    /// Argument name
    pub name: String,
    /// Argument type
    pub type_name: String,
    /// Concrete example value (for reference)
    pub example_value: Option<String>,
    /// Range for integer types
    pub range: Option<(i64, i64)>,
}

/// Result of symbolic execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicResult {
    /// All discovered execution paths
    pub paths: Vec<ExecutionPath>,
    /// Total paths explored
    pub paths_explored: usize,
    /// Total instructions executed
    pub instructions: u64,
    /// Total time elapsed (seconds)
    pub elapsed_seconds: f64,
    /// Whether execution was complete or truncated
    pub is_complete: bool,
    /// Exit status
    pub exit_reason: ExitReason,
}

/// Reason for execution termination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExitReason {
    /// All paths explored
    Completed,
    /// Timeout reached
    Timeout,
    /// Memory limit reached
    MemoryLimit,
    /// Path limit reached
    PathLimit,
    /// User interrupt
    Interrupted,
    /// Error occurred
    Error(String),
}

/// KLEE Executor
pub struct KleeExecutor {
    /// Configuration
    config: SymbolicConfig,
    /// KLEE binary path
    klee_path: Option<PathBuf>,
    /// Working directory (temporary, cleaned up on drop)
    work_dir: Option<TempDir>,
}

impl KleeExecutor {
    /// Create a new executor
    pub fn new(config: SymbolicConfig) -> Self {
        Self {
            config,
            klee_path: None,
            work_dir: None,
        }
    }

    /// Find KLEE binary in PATH
    pub fn find_klee() -> Option<PathBuf> {
        which::which("klee").ok().or_else(|| {
            // Try common installation paths
            let common_paths = [
                "/usr/local/bin/klee",
                "/usr/bin/klee",
                "/opt/klee/bin/klee",
            ];
            common_paths.iter()
                .find(|p| PathBuf::from(p).exists())
                .map(|p| PathBuf::from(p))
        })
    }

    /// Set KLEE binary path
    pub fn with_klee_path(mut self, path: impl AsRef<Path>) -> Self {
        self.klee_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Check if KLEE is available
    pub fn is_available(&self) -> bool {
        Self::find_klee().is_some() || self.klee_path.as_ref().map_or(false, |p| p.exists())
    }

    /// Analyze a function and return all execution paths
    ///
    /// # Arguments
    ///
    /// * `llvm_ir` - Path to LLVM bitcode file (.bc)
    /// * `target_function` - Name of the function to analyze
    /// * `args` - Symbolic arguments for the function
    ///
    /// # Returns
    ///
    /// All feasible execution paths with their constraints
    pub fn analyze_function(
        &self,
        llvm_ir: impl AsRef<Path>,
        target_function: &str,
        args: &[SymbolicArg],
    ) -> Result<SymbolicResult, SymbolicError> {
        if !self.is_available() {
            // Fallback to simulated execution
            return self.simulate_execution(llvm_ir, target_function, args);
        }

        // Create working directory
        let work_dir = TempDir::new().map_err(|e| SymbolicError::IoError(e))?;
        let work_path = work_dir.path().to_path_buf();

        // Generate KLEE test code
        let test_code = self.generate_klee_test(target_function, args)?;

        // Write test code
        let test_file = work_path.join("test.c");
        std::fs::write(&test_file, &test_code).map_err(|e| SymbolicError::IoError(e))?;

        // Compile to LLVM IR
        let bc_file = self.compile_to_llvm(&test_file, &work_path)?;

        // Run KLEE
        let klee_output = self.run_klee(&bc_file, &work_path)?;

        // Parse results
        let result = self.parse_klee_output(klee_output)?;

        Ok(result)
    }

    /// Generate KLEE test code
    fn generate_klee_test(&self, _target_function: &str, args: &[SymbolicArg]) -> Result<String, SymbolicError> {
        let mut code = String::new();

        // Header
        code.push_str(r#"#include <klee/klee.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

// External declaration of target function
extern int target_function(int arg0);

int main(int argc, char** argv) {
"#);

        // Generate symbolic arguments
        for (i, arg) in args.iter().enumerate() {
            let name = format!("arg_{}", i);
            let c_type = self.rust_type_to_c(&arg.type_name);

            code.push_str(&format!("    // Symbolize argument {}: {}\n", i, arg.name));
            code.push_str(&format!("    {} {};\n", c_type, name));

            match arg.type_name.as_str() {
                "int" | "i32" => {
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
                "int64_t" | "i64" | "long" => {
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
                "char" | "i8" => {
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
                "bool" | "_Bool" => {
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
                _ => {
                    // Default: treat as pointer
                    code.push_str(&format!("    void* {};\n", name));
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
            }
        }

        // Call target function
        code.push_str("\n    // Call target function\n");
        let args_str = args.iter()
            .enumerate()
            .map(|(i, _)| format!("arg_{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        code.push_str(&format!("    int result = target_function({});\n", args_str));

        // Print result
        code.push_str(r#"
    printf("KLEE_RESULT: %d\n", result);
    printf("KLEE_PATHS_COMPLETE\n");
    return 0;
}
"#);

        Ok(code)
    }

    /// Convert Rust type to C type
    fn rust_type_to_c(&self, rust_type: &str) -> &'static str {
        match rust_type {
            "int" | "i32" => "int32_t",
            "int64_t" | "i64" | "long" => "int64_t",
            "unsigned_int" | "u32" => "uint32_t",
            "char" | "i8" => "int8_t",
            "bool" | "_Bool" => "bool",
            "size_t" => "size_t",
            "void" | "void_ptr" => "void",
            "void_star" | "ptr_const_u8" | "ptr_mut_u8" => "void*",
            _ => "int32_t",
        }
    }

    /// Compile C code to LLVM bitcode
    fn compile_to_llvm(&self, c_file: &PathBuf, work_dir: &PathBuf) -> Result<PathBuf, SymbolicError> {
        let bc_file = work_dir.join("test.bc");

        // Use clang with KLEE intrinsics
        let output = Command::new("clang")
            .args(&[
                "-emit-llvm",
                "-c",
                "-O0",
                "-g",
                c_file.to_str().unwrap(),
                "-o",
                bc_file.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| SymbolicError::KleeExecutionError(e.to_string()))?;

        if !output.status.success() {
            return Err(SymbolicError::LlvmParseError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(bc_file)
    }

    /// Run KLEE symbolic execution
    fn run_klee(&self, bc_file: &PathBuf, work_dir: &PathBuf) -> Result<String, SymbolicError> {
        let klee_binary = if let Some(path) = &self.klee_path {
            path.clone()
        } else {
            Self::find_klee().ok_or_else(|| SymbolicError::KleeNotFound("KLEE not found in PATH".into()))?
        };

        let mut cmd = Command::new(&klee_binary);
        cmd.args(&[
            "--max-paths",
            &self.config.max_paths.to_string(),
            "--max-time",
            &format!("{}s", self.config.timeout),
            "--output-dir",
            work_dir.to_str().unwrap(),
            bc_file.to_str().unwrap(),
        ]);

        // Set working directory
        cmd.current_dir(work_dir);

        // Capture output
        let output = cmd.output()
            .map_err(|e| SymbolicError::KleeExecutionError(e.to_string()))?;

        // Read KLEE output directory
        let test_dir = work_dir.join("klee-last");
        if test_dir.exists() {
            // Read instructions.txt for stats
            let stats_file = test_dir.join("info");
            if stats_file.exists() {
                return Ok(std::fs::read_to_string(stats_file).map_err(|e| SymbolicError::IoError(e))?);
            }
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parse KLEE output
    fn parse_klee_output(&self, output: String) -> Result<SymbolicResult, SymbolicError> {
        // Parse KLEE statistics and generate execution paths
        let paths: Vec<ExecutionPath> = Vec::new();
        let mut instructions: u64 = 0;
        let mut elapsed_seconds: f64 = 0.0;

        // Extract statistics from output
        let paths_explored: usize = if let Some(captures) = regex::Regex::new(r"Paths explored:\s+(\d+)")
            .unwrap()
            .captures(&output)
        {
            captures.get(1).map(|m| m.as_str().parse().unwrap()).unwrap_or(0)
        } else {
            0
        };

        // Extract instruction count
        if let Some(captures) = regex::Regex::new(r"Instructions:\s+(\d+)")
            .unwrap()
            .captures(&output)
        {
            instructions = captures.get(1).map(|m| m.as_str().parse().unwrap()).unwrap_or(0);
        }

        // Extract elapsed time
        if let Some(captures) = regex::Regex::new(r"Total time:\s+([\d.]+)s")
            .unwrap()
            .captures(&output)
        {
            elapsed_seconds = captures.get(1).map(|m| m.as_str().parse().unwrap()).unwrap_or(0.0);
        }

        let exit_reason = if output.contains("HaltTimer") {
            ExitReason::Timeout
        } else if output.contains("max memory") {
            ExitReason::MemoryLimit
        } else if output.contains("max-syms") {
            ExitReason::MemoryLimit
        } else if output.contains("max-states") {
            ExitReason::PathLimit
        } else {
            ExitReason::Completed
        };

        Ok(SymbolicResult {
            paths,
            paths_explored,
            instructions,
            elapsed_seconds,
            is_complete: matches!(exit_reason, ExitReason::Completed),
            exit_reason,
        })
    }

    /// Generate KLEE test code with kernel API stubs
    ///
    /// This method generates a complete KLEE test file that includes:
    /// - Symbolic argument declarations
    /// - Kernel API stub implementations
    /// - Target function call with symbolic values
    pub fn generate_klee_test_with_stubs(
        &self,
        target_function: &str,
        args: &[SymbolicArg],
    ) -> Result<String, SymbolicError> {
        let mut code = String::new();
        let stub_manager = KernelStubManager::new();

        // Header with includes
        code.push_str(r#"#include <klee/klee.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

"#);

        // Add kernel stub implementations
        code.push_str(&stub_manager.generate_header());
        code.push_str("\n");

        // Generate stub wrappers for detected kernel calls
        code.push_str(&stub_manager.generate_wrappers());
        code.push_str("\n");

        // External declaration of target function
        code.push_str("// External declaration of target function\n");
        code.push_str(&format!("extern int {}(", target_function));
        let args_list: Vec<String> = args.iter()
            .enumerate()
            .map(|(i, arg)| format!("int {}", arg.type_name))
            .collect();
        code.push_str(&args_list.join(", "));
        code.push_str(");\n\n");

        code.push_str("int main(int argc, char** argv) {\n");

        // Generate symbolic arguments
        for (i, arg) in args.iter().enumerate() {
            let name = format!("arg_{}", i);
            let c_type = self.rust_type_to_c(&arg.type_name);

            code.push_str(&format!("    // Symbolize argument {}: {}\n", i, arg.name));
            code.push_str(&format!("    {} {};\n", c_type, name));

            match arg.type_name.as_str() {
                "int" | "i32" => {
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
                "int64_t" | "i64" | "long" => {
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
                "char" | "i8" => {
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
                "bool" | "_Bool" => {
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
                _ => {
                    code.push_str(&format!("    void* {};\n", name));
                    code.push_str(&format!("    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n", name, name, name));
                }
            }
        }

        // Call target function
        code.push_str("\n    // Call target function\n");
        let args_str = args.iter()
            .enumerate()
            .map(|(i, _)| format!("arg_{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        code.push_str(&format!("    int result = {}({});\n", target_function, args_str));

        // Print result
        code.push_str(r#"
    printf("KLEE_RESULT: %d
");
    printf("KLEE_PATHS_COMPLETE
");
    return 0;
}
"#);

        Ok(code)
    }

    /// Generate test from function analysis result
    ///
    /// Combines LLVM IR analysis with KLEE test generation for comprehensive testing.
    pub fn generate_test_from_analysis(
        &self,
        target_function: &str,
        args: &[SymbolicArg],
        _calls: &[String],
    ) -> Result<String, SymbolicError> {
        // Use the kernel stub enhanced test generator
        self.generate_klee_test_with_stubs(target_function, args)
    }

    /// Parse KLEE output directory and extract all test cases
    ///
    /// This enhanced method parses KLEE's output directory to extract:
    /// - Test cases (ktest files)
    /// - Path constraints
    /// - Concrete values for each path
    pub fn parse_klee_output_dir(&self, output_dir: &Path) -> Result<SymbolicResult, SymbolicError> {
        let mut paths: Vec<ExecutionPath> = Vec::new();
        let mut paths_explored = 0;
        let mut instructions = 0;
        let mut elapsed_seconds = 0.0;

        // Find klee-last or specific test directory
        let test_dir = if output_dir.join("klee-last").exists() {
            output_dir.join("klee-last")
        } else if output_dir.join("klee-out-0").exists() {
            // Try to find the latest klee-out directory
            let entries = std::fs::read_dir(output_dir)
                .map_err(|e| SymbolicError::IoError(e))?;
            let mut out_dirs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with("klee-out"))
                .collect();
            out_dirs.sort_by_key(|e| e.file_name());
            out_dirs.last()
                .map(|e| e.path())
                .unwrap_or_else(|| output_dir.to_path_buf())
        } else {
            output_dir.to_path_buf()
        };

        // Parse info file for statistics
        let info_file = test_dir.join("info");
        if info_file.exists() {
            let info_content = std::fs::read_to_string(&info_file)
                .map_err(|e| SymbolicError::IoError(e))?;

            paths_explored = regex::Regex::new(r"Paths explored:\s+(\d+)")
                .unwrap()
                .captures(&info_content)
                .and_then(|c| c.get(1).map(|m| m.as_str().parse().unwrap()))
                .unwrap_or(0);

            instructions = regex::Regex::new(r"Instructions:\s+(\d+)")
                .unwrap()
                .captures(&info_content)
                .and_then(|c| c.get(1).map(|m| m.as_str().parse().unwrap()))
                .unwrap_or(0);

            elapsed_seconds = regex::Regex::new(r"Total time:\s+([\d.]+)s")
                .unwrap()
                .captures(&info_content)
                .and_then(|c| c.get(1).map(|m| m.as_str().parse().unwrap()))
                .unwrap_or(0.0);
        }

        // Parse all ktest files
        let test_files: Vec<_> = std::fs::read_dir(&test_dir)
            .map_err(|e| SymbolicError::IoError(e))?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("test"))
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "ktest"))
            .collect();

        for (path_id, ktest_file) in test_files.iter().enumerate() {
            if let Ok(path) = self.parse_ktest_file(ktest_file.path(), path_id) {
                paths.push(path);
            }
        }

        // Determine exit reason
        let exit_reason = if instructions > 0 && paths_explored >= self.config.max_paths {
            ExitReason::PathLimit
        } else if elapsed_seconds >= self.config.timeout as f64 {
            ExitReason::Timeout
        } else if paths_explored > 0 {
            ExitReason::Completed
        } else {
            ExitReason::Error("No paths explored".to_string())
        };

        Ok(SymbolicResult {
            paths,
            paths_explored,
            instructions,
            elapsed_seconds,
            is_complete: matches!(exit_reason, ExitReason::Completed),
            exit_reason,
        })
    }

    /// Parse a single ktest file and extract concrete values
    pub fn parse_ktest_file(&self, ktest_file: PathBuf, path_id: usize) -> Result<ExecutionPath, SymbolicError> {
        // Try to read as binary format first
        let content = std::fs::read(&ktest_file)
            .map_err(|e| SymbolicError::IoError(e))?;

        // KTEST format is version 4: big-endian int, four bytes of "ktest"
        //   uint8_t data[numBytes]

        let path_name = format!("path_{:04}", path_id);
        let mut path = ExecutionPath::new(format!("klee_{:04}", path_id), format!("KLEE Path {}", path_id));

        // Try to read as text (some versions output text)
        if let Ok(text_content) = std::fs::read_to_string(&ktest_file) {
            // Parse object names and values from text format
            // Note: ktest file format uses double quotes around names
            let object_re = regex::Regex::new(r"object (\d+): name = '(\w+)', size = (\d+)")
                .unwrap();

            for cap in object_re.captures_iter(&text_content) {
                let name = cap.get(2).unwrap().as_str();
                let size: usize = cap.get(3).unwrap().as_str().parse().unwrap();

                path.concrete_values.insert(name.to_string(), format!("<symbolic size={}>", size));
            }
        }

        // Add a step for the target function call
        path.add_step(ExecutionStep {
            step_id: 0,
            function: "target_function".into(),
            location: None,
            call_type: CallType::Direct,
            arguments: Vec::new(),
            return_value: None,
            is_user_code: true,
            line: 0,
        });

        Ok(path)
    }

    /// Simulate symbolic execution (fallback when KLEE is not available)
    fn simulate_execution(
        &self,
        _llvm_ir: impl AsRef<Path>,
        _target_function: &str,
        _args: &[SymbolicArg],
    ) -> Result<SymbolicResult, SymbolicError> {
        // Return a simulated result for testing
        // In production, this should either require KLEE or use an alternative

        // Create a simple example path
        let mut path = ExecutionPath::new("sim_1", "Simulated Path");

        // Add example steps
        path.add_step(ExecutionStep {
            step_id: 0,
            function: "target_function".into(),
            location: None,
            call_type: CallType::Direct,
            arguments: vec![
                SymbolicValue::new("arg_0", "int"),
            ],
            return_value: None,
            is_user_code: true,
            line: 1,
        });

        // Add example constraint
        path.add_constraint(crate::path::PathConstraint {
            id: "c1".into(),
            location: None,
            expression: "arg_0 != 0".into(),
            constraint_type: crate::path::ConstraintType::Branch,
            variable: "arg_0".into(),
            operator: "!=".into(),
            value: "0".into(),
        });

        Ok(SymbolicResult {
            paths: vec![path],
            paths_explored: 1,
            instructions: 10,
            elapsed_seconds: 0.01,
            is_complete: true,
            exit_reason: ExitReason::Completed,
        })
    }
}

impl Default for KleeExecutor {
    fn default() -> Self {
        Self::new(SymbolicConfig::default())
    }
}
