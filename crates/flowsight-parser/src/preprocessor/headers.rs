//! Header File Resolver
//!
//! Resolves header file paths for Linux kernel and other C projects.

use std::path::{Path, PathBuf};
use tracing::debug;

use super::config::Architecture;

/// Header file resolver for C projects
pub struct HeaderResolver {
    /// Project root directory
    root: PathBuf,
    /// Include search paths
    include_paths: Vec<PathBuf>,
}

impl HeaderResolver {
    /// Create a new header resolver
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            include_paths: Vec::new(),
        }
    }

    /// Create a resolver for Linux kernel source
    pub fn for_kernel(kernel_root: &Path, arch: Architecture) -> Self {
        let arch_dir = arch.kernel_arch_dir();

        let include_paths = vec![
            // Generic includes
            kernel_root.join("include"),
            kernel_root.join("include/uapi"),
            // Architecture-specific includes
            kernel_root.join(format!("arch/{}/include", arch_dir)),
            kernel_root.join(format!("arch/{}/include/uapi", arch_dir)),
            // Generated includes (from kernel build)
            kernel_root.join(format!("arch/{}/include/generated", arch_dir)),
            kernel_root.join(format!("arch/{}/include/generated/uapi", arch_dir)),
            kernel_root.join("include/generated"),
            kernel_root.join("include/generated/uapi"),
        ];

        Self {
            root: kernel_root.to_path_buf(),
            include_paths,
        }
    }

    /// Add an include path
    pub fn add_include_path(&mut self, path: PathBuf) {
        if !self.include_paths.contains(&path) {
            self.include_paths.push(path);
        }
    }

    /// Get all include paths
    pub fn include_paths(&self) -> &[PathBuf] {
        &self.include_paths
    }

    /// Get include paths that actually exist
    pub fn existing_include_paths(&self) -> Vec<PathBuf> {
        self.include_paths
            .iter()
            .filter(|p| p.exists())
            .cloned()
            .collect()
    }

    /// Resolve a header file path
    ///
    /// Given an include directive like `#include <linux/kernel.h>` or
    /// `#include "myheader.h"`, find the actual file path.
    pub fn resolve(&self, header: &str, from_file: Option<&Path>) -> Option<PathBuf> {
        // For quoted includes, first try relative to the including file
        if let Some(from) = from_file {
            if let Some(parent) = from.parent() {
                let relative_path = parent.join(header);
                if relative_path.exists() {
                    debug!("Resolved {} relative to {:?}", header, from);
                    return Some(relative_path);
                }
            }
        }

        // Search in include paths
        for include_path in &self.include_paths {
            let full_path = include_path.join(header);
            if full_path.exists() {
                debug!("Resolved {} in {:?}", header, include_path);
                return Some(full_path);
            }
        }

        // Try relative to project root
        let root_relative = self.root.join(header);
        if root_relative.exists() {
            debug!("Resolved {} relative to root", header);
            return Some(root_relative);
        }

        debug!("Failed to resolve header: {}", header);
        None
    }

    /// Check if this looks like a Linux kernel source tree
    pub fn is_kernel_source(&self) -> bool {
        // Check for characteristic kernel files
        let kernel_markers = [
            "Kconfig",
            "Makefile",
            "include/linux/kernel.h",
            "arch/x86/Kconfig",
        ];

        kernel_markers.iter().any(|marker| self.root.join(marker).exists())
    }

    /// Detect the kernel version from source
    pub fn detect_kernel_version(&self) -> Option<String> {
        let makefile = self.root.join("Makefile");
        if !makefile.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&makefile).ok()?;

        let mut version = None;
        let mut patchlevel = None;
        let mut sublevel = None;

        for line in content.lines().take(10) {
            if let Some(v) = line.strip_prefix("VERSION = ") {
                version = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("PATCHLEVEL = ") {
                patchlevel = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("SUBLEVEL = ") {
                sublevel = Some(v.trim().to_string());
            }
        }

        match (version, patchlevel, sublevel) {
            (Some(v), Some(p), Some(s)) => Some(format!("{}.{}.{}", v, p, s)),
            (Some(v), Some(p), None) => Some(format!("{}.{}", v, p)),
            (Some(v), None, None) => Some(v),
            _ => None,
        }
    }

    /// Get common kernel subsystem paths
    pub fn kernel_subsystem_paths(&self) -> Vec<(&'static str, PathBuf)> {
        vec![
            ("drivers", self.root.join("drivers")),
            ("fs", self.root.join("fs")),
            ("net", self.root.join("net")),
            ("kernel", self.root.join("kernel")),
            ("mm", self.root.join("mm")),
            ("block", self.root.join("block")),
            ("sound", self.root.join("sound")),
            ("crypto", self.root.join("crypto")),
            ("security", self.root.join("security")),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_kernel_tree() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Create kernel-like structure
        fs::create_dir_all(root.join("include/linux")).unwrap();
        fs::create_dir_all(root.join("arch/x86/include/asm")).unwrap();
        fs::create_dir_all(root.join("drivers/usb/core")).unwrap();

        fs::write(root.join("Kconfig"), "# Kernel config").unwrap();
        fs::write(root.join("Makefile"), "VERSION = 6\nPATCHLEVEL = 1\nSUBLEVEL = 0\n").unwrap();
        fs::write(root.join("include/linux/kernel.h"), "// kernel.h").unwrap();
        fs::write(root.join("arch/x86/include/asm/types.h"), "// types.h").unwrap();

        temp
    }

    #[test]
    fn test_is_kernel_source() {
        let temp = create_test_kernel_tree();
        let resolver = HeaderResolver::new(temp.path().to_path_buf());
        assert!(resolver.is_kernel_source());
    }

    #[test]
    fn test_detect_kernel_version() {
        let temp = create_test_kernel_tree();
        let resolver = HeaderResolver::new(temp.path().to_path_buf());
        assert_eq!(resolver.detect_kernel_version(), Some("6.1.0".to_string()));
    }

    #[test]
    fn test_resolve_header() {
        let temp = create_test_kernel_tree();
        let resolver = HeaderResolver::for_kernel(temp.path(), Architecture::X86_64);

        let resolved = resolver.resolve("linux/kernel.h", None);
        assert!(resolved.is_some());
        assert!(resolved.unwrap().ends_with("include/linux/kernel.h"));
    }

    #[test]
    fn test_existing_include_paths() {
        let temp = create_test_kernel_tree();
        let resolver = HeaderResolver::for_kernel(temp.path(), Architecture::X86_64);

        let existing = resolver.existing_include_paths();
        // Should have at least include/ and arch/x86/include/
        assert!(existing.len() >= 2);
    }
}
