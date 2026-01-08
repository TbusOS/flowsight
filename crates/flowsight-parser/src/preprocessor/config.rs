//! Configuration Extractor
//!
//! Extracts macro definitions from Linux kernel configuration files (.config, Kconfig).

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Supported target architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Architecture {
    X86_64,
    I386,
    Arm64,
    Arm,
    Riscv64,
    Riscv32,
    Mips,
    PowerPC,
}

impl Architecture {
    /// Get the Clang target triple for this architecture
    pub fn target_triple(&self) -> &'static str {
        match self {
            Architecture::X86_64 => "x86_64-linux-gnu",
            Architecture::I386 => "i386-linux-gnu",
            Architecture::Arm64 => "aarch64-linux-gnu",
            Architecture::Arm => "arm-linux-gnueabi",
            Architecture::Riscv64 => "riscv64-linux-gnu",
            Architecture::Riscv32 => "riscv32-linux-gnu",
            Architecture::Mips => "mips-linux-gnu",
            Architecture::PowerPC => "powerpc64-linux-gnu",
        }
    }

    /// Get the kernel arch directory name
    pub fn kernel_arch_dir(&self) -> &'static str {
        match self {
            Architecture::X86_64 | Architecture::I386 => "x86",
            Architecture::Arm64 => "arm64",
            Architecture::Arm => "arm",
            Architecture::Riscv64 | Architecture::Riscv32 => "riscv",
            Architecture::Mips => "mips",
            Architecture::PowerPC => "powerpc",
        }
    }

    /// Get architecture-specific predefined macros
    pub fn predefined_macros(&self) -> Vec<MacroDefinition> {
        match self {
            Architecture::X86_64 => vec![
                MacroDefinition::defined("__x86_64__"),
                MacroDefinition::defined("__amd64__"),
                MacroDefinition::with_value("__LP64__", "1"),
            ],
            Architecture::I386 => vec![
                MacroDefinition::defined("__i386__"),
                MacroDefinition::defined("__i686__"),
            ],
            Architecture::Arm64 => vec![
                MacroDefinition::defined("__aarch64__"),
                MacroDefinition::with_value("__LP64__", "1"),
            ],
            Architecture::Arm => vec![
                MacroDefinition::defined("__arm__"),
            ],
            Architecture::Riscv64 => vec![
                MacroDefinition::defined("__riscv"),
                MacroDefinition::with_value("__riscv_xlen", "64"),
                MacroDefinition::with_value("__LP64__", "1"),
            ],
            Architecture::Riscv32 => vec![
                MacroDefinition::defined("__riscv"),
                MacroDefinition::with_value("__riscv_xlen", "32"),
            ],
            Architecture::Mips => vec![
                MacroDefinition::defined("__mips__"),
            ],
            Architecture::PowerPC => vec![
                MacroDefinition::defined("__powerpc__"),
                MacroDefinition::defined("__powerpc64__"),
            ],
        }
    }
}

impl std::str::FromStr for Architecture {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "x86_64" | "amd64" => Ok(Architecture::X86_64),
            "i386" | "i686" | "x86" => Ok(Architecture::I386),
            "arm64" | "aarch64" => Ok(Architecture::Arm64),
            "arm" | "arm32" => Ok(Architecture::Arm),
            "riscv64" => Ok(Architecture::Riscv64),
            "riscv32" | "riscv" => Ok(Architecture::Riscv32),
            "mips" => Ok(Architecture::Mips),
            "powerpc" | "ppc64" => Ok(Architecture::PowerPC),
            _ => Err(ConfigError::UnknownArchitecture(s.to_string())),
        }
    }
}

/// A macro definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroDefinition {
    pub name: String,
    pub value: Option<String>,
}

impl MacroDefinition {
    /// Create a macro that is simply defined (no value)
    pub fn defined(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: Some("1".to_string()),
        }
    }

    /// Create a macro with a specific value
    pub fn with_value(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: Some(value.to_string()),
        }
    }

    /// Create an undefined macro (for -U flag)
    pub fn undefined(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: None,
        }
    }

    /// Convert to clang -D/-U argument
    pub fn to_clang_arg(&self) -> String {
        match &self.value {
            Some(v) => format!("-D{}={}", self.name, v),
            None => format!("-U{}", self.name),
        }
    }
}

/// Errors that can occur during configuration extraction
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown architecture: {0}")]
    UnknownArchitecture(String),

    #[error("Invalid config line: {0}")]
    InvalidLine(String),
}

/// Configuration extractor for Linux kernel
pub struct ConfigExtractor {
    /// Extracted macro definitions
    macros: HashMap<String, MacroDefinition>,
}

impl ConfigExtractor {
    /// Create a new empty config extractor
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    /// Extract configuration from a .config file
    pub fn from_dot_config(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let mut extractor = Self::new();
        extractor.parse_dot_config(&content)?;
        Ok(extractor)
    }

