//! Clang Preprocessor Integration
//!
//! Wraps the Clang preprocessor for accurate C code preprocessing,
//! handling macros, conditional compilation, and header file inclusion.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;
use tracing::{debug, warn};

use super::config::{Architecture, MacroDefinition};

/// Errors that can occur during preprocessing
#[derive(Debug, Error)]
pub enum PreprocessError {
    #[error("Clang not found. Please install clang.")]
    ClangNotFound,

    #[error("Preprocessing failed: {0}")]
    PreprocessFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid source file: {0}")]
    InvalidSource(String),
}

/// Options for preprocessing
#[derive(Debug, Clone)]
pub struct PreprocessOptions {
    /// Target architecture
    pub target: Architecture,
    /// Macro definitions (-D flags)
    pub defines: Vec<MacroDefinition>,
    /// Include paths (-I flags)
    pub includes: Vec<PathBuf>,
    /// System include paths (-isystem flags)
    pub system_includes: Vec<PathBuf>,
    /// Additional clang arguments
    pub extra_args: Vec<String>,
    /// Keep comments in output
    pub keep_comments: bool,
    /// Generate line markers
    pub line_markers: bool,
}

impl Default for PreprocessOptions {
    fn default() -> Self {
        Self {
            target: Architecture::X86_64,
            defines: Vec::new(),
            includes: Vec::new(),
            system_includes: Vec::new(),
            extra_args: Vec::new(),
            keep_comments: false,
            line_markers: true,
        }
    }
}

impl PreprocessOptions {
    /// Create options for Linux kernel preprocessing
    pub fn for_kernel(kernel_root: &Path, arch: Architecture) -> Self {
        let arch_dir = arch.kernel_arch_dir();

        let includes = vec![
            kernel_root.join("include"),
            kernel_root.join("include/uapi"),
            kernel_root.join(format!("arch/{}/include", arch_dir)),
            kernel_root.join(format!("arch/{}/include/uapi", arch_dir)),
            kernel_root.join(format!("arch/{}/include/generated", arch_dir)),
            kernel_root.join(format!("arch/{}/include/generated/uapi", arch_dir)),
            kernel_root.join("include/generated"),
            kernel_root.join("include/generated/uapi"),
        ];

        let mut defines = arch.predefined_macros();
        defines.push(MacroDefinition::defined("__KERNEL__"));
        defines.push(MacroDefinition::with_value("__GNUC__", "12"));
        defines.push(MacroDefinition::with_value("__GNUC_MINOR__", "0"));

        Self {
            target: arch,
            defines,
            includes,
            system_includes: Vec::new(),
            extra_args: vec![
                "-nostdinc".to_string(),
                "-fno-builtin".to_string(),
            ],
            keep_comments: false,
            line_markers: true,
        }
    }
}

/// Result of preprocessing
#[derive(Debug)]
pub struct PreprocessResult {
    /// Preprocessed source code
    pub code: String,
    /// Files included during preprocessing
    pub included_files: Vec<PathBuf>,
    /// Warnings generated during preprocessing
    pub warnings: Vec<String>,
}

/// Clang preprocessor wrapper
pub struct ClangPreprocessor {
    /// Path to clang executable
    clang_path: PathBuf,
}

impl ClangPreprocessor {
    /// Create a new preprocessor, auto-detecting clang location
    pub fn new() -> Result<Self, PreprocessError> {
        let clang_path = Self::find_clang()?;
        debug!("Found clang at: {:?}", clang_path);
        Ok(Self { clang_path })
    }

    /// Create a preprocessor with a specific clang path
    pub fn with_path(clang_path: PathBuf) -> Self {
        Self { clang_path }
    }

    /// Find clang executable
    fn find_clang() -> Result<PathBuf, PreprocessError> {
        // Try common locations
        let candidates = [
            "clang",
            "/usr/bin/clang",
            "/usr/local/bin/clang",
            "/opt/homebrew/bin/clang",  // macOS ARM
            "/opt/homebrew/opt/llvm/bin/clang",
        ];

        for candidate in candidates {
            if let Ok(output) = Command::new(candidate).arg("--version").output() {
                if output.status.success() {
                    return Ok(PathBuf::from(candidate));
                }
            }
        }

        Err(PreprocessError::ClangNotFound)
    }

