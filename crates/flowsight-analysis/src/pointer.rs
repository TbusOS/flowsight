//! Andersen-style Pointer Analysis
//!
//! Implements a flow-insensitive, context-insensitive pointer analysis
//! based on Andersen's algorithm for resolving function pointer targets.
//!
//! ## Algorithm Overview
//!
//! 1. Collect constraints from source code:
//!    - AddressOf: `p = &x` → x ∈ pts(p)
//!    - Copy: `p = q` → pts(q) ⊆ pts(p)
//!    - Load: `p = *q` → ∀o ∈ pts(q): pts(o) ⊆ pts(p)
//!    - Store: `*p = q` → ∀o ∈ pts(p): pts(q) ⊆ pts(o)
//!
//! 2. Iteratively solve constraints until fixed point
//!
//! 3. Output: points-to set for each pointer variable

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

/// A variable or memory location in the analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Location {
    /// A named variable (local or global)
    Variable(String),
    /// A struct field: (base_type, field_name)
    Field(String, String),
    /// A function (can be pointed to)
    Function(String),
    /// An allocation site (for heap objects)
    Alloc(String),
    /// Array element (treated as single location)
    ArrayElement(String),
}

impl Location {
    pub fn var(name: &str) -> Self {
        Location::Variable(name.to_string())
    }

    pub fn field(base: &str, field: &str) -> Self {
        Location::Field(base.to_string(), field.to_string())
    }

    pub fn func(name: &str) -> Self {
        Location::Function(name.to_string())
    }
}

/// Constraint types for pointer analysis
#[derive(Debug, Clone)]
pub enum Constraint {
    /// p = &x: x is added to pts(p)
    AddressOf {
        pointer: Location,
        target: Location,
    },
    /// p = q: pts(q) ⊆ pts(p)
    Copy {
        dest: Location,
        src: Location,
    },
    /// p = *q: for all o in pts(q), pts(o) ⊆ pts(p)
    Load {
        dest: Location,
        src_ptr: Location,
    },
    /// *p = q: for all o in pts(p), pts(q) ⊆ pts(o)
    Store {
        dest_ptr: Location,
        src: Location,
    },
    /// p = q->field: field-sensitive load
    FieldLoad {
        dest: Location,
        base_ptr: Location,
        field: String,
    },
    /// p->field = q: field-sensitive store
    FieldStore {
        base_ptr: Location,
        field: String,
        src: Location,
    },
}

/// Result of pointer analysis
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PointsToResult {
    /// Points-to sets for each location
    pub points_to: HashMap<String, HashSet<String>>,
    /// Function pointer targets: call_site -> possible functions
    pub func_ptr_targets: HashMap<String, HashSet<String>>,
}

impl PointsToResult {
    /// Get possible targets for a function pointer
    pub fn get_targets(&self, ptr: &str) -> Option<&HashSet<String>> {
        self.points_to.get(ptr)
    }

