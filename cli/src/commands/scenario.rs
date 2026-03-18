//! `flowsight scenario` command group - kernel execution flow scenario engine
//!
//! Pre-built analysis scenarios that trace common Linux kernel execution flows
//! across subsystems. Kernel developers and learners can run a scenario to see
//! the complete execution path for common operations.

use crate::context::AnalysisContext;
use crate::output::{json, OutputFormat};
use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ─── Scenario data types ────────────────────────────────────────────────────

/// A single step in a scenario phase
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioStep {
    /// Function name
    function: String,
    /// Source file (kernel source path)
    file: Option<String>,
    /// Short description
    description: Option<String>,
    /// Whether this is a user-bindable entry point
    is_bind_point: bool,
    /// Execution context tag (e.g., "process", "hardirq", "softirq")
    context: String,
}

/// A phase within a scenario (e.g., "Device Connection", "Driver Binding")
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioPhase {
    /// Phase name
    name: String,
    /// Steps in this phase
    steps: Vec<ScenarioStep>,
}

/// An async handler discovered during scenario execution
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioAsyncHandler {
    /// Handler function name
    function: String,
    /// Mechanism tag (e.g., "WQ", "IRQ", "TM")
    mechanism: String,
    /// Whether the handler can sleep
    can_sleep: bool,
}

/// A callback registration discovered during scenario execution
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioCallback {
    /// The ops struct field (e.g., "usb_gadget_ops.pullup")
    ops_field: String,
    /// The implementation function
    handler: String,
}

/// Complete scenario definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioDefinition {
    /// Unique identifier (e.g., "usb-enumeration")
    id: String,
    /// Human-readable name
    name: String,
    /// Short description
    description: String,
    /// Category tag (e.g., "drivers", "memory", "net", "scheduler", "fs")
    category: String,
    /// Execution phases
    phases: Vec<ScenarioPhase>,
    /// Bindable parameters (key = bind name, value = description)
    bind_params: HashMap<String, String>,
    /// Related kernel subsystems
    subsystems: Vec<String>,
}

/// Result of running a scenario against actual code
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioRunResult {
    /// The scenario definition
    scenario: ScenarioDefinition,
    /// Bound file path (if any)
    bound_file: Option<String>,
    /// Bound function name (if any)
    bound_function: Option<String>,
    /// Async handlers found in the bound file
    async_handlers: Vec<ScenarioAsyncHandler>,
    /// Callbacks found in the bound file
    callbacks: Vec<ScenarioCallback>,
    /// Functions from the bound file that appear in the scenario
    matched_functions: Vec<String>,
}

// ─── Scenario options ───────────────────────────────────────────────────────

/// Options for `scenario run`
pub struct ScenarioRunOptions {
    /// Bind a driver file to the scenario
    pub bind_driver: Option<String>,
    /// Bind a specific function
    pub bind_function: Option<String>,
    /// Maximum depth for call expansion
    pub max_depth: Option<usize>,
}

// ─── Public command entry points ────────────────────────────────────────────

/// List all available scenarios
pub fn run_list(format: &OutputFormat) -> Result<()> {
    let scenarios = builtin_scenarios();

    match format {
        OutputFormat::Json => {
            let items: Vec<_> = scenarios
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "name": s.name,
                        "description": s.description,
                        "category": s.category,
                        "phases": s.phases.len(),
                        "bind_params": s.bind_params,
                        "subsystems": s.subsystems,
                    })
                })
                .collect();
            println!("{}", json::to_pretty_json(&items)?);
        }
        _ => {
            print_scenario_list(&scenarios);
        }
    }

    Ok(())
}

/// Show details of a specific scenario
pub fn run_show(name: &str, format: &OutputFormat) -> Result<()> {
    let scenarios = builtin_scenarios();
    let scenario = find_scenario(&scenarios, name)?;

    match format {
        OutputFormat::Json => {
            println!("{}", json::to_pretty_json(&scenario)?);
        }
        _ => {
            print_scenario_detail(scenario);
        }
    }

    Ok(())
}

/// Run a scenario, optionally binding to a real source file
pub fn run_scenario(
    name: &str,
    format: &OutputFormat,
    opts: &ScenarioRunOptions,
) -> Result<()> {
    let scenarios = builtin_scenarios();
    let scenario = find_scenario(&scenarios, name)?.clone();

    let run_result = execute_scenario(&scenario, opts)?;

    match format {
        OutputFormat::Json => {
            println!("{}", json::to_pretty_json(&run_result)?);
        }
        OutputFormat::Sequence => {
            print_scenario_sequence(&run_result);
        }
        _ => {
            print_scenario_run(&run_result);
        }
    }

    Ok(())
}

/// Create a custom scenario template file
pub fn run_create(name: &str) -> Result<()> {
    let template = create_scenario_template(name);
    let filename = format!("{}.json", name);

    let content = serde_json::to_string_pretty(&template)
        .context("Failed to serialize scenario template")?;

    std::fs::write(&filename, &content)
        .with_context(|| format!("Failed to write {}", filename))?;

    let label = Color::DarkCyan;
    let value = Color::Grey;
    println!(
        "{} {}",
        "Created scenario template:".with(label),
        filename.as_str().with(value)
    );
    println!(
        "{}",
        "Edit the file, then run with: flowsight scenario run <name> --from-file <file>"
            .with(Color::DarkGrey)
    );

    Ok(())
}

// ─── Scenario execution ─────────────────────────────────────────────────────