    /// Check if clang is available
    pub fn is_available(&self) -> bool {
        Command::new(&self.clang_path)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get clang version
    pub fn version(&self) -> Option<String> {
        Command::new(&self.clang_path)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8(o.stdout)
                    .ok()
                    .and_then(|s| s.lines().next().map(|l| l.to_string()))
            })
    }

    /// Preprocess a source file
    pub fn preprocess_file(
        &self,
        source_path: &Path,
        options: &PreprocessOptions,
    ) -> Result<PreprocessResult, PreprocessError> {
        if !source_path.exists() {
            return Err(PreprocessError::InvalidSource(
                format!("File not found: {:?}", source_path)
            ));
        }

        let args = self.build_args(options);
        debug!("Preprocessing {:?} with args: {:?}", source_path, args);

        let output = Command::new(&self.clang_path)
            .args(&args)
            .arg(source_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PreprocessError::PreprocessFailed(stderr.to_string()));
        }

        let code = String::from_utf8_lossy(&output.stdout).to_string();
        let warnings = self.parse_warnings(&output.stderr);
        let included_files = self.extract_included_files(&code);

        Ok(PreprocessResult {
            code,
            included_files,
            warnings,
        })
    }

    /// Preprocess source code from a string
    pub fn preprocess_string(
        &self,
        source: &str,
        filename: &str,
        options: &PreprocessOptions,
    ) -> Result<PreprocessResult, PreprocessError> {
        let args = self.build_args(options);

        let mut cmd = Command::new(&self.clang_path);
        cmd.args(&args)
            .arg("-x").arg("c")  // Treat input as C
            .arg("-")           // Read from stdin
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set filename for error messages
        cmd.arg(format!("-ffile-prefix-map=-={}", filename));

        let mut child = cmd.spawn()?;

        // Write source to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(source.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PreprocessError::PreprocessFailed(stderr.to_string()));
        }

        let code = String::from_utf8_lossy(&output.stdout).to_string();
        let warnings = self.parse_warnings(&output.stderr);
        let included_files = self.extract_included_files(&code);

        Ok(PreprocessResult {
            code,
            included_files,
            warnings,
        })
    }

    /// Build clang command line arguments
    fn build_args(&self, options: &PreprocessOptions) -> Vec<String> {
        let mut args = vec![
            "-E".to_string(),  // Preprocess only
            format!("-target {}", options.target.target_triple()),
        ];

        // Add macro definitions
        for macro_def in &options.defines {
            args.push(macro_def.to_clang_arg());
        }

        // Add include paths
        for include in &options.includes {
            args.push(format!("-I{}", include.display()));
        }

        // Add system include paths
        for sys_include in &options.system_includes {
            args.push(format!("-isystem {}", sys_include.display()));
        }

        // Add extra args
        args.extend(options.extra_args.clone());

        // Comments and line markers
        if options.keep_comments {
            args.push("-C".to_string());
        }
        if !options.line_markers {
            args.push("-P".to_string());
        }

        args
    }

    /// Parse warnings from stderr
    fn parse_warnings(&self, stderr: &[u8]) -> Vec<String> {
        let stderr_str = String::from_utf8_lossy(stderr);
        stderr_str
            .lines()
            .filter(|line| line.contains("warning:"))
            .map(|s| s.to_string())
            .collect()
    }

    /// Extract included files from preprocessed output (from # line markers)
    fn extract_included_files(&self, code: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for line in code.lines() {
            // Line markers look like: # 1 "/path/to/file.h" 1
            if line.starts_with("# ") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let path = &line[start + 1..start + 1 + end];
                        if !path.starts_with('<') && !path.is_empty() {
                            let path_buf = PathBuf::from(path);
                            if !files.contains(&path_buf) {
                                files.push(path_buf);
                            }
                        }
                    }
                }
            }
        }

        files
    }
}

impl Default for ClangPreprocessor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            warn!("Clang not found, using placeholder path");
            Self::with_path(PathBuf::from("clang"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_args() {
        let preprocessor = ClangPreprocessor::with_path(PathBuf::from("clang"));
        let mut options = PreprocessOptions::default();
        options.defines.push(MacroDefinition::defined("FOO"));
        options.includes.push(PathBuf::from("/usr/include"));

        let args = preprocessor.build_args(&options);

        assert!(args.contains(&"-E".to_string()));
        assert!(args.contains(&"-DFOO=1".to_string()));
        assert!(args.contains(&"-I/usr/include".to_string()));
    }

    #[test]
    fn test_extract_included_files() {
        let preprocessor = ClangPreprocessor::with_path(PathBuf::from("clang"));
        let code = r#"
# 1 "test.c"
# 1 "<built-in>" 1
# 1 "/usr/include/stdio.h" 1
# 42 "/usr/include/stdio.h"
# 1 "/usr/include/bits/types.h" 1
"#;

        let files = preprocessor.extract_included_files(code);

        assert!(files.contains(&PathBuf::from("test.c")));
        assert!(files.contains(&PathBuf::from("/usr/include/stdio.h")));
        assert!(files.contains(&PathBuf::from("/usr/include/bits/types.h")));
    }

    #[test]
    fn test_kernel_options() {
        let kernel_root = PathBuf::from("/usr/src/linux");
        let options = PreprocessOptions::for_kernel(&kernel_root, Architecture::X86_64);

        assert!(options.defines.iter().any(|m| m.name == "__KERNEL__"));
        assert!(options.defines.iter().any(|m| m.name == "__x86_64__"));
        assert!(options.includes.iter().any(|p| p.ends_with("include")));
    }
}
