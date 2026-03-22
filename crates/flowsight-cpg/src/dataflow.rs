//! Data flow analysis — reaching definitions and def-use chains
//!
//! Uses a worklist algorithm on the CFG to compute which variable
//! definitions can reach which variable uses.

use crate::types::*;
use flowsight_cfg::ControlFlowGraph;
use std::collections::{HashMap, HashSet};

/// Compute reaching definitions and build def-use chains
///
/// Algorithm (simplified reaching definitions):
/// 1. For each block, collect GEN (definitions created) and KILL (definitions overwritten)
/// 2. Iterate until fixed point: IN[B] = union(OUT[P]) for all predecessors P
/// 3. OUT[B] = GEN[B] union (IN[B] - KILL[B])
/// 4. For each use, find which definitions in IN[block] match the variable name
pub fn compute_reaching_definitions(
    cfg: &ControlFlowGraph,
    cpg: &mut CodePropertyGraph,
) {
    if cpg.definitions.is_empty() || cpg.uses.is_empty() {
        return;
    }

    // Step 1: Build per-block GEN and KILL sets
    let mut gen: HashMap<usize, Vec<NodeId>> = HashMap::new();     // block → [def_ids]
    let mut kill: HashMap<usize, HashSet<String>> = HashMap::new(); // block → {var_names killed}

    for def in &cpg.definitions {
        gen.entry(def.block_id).or_default().push(def.id);
        // A definition kills all previous definitions of the same variable
        // (except for field assignments which are partial)
        if !matches!(def.kind, DefKind::FieldAssignment { .. }) {
            kill.entry(def.block_id)
                .or_default()
                .insert(def.var_name.clone());
        }
    }

    // Step 2: Worklist algorithm for reaching definitions
    // IN[block] = set of (def_id, var_name) pairs that reach block entry
    let block_ids: Vec<usize> = cfg.blocks.iter().map(|b| b.id).collect();
    let mut in_sets: HashMap<usize, HashSet<NodeId>> = HashMap::new();
    let mut out_sets: HashMap<usize, HashSet<NodeId>> = HashMap::new();

    for &bid in &block_ids {
        in_sets.insert(bid, HashSet::new());
        out_sets.insert(bid, HashSet::new());
    }

    // Build a lookup: def_id → var_name
    let def_var: HashMap<NodeId, &str> = cpg
        .definitions
        .iter()
        .map(|d| (d.id, d.var_name.as_str()))
        .collect();

    let mut changed = true;
    let mut iterations = 0;
    while changed && iterations < 100 {
        changed = false;
        iterations += 1;

        for &bid in &block_ids {
            // IN[B] = union of OUT[P] for all predecessors P
            let preds = cfg.predecessors(bid);
            let mut new_in: HashSet<NodeId> = HashSet::new();
            for pred in &preds {
                if let Some(pred_out) = out_sets.get(pred) {
                    new_in.extend(pred_out);
                }
            }

            // OUT[B] = GEN[B] union (IN[B] - KILL[B])
            let killed_vars = kill.get(&bid);
            let mut new_out: HashSet<NodeId> = HashSet::new();

            // Add surviving defs from IN (not killed)
            for &def_id in &new_in {
                let var_name = def_var.get(&def_id).copied().unwrap_or("");
                let is_killed = killed_vars
                    .map(|k| k.contains(var_name))
                    .unwrap_or(false);
                if !is_killed {
                    new_out.insert(def_id);
                }
            }

            // Add GEN defs
            if let Some(gen_defs) = gen.get(&bid) {
                new_out.extend(gen_defs);
            }

            if new_in != *in_sets.get(&bid).unwrap_or(&HashSet::new()) {
                changed = true;
            }
            if new_out != *out_sets.get(&bid).unwrap_or(&HashSet::new()) {
                changed = true;
            }

            in_sets.insert(bid, new_in);
            out_sets.insert(bid, new_out);
        }
    }

    // Step 3: For each use, find reaching definitions
    for use_node in &cpg.uses {
        let block_in = in_sets.get(&use_node.block_id).cloned().unwrap_or_default();

        // Also include definitions from the same block that come BEFORE this use
        let mut reachable_defs: Vec<NodeId> = Vec::new();

        // Defs from IN set
        for &def_id in &block_in {
            if let Some(&var) = def_var.get(&def_id) {
                if var == use_node.var_name {
                    reachable_defs.push(def_id);
                }
            }
        }

        // Defs from same block, before this use's line
        if let Some(gen_defs) = gen.get(&use_node.block_id) {
            for &def_id in gen_defs {
                if let Some(&var) = def_var.get(&def_id) {
                    if var == use_node.var_name {
                        // Check if def is before use (by line number)
                        if let Some(def) = cpg.definitions.iter().find(|d| d.id == def_id) {
                            if def.line <= use_node.line {
                                if !reachable_defs.contains(&def_id) {
                                    reachable_defs.push(def_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        if !reachable_defs.is_empty() {
            let def_lines: Vec<usize> = reachable_defs
                .iter()
                .filter_map(|&did| cpg.definitions.iter().find(|d| d.id == did))
                .map(|d| d.line)
                .collect();

            cpg.reaching_defs.push(ReachingDef {
                use_id: use_node.id,
                var_name: use_node.var_name.clone(),
                use_line: use_node.line,
                def_ids: reachable_defs.clone(),
                def_lines,
            });

            // Create data flow edges
            for &def_id in &reachable_defs {
                if let Some(def) = cpg.definitions.iter().find(|d| d.id == def_id) {
                    cpg.data_edges.push(DataFlowEdge {
                        from_def: def_id,
                        to_use: use_node.id,
                        var_name: use_node.var_name.clone(),
                        def_line: def.line,
                        use_line: use_node.line,
                    });
                }
            }
        }
    }

    // Step 4: Build def-use chains (invert reaching definitions)
    let mut chains: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for rd in &cpg.reaching_defs {
        for &def_id in &rd.def_ids {
            chains.entry(def_id).or_default().push(rd.use_id);
        }
    }

    for (def_id, use_ids) in chains {
        if let Some(def) = cpg.definitions.iter().find(|d| d.id == def_id) {
            let use_lines: Vec<usize> = use_ids
                .iter()
                .filter_map(|&uid| cpg.uses.iter().find(|u| u.id == uid))
                .map(|u| u.line)
                .collect();

            cpg.def_use_chains.push(DefUseChain {
                def_id,
                var_name: def.var_name.clone(),
                def_line: def.line,
                use_ids,
                use_lines,
            });
        }
    }

    // Update stats
    cpg.stats.total_data_edges = cpg.data_edges.len();
    cpg.stats.total_def_use_chains = cpg.def_use_chains.len();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::CpgBuilder;

    #[test]
    fn test_reaching_defs_simple() {
        let source = r#"
int foo(int x) {
    int y = x + 1;
    int z = y * 2;
    return z;
}
"#;
        let mut builder = CpgBuilder::new();
        let cpg = builder.build_function_cpg(source, "foo").unwrap();

        // x is used in y = x + 1
        let x_uses = cpg.uses_of("x");
        assert!(!x_uses.is_empty(), "x should be used");

        // y is used in z = y * 2
        let y_uses = cpg.uses_of("y");
        assert!(!y_uses.is_empty(), "y should be used");

        // z is used in return z
        let z_uses = cpg.uses_of("z");
        assert!(!z_uses.is_empty(), "z should be used");

        // Data flow edges should exist
        assert!(
            cpg.data_edges.len() >= 3,
            "Expected at least 3 data edges (x→y, y→z, z→return), got {}",
            cpg.data_edges.len()
        );
    }

    #[test]
    fn test_def_use_chain() {
        let source = r#"
void bar(int a) {
    int b = a;
    int c = b;
    int d = b + c;
}
"#;
        let mut builder = CpgBuilder::new();
        let cpg = builder.build_function_cpg(source, "bar").unwrap();

        // b should have a def-use chain with uses in c=b and d=b+c
        let b_chains: Vec<_> = cpg
            .def_use_chains
            .iter()
            .filter(|c| c.var_name == "b")
            .collect();

        assert!(
            !b_chains.is_empty(),
            "Expected def-use chain for 'b'"
        );

        // b should be used at least twice
        let total_b_uses: usize = b_chains.iter().map(|c| c.use_ids.len()).sum();
        assert!(
            total_b_uses >= 2,
            "b should be used at least twice, got {}",
            total_b_uses
        );
    }

    #[test]
    fn test_backward_slice() {
        let source = r#"
int compute(int a, int b) {
    int x = a + 1;
    int y = b + 2;
    int z = x + y;
    return z;
}
"#;
        let mut builder = CpgBuilder::new();
        let cpg = builder.build_function_cpg(source, "compute").unwrap();

        // Find a use of z (in return)
        let z_return_use = cpg.uses.iter().find(|u| {
            u.var_name == "z" && matches!(u.kind, UseKind::ReturnValue)
        });

        if let Some(z_use) = z_return_use {
            let slice = cpg.backward_slice(z_use.id);
            // Backward slice from z should include definition of z
            assert!(
                !slice.is_empty(),
                "Backward slice from z return should be non-empty"
            );
        }
    }
}
