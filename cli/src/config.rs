//! Project-level configuration support
//!
//! Loads `.flowsight.toml` from the current directory or any ancestor,
//! providing defaults that CLI flags can override.
//!
//! Priority: CLI flag > config file > built-in default

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Configuration file name searched in parent directories
const CONFIG_FILENAME: &str = ".flowsight.toml";

/// Template content written by `config init`
const CONFIG_TEMPLATE: &str = r#"# FlowSight project configuration
# Place this file in your project root.
# CLI flags always override values set here.

[global]
# kernel_path = "/usr/src/linux"
# format = "text"          # text | json | ftrace | sequence | markdown | dot
# verbose = false

[index]
# db_path = ".flowsight/index.db"
# pattern = "*.c"
# subsystem = true
# parallel = 8

[analysis]
# max_depth = 10
# no_kernel = false
# expand_async = true
# exclude = ["tools/", "scripts/", "Documentation/"]

[output]
# color = true
# column_width = 36

[llm]
# default_provider = "openai"    # openai | claude | ollama | custom
# temperature = 0.3
# max_tokens = 4096
# stream = true

# [llm.providers.openai]
# provider_type = "openai"
# api_key_env = "OPENAI_API_KEY"
# model = "gpt-4o"

# [llm.providers.claude]
# provider_type = "anthropic"
# api_key_env = "ANTHROPIC_API_KEY"
# model = "claude-sonnet-4-20250514"

# [llm.providers.ollama]
# provider_type = "ollama"
# endpoint = "http://localhost:11434"
# model = "llama3.2"

# [llm.providers.deepseek]
# provider_type = "openai"
# api_key_env = "DEEPSEEK_API_KEY"
# base_url = "https://api.deepseek.com"
# model = "deepseek-chat"
"#;

// ---------------------------------------------------------------------------
// Config structs
// ---------------------------------------------------------------------------

/// Top-level configuration deserialized from `.flowsight.toml`
#[derive(Debug, Deserialize, Default, Clone)]
pub struct FlowSightConfig {
    /// Global settings
    #[serde(default)]
    pub global: Option<GlobalConfig>,

    /// Index settings
    #[serde(default)]
    pub index: Option<IndexConfig>,

    /// Analysis settings
    #[serde(default)]
    pub analysis: Option<AnalysisConfig>,

    /// Output settings
    #[serde(default)]
    pub output: Option<OutputConfig>,

    /// LLM provider settings
    #[serde(default)]
    pub llm: Option<flowsight_llm::config::LlmConfig>,

    /// Path to the config file that was loaded (not deserialized from TOML)
    #[serde(skip)]
    pub source_path: Option<PathBuf>,
}

/// Global defaults
#[derive(Debug, Deserialize, Default, Clone)]
pub struct GlobalConfig {
    /// Default kernel source path
    pub kernel_path: Option<PathBuf>,
    /// Default output format (text, json, ftrace, sequence, markdown, dot)
    pub format: Option<String>,
    /// Verbose mode
    pub verbose: Option<bool>,
}

/// Index command defaults
#[derive(Debug, Deserialize, Default, Clone)]
pub struct IndexConfig {
    /// Default database path
    pub db_path: Option<PathBuf>,
    /// File pattern for directory scan
    pub pattern: Option<String>,
    /// Auto-detect kernel subsystem boundaries
    pub subsystem: Option<bool>,
    /// Number of parallel workers
    pub parallel: Option<usize>,
}

/// Analysis defaults
#[derive(Debug, Deserialize, Default, Clone)]
pub struct AnalysisConfig {
    /// Default depth limit
    pub max_depth: Option<usize>,
    /// Hide kernel internal calls
    pub no_kernel: Option<bool>,
    /// Expand async handlers
    pub expand_async: Option<bool>,
    /// Excluded directories
    pub exclude: Option<Vec<String>>,
}

/// Output presentation
#[derive(Debug, Deserialize, Default, Clone)]
pub struct OutputConfig {
    /// Enable colors
    pub color: Option<bool>,
    /// Sequence diagram column width
    pub column_width: Option<usize>,
}

// ---------------------------------------------------------------------------
// Discovery & loading
// ---------------------------------------------------------------------------