    /// Parse .config file content
    fn parse_dot_config(&mut self, content: &str) -> Result<(), ConfigError> {
        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                // Check for "# CONFIG_XXX is not set" pattern
                if let Some(rest) = line.strip_prefix("# CONFIG_") {
                    if let Some(name) = rest.strip_suffix(" is not set") {
                        let macro_name = format!("CONFIG_{}", name);
                        self.macros.insert(
                            macro_name.clone(),
                            MacroDefinition::undefined(&macro_name),
                        );
                    }
                }
                continue;
            }

            // Parse CONFIG_XXX=value lines
            if let Some((key, value)) = line.split_once('=') {
                if key.starts_with("CONFIG_") {
                    let value = self.parse_config_value(value);
                    self.macros.insert(
                        key.to_string(),
                        MacroDefinition {
                            name: key.to_string(),
                            value: Some(value),
                        },
                    );
                }
            }
        }

        Ok(())
    }

    /// Parse a config value, handling strings and special values
    fn parse_config_value(&self, value: &str) -> String {
        let value = value.trim();

        // Handle quoted strings
        if value.starts_with('"') && value.ends_with('"') {
            return value[1..value.len() - 1].to_string();
        }

        // Handle y/n/m values
        match value {
            "y" => "1".to_string(),
            "n" => "0".to_string(),
            "m" => "1".to_string(), // Module, treat as enabled for preprocessing
            _ => value.to_string(),
        }
    }

    /// Add kernel-specific macros
    pub fn add_kernel_macros(&mut self) {
        // Always defined for kernel code
        self.macros.insert(
            "__KERNEL__".to_string(),
            MacroDefinition::defined("__KERNEL__"),
        );

        // GNU C version (simulate GCC)
        self.macros.insert(
            "__GNUC__".to_string(),
            MacroDefinition::with_value("__GNUC__", "12"),
        );
        self.macros.insert(
            "__GNUC_MINOR__".to_string(),
            MacroDefinition::with_value("__GNUC_MINOR__", "0"),
        );
    }

    /// Add architecture-specific macros
    pub fn add_arch_macros(&mut self, arch: Architecture) {
        for macro_def in arch.predefined_macros() {
            self.macros.insert(macro_def.name.clone(), macro_def);
        }
    }

    /// Get all macro definitions
    pub fn macros(&self) -> &HashMap<String, MacroDefinition> {
        &self.macros
    }

    /// Get macro definitions as a vector
    pub fn macro_list(&self) -> Vec<&MacroDefinition> {
        self.macros.values().collect()
    }

    /// Check if a config option is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.macros
            .get(name)
            .map(|m| m.value.as_ref().map(|v| v != "0").unwrap_or(false))
            .unwrap_or(false)
    }

    /// Get a config value
    pub fn get_value(&self, name: &str) -> Option<&str> {
        self.macros
            .get(name)
            .and_then(|m| m.value.as_deref())
    }
}

impl Default for ConfigExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dot_config() {
        let content = r#"
#
# Automatically generated file; DO NOT EDIT.
# Linux/x86 6.1.0 Kernel Configuration
#
CONFIG_CC_VERSION_TEXT="gcc (Ubuntu 11.4.0-1ubuntu1~22.04) 11.4.0"
CONFIG_CC_IS_GCC=y
CONFIG_GCC_VERSION=110400
CONFIG_CLANG_VERSION=0
CONFIG_AS_IS_GNU=y
# CONFIG_DEBUG_INFO is not set
CONFIG_MODULES=y
CONFIG_MODULE_SIG=y
CONFIG_LOCALVERSION="-custom"
"#;

        let mut extractor = ConfigExtractor::new();
        extractor.parse_dot_config(content).unwrap();

        assert!(extractor.is_enabled("CONFIG_CC_IS_GCC"));
        assert!(extractor.is_enabled("CONFIG_MODULES"));
        assert!(!extractor.is_enabled("CONFIG_DEBUG_INFO"));
        assert_eq!(extractor.get_value("CONFIG_GCC_VERSION"), Some("110400"));
        assert_eq!(extractor.get_value("CONFIG_LOCALVERSION"), Some("-custom"));
    }

    #[test]
    fn test_architecture_macros() {
        let arch = Architecture::X86_64;
        let macros = arch.predefined_macros();

        assert!(macros.iter().any(|m| m.name == "__x86_64__"));
        assert_eq!(arch.target_triple(), "x86_64-linux-gnu");
        assert_eq!(arch.kernel_arch_dir(), "x86");
    }

    #[test]
    fn test_macro_to_clang_arg() {
        let defined = MacroDefinition::defined("FOO");
        assert_eq!(defined.to_clang_arg(), "-DFOO=1");

        let with_value = MacroDefinition::with_value("BAR", "42");
        assert_eq!(with_value.to_clang_arg(), "-DBAR=42");

        let undefined = MacroDefinition::undefined("BAZ");
        assert_eq!(undefined.to_clang_arg(), "-UBAZ");
    }
}