fn execute_scenario(
    scenario: &ScenarioDefinition,
    opts: &ScenarioRunOptions,
) -> Result<ScenarioRunResult> {
    let max_steps = opts.max_depth.unwrap_or(usize::MAX);
    let mut async_handlers = Vec::new();
    let mut callbacks = Vec::new();
    let mut matched_functions = Vec::new();
    let mut bound_file_display: Option<String> = None;
    let mut bound_function_display: Option<String> = None;

    // If a driver file is bound, analyze it and merge results
    if let Some(ref driver_path) = opts.bind_driver {
        let path = Path::new(driver_path);

        if path.exists() {
            let mut ctx = AnalysisContext::new();
            let analysis = ctx.analyze_file(path)?;

            bound_file_display = Some(driver_path.clone());

            // Collect async handlers from the file
            for binding in &analysis.analysis.async_bindings {
                let mechanism_tag = format!("{:?}", binding.mechanism);
                let tag = mechanism_short_tag(&mechanism_tag);
                async_handlers.push(ScenarioAsyncHandler {
                    function: binding.handler.clone(),
                    mechanism: tag,
                    can_sleep: binding.context.can_sleep(),
                });
            }

            // Collect callbacks from the file
            let kb = ctx.knowledge_base();
            for (name, func) in &analysis.parse_result.functions {
                if func.is_callback {
                    if let Some((fw, cb, _)) = kb.identify_callback(name, &analysis.source) {
                        callbacks.push(ScenarioCallback {
                            ops_field: format!("{}.{}", fw, cb),
                            handler: name.clone(),
                        });
                    }
                }
            }

            // Find functions in the file that match scenario steps
            let file_functions: Vec<String> = analysis
                .parse_result
                .functions
                .keys()
                .cloned()
                .collect();

            for phase in &scenario.phases {
                for step in &phase.steps {
                    if step.is_bind_point {
                        // Check if any function in the file could fill this bind point
                        if let Some(ref bind_fn) = opts.bind_function {
                            if file_functions.contains(bind_fn) {
                                matched_functions.push(bind_fn.clone());
                                bound_function_display = Some(bind_fn.clone());
                            }
                        } else {
                            // Auto-detect: look for callbacks that match the step's context
                            for (name, func) in &analysis.parse_result.functions {
                                if func.is_callback {
                                    matched_functions.push(name.clone());
                                }
                            }
                        }
                    }
                }
            }

            // If a specific function is bound, also add its callees
            if let Some(ref bind_fn) = opts.bind_function {
                if let Some(func) = analysis.parse_result.functions.get(bind_fn.as_str()) {
                    for callee in &func.calls {
                        matched_functions.push(callee.clone());
                    }
                }
                bound_function_display = Some(bind_fn.clone());
            }

            // Dedup
            matched_functions.sort();
            matched_functions.dedup();
        }
    }

    // Apply depth limit: truncate steps per phase
    let trimmed_scenario = if max_steps < usize::MAX {
        let trimmed_phases: Vec<ScenarioPhase> = scenario
            .phases
            .iter()
            .map(|phase| {
                let steps: Vec<ScenarioStep> = phase
                    .steps
                    .iter()
                    .take(max_steps)
                    .cloned()
                    .collect();
                ScenarioPhase {
                    name: phase.name.clone(),
                    steps,
                }
            })
            .collect();

        ScenarioDefinition {
            phases: trimmed_phases,
            ..scenario.clone()
        }
    } else {
        scenario.clone()
    };

    Ok(ScenarioRunResult {
        scenario: trimmed_scenario,
        bound_file: bound_file_display,
        bound_function: bound_function_display,
        async_handlers,
        callbacks,
        matched_functions,
    })
}

// ─── Text output formatters ─────────────────────────────────────────────────

fn print_scenario_list(scenarios: &[ScenarioDefinition]) {
    let title = Color::DarkCyan;
    let dim = Color::DarkGrey;
    let value = Color::Grey;

    println!(
        "{}",
        "Available Kernel Scenarios".with(title)
    );
    println!(
        "{}",
        "\u{2550}".repeat(50).as_str().with(title)
    );
    println!();

    // Group by category
    let categories = [
        ("drivers", "Driver Subsystem"),
        ("memory", "Memory Management"),
        ("net", "Networking"),
        ("fs", "File Systems"),
        ("scheduler", "Process Scheduler"),
        ("irq", "Interrupt Handling"),
    ];

    for (cat_id, cat_name) in &categories {
        let in_category: Vec<_> = scenarios
            .iter()
            .filter(|s| s.category == *cat_id)
            .collect();

        if in_category.is_empty() {
            continue;
        }

        println!(
            "  {}",
            cat_name.with(Color::DarkYellow)
        );

        for s in &in_category {
            println!(
                "    {:<24} {}",
                s.id.as_str().with(value),
                s.description.as_str().with(dim)
            );
        }
        println!();
    }

    println!(
        "{}",
        "Use 'flowsight scenario show <name>' for details".with(dim)
    );
}

fn print_scenario_detail(scenario: &ScenarioDefinition) {
    let title = Color::DarkCyan;
    let dim = Color::DarkGrey;
    let value = Color::Grey;
    let accent = Color::DarkYellow;

    let bar = "\u{2550}".repeat(50);
    println!("{}", bar.as_str().with(title));
    println!(
        "{} {}",
        "Scenario:".with(title),
        scenario.name.as_str().with(value)
    );
    println!(
        "{} {}",
        "ID:".with(title),
        scenario.id.as_str().with(value)
    );
    println!(
        "{} {}",
        "Category:".with(title),
        scenario.category.as_str().with(value)
    );
    println!("{}", bar.as_str().with(title));
    println!();
    println!(
        "  {}",
        scenario.description.as_str().with(dim)
    );
    println!();

    // Show phases
    for (idx, phase) in scenario.phases.iter().enumerate() {
        let phase_header = format!("Phase {}: {}", idx + 1, phase.name);
        let underline = "\u{2500}".repeat(phase_header.len());
        println!("  {}", phase_header.as_str().with(accent));
        println!("  {}", underline.as_str().with(dim));

        for step in &phase.steps {
            let bind_tag = if step.is_bind_point {
                " [bindable]".with(Color::DarkYellow).to_string()
            } else {
                String::new()
            };

            let ctx_tag = match step.context.as_str() {
                "hardirq" => " [hardirq]",
                "softirq" => " [softirq]",
                "workqueue" => " [workqueue]",
                _ => "",
            };

            let file_info = step
                .file
                .as_deref()
                .map(|f| format!("  // {}", f))
                .unwrap_or_default();

            println!(
                "    {}{}(){}{}",
                "\u{2192} ".with(dim),
                step.function.as_str().with(value),
                ctx_tag.with(dim),
                bind_tag
            );

            if let Some(ref desc) = step.description {
                println!(
                    "      {}",
                    desc.as_str().with(dim)
                );
            }
            if !file_info.is_empty() {
                println!(
                    "      {}",
                    file_info.as_str().with(dim)
                );
            }
        }
        println!();
    }

    // Show bind params
    if !scenario.bind_params.is_empty() {
        println!(
            "  {}",
            "Bindable Parameters:".with(accent)
        );
        for (key, desc) in &scenario.bind_params {
            println!(
                "    --bind {}=<{}>",
                key.as_str().with(value),
                desc.as_str().with(dim)
            );
        }
        println!();
    }

    println!(
        "  {}",
        "Subsystems:".with(accent)
    );
    for sub in &scenario.subsystems {
        println!(
            "    {}",
            sub.as_str().with(dim)
        );
    }
}

