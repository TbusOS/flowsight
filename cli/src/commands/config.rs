//! `flowsight config` subcommand
//!
//! - `config show`  — display the active (merged) configuration
//! - `config init`  — create a template `.flowsight.toml` in the current directory
//! - `config path`  — print the location of the active config file

use anyhow::Result;

use crate::config::{self, FlowSightConfig};

/// Run `config show` — print the active config.
pub fn run_show(config: &FlowSightConfig) -> Result<()> {
    if let Some(ref path) = config.source_path {
        println!("# Loaded from: {}", path.display());
        println!();
    } else {
        println!("# No .flowsight.toml found (using defaults)");
        println!();
    }
    println!("{}", config.to_display_string());
    Ok(())
}

/// Run `config init` — write a template config file.
pub fn run_init() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let path = config::write_template(&cwd)?;
    println!("Created {}", path.display());
    Ok(())
}

/// Run `config path` — print where the active config file lives.
pub fn run_path(config: &FlowSightConfig) -> Result<()> {
    match config.source_path {
        Some(ref path) => println!("{}", path.display()),
        None => println!("(no config file found)"),
    }
    Ok(())
}
