//! Async mechanism tracker
//!
//! Tracks async patterns like:
//! - Work queues (INIT_WORK, schedule_work)
//! - Timers (timer_setup, mod_timer)
//! - Interrupts (request_irq)
//! - Tasklets (tasklet_init)
//! - Kernel threads (kthread_run)
//!
//! Integrates with KnowledgeBase for enhanced pattern matching.

use flowsight_core::{AsyncBinding, AsyncMechanism, ExecutionContext, FunctionDef, Location};
use flowsight_knowledge::KnowledgeBase;
use regex::Regex;
use std::collections::HashMap;

/// Pattern definition for async mechanism
struct AsyncPattern {
    mechanism: AsyncMechanism,
    context: ExecutionContext,
    bind_patterns: Vec<Regex>,
    trigger_patterns: Vec<Regex>,
}

/// Async mechanism tracker
pub struct AsyncTracker {
    patterns: Vec<AsyncPattern>,
    /// Additional patterns from knowledge base
    kb_patterns: Vec<KbAsyncPattern>,
}

/// Async pattern from knowledge base
struct KbAsyncPattern {
    /// Pattern name from KB (e.g., "work_struct", "timer_list")
    name: String,
    /// Async mechanism type
    mechanism: AsyncMechanism,
    /// Execution context
    context: ExecutionContext,
    /// Bind patterns (regex)
    bind_patterns: Vec<Regex>,
    /// Trigger patterns (regex)
    trigger_patterns: Vec<Regex>,
}