fn print_scenario_run(result: &ScenarioRunResult) {
    let title = Color::DarkCyan;
    let dim = Color::DarkGrey;
    let value = Color::Grey;
    let accent = Color::DarkYellow;
    let highlight = Color::White;

    let bar = "\u{2550}".repeat(50);
    println!("{}", bar.as_str().with(title));
    println!(
        "{} {}",
        "Scenario:".with(title),
        result.scenario.name.as_str().with(value)
    );
    if let Some(ref f) = result.bound_file {
        println!(
            "{} {}",
            "Driver:".with(title),
            f.as_str().with(value)
        );
    }
    if let Some(ref f) = result.bound_function {
        println!(
            "{} {}",
            "Function:".with(title),
            f.as_str().with(value)
        );
    }
    println!("{}", bar.as_str().with(title));
    println!();

    // Print phases
    for (idx, phase) in result.scenario.phases.iter().enumerate() {
        let phase_header = format!("Phase {}: {}", idx + 1, phase.name);
        let underline = "\u{2500}".repeat(phase_header.len());
        println!("{}", phase_header.as_str().with(accent));
        println!("{}", underline.as_str().with(dim));

        for step in &phase.steps {
            let depth = step_indent_depth(step);
            let indent = "  ".repeat(depth);

            let is_matched = result.matched_functions.contains(&step.function);
            let is_bound = step.is_bind_point
                && result
                    .bound_function
                    .as_ref()
                    .map(|_| true)
                    .unwrap_or(false);

            let fn_color = if is_matched || is_bound {
                highlight
            } else {
                value
            };

            let ctx_suffix = match step.context.as_str() {
                "hardirq" => "  [hardirq]",
                "softirq" => "  [softirq]",
                "workqueue" => "  [workqueue context]",
                _ => "",
            };

            let bind_marker = if is_bound {
                if let Some(ref bound_fn) = result.bound_function {
                    format!("  [bound: {}]", bound_fn)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let desc_suffix = step
                .description
                .as_deref()
                .map(|d| format!("     {}", d))
                .unwrap_or_default();

            println!(
                "{}\u{2192} {}(){}{}",
                indent,
                step.function.as_str().with(fn_color),
                ctx_suffix.with(dim),
                bind_marker.as_str().with(accent)
            );

            if !desc_suffix.is_empty() {
                println!(
                    "{}{}",
                    indent,
                    desc_suffix.as_str().with(dim)
                );
            }
        }
        println!();
    }

    // Print bound file analysis results
    if result.bound_file.is_some() {
        if !result.async_handlers.is_empty() {
            println!(
                "{}",
                "Async Handlers Found:".with(accent)
            );
            for handler in &result.async_handlers {
                let sleep = if handler.can_sleep {
                    "can_sleep: true"
                } else {
                    "can_sleep: false"
                };
                println!(
                    "  [{}] {}  ({})",
                    handler.mechanism.as_str().with(value),
                    handler.function.as_str().with(highlight),
                    sleep.with(dim)
                );
            }
            println!();
        }

        if !result.callbacks.is_empty() {
            println!(
                "{}",
                "Callbacks Registered:".with(accent)
            );
            for cb in &result.callbacks {
                println!(
                    "  {} = {}",
                    cb.ops_field.as_str().with(dim),
                    cb.handler.as_str().with(highlight)
                );
            }
            println!();
        }

        if result.async_handlers.is_empty() && result.callbacks.is_empty() {
            println!(
                "  {}",
                "(No async handlers or callbacks detected in bound file)".with(dim)
            );
            println!();
        }
    }
}

fn print_scenario_sequence(result: &ScenarioRunResult) {
    // Simple sequence-style output
    let dim = Color::DarkGrey;
    let value = Color::Grey;
    let accent = Color::DarkYellow;

    let total_width = 60;
    let bar = "\u{2500}".repeat(total_width);

    println!(
        "{}",
        format!("=== {} ===", result.scenario.name).as_str().with(accent)
    );
    println!();

    for (idx, phase) in result.scenario.phases.iter().enumerate() {
        println!(
            "{}",
            format!("[Phase {}] {}", idx + 1, phase.name).as_str().with(accent)
        );
        println!("{}", bar.as_str().with(dim));

        for step in &phase.steps {
            let depth = step_indent_depth(step);
            let indent = "  ".repeat(depth);
            let arrow = "\u{2192}";

            let is_bound = step.is_bind_point
                && result
                    .bound_function
                    .as_ref()
                    .map(|_| true)
                    .unwrap_or(false);

            let marker = if is_bound {
                " <-- YOUR CODE"
            } else {
                ""
            };

            println!(
                "{}{} {}(){}",
                indent,
                arrow.with(dim),
                step.function.as_str().with(value),
                marker.with(accent)
            );
        }
        println!();
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn find_scenario<'a>(
    scenarios: &'a [ScenarioDefinition],
    name: &str,
) -> Result<&'a ScenarioDefinition> {
    let name_lower = name.to_lowercase();

    // Exact match
    if let Some(s) = scenarios.iter().find(|s| s.id == name_lower) {
        return Ok(s);
    }

    // Partial match
    let matches: Vec<_> = scenarios
        .iter()
        .filter(|s| s.id.contains(&name_lower) || s.name.to_lowercase().contains(&name_lower))
        .collect();

    match matches.len() {
        0 => {
            let available: Vec<_> = scenarios.iter().map(|s| s.id.as_str()).collect();
            anyhow::bail!(
                "Scenario '{}' not found. Available: {}",
                name,
                available.join(", ")
            )
        }
        1 => Ok(matches[0]),
        _ => {
            let matched_names: Vec<_> = matches.iter().map(|s| s.id.as_str()).collect();
            anyhow::bail!(
                "Ambiguous scenario '{}'. Matches: {}",
                name,
                matched_names.join(", ")
            )
        }
    }
}

fn mechanism_short_tag(mechanism: &str) -> String {
    if mechanism.contains("WorkQueue") {
        "WQ".to_string()
    } else if mechanism.contains("Timer") {
        "TM".to_string()
    } else if mechanism.contains("Interrupt") {
        "IRQ".to_string()
    } else if mechanism.contains("Tasklet") {
        "TL".to_string()
    } else if mechanism.contains("KThread") {
        "KT".to_string()
    } else {
        "ASYNC".to_string()
    }
}

fn step_indent_depth(step: &ScenarioStep) -> usize {
    // Compute depth from context: deeper steps get more indentation
    // Use a simple heuristic based on whether the step has a file path
    if step.is_bind_point {
        3
    } else if step.file.is_some() {
        2
    } else {
        1
    }
}

fn create_scenario_template(name: &str) -> ScenarioDefinition {
    ScenarioDefinition {
        id: name.to_string(),
        name: format!("Custom: {}", name),
        description: "Custom scenario - edit this template".to_string(),
        category: "custom".to_string(),
        phases: vec![ScenarioPhase {
            name: "Phase 1".to_string(),
            steps: vec![
                ScenarioStep {
                    function: "entry_function".to_string(),
                    file: Some("path/to/file.c".to_string()),
                    description: Some("Entry point".to_string()),
                    is_bind_point: false,
                    context: "process".to_string(),
                },
                ScenarioStep {
                    function: "your_function".to_string(),
                    file: None,
                    description: Some("Your driver code here".to_string()),
                    is_bind_point: true,
                    context: "process".to_string(),
                },
            ],
        }],
        bind_params: HashMap::from([
            (
                "driver".to_string(),
                "path to your driver source file".to_string(),
            ),
            (
                "function".to_string(),
                "entry function name".to_string(),
            ),
        ]),
        subsystems: vec!["custom".to_string()],
    }
}

// ─── Built-in scenario definitions ──────────────────────────────────────────

fn builtin_scenarios() -> Vec<ScenarioDefinition> {
    vec![
        scenario_usb_enumeration(),
        scenario_driver_probe(),
        scenario_irq_handling(),
        scenario_memory_alloc(),
        scenario_net_rx(),
        scenario_fs_read(),
        scenario_sched_switch(),
        scenario_platform_device(),
        scenario_clock_init(),
        scenario_gpio_ops(),
    ]
}

fn scenario_usb_enumeration() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "usb-enumeration".to_string(),
        name: "USB Device Enumeration".to_string(),
        description: "Complete USB device plug-in flow: detection, addressing, descriptor read, driver binding".to_string(),
        category: "drivers".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "Device Connection".to_string(),
                steps: vec![
                    step("usb_submit_urb", Some("drivers/usb/core/urb.c"), Some("Submit USB Request Block"), false, "process"),
                    step("usb_hcd_submit_urb", Some("drivers/usb/core/hcd.c"), Some("Host controller dispatch"), false, "process"),
                    step("ehci_urb_enqueue", Some("drivers/usb/host/ehci-hcd.c"), Some("HCD-specific URB enqueue"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Device Initialization".to_string(),
                steps: vec![
                    step("hub_event", Some("drivers/usb/core/hub.c"), Some("Hub event workqueue handler"), false, "workqueue"),
                    step("hub_port_connect", Some("drivers/usb/core/hub.c"), Some("Port connection handling"), false, "process"),
                    step("usb_new_device", Some("drivers/usb/core/hub.c"), Some("Create new USB device structure"), false, "process"),
                    step("usb_enumerate_device", Some("drivers/usb/core/hub.c"), Some("Read device descriptors"), false, "process"),
                    step("usb_get_device_descriptor", Some("drivers/usb/core/message.c"), Some("GET_DESCRIPTOR control transfer"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Driver Binding".to_string(),
                steps: vec![
                    step("device_add", Some("drivers/base/core.c"), Some("Add device to driver model"), false, "process"),
                    step("bus_probe_device", Some("drivers/base/bus.c"), Some("Bus layer probe"), false, "process"),
                    step("usb_probe_interface", Some("drivers/usb/core/driver.c"), Some("USB interface-level probe"), false, "process"),
                    step("drv->probe", None, Some("Your driver probe function"), true, "process"),
                ],
            },
            ScenarioPhase {
                name: "Configuration".to_string(),
                steps: vec![
                    step("usb_set_configuration", Some("drivers/usb/core/message.c"), Some("SET_CONFIGURATION request"), false, "process"),
                    step("usb_ep_autoconfig", Some("drivers/usb/gadget/epautoconf.c"), Some("Auto-configure endpoints"), false, "process"),
                    step("request_irq", Some("kernel/irq/manage.c"), Some("Register interrupt handler"), false, "process"),
                ],
            },
        ],
        bind_params: HashMap::from([
            ("driver".to_string(), "path to USB driver source file".to_string()),
            ("function".to_string(), "probe function name".to_string()),
        ]),
        subsystems: vec![
            "drivers/usb/core".to_string(),
            "drivers/usb/host".to_string(),
            "drivers/base".to_string(),
        ],
    }
}

fn scenario_driver_probe() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "driver-probe".to_string(),
        name: "Driver Probe Lifecycle".to_string(),
        description: "Module load to driver probe: init, registration, bus matching, resource setup".to_string(),
        category: "drivers".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "Module Loading".to_string(),
                steps: vec![
                    step("sys_init_module", Some("kernel/module.c"), Some("System call entry"), false, "process"),
                    step("load_module", Some("kernel/module.c"), Some("Load ELF, resolve symbols"), false, "process"),
                    step("do_init_module", Some("kernel/module.c"), Some("Call module_init()"), false, "process"),
                    step("module_init", None, Some("Your __init function"), true, "process"),
                ],
            },
            ScenarioPhase {
                name: "Driver Registration".to_string(),
                steps: vec![
                    step("register_driver", None, Some("Bus-specific registration (e.g., usb_register)"), true, "process"),
                    step("driver_register", Some("drivers/base/driver.c"), Some("Core driver model registration"), false, "process"),
                    step("bus_add_driver", Some("drivers/base/bus.c"), Some("Add driver to bus driver list"), false, "process"),
                    step("driver_attach", Some("drivers/base/dd.c"), Some("Try to match existing devices"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Device Matching".to_string(),
                steps: vec![
                    step("__driver_attach", Some("drivers/base/dd.c"), Some("Iterate devices on the bus"), false, "process"),
                    step("driver_match_device", Some("drivers/base/base.h"), Some("Call bus->match()"), false, "process"),
                    step("driver_probe_device", Some("drivers/base/dd.c"), Some("Attempt probe"), false, "process"),
                    step("really_probe", Some("drivers/base/dd.c"), Some("Actual probe execution"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Resource Setup (in probe)".to_string(),
                steps: vec![
                    step("drv->probe", None, Some("Your probe function"), true, "process"),
                    step("devm_kzalloc", Some("drivers/base/devres.c"), Some("Device-managed memory allocation"), false, "process"),
                    step("clk_prepare_enable", Some("drivers/clk/clk.c"), Some("Enable device clock"), false, "process"),
                    step("devm_ioremap_resource", Some("lib/devres.c"), Some("Map device registers"), false, "process"),
                    step("devm_request_irq", Some("kernel/irq/devres.c"), Some("Register device IRQ handler"), false, "process"),
                ],
            },
        ],
        bind_params: HashMap::from([
            ("driver".to_string(), "path to driver source file".to_string()),
            ("function".to_string(), "probe function name".to_string()),
        ]),
        subsystems: vec![
            "drivers/base".to_string(),
            "kernel/module.c".to_string(),
        ],
    }
}

fn scenario_irq_handling() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "irq-handling".to_string(),
        name: "Interrupt Handling".to_string(),
        description: "Hardware interrupt to deferred work: hardirq, softirq, threaded IRQ paths".to_string(),
        category: "irq".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "Hardware Interrupt Entry".to_string(),
                steps: vec![
                    step("asm_do_IRQ", Some("arch/arm/kernel/irq.c"), Some("Architecture-specific IRQ entry"), false, "hardirq"),
                    step("handle_domain_irq", Some("kernel/irq/irqdesc.c"), Some("IRQ domain dispatch"), false, "hardirq"),
                    step("generic_handle_irq", Some("kernel/irq/irqdesc.c"), Some("Generic IRQ handler"), false, "hardirq"),
                    step("handle_fasteoi_irq", Some("kernel/irq/chip.c"), Some("Flow handler (edge/level/fasteoi)"), false, "hardirq"),
                    step("handle_irq_event", Some("kernel/irq/handle.c"), Some("Execute IRQ action chain"), false, "hardirq"),
                    step("action->handler", None, Some("Your hardirq handler"), true, "hardirq"),
                ],
            },
            ScenarioPhase {
                name: "Softirq Processing".to_string(),
                steps: vec![
                    step("irq_exit", Some("kernel/softirq.c"), Some("Leaving hardirq, check pending softirqs"), false, "hardirq"),
                    step("__do_softirq", Some("kernel/softirq.c"), Some("Process pending softirqs"), false, "softirq"),
                    step("tasklet_action", Some("kernel/softirq.c"), Some("Run scheduled tasklets"), false, "softirq"),
                    step("tasklet->func", None, Some("Your tasklet handler (cannot sleep!)"), true, "softirq"),
                ],
            },
            ScenarioPhase {
                name: "Threaded IRQ (deferred)".to_string(),
                steps: vec![
                    step("irq_thread", Some("kernel/irq/manage.c"), Some("Kernel thread for threaded IRQs"), false, "process"),
                    step("irq_thread_fn", Some("kernel/irq/manage.c"), Some("Thread function wrapper"), false, "process"),
                    step("action->thread_fn", None, Some("Your threaded IRQ handler (can sleep)"), true, "process"),
                ],
            },
            ScenarioPhase {
                name: "WorkQueue Deferral".to_string(),
                steps: vec![
                    step("kworker_thread", Some("kernel/workqueue.c"), Some("Worker thread wakes up"), false, "process"),
                    step("process_one_work", Some("kernel/workqueue.c"), Some("Dequeue and execute work"), false, "process"),
                    step("work->func", None, Some("Your work handler (can sleep)"), true, "process"),
                ],
            },
        ],
        bind_params: HashMap::from([
            ("driver".to_string(), "path to driver with IRQ handling".to_string()),
            ("function".to_string(), "IRQ handler function name".to_string()),
        ]),
        subsystems: vec![
            "kernel/irq".to_string(),
            "kernel/softirq.c".to_string(),
            "kernel/workqueue.c".to_string(),
        ],
    }
}

fn scenario_memory_alloc() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "memory-alloc".to_string(),
        name: "Memory Allocation Path".to_string(),
        description: "kmalloc to page allocation: slab cache, buddy allocator, OOM killer path".to_string(),
        category: "memory".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "Slab Allocator".to_string(),
                steps: vec![
                    step("kmalloc", Some("include/linux/slab.h"), Some("Kernel memory allocation entry"), false, "process"),
                    step("__kmalloc", Some("mm/slab.c"), Some("Internal kmalloc dispatch"), false, "process"),
                    step("kmem_cache_alloc_trace", Some("mm/slub.c"), Some("SLUB allocator fast path"), false, "process"),
                    step("slab_alloc_node", Some("mm/slub.c"), Some("Try per-CPU slab freelist"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Slab Refill".to_string(),
                steps: vec![
                    step("__slab_alloc", Some("mm/slub.c"), Some("Slow path: need new slab"), false, "process"),
                    step("new_slab", Some("mm/slub.c"), Some("Allocate a new slab page"), false, "process"),
                    step("allocate_slab", Some("mm/slub.c"), Some("Request pages from buddy"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Page Allocator (Buddy System)".to_string(),
                steps: vec![
                    step("alloc_pages", Some("mm/page_alloc.c"), Some("Buddy allocator entry"), false, "process"),
                    step("__alloc_pages_nodemask", Some("mm/page_alloc.c"), Some("Core page allocation"), false, "process"),
                    step("get_page_from_freelist", Some("mm/page_alloc.c"), Some("Fast path: try freelists"), false, "process"),
                    step("rmqueue", Some("mm/page_alloc.c"), Some("Remove page from buddy freelist"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "OOM Path (when memory exhausted)".to_string(),
                steps: vec![
                    step("__alloc_pages_slowpath", Some("mm/page_alloc.c"), Some("Slow path: reclaim + compact"), false, "process"),
                    step("__perform_reclaim", Some("mm/page_alloc.c"), Some("Try to reclaim pages"), false, "process"),
                    step("out_of_memory", Some("mm/oom_kill.c"), Some("OOM killer activation"), false, "process"),
                    step("oom_kill_process", Some("mm/oom_kill.c"), Some("Select and kill a process"), false, "process"),
                ],
            },
        ],
        bind_params: HashMap::new(),
        subsystems: vec![
            "mm/slub.c".to_string(),
            "mm/page_alloc.c".to_string(),
            "mm/oom_kill.c".to_string(),
        ],
    }
}

fn scenario_net_rx() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "net-rx".to_string(),
        name: "Network Packet Receive".to_string(),
        description: "NIC interrupt to TCP socket: NAPI poll, protocol demux, socket delivery".to_string(),
        category: "net".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "NIC Interrupt".to_string(),
                steps: vec![
                    step("nic_irq_handler", None, Some("NIC hardware interrupt fires"), true, "hardirq"),
                    step("napi_schedule", Some("include/linux/netdevice.h"), Some("Schedule NAPI poll"), false, "hardirq"),
                    step("__raise_softirq_irqoff", Some("kernel/softirq.c"), Some("Raise NET_RX_SOFTIRQ"), false, "hardirq"),
                ],
            },
            ScenarioPhase {
                name: "NAPI Poll (softirq)".to_string(),
                steps: vec![
                    step("net_rx_action", Some("net/core/dev.c"), Some("NET_RX softirq handler"), false, "softirq"),
                    step("napi_poll", Some("net/core/dev.c"), Some("Call driver NAPI poll"), false, "softirq"),
                    step("napi->poll", None, Some("Driver poll function (read packets from ring)"), true, "softirq"),
                    step("napi_gro_receive", Some("net/core/gro.c"), Some("Generic Receive Offload aggregation"), false, "softirq"),
                    step("netif_receive_skb", Some("net/core/dev.c"), Some("Hand skb to protocol stack"), false, "softirq"),
                ],
            },
            ScenarioPhase {
                name: "Protocol Demux".to_string(),
                steps: vec![
                    step("__netif_receive_skb_core", Some("net/core/dev.c"), Some("L2 processing + protocol dispatch"), false, "softirq"),
                    step("ip_rcv", Some("net/ipv4/ip_input.c"), Some("IPv4 receive entry"), false, "softirq"),
                    step("ip_rcv_finish", Some("net/ipv4/ip_input.c"), Some("Routing decision"), false, "softirq"),
                    step("ip_local_deliver", Some("net/ipv4/ip_input.c"), Some("Deliver to local protocols"), false, "softirq"),
                    step("ip_local_deliver_finish", Some("net/ipv4/ip_input.c"), Some("Protocol handler dispatch"), false, "softirq"),
                ],
            },
            ScenarioPhase {
                name: "TCP/Socket Delivery".to_string(),
                steps: vec![
                    step("tcp_v4_rcv", Some("net/ipv4/tcp_ipv4.c"), Some("TCP receive entry"), false, "softirq"),
                    step("tcp_v4_do_rcv", Some("net/ipv4/tcp_ipv4.c"), Some("TCP state machine processing"), false, "softirq"),
                    step("tcp_rcv_established", Some("net/ipv4/tcp_input.c"), Some("Fast path for established connections"), false, "softirq"),
                    step("tcp_queue_rcv", Some("net/ipv4/tcp_input.c"), Some("Queue data to socket receive buffer"), false, "softirq"),
                    step("sk_data_ready", Some("net/core/sock.c"), Some("Wake up blocked reader"), false, "softirq"),
                ],
            },
        ],
        bind_params: HashMap::from([
            ("driver".to_string(), "path to NIC driver source".to_string()),
            ("function".to_string(), "NAPI poll function name".to_string()),
        ]),
        subsystems: vec![
            "net/core".to_string(),
            "net/ipv4".to_string(),
            "drivers/net".to_string(),
        ],
    }
}

fn scenario_fs_read() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "fs-read".to_string(),
        name: "File System Read".to_string(),
        description: "VFS read to block I/O: page cache, filesystem ops, block layer submission".to_string(),
        category: "fs".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "VFS Layer".to_string(),
                steps: vec![
                    step("sys_read", Some("fs/read_write.c"), Some("read() system call entry"), false, "process"),
                    step("ksys_read", Some("fs/read_write.c"), Some("Internal read dispatch"), false, "process"),
                    step("vfs_read", Some("fs/read_write.c"), Some("VFS read entry point"), false, "process"),
                    step("new_sync_read", Some("fs/read_write.c"), Some("Convert to iov_iter based read"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Filesystem-Specific".to_string(),
                steps: vec![
                    step("file->f_op->read_iter", None, Some("Filesystem read_iter callback"), true, "process"),
                    step("generic_file_read_iter", Some("mm/filemap.c"), Some("Generic implementation (most filesystems)"), false, "process"),
                    step("filemap_read", Some("mm/filemap.c"), Some("Page cache read path"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Page Cache".to_string(),
                steps: vec![
                    step("filemap_get_pages", Some("mm/filemap.c"), Some("Look up pages in cache"), false, "process"),
                    step("page_cache_sync_readahead", Some("mm/readahead.c"), Some("Synchronous readahead if cache miss"), false, "process"),
                    step("readahead_expand", Some("mm/readahead.c"), Some("Expand readahead window"), false, "process"),
                    step("read_pages", Some("mm/readahead.c"), Some("Submit pages for I/O"), false, "process"),
                    step("aops->readahead", None, Some("Filesystem-specific readahead"), true, "process"),
                ],
            },
            ScenarioPhase {
                name: "Block I/O".to_string(),
                steps: vec![
                    step("submit_bio", Some("block/blk-core.c"), Some("Submit block I/O request"), false, "process"),
                    step("blk_mq_submit_bio", Some("block/blk-mq.c"), Some("Multi-queue dispatch"), false, "process"),
                    step("blk_mq_try_issue_directly", Some("block/blk-mq.c"), Some("Try direct issue to hardware"), false, "process"),
                    step("scsi_queue_rq", Some("drivers/scsi/scsi_lib.c"), Some("SCSI layer command submission"), false, "process"),
                ],
            },
        ],
        bind_params: HashMap::from([
            ("driver".to_string(), "path to filesystem source".to_string()),
            ("function".to_string(), "read_iter implementation".to_string()),
        ]),
        subsystems: vec![
            "fs/read_write.c".to_string(),
            "mm/filemap.c".to_string(),
            "block/blk-mq.c".to_string(),
        ],
    }
}

fn scenario_sched_switch() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "sched-switch".to_string(),
        name: "Scheduler Context Switch".to_string(),
        description: "Voluntary/involuntary context switch: scheduler classes, task selection, CPU migration".to_string(),
        category: "scheduler".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "Schedule Entry".to_string(),
                steps: vec![
                    step("schedule", Some("kernel/sched/core.c"), Some("Main scheduler entry (voluntary or preempt)"), false, "process"),
                    step("__schedule", Some("kernel/sched/core.c"), Some("Core scheduling logic"), false, "process"),
                    step("deactivate_task", Some("kernel/sched/core.c"), Some("Remove current from runqueue"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Task Selection".to_string(),
                steps: vec![
                    step("pick_next_task", Some("kernel/sched/core.c"), Some("Iterate scheduler classes"), false, "process"),
                    step("pick_next_task_fair", Some("kernel/sched/fair.c"), Some("CFS: pick task with smallest vruntime"), false, "process"),
                    step("pick_next_entity", Some("kernel/sched/fair.c"), Some("Select from CFS red-black tree"), false, "process"),
                    step("set_next_entity", Some("kernel/sched/fair.c"), Some("Mark task as running"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Context Switch".to_string(),
                steps: vec![
                    step("context_switch", Some("kernel/sched/core.c"), Some("Perform the actual switch"), false, "process"),
                    step("switch_mm_irqs_off", Some("arch/arm/include/asm/mmu_context.h"), Some("Switch memory map (page tables)"), false, "process"),
                    step("switch_to", Some("arch/arm/include/asm/switch_to.h"), Some("Switch CPU registers"), false, "process"),
                    step("__switch_to", Some("arch/arm/kernel/entry-armv.S"), Some("Architecture-specific register save/restore"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "CPU Migration (if needed)".to_string(),
                steps: vec![
                    step("select_task_rq_fair", Some("kernel/sched/fair.c"), Some("Select target CPU for task"), false, "process"),
                    step("select_idle_sibling", Some("kernel/sched/fair.c"), Some("Find idle CPU in topology"), false, "process"),
                    step("migrate_task_to", Some("kernel/sched/core.c"), Some("Move task to new CPU"), false, "process"),
                ],
            },
        ],
        bind_params: HashMap::new(),
        subsystems: vec![
            "kernel/sched/core.c".to_string(),
            "kernel/sched/fair.c".to_string(),
            "arch/arm/kernel".to_string(),
        ],
    }
}

fn scenario_platform_device() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "platform-device".to_string(),
        name: "Platform Device Registration".to_string(),
        description: "Device-tree to probe: OF matching, platform bus, resource parsing".to_string(),
        category: "drivers".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "Device Tree Parsing".to_string(),
                steps: vec![
                    step("of_platform_populate", Some("drivers/of/platform.c"), Some("Parse DT nodes into platform devices"), false, "process"),
                    step("of_platform_bus_create", Some("drivers/of/platform.c"), Some("Create bus from DT node"), false, "process"),
                    step("of_platform_device_create_pdata", Some("drivers/of/platform.c"), Some("Create platform_device from DT"), false, "process"),
                    step("of_device_alloc", Some("drivers/of/platform.c"), Some("Allocate and populate resources from DT"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Device Registration".to_string(),
                steps: vec![
                    step("platform_device_register", Some("drivers/base/platform.c"), Some("Register platform device"), false, "process"),
                    step("platform_device_add", Some("drivers/base/platform.c"), Some("Add to platform bus"), false, "process"),
                    step("device_add", Some("drivers/base/core.c"), Some("Core device model add"), false, "process"),
                    step("bus_probe_device", Some("drivers/base/bus.c"), Some("Trigger bus probing"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Driver Matching".to_string(),
                steps: vec![
                    step("platform_match", Some("drivers/base/platform.c"), Some("Platform bus match function"), false, "process"),
                    step("of_driver_match_device", Some("drivers/of/device.c"), Some("Match against compatible strings"), false, "process"),
                    step("of_match_device", Some("drivers/of/device.c"), Some("DT compatible table lookup"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Probe Execution".to_string(),
                steps: vec![
                    step("driver_probe_device", Some("drivers/base/dd.c"), Some("Execute probe sequence"), false, "process"),
                    step("really_probe", Some("drivers/base/dd.c"), Some("Actual probe with error handling"), false, "process"),
                    step("platform_drv_probe", Some("drivers/base/platform.c"), Some("Platform-specific probe wrapper"), false, "process"),
                    step("drv->probe", None, Some("Your platform driver probe"), true, "process"),
                ],
            },
        ],
        bind_params: HashMap::from([
            ("driver".to_string(), "path to platform driver source".to_string()),
            ("function".to_string(), "probe function name".to_string()),
        ]),
        subsystems: vec![
            "drivers/base/platform.c".to_string(),
            "drivers/of".to_string(),
            "drivers/base/dd.c".to_string(),
        ],
    }
}

fn scenario_clock_init() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "clock-init".to_string(),
        name: "Clock Framework".to_string(),
        description: "Clock provider registration and tree construction: clk_hw, parent relationships, rate calculation".to_string(),
        category: "drivers".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "Clock Registration".to_string(),
                steps: vec![
                    step("clk_register", Some("drivers/clk/clk.c"), Some("Legacy clock registration"), false, "process"),
                    step("clk_hw_register", Some("drivers/clk/clk.c"), Some("Hardware clock registration (preferred)"), false, "process"),
                    step("clk_core_create", Some("drivers/clk/clk.c"), Some("Create clk_core structure"), false, "process"),
                    step("__clk_core_init", Some("drivers/clk/clk.c"), Some("Initialize clock: ops, parent lookup"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Parent Resolution".to_string(),
                steps: vec![
                    step("clk_core_get_parent_by_index", Some("drivers/clk/clk.c"), Some("Resolve parent clock"), false, "process"),
                    step("clk_core_fill_parent_index", Some("drivers/clk/clk.c"), Some("Fill parent from DT/name"), false, "process"),
                    step("of_clk_get_parent_name", Some("drivers/clk/clk.c"), Some("Get parent name from device tree"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Rate Calculation".to_string(),
                steps: vec![
                    step("clk_set_rate", Some("drivers/clk/clk.c"), Some("Request rate change"), false, "process"),
                    step("clk_core_set_rate_nolock", Some("drivers/clk/clk.c"), Some("Core rate-setting logic"), false, "process"),
                    step("clk_calc_new_rates", Some("drivers/clk/clk.c"), Some("Walk tree to compute new rates"), false, "process"),
                    step("ops->determine_rate", None, Some("Your clock determine_rate callback"), true, "process"),
                    step("ops->set_rate", None, Some("Your clock set_rate callback"), true, "process"),
                ],
            },
            ScenarioPhase {
                name: "Clock Enable".to_string(),
                steps: vec![
                    step("clk_prepare_enable", Some("include/linux/clk.h"), Some("Prepare + enable convenience wrapper"), false, "process"),
                    step("clk_core_prepare", Some("drivers/clk/clk.c"), Some("Recursive prepare (can sleep)"), false, "process"),
                    step("ops->prepare", None, Some("Your clock prepare callback"), true, "process"),
                    step("clk_core_enable", Some("drivers/clk/clk.c"), Some("Recursive enable (atomic)"), false, "process"),
                    step("ops->enable", None, Some("Your clock enable callback"), true, "process"),
                ],
            },
        ],
        bind_params: HashMap::from([
            ("driver".to_string(), "path to clock driver source".to_string()),
            ("function".to_string(), "clock ops function name".to_string()),
        ]),
        subsystems: vec![
            "drivers/clk".to_string(),
            "include/linux/clk-provider.h".to_string(),
        ],
    }
}

fn scenario_gpio_ops() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "gpio-ops".to_string(),
        name: "GPIO Operations".to_string(),
        description: "GPIO request to I/O: gpiod API, pin controller, chip operations".to_string(),
        category: "drivers".to_string(),
        phases: vec![
            ScenarioPhase {
                name: "GPIO Request".to_string(),
                steps: vec![
                    step("gpiod_get", Some("drivers/gpio/gpiolib-devres.c"), Some("Device-managed GPIO request"), false, "process"),
                    step("gpiod_get_index", Some("drivers/gpio/gpiolib-devres.c"), Some("Get GPIO by index"), false, "process"),
                    step("gpio_device_get_desc", Some("drivers/gpio/gpiolib.c"), Some("Resolve GPIO descriptor"), false, "process"),
                    step("gpiod_request", Some("drivers/gpio/gpiolib.c"), Some("Request exclusive GPIO access"), false, "process"),
                    step("gpiochip_request_own_desc", Some("drivers/gpio/gpiolib.c"), Some("Chip internal request"), false, "process"),
                ],
            },
            ScenarioPhase {
                name: "Pin Multiplexing".to_string(),
                steps: vec![
                    step("pinctrl_gpio_request", Some("drivers/pinctrl/core.c"), Some("Request pin from pinctrl"), false, "process"),
                    step("pinmux_request_gpio", Some("drivers/pinctrl/pinmux.c"), Some("Set pin to GPIO function"), false, "process"),
                    step("ops->gpio_request_enable", None, Some("Pin controller mux callback"), true, "process"),
                ],
            },
            ScenarioPhase {
                name: "Direction Configuration".to_string(),
                steps: vec![
                    step("gpiod_direction_output", Some("drivers/gpio/gpiolib.c"), Some("Set GPIO as output"), false, "process"),
                    step("gpio_set_config", Some("drivers/gpio/gpiolib.c"), Some("Apply configuration to pin"), false, "process"),
                    step("chip->direction_output", None, Some("GPIO chip direction_output callback"), true, "process"),
                ],
            },
            ScenarioPhase {
                name: "I/O Operations".to_string(),
                steps: vec![
                    step("gpiod_set_value", Some("drivers/gpio/gpiolib.c"), Some("Set GPIO pin value"), false, "process"),
                    step("gpio_chip_set_value", Some("drivers/gpio/gpiolib.c"), Some("Dispatch to chip ops"), false, "process"),
                    step("chip->set", None, Some("GPIO chip set callback (register write)"), true, "process"),
                    step("gpiod_get_value", Some("drivers/gpio/gpiolib.c"), Some("Read GPIO pin value"), false, "process"),
                    step("chip->get", None, Some("GPIO chip get callback (register read)"), true, "process"),
                ],
            },
        ],
        bind_params: HashMap::from([
            ("driver".to_string(), "path to GPIO chip driver source".to_string()),
            ("function".to_string(), "GPIO chip ops function name".to_string()),
        ]),
        subsystems: vec![
            "drivers/gpio".to_string(),
            "drivers/pinctrl".to_string(),
        ],
    }
}

// ─── Step builder ───────────────────────────────────────────────────────────

fn step(
    function: &str,
    file: Option<&str>,
    description: Option<&str>,
    is_bind_point: bool,
    context: &str,
) -> ScenarioStep {
    ScenarioStep {
        function: function.to_string(),
        file: file.map(String::from),
        description: description.map(String::from),
        is_bind_point,
        context: context.to_string(),
    }
}
