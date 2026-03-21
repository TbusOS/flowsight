//! `flowsight subsystem-deps` — subsystem dependency graph from index

use crate::index_db::IndexDb;
use crate::output::OutputFormat;
use anyhow::Result;
use crossterm::style::{Color, Stylize};
use std::path::Path;

const C_TITLE: Color = Color::Rgb { r: 140, g: 185, b: 165 };
const C_FILE: Color = Color::Rgb { r: 155, g: 160, b: 185 };
const C_DIM: Color = Color::Rgb { r: 110, g: 115, b: 120 };
const C_OK: Color = Color::Rgb { r: 130, g: 175, b: 140 };

/// Options for subsystem dependency display
pub struct SubsystemDepsOptions {
    /// Only show edges involving these subsystems
    pub focus: Option<Vec<String>>,
    /// Minimum call count to show an edge
    pub min_calls: usize,
}

impl Default for SubsystemDepsOptions {
    fn default() -> Self {
        Self {
            focus: None,
            min_calls: 1,
        }
    }
}

/// Show subsystem dependency graph
pub fn run(
    db_path: &Path,
    format: &OutputFormat,
    opts: &SubsystemDepsOptions,
) -> Result<()> {
    let db = IndexDb::open_readonly(db_path)?;
    let mut edges = db.query_subsystem_deps()?;
    let subsystems = db.query_subsystems()?;

    // Apply filters
    if let Some(focus) = &opts.focus {
        edges.retain(|e| focus.iter().any(|f| e.from.contains(f) || e.to.contains(f)));
    }
    edges.retain(|e| e.call_count >= opts.min_calls);

    match format {
        OutputFormat::Dot => {
            print_dot(&subsystems, &edges);
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "subsystems": subsystems.iter().map(|(name, count)| {
                    serde_json::json!({ "name": name, "files": count })
                }).collect::<Vec<_>>(),
                "edges": edges.iter().map(|e| {
                    serde_json::json!({
                        "from": e.from,
                        "to": e.to,
                        "calls": e.call_count,
                    })
                }).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            print_text(&subsystems, &edges);
        }
    }

    Ok(())
}

fn print_text(subsystems: &[(String, usize)], edges: &[crate::index_db::SubsystemEdge]) {
    println!(
        "{} ({} subsystems, {} cross-subsystem edges)",
        "Subsystem Dependencies".with(C_TITLE),
        subsystems.len().to_string().with(C_OK),
        edges.len().to_string().with(C_OK),
    );
    println!();

    if edges.is_empty() {
        println!("  {}", "(No cross-subsystem call edges found)".with(C_DIM));
        println!();
        println!("{}", "Subsystems:".with(C_TITLE));
        for (name, count) in subsystems {
            println!(
                "  {} ({} files)",
                name.as_str().with(C_FILE),
                count.to_string().with(C_DIM),
            );
        }
        return;
    }

    // Group edges by source subsystem
    let mut by_source: std::collections::HashMap<&str, Vec<(&str, usize)>> =
        std::collections::HashMap::new();
    for edge in edges {
        by_source
            .entry(&edge.from)
            .or_default()
            .push((&edge.to, edge.call_count));
    }

    let mut sources: Vec<_> = by_source.keys().copied().collect();
    sources.sort();

    for src in sources {
        let targets = by_source.get(src).unwrap();
        let total: usize = targets.iter().map(|(_, c)| c).sum();
        println!(
            "  {} ({} calls out)",
            src.with(C_TITLE),
            total.to_string().with(C_DIM),
        );

        let mut sorted_targets = targets.clone();
        sorted_targets.sort_by(|a, b| b.1.cmp(&a.1));

        for (i, (target, count)) in sorted_targets.iter().enumerate() {
            let is_last = i == sorted_targets.len() - 1;
            let prefix = if is_last { "    └── " } else { "    ├── " };
            println!(
                "{}{} ({})",
                prefix.with(C_DIM),
                target.with(C_FILE),
                format!("{} calls", count).with(C_DIM),
            );
        }
    }
}

fn print_dot(subsystems: &[(String, usize)], edges: &[crate::index_db::SubsystemEdge]) {
    println!("digraph subsystem_deps {{");
    println!("  rankdir=LR;");
    println!("  node [shape=box, style=\"filled,rounded\", fontname=\"monospace\", fontsize=11];");
    println!("  edge [fontname=\"monospace\", fontsize=9];");
    println!();

    // Assign colors by index
    let colors = [
        "#b8d4b8", "#d4c8b8", "#b8c8d4", "#d4b8c8", "#c8d4b8",
        "#c8b8d4", "#d4d4b8", "#b8d4d4", "#d4b8b8", "#b8b8d4",
    ];

    for (i, (name, count)) in subsystems.iter().enumerate() {
        let color = colors[i % colors.len()];
        println!(
            "  \"{}\" [label=\"{}\\n({} files)\", fillcolor=\"{}\"];",
            name, name, count, color
        );
    }
    println!();

    for edge in edges {
        let width = if edge.call_count > 50 {
            3.0
        } else if edge.call_count > 10 {
            2.0
        } else {
            1.0
        };
        println!(
            "  \"{}\" -> \"{}\" [label=\"{}\", penwidth={:.1}];",
            edge.from, edge.to, edge.call_count, width
        );
    }

    println!("}}");
}