impl AsyncTracker {
    /// Create a new async tracker with default patterns
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
            kb_patterns: Vec::new(),
        }
    }

    /// Create an async tracker with knowledge base integration
    pub fn with_knowledge_base(kb: &KnowledgeBase) -> Self {
        let mut tracker = Self::new();
        tracker.load_from_knowledge_base(kb);
        tracker
    }

    /// Load async patterns from knowledge base
    pub fn load_from_knowledge_base(&mut self, kb: &KnowledgeBase) {
        for (pattern_name, async_pattern) in &kb.async_patterns {
            // Convert KB execution context to core type
            let context = match async_pattern.context {
                flowsight_knowledge::ExecutionContext::Process => ExecutionContext::Process,
                flowsight_knowledge::ExecutionContext::SoftIrq => ExecutionContext::SoftIrq,
                flowsight_knowledge::ExecutionContext::HardIrq => ExecutionContext::HardIrq,
                flowsight_knowledge::ExecutionContext::User => ExecutionContext::Process,
                flowsight_knowledge::ExecutionContext::Unknown => ExecutionContext::Unknown,
            };

            // Determine mechanism from pattern name
            let mechanism = Self::infer_mechanism_from_name(pattern_name);

            // Compile bind patterns
            let bind_patterns: Vec<Regex> = async_pattern
                .bind_patterns
                .iter()
                .filter_map(|p| Regex::new(p).ok())
                .collect();

            // Compile trigger patterns
            let trigger_patterns: Vec<Regex> = async_pattern
                .trigger_patterns
                .iter()
                .filter_map(|p| Regex::new(p).ok())
                .collect();

            if !bind_patterns.is_empty() {
                self.kb_patterns.push(KbAsyncPattern {
                    name: pattern_name.clone(),
                    mechanism,
                    context,
                    bind_patterns,
                    trigger_patterns,
                });
            }
        }
    }

    /// Infer async mechanism from pattern name
    fn infer_mechanism_from_name(name: &str) -> AsyncMechanism {
        let name_lower = name.to_lowercase();
        if name_lower.contains("work") || name_lower.contains("delayed_work") {
            AsyncMechanism::WorkQueue {
                delayed: name_lower.contains("delayed"),
            }
        } else if name_lower.contains("timer") || name_lower.contains("hrtimer") {
            AsyncMechanism::Timer {
                high_resolution: name_lower.contains("hr"),
            }
        } else if name_lower.contains("irq") || name_lower.contains("interrupt") {
            AsyncMechanism::Interrupt {
                threaded: name_lower.contains("threaded"),
            }
        } else if name_lower.contains("tasklet") {
            AsyncMechanism::Tasklet
        } else if name_lower.contains("kthread") || name_lower.contains("thread") {
            AsyncMechanism::KThread
        } else if name_lower.contains("rcu") {
            AsyncMechanism::RcuCallback
        } else if name_lower.contains("notifier") {
            AsyncMechanism::Notifier
        } else if name_lower.contains("softirq") {
            AsyncMechanism::Softirq
        } else {
            AsyncMechanism::Custom(name.to_string())
        }
    }

    fn default_patterns() -> Vec<AsyncPattern> {
        vec![
            // Work queue
            AsyncPattern {
                mechanism: AsyncMechanism::WorkQueue { delayed: false },
                context: ExecutionContext::Process,
                bind_patterns: vec![Regex::new(
                    r"INIT_WORK\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)",
                )
                .unwrap()],
                trigger_patterns: vec![
                    Regex::new(r"schedule_work\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"queue_work\s*\([^,]+,\s*&?([\w\.\->]+)\s*\)").unwrap(),
                ],
            },
            // Delayed work
            AsyncPattern {
                mechanism: AsyncMechanism::WorkQueue { delayed: true },
                context: ExecutionContext::Process,
                bind_patterns: vec![Regex::new(
                    r"INIT_DELAYED_WORK\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)",
                )
                .unwrap()],
                trigger_patterns: vec![Regex::new(
                    r"schedule_delayed_work\s*\(\s*&?([\w\.\->]+)\s*,",
                )
                .unwrap()],
            },
            // Timer
            AsyncPattern {
                mechanism: AsyncMechanism::Timer {
                    high_resolution: false,
                },
                context: ExecutionContext::SoftIrq,
                bind_patterns: vec![
                    Regex::new(r"timer_setup\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*,").unwrap(),
                    Regex::new(r"DEFINE_TIMER\s*\(\s*(\w+)\s*,\s*(\w+)\s*\)").unwrap(),
                ],
                trigger_patterns: vec![
                    Regex::new(r"mod_timer\s*\(\s*&?([\w\.\->]+)\s*,").unwrap(),
                    Regex::new(r"add_timer\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                ],
            },
            // High-resolution timer
            AsyncPattern {
                mechanism: AsyncMechanism::Timer {
                    high_resolution: true,
                },
                context: ExecutionContext::HardIrq,
                bind_patterns: vec![Regex::new(r"([\w\.\->]+)\.function\s*=\s*(\w+)").unwrap()],
                trigger_patterns: vec![
                    Regex::new(r"hrtimer_start\s*\(\s*&?([\w\.\->]+)\s*,").unwrap()
                ],
            },
            // Interrupt
            AsyncPattern {
                mechanism: AsyncMechanism::Interrupt { threaded: false },
                context: ExecutionContext::HardIrq,
                bind_patterns: vec![
                    Regex::new(r"request_irq\s*\([^,]+,\s*(\w+)\s*,").unwrap(),
                    Regex::new(r"devm_request_irq\s*\([^,]+,\s*[^,]+,\s*(\w+)\s*,").unwrap(),
                ],
                trigger_patterns: vec![],
            },
            // Threaded interrupt
            AsyncPattern {
                mechanism: AsyncMechanism::Interrupt { threaded: true },
                context: ExecutionContext::Process,
                bind_patterns: vec![Regex::new(
                    r"request_threaded_irq\s*\([^,]+,\s*\w+\s*,\s*(\w+)\s*,",
                )
                .unwrap()],
                trigger_patterns: vec![],
            },
            // Tasklet
            AsyncPattern {
                mechanism: AsyncMechanism::Tasklet,
                context: ExecutionContext::SoftIrq,
                bind_patterns: vec![
                    Regex::new(r"tasklet_init\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*,").unwrap(),
                    Regex::new(r"DECLARE_TASKLET\s*\(\s*(\w+)\s*,\s*(\w+)\s*\)").unwrap(),
                ],
                trigger_patterns: vec![
                    Regex::new(r"tasklet_schedule\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap()
                ],
            },
            // Kernel thread
            AsyncPattern {
                mechanism: AsyncMechanism::KThread,
                context: ExecutionContext::Process,
                bind_patterns: vec![
                    Regex::new(r"kthread_run\s*\(\s*(\w+)\s*,").unwrap(),
                    Regex::new(r"kthread_create\s*\(\s*(\w+)\s*,").unwrap(),
                ],
                trigger_patterns: vec![Regex::new(r"wake_up_process\s*\(").unwrap()],
            },
            // RCU callback
            AsyncPattern {
                mechanism: AsyncMechanism::RcuCallback,
                context: ExecutionContext::SoftIrq,
                bind_patterns: vec![
                    Regex::new(r"call_rcu\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)").unwrap(),
                    Regex::new(r"call_rcu_sched\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)").unwrap(),
                ],
                trigger_patterns: vec![],
            },
            // Notifier chain
            AsyncPattern {
                mechanism: AsyncMechanism::Notifier,
                context: ExecutionContext::Process,
                bind_patterns: vec![
                    Regex::new(r"register_reboot_notifier\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"register_netdevice_notifier\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(
                        r"blocking_notifier_chain_register\s*\([^,]+,\s*&?([\w\.\->]+)\s*\)",
                    )
                    .unwrap(),
                    Regex::new(r"atomic_notifier_chain_register\s*\([^,]+,\s*&?([\w\.\->]+)\s*\)")
                        .unwrap(),
                ],
                trigger_patterns: vec![],
            },
            // Softirq
            AsyncPattern {
                mechanism: AsyncMechanism::Softirq,
                context: ExecutionContext::SoftIrq,
                bind_patterns: vec![
                    Regex::new(r"open_softirq\s*\(\s*\w+\s*,\s*(\w+)\s*\)").unwrap()
                ],
                trigger_patterns: vec![
                    Regex::new(r"raise_softirq\s*\(").unwrap(),
                    Regex::new(r"raise_softirq_irqoff\s*\(").unwrap(),
                ],
            },
            // Completion (synchronization but often used with async)
            AsyncPattern {
                mechanism: AsyncMechanism::Custom("completion".to_string()),
                context: ExecutionContext::Process,
                bind_patterns: vec![
                    Regex::new(r"init_completion\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"DECLARE_COMPLETION\s*\(\s*(\w+)\s*\)").unwrap(),
                    Regex::new(r"reinit_completion\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                ],
                trigger_patterns: vec![
                    Regex::new(r"complete\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"complete_all\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                ],
            },
            // Wait queue
            AsyncPattern {
                mechanism: AsyncMechanism::Custom("waitqueue".to_string()),
                context: ExecutionContext::Process,
                bind_patterns: vec![
                    Regex::new(r"init_waitqueue_head\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"DECLARE_WAIT_QUEUE_HEAD\s*\(\s*(\w+)\s*\)").unwrap(),
                ],
                trigger_patterns: vec![
                    Regex::new(r"wake_up\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"wake_up_interruptible\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"wake_up_all\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                ],
            },
            // Deferred work (system_wq)
            AsyncPattern {
                mechanism: AsyncMechanism::WorkQueue { delayed: false },
                context: ExecutionContext::Process,
                bind_patterns: vec![Regex::new(
                    r"INIT_WORK_ONSTACK\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)",
                )
                .unwrap()],
                trigger_patterns: vec![
                    Regex::new(r"schedule_work_on\s*\([^,]+,\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"queue_work_on\s*\([^,]+,\s*[^,]+,\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"flush_work\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                ],
            },
            // IRQ work (runs in IRQ context but deferred)
            AsyncPattern {
                mechanism: AsyncMechanism::Custom("irq_work".to_string()),
                context: ExecutionContext::HardIrq,
                bind_patterns: vec![Regex::new(
                    r"init_irq_work\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)",
                )
                .unwrap()],
                trigger_patterns: vec![
                    Regex::new(r"irq_work_queue\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap()
                ],
            },
        ]
    }

    /// Analyze source code for async patterns
    pub fn analyze(
        &self,
        source: &str,
        functions: &HashMap<String, FunctionDef>,
    ) -> Vec<AsyncBinding> {
        let mut bindings = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        // Analyze with hardcoded patterns
        for pattern in &self.patterns {
            self.analyze_pattern(
                &lines,
                source,
                functions,
                &mut bindings,
                &pattern.mechanism,
                &pattern.context,
                &pattern.bind_patterns,
                &pattern.trigger_patterns,
            );
        }

        // Analyze with knowledge base patterns
        for kb_pattern in &self.kb_patterns {
            self.analyze_pattern(
                &lines,
                source,
                functions,
                &mut bindings,
                &kb_pattern.mechanism,
                &kb_pattern.context,
                &kb_pattern.bind_patterns,
                &kb_pattern.trigger_patterns,
            );
        }

        bindings
    }

    /// Analyze source with a specific pattern set
    fn analyze_pattern(
        &self,
        lines: &[&str],
        source: &str,
        functions: &HashMap<String, FunctionDef>,
        bindings: &mut Vec<AsyncBinding>,
        mechanism: &AsyncMechanism,
        context: &ExecutionContext,
        bind_patterns: &[Regex],
        trigger_patterns: &[Regex],
    ) {
        for bind_re in bind_patterns {
            for (line_num, line) in lines.iter().enumerate() {
                if let Some(caps) = bind_re.captures(line) {
                    // Extract handler name (usually last capture group)
                    let handler = caps
                        .get(caps.len() - 1)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();

                    // Extract variable (if present)
                    let variable = if caps.len() > 2 {
                        caps.get(1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };

                    if !handler.is_empty() && handler != "NULL" && functions.contains_key(&handler)
                    {
                        // Avoid duplicates
                        if bindings
                            .iter()
                            .any(|b| b.handler == handler && b.variable == variable)
                        {
                            continue;
                        }

                        // Find trigger locations
                        let trigger_locations =
                            self.find_triggers(source, trigger_patterns, &variable);

                        bindings.push(AsyncBinding {
                            mechanism: mechanism.clone(),
                            variable,
                            handler,
                            bind_location: Some(Location::new("", (line_num + 1) as u32, 0)),
                            trigger_locations,
                            context: context.clone(),
                        });
                    }
                }
            }
        }
    }

    fn find_triggers(&self, source: &str, patterns: &[Regex], variable: &str) -> Vec<Location> {
        let mut locations = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for trigger_re in patterns {
            for (line_num, line) in lines.iter().enumerate() {
                if let Some(caps) = trigger_re.captures(line) {
                    // Check if variable matches (if specified)
                    if !variable.is_empty() {
                        if let Some(var_match) = caps.get(1) {
                            if !Self::variables_match(variable, var_match.as_str()) {
                                continue;
                            }
                        }
                    }
                    locations.push(Location::new("", (line_num + 1) as u32, 0));
                }
            }
        }

        locations
    }

    fn variables_match(var1: &str, var2: &str) -> bool {
        // Normalize and compare variables
        let normalize = |s: &str| s.replace("&", "").replace("->", ".").replace(" ", "");
        normalize(var1) == normalize(var2)
    }
}

impl Default for AsyncTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_queue_detection() {
        let tracker = AsyncTracker::new();
        let source = r#"
static void my_work_handler(struct work_struct *work) {
    printk("work\n");
}

static int my_probe(void) {
    INIT_WORK(&dev->work, my_work_handler);
    schedule_work(&dev->work);
    return 0;
}
"#;

        let mut functions = HashMap::new();
        functions.insert(
            "my_work_handler".to_string(),
            FunctionDef {
                name: "my_work_handler".to_string(),
                return_type: "void".to_string(),
                params: vec![],
                location: None,
                calls: vec![],
                called_by: vec![],
                is_callback: false,
                callback_context: None,
                attributes: vec![],
            },
        );

        let bindings = tracker.analyze(source, &functions);
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].handler, "my_work_handler");
    }
}