/// Walk up the directory tree from `start` looking for `.flowsight.toml`.
///
/// Returns the parsed config if found, or `None` if no config file exists.
pub fn find_config(start: &Path) -> Option<FlowSightConfig> {
    let canonical = start.canonicalize().ok()?;
    let mut dir = if canonical.is_file() {
        canonical.parent()?.to_path_buf()
    } else {
        canonical
    };

    loop {
        let config_path = dir.join(CONFIG_FILENAME);
        if config_path.is_file() {
            return load_config(&config_path).ok();
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Load and parse a specific config file.
fn load_config(path: &Path) -> Result<FlowSightConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let mut config: FlowSightConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config: {}", path.display()))?;
    config.source_path = Some(path.to_path_buf());
    Ok(config)
}

/// Load config from the current working directory (walking upward).
pub fn load_from_cwd() -> Option<FlowSightConfig> {
    let cwd = std::env::current_dir().ok()?;
    find_config(&cwd)
}

// ---------------------------------------------------------------------------
// Config init
// ---------------------------------------------------------------------------

/// Write a template `.flowsight.toml` into the given directory.
///
/// Returns an error if the file already exists.
pub fn write_template(dir: &Path) -> Result<PathBuf> {
    let dest = dir.join(CONFIG_FILENAME);
    if dest.exists() {
        anyhow::bail!(
            "Config file already exists: {}",
            dest.display()
        );
    }
    std::fs::write(&dest, CONFIG_TEMPLATE)
        .with_context(|| format!("Failed to write {}", dest.display()))?;
    Ok(dest)
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

impl FlowSightConfig {
    /// Render the active (merged) config as a readable TOML string.
    pub fn to_display_string(&self) -> String {
        let mut lines = Vec::new();

        if let Some(ref g) = self.global {
            lines.push("[global]".to_string());
            if let Some(ref p) = g.kernel_path {
                lines.push(format!("kernel_path = {:?}", p.display().to_string()));
            }
            if let Some(ref f) = g.format {
                lines.push(format!("format = {:?}", f));
            }
            if let Some(v) = g.verbose {
                lines.push(format!("verbose = {}", v));
            }
            lines.push(String::new());
        }

        if let Some(ref i) = self.index {
            lines.push("[index]".to_string());
            if let Some(ref p) = i.db_path {
                lines.push(format!("db_path = {:?}", p.display().to_string()));
            }
            if let Some(ref p) = i.pattern {
                lines.push(format!("pattern = {:?}", p));
            }
            if let Some(s) = i.subsystem {
                lines.push(format!("subsystem = {}", s));
            }
            if let Some(p) = i.parallel {
                lines.push(format!("parallel = {}", p));
            }
            lines.push(String::new());
        }

        if let Some(ref a) = self.analysis {
            lines.push("[analysis]".to_string());
            if let Some(d) = a.max_depth {
                lines.push(format!("max_depth = {}", d));
            }
            if let Some(nk) = a.no_kernel {
                lines.push(format!("no_kernel = {}", nk));
            }
            if let Some(ea) = a.expand_async {
                lines.push(format!("expand_async = {}", ea));
            }
            if let Some(ref ex) = a.exclude {
                lines.push(format!("exclude = {:?}", ex));
            }
            lines.push(String::new());
        }

        if let Some(ref o) = self.output {
            lines.push("[output]".to_string());
            if let Some(c) = o.color {
                lines.push(format!("color = {}", c));
            }
            if let Some(cw) = o.column_width {
                lines.push(format!("column_width = {}", cw));
            }
            lines.push(String::new());
        }

        if lines.is_empty() {
            "(no configuration set)".to_string()
        } else {
            lines.join("\n")
        }
    }

    /// Check if the config is completely empty (all sections None).
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.global.is_none()
            && self.index.is_none()
            && self.analysis.is_none()
            && self.output.is_none()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_full_config() {
        let toml_str = r#"
[global]
kernel_path = "/usr/src/linux"
format = "json"
verbose = true

[index]
db_path = ".flowsight/index.db"
pattern = "*.c"
subsystem = true
parallel = 4

[analysis]
max_depth = 8
no_kernel = true
expand_async = false
exclude = ["tools/", "Documentation/"]

[output]
color = false
column_width = 40
"#;
        let config: FlowSightConfig = toml::from_str(toml_str).expect("parse failed");

        let g = config.global.as_ref().expect("global missing");
        assert_eq!(
            g.kernel_path.as_ref().map(|p| p.to_str().unwrap()),
            Some("/usr/src/linux")
        );
        assert_eq!(g.format.as_deref(), Some("json"));
        assert_eq!(g.verbose, Some(true));

        let i = config.index.as_ref().expect("index missing");
        assert_eq!(i.pattern.as_deref(), Some("*.c"));
        assert_eq!(i.parallel, Some(4));
        assert_eq!(i.subsystem, Some(true));

        let a = config.analysis.as_ref().expect("analysis missing");
        assert_eq!(a.max_depth, Some(8));
        assert_eq!(a.no_kernel, Some(true));
        assert_eq!(a.expand_async, Some(false));
        assert_eq!(
            a.exclude.as_deref(),
            Some(&["tools/".to_string(), "Documentation/".to_string()][..])
        );

        let o = config.output.as_ref().expect("output missing");
        assert_eq!(o.color, Some(false));
        assert_eq!(o.column_width, Some(40));
    }

    #[test]
    fn parse_empty_config() {
        let config: FlowSightConfig = toml::from_str("").expect("parse failed");
        assert!(config.is_empty());
    }

    #[test]
    fn parse_partial_config() {
        let toml_str = r#"
[global]
format = "ftrace"
"#;
        let config: FlowSightConfig = toml::from_str(toml_str).expect("parse failed");
        let g = config.global.as_ref().expect("global missing");
        assert_eq!(g.format.as_deref(), Some("ftrace"));
        assert!(g.kernel_path.is_none());
        assert!(config.index.is_none());
    }

    #[test]
    fn find_config_walks_upward() {
        let tmp = std::env::temp_dir().join("flowsight_cfg_test");
        let sub = tmp.join("a").join("b").join("c");
        fs::create_dir_all(&sub).expect("mkdir");

        let cfg_path = tmp.join(CONFIG_FILENAME);
        fs::write(&cfg_path, "[global]\nformat = \"json\"\n").expect("write");

        let found = find_config(&sub);
        assert!(found.is_some());
        let g = found
            .as_ref()
            .and_then(|c| c.global.as_ref())
            .expect("global");
        assert_eq!(g.format.as_deref(), Some("json"));

        // cleanup
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn write_template_creates_file() {
        let tmp = std::env::temp_dir().join("flowsight_init_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("mkdir");

        let path = write_template(&tmp).expect("init");
        assert!(path.exists());

        // Parse the template to make sure it's valid TOML
        let content = fs::read_to_string(&path).expect("read");
        let _: FlowSightConfig = toml::from_str(&content).expect("template must be valid TOML");

        // Second call should fail (already exists)
        assert!(write_template(&tmp).is_err());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn display_string_empty() {
        let config = FlowSightConfig::default();
        assert_eq!(config.to_display_string(), "(no configuration set)");
    }
}
