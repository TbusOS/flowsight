//! `flowsight path` command — find call chain between two functions
//!
//! Uses BFS on the SQLite index call graph to find the shortest path
//! from function A to function B.

use crate::index_db::IndexDb;
use crate::output::OutputFormat;
use anyhow::Result;
use crossterm::style::{Color, Stylize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

const C_TITLE: Color = Color::Rgb { r: 140, g: 185, b: 165 };
const C_FILE: Color = Color::Rgb { r: 155, g: 160, b: 185 };
const C_DIM: Color = Color::Rgb { r: 110, g: 115, b: 120 };
const C_OK: Color = Color::Rgb { r: 130, g: 175, b: 140 };
const C_ERR: Color = Color::Rgb { r: 195, g: 120, b: 120 };

/// Options for path finding
pub struct PathOptions {
    pub max_depth: usize,
    pub all_paths: bool,
}

impl Default for PathOptions {
    fn default() -> Self {
        Self {
            max_depth: 10,
            all_paths: false,
        }
    }
}

/// A single step in a call path
#[derive(Debug, Clone)]
struct PathStep {
    function: String,
    file: Option<String>,
}

/// Find shortest call path from `from` to `to` using BFS
pub fn run(
    from: &str,
    to: &str,
    db_path: &Path,
    format: &OutputFormat,
    opts: &PathOptions,
) -> Result<()> {
    let db = IndexDb::open_readonly(db_path)?;

    if opts.all_paths {
        let paths = find_all_paths(&db, from, to, opts.max_depth)?;
        print_paths(from, to, &paths, format)?;
    } else {
        let path = find_shortest_path(&db, from, to, opts.max_depth)?;
        match path {
            Some(p) => print_single_path(from, to, &p, format)?,
            None => {
                match format {
                    OutputFormat::Json => {
                        println!("{}", serde_json::json!({
                            "from": from,
                            "to": to,
                            "found": false,
                            "max_depth": opts.max_depth,
                        }));
                    }
                    _ => {
                        println!(
                            "{} {} -> {} {}",
                            "No path found:".with(C_ERR),
                            from.with(C_FILE),
                            to.with(C_FILE),
                            format!("(max depth {})", opts.max_depth).with(C_DIM),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// BFS to find shortest path
fn find_shortest_path(
    db: &IndexDb,
    from: &str,
    to: &str,
    max_depth: usize,
) -> Result<Option<Vec<PathStep>>> {
    if from == to {
        return Ok(Some(vec![PathStep { function: from.to_string(), file: None }]));
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut parent: HashMap<String, PathStep> = HashMap::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();

    visited.insert(from.to_string());
    queue.push_back((from.to_string(), 0));

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }

        // Get all callees of current function
        let callees = db.query_callees(&current)?;

        for callee in &callees {
            if visited.contains(&callee.callee_name) {
                continue;
            }

            visited.insert(callee.callee_name.clone());
            parent.insert(callee.callee_name.clone(), PathStep {
                function: current.clone(),
                file: Some(callee.caller_file.clone()),
            });

            if callee.callee_name == to {
                // Found it — reconstruct path
                return Ok(Some(reconstruct_path(&parent, from, to)));
            }

            queue.push_back((callee.callee_name.clone(), depth + 1));
        }
    }

    Ok(None)
}

/// Find all paths (DFS with depth limit)
fn find_all_paths(
    db: &IndexDb,
    from: &str,
    to: &str,
    max_depth: usize,
) -> Result<Vec<Vec<PathStep>>> {
    let mut all_paths = Vec::new();
    let mut current_path = vec![PathStep { function: from.to_string(), file: None }];
    let mut visited = HashSet::new();
    visited.insert(from.to_string());

    dfs_all_paths(db, to, &mut current_path, &mut visited, &mut all_paths, max_depth)?;

    // Sort by path length
    all_paths.sort_by_key(|p| p.len());
    Ok(all_paths)
}

fn dfs_all_paths(
    db: &IndexDb,
    target: &str,
    current_path: &mut Vec<PathStep>,
    visited: &mut HashSet<String>,
    all_paths: &mut Vec<Vec<PathStep>>,
    max_depth: usize,
) -> Result<()> {
    if current_path.len() > max_depth {
        return Ok(());
    }

    // Limit total results
    if all_paths.len() >= 20 {
        return Ok(());
    }

    let current = current_path.last().unwrap().function.clone();
    let callees = db.query_callees(&current)?;

    for callee in &callees {
        if callee.callee_name == target {
            let mut path = current_path.clone();
            path.push(PathStep {
                function: target.to_string(),
                file: Some(callee.caller_file.clone()),
            });
            all_paths.push(path);
            continue;
        }

        if visited.contains(&callee.callee_name) {
            continue;
        }

        visited.insert(callee.callee_name.clone());
        current_path.push(PathStep {
            function: callee.callee_name.clone(),
            file: Some(callee.caller_file.clone()),
        });

        dfs_all_paths(db, target, current_path, visited, all_paths, max_depth)?;

        current_path.pop();
        visited.remove(&callee.callee_name);
    }

    Ok(())
}

/// Reconstruct path from parent map
fn reconstruct_path(
    parent: &HashMap<String, PathStep>,
    from: &str,
    to: &str,
) -> Vec<PathStep> {
    let mut path = Vec::new();
    let mut current = to.to_string();

    while current != from {
        if let Some(step) = parent.get(&current) {
            path.push(PathStep {
                function: current.clone(),
                file: step.file.clone(),
            });
            current = step.function.clone();
        } else {
            break;
        }
    }

    path.push(PathStep { function: from.to_string(), file: None });
    path.reverse();
    path
}

/// Print a single path
fn print_single_path(
    from: &str,
    to: &str,
    path: &[PathStep],
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let steps: Vec<_> = path.iter().map(|s| {
                serde_json::json!({
                    "function": s.function,
                    "file": s.file,
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "from": from,
                "to": to,
                "found": true,
                "depth": path.len() - 1,
                "path": steps,
            }))?);
        }
        _ => {
            println!(
                "{} {} -> {} ({} hops)",
                "Path found:".with(C_OK),
                from.with(C_FILE),
                to.with(C_FILE),
                (path.len() - 1).to_string().with(C_TITLE),
            );
            println!();

            for (i, step) in path.iter().enumerate() {
                let is_last = i == path.len() - 1;
                let _connector = if is_last { "  " } else { "  -> " };
                let file_info = step.file.as_ref()
                    .map(|f| {
                        let short = Path::new(f)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(f);
                        format!("  ({})", short)
                    })
                    .unwrap_or_default();

                if i == 0 {
                    println!(
                        "  {}(){}",
                        step.function.as_str().with(C_OK),
                        file_info.with(C_DIM),
                    );
                } else if is_last {
                    println!(
                        "  -> {}(){}",
                        step.function.as_str().with(C_OK),
                        file_info.with(C_DIM),
                    );
                } else {
                    println!(
                        "  -> {}(){}",
                        step.function.as_str().with(C_FILE),
                        file_info.with(C_DIM),
                    );
                }
            }
        }
    }

    Ok(())
}

/// Print multiple paths
fn print_paths(
    from: &str,
    to: &str,
    paths: &[Vec<PathStep>],
    format: &OutputFormat,
) -> Result<()> {
    if paths.is_empty() {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::json!({
                    "from": from,
                    "to": to,
                    "found": false,
                    "paths": [],
                }));
            }
            _ => {
                println!(
                    "{} {} -> {}",
                    "No paths found:".with(C_ERR),
                    from.with(C_FILE),
                    to.with(C_FILE),
                );
            }
        }
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let json_paths: Vec<_> = paths.iter().map(|p| {
                p.iter().map(|s| {
                    serde_json::json!({
                        "function": s.function,
                        "file": s.file,
                    })
                }).collect::<Vec<_>>()
            }).collect();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "from": from,
                "to": to,
                "found": true,
                "count": paths.len(),
                "paths": json_paths,
            }))?);
        }
        _ => {
            println!(
                "{} {} paths from {} -> {}",
                "Found:".with(C_OK),
                paths.len().to_string().with(C_TITLE),
                from.with(C_FILE),
                to.with(C_FILE),
            );
            println!();

            for (pi, path) in paths.iter().enumerate() {
                println!(
                    "  {} ({} hops)",
                    format!("Path #{}", pi + 1).with(C_TITLE),
                    (path.len() - 1).to_string().with(C_DIM),
                );
                let chain: Vec<String> = path.iter().map(|s| format!("{}()", s.function)).collect();
                println!("    {}", chain.join(" -> ").with(C_FILE));
                println!();
            }
        }
    }

    Ok(())
}