    /// Get all function names that a pointer might point to
    pub fn get_function_targets(&self, ptr: &str) -> Vec<String> {
        self.points_to
            .get(ptr)
            .map(|set| {
                set.iter()
                    .filter(|s| !s.starts_with("alloc:") && !s.contains('.'))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Andersen-style pointer analysis solver
pub struct AndersenSolver {
    /// All constraints
    constraints: Vec<Constraint>,
    /// Points-to sets (location_key -> set of target_keys)
    pts: HashMap<String, HashSet<String>>,
    /// Worklist for iterative solving
    worklist: VecDeque<String>,
    /// Track which locations have been modified
    changed: HashSet<String>,
}

impl AndersenSolver {
    /// Create a new solver
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            pts: HashMap::new(),
            worklist: VecDeque::new(),
            changed: HashSet::new(),
        }
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Add multiple constraints
    pub fn add_constraints(&mut self, constraints: impl IntoIterator<Item = Constraint>) {
        self.constraints.extend(constraints);
    }

    /// Get the key for a location
    fn loc_key(loc: &Location) -> String {
        match loc {
            Location::Variable(name) => name.clone(),
            Location::Field(base, field) => format!("{}.{}", base, field),
            Location::Function(name) => name.clone(),
            Location::Alloc(site) => format!("alloc:{}", site),
            Location::ArrayElement(name) => format!("{}[]", name),
        }
    }

    /// Initialize points-to sets from AddressOf constraints
    fn initialize(&mut self) {
        for constraint in &self.constraints {
            if let Constraint::AddressOf { pointer, target } = constraint {
                let ptr_key = Self::loc_key(pointer);
                let tgt_key = Self::loc_key(target);

                self.pts
                    .entry(ptr_key.clone())
                    .or_default()
                    .insert(tgt_key);

                self.worklist.push_back(ptr_key);
            }
        }
    }

    /// Add target to pts(loc), return true if changed
    fn add_to_pts(&mut self, loc: &str, target: &str) -> bool {
        let set = self.pts.entry(loc.to_string()).or_default();
        if set.insert(target.to_string()) {
            self.changed.insert(loc.to_string());
            true
        } else {
            false
        }
    }

    /// Union pts(src) into pts(dest), return true if changed
    fn union_pts(&mut self, dest: &str, src: &str) -> bool {
        let src_set = self.pts.get(src).cloned().unwrap_or_default();
        let dest_set = self.pts.entry(dest.to_string()).or_default();

        let old_size = dest_set.len();
        dest_set.extend(src_set);

        if dest_set.len() > old_size {
            self.changed.insert(dest.to_string());
            true
        } else {
            false
        }
    }

    /// Solve all constraints to fixed point
    pub fn solve(&mut self) -> PointsToResult {
        // Initialize from AddressOf constraints
        self.initialize();

        // Iterate until fixed point
        let max_iterations = 1000;
        let mut iteration = 0;

        loop {
            self.changed.clear();
            iteration += 1;

            // Process each constraint
            for constraint in self.constraints.clone() {
                match constraint {
                    Constraint::AddressOf { .. } => {
                        // Already handled in initialize
                    }
                    Constraint::Copy { dest, src } => {
                        let dest_key = Self::loc_key(&dest);
                        let src_key = Self::loc_key(&src);
                        self.union_pts(&dest_key, &src_key);
                    }
                    Constraint::Load { dest, src_ptr } => {
                        let dest_key = Self::loc_key(&dest);
                        let src_ptr_key = Self::loc_key(&src_ptr);

                        // For each o in pts(src_ptr), union pts(o) into pts(dest)
                        if let Some(targets) = self.pts.get(&src_ptr_key).cloned() {
                            for target in targets {
                                self.union_pts(&dest_key, &target);
                            }
                        }
                    }
                    Constraint::Store { dest_ptr, src } => {
                        let dest_ptr_key = Self::loc_key(&dest_ptr);
                        let src_key = Self::loc_key(&src);

                        // For each o in pts(dest_ptr), union pts(src) into pts(o)
                        if let Some(targets) = self.pts.get(&dest_ptr_key).cloned() {
                            for target in targets {
                                self.union_pts(&target, &src_key);
                            }
                        }
                    }
                    Constraint::FieldLoad { dest, base_ptr, field } => {
                        let dest_key = Self::loc_key(&dest);
                        let base_ptr_key = Self::loc_key(&base_ptr);

                        // For each o in pts(base_ptr), union pts(o.field) into pts(dest)
                        if let Some(bases) = self.pts.get(&base_ptr_key).cloned() {
                            for base in bases {
                                let field_key = format!("{}.{}", base, field);
                                self.union_pts(&dest_key, &field_key);
                            }
                        }
                    }
                    Constraint::FieldStore { base_ptr, field, src } => {
                        let base_ptr_key = Self::loc_key(&base_ptr);
                        let src_key = Self::loc_key(&src);

                        // For each o in pts(base_ptr), union pts(src) into pts(o.field)
                        if let Some(bases) = self.pts.get(&base_ptr_key).cloned() {
                            for base in bases {
                                let field_key = format!("{}.{}", base, field);
                                self.union_pts(&field_key, &src_key);
                            }
                        }
                    }
                }
            }

            // Check for fixed point
            if self.changed.is_empty() || iteration >= max_iterations {
                break;
            }
        }

        // Build result
        let mut result = PointsToResult::default();
        result.points_to = self.pts.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Extract function pointer targets
        for (loc, targets) in &self.pts {
            let func_targets: HashSet<String> = targets
                .iter()
                .filter(|t| !t.starts_with("alloc:") && !t.contains('.'))
                .cloned()
                .collect();

            if !func_targets.is_empty() {
                result.func_ptr_targets.insert(loc.clone(), func_targets);
            }
        }

        result
    }
}

impl Default for AndersenSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_address_of() {
        let mut solver = AndersenSolver::new();

        // fp = &my_func
        solver.add_constraint(Constraint::AddressOf {
            pointer: Location::var("fp"),
            target: Location::func("my_func"),
        });

        let result = solver.solve();

        let targets = result.get_function_targets("fp");
        assert!(targets.contains(&"my_func".to_string()));
    }

    #[test]
    fn test_copy_constraint() {
        let mut solver = AndersenSolver::new();

        // fp = &func_a
        solver.add_constraint(Constraint::AddressOf {
            pointer: Location::var("fp"),
            target: Location::func("func_a"),
        });

        // gp = fp
        solver.add_constraint(Constraint::Copy {
            dest: Location::var("gp"),
            src: Location::var("fp"),
        });

        let result = solver.solve();

        // Both fp and gp should point to func_a
        assert!(result.get_function_targets("fp").contains(&"func_a".to_string()));
        assert!(result.get_function_targets("gp").contains(&"func_a".to_string()));
    }

    #[test]
    fn test_conditional_assignment() {
        let mut solver = AndersenSolver::new();

        // if (cond) fp = &func_a; else fp = &func_b;
        // Flow-insensitive: both are possible
        solver.add_constraint(Constraint::AddressOf {
            pointer: Location::var("fp"),
            target: Location::func("func_a"),
        });
        solver.add_constraint(Constraint::AddressOf {
            pointer: Location::var("fp"),
            target: Location::func("func_b"),
        });

        let result = solver.solve();

        let targets = result.get_function_targets("fp");
        assert!(targets.contains(&"func_a".to_string()));
        assert!(targets.contains(&"func_b".to_string()));
    }

    #[test]
    fn test_field_store_load() {
        let mut solver = AndersenSolver::new();

        // dev = alloc
        solver.add_constraint(Constraint::AddressOf {
            pointer: Location::var("dev"),
            target: Location::Alloc("dev_alloc".to_string()),
        });

        // dev->callback = my_handler
        solver.add_constraint(Constraint::FieldStore {
            base_ptr: Location::var("dev"),
            field: "callback".to_string(),
            src: Location::func("my_handler"),
        });

        // Need to add the function as addressable
        solver.add_constraint(Constraint::AddressOf {
            pointer: Location::func("my_handler"),
            target: Location::func("my_handler"),
        });

        let result = solver.solve();

        // dev_alloc.callback should point to my_handler
        let field_targets = result.points_to.get("alloc:dev_alloc.callback");
        assert!(field_targets.is_some());
        assert!(field_targets.unwrap().contains("my_handler"));
    }

    #[test]
    fn test_transitive_copy() {
        let mut solver = AndersenSolver::new();

        // a = &func
        solver.add_constraint(Constraint::AddressOf {
            pointer: Location::var("a"),
            target: Location::func("func"),
        });

        // b = a
        solver.add_constraint(Constraint::Copy {
            dest: Location::var("b"),
            src: Location::var("a"),
        });

        // c = b
        solver.add_constraint(Constraint::Copy {
            dest: Location::var("c"),
            src: Location::var("b"),
        });

        let result = solver.solve();

        // All should point to func
        assert!(result.get_function_targets("a").contains(&"func".to_string()));
        assert!(result.get_function_targets("b").contains(&"func".to_string()));
        assert!(result.get_function_targets("c").contains(&"func".to_string()));
    }
}
