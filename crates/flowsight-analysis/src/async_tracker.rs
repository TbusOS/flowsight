//! Async mechanism tracker
//!
//! Tracks async patterns like:
//! - Work queues (INIT_WORK, schedule_work)
//! - Timers (timer_setup, mod_timer)
//! - Interrupts (request_irq)
//! - Tasklets (tasklet_init)
//! - Kernel threads (kthread_run)

use flowsight_core::{AsyncBinding, AsyncMechanism, ExecutionContext, Location, FunctionDef};
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
}

impl AsyncTracker {
    /// Create a new async tracker with default patterns
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    fn default_patterns() -> Vec<AsyncPattern> {
        vec![
            // Work queue
            AsyncPattern {
                mechanism: AsyncMechanism::WorkQueue { delayed: false },
                context: ExecutionContext::Process,
                bind_patterns: vec![
                    Regex::new(r"INIT_WORK\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)").unwrap(),
                ],
                trigger_patterns: vec![
                    Regex::new(r"schedule_work\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"queue_work\s*\([^,]+,\s*&?([\w\.\->]+)\s*\)").unwrap(),
                ],
            },
            // Delayed work
            AsyncPattern {
                mechanism: AsyncMechanism::WorkQueue { delayed: true },
                context: ExecutionContext::Process,
                bind_patterns: vec![
                    Regex::new(r"INIT_DELAYED_WORK\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)").unwrap(),
                ],
                trigger_patterns: vec![
                    Regex::new(r"schedule_delayed_work\s*\(\s*&?([\w\.\->]+)\s*,").unwrap(),
                ],
            },
            // Timer
            AsyncPattern {
                mechanism: AsyncMechanism::Timer { high_resolution: false },
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
                mechanism: AsyncMechanism::Timer { high_resolution: true },
                context: ExecutionContext::HardIrq,
                bind_patterns: vec![
                    Regex::new(r"([\w\.\->]+)\.function\s*=\s*(\w+)").unwrap(),
                ],
                trigger_patterns: vec![
                    Regex::new(r"hrtimer_start\s*\(\s*&?([\w\.\->]+)\s*,").unwrap(),
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
                bind_patterns: vec![
                    Regex::new(r"request_threaded_irq\s*\([^,]+,\s*\w+\s*,\s*(\w+)\s*,").unwrap(),
                ],
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
                    Regex::new(r"tasklet_schedule\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
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
                trigger_patterns: vec![
                    Regex::new(r"wake_up_process\s*\(").unwrap(),
                ],
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
                    Regex::new(r"blocking_notifier_chain_register\s*\([^,]+,\s*&?([\w\.\->]+)\s*\)").unwrap(),
                    Regex::new(r"atomic_notifier_chain_register\s*\([^,]+,\s*&?([\w\.\->]+)\s*\)").unwrap(),
                ],
                trigger_patterns: vec![],
            },
            // Softirq
            AsyncPattern {
                mechanism: AsyncMechanism::Softirq,
                context: ExecutionContext::SoftIrq,
                bind_patterns: vec![
                    Regex::new(r"open_softirq\s*\(\s*\w+\s*,\s*(\w+)\s*\)").unwrap(),
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
                bind_patterns: vec![
                    Regex::new(r"INIT_WORK_ONSTACK\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)").unwrap(),
                ],
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
                bind_patterns: vec![
                    Regex::new(r"init_irq_work\s*\(\s*&?([\w\.\->]+)\s*,\s*(\w+)\s*\)").unwrap(),
                ],
                trigger_patterns: vec![
                    Regex::new(r"irq_work_queue\s*\(\s*&?([\w\.\->]+)\s*\)").unwrap(),
                ],
            },
        ]
    }

    /// Analyze source code for async patterns
    pub fn analyze(&self, source: &str, functions: &HashMap<String, FunctionDef>) -> Vec<AsyncBinding> {
        let mut bindings = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for pattern in &self.patterns {
            for bind_re in &pattern.bind_patterns {
                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(caps) = bind_re.captures(line) {
                        // Extract handler name (usually last capture group)
                        let handler = caps.get(caps.len() - 1)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();

                        // Extract variable (if present)
                        let variable = if caps.len() > 2 {
                            caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default()
                        } else {
                            String::new()
                        };

                        if !handler.is_empty() && handler != "NULL" && functions.contains_key(&handler) {
                            // Find trigger locations
                            let trigger_locations = self.find_triggers(source, &pattern.trigger_patterns, &variable);

                            bindings.push(AsyncBinding {
                                mechanism: pattern.mechanism.clone(),
                                variable,
                                handler,
                                bind_location: Some(Location::new("", (line_num + 1) as u32, 0)),
                                trigger_locations,
                                context: pattern.context.clone(),
                            });
                        }
                    }
                }
            }
        }

        bindings
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
        functions.insert("my_work_handler".to_string(), FunctionDef {
            name: "my_work_handler".to_string(),
            return_type: "void".to_string(),
            params: vec![],
            location: None,
            calls: vec![],
            called_by: vec![],
            is_callback: false,
            callback_context: None,
            attributes: vec![],
        });

        let bindings = tracker.analyze(source, &functions);
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].handler, "my_work_handler");
    }
}

