//! 知识库模式匹配器
//!
//! 用于在代码中识别异步机制和框架回调模式
//!
//! ## 支持的模式
//!
//! - WorkQueue: `INIT_WORK`, `schedule_work`, `queue_work`
//! - Timer: `timer_setup`, `mod_timer`, `add_timer`
//! - IRQ: `request_irq`, `request_threaded_irq`
//! - 框架回调: USB driver, file_operations 等

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模式匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// 匹配的模式类型
    pub pattern_type: PatternType,

    /// 匹配的原始文本
    pub matched_text: String,

    /// 匹配的起始位置（字节偏移）
    pub start: usize,

    /// 匹配的结束位置
    pub end: usize,

    /// 提取的变量名（如 work_struct 变量）
    pub variable: Option<String>,

    /// 提取的处理函数名
    pub handler: Option<String>,

    /// 置信度
    pub confidence: MatchConfidence,
}

/// 模式类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    // 异步机制绑定
    WorkQueueBind, // INIT_WORK, INIT_DELAYED_WORK
    TimerBind,     // timer_setup, setup_timer
    IrqBind,       // request_irq, request_threaded_irq
    TaskletBind,   // tasklet_setup

    // 异步机制触发
    WorkQueueTrigger, // schedule_work, queue_work
    TimerTrigger,     // mod_timer, add_timer

    // 框架注册
    UsbDriverRegister, // usb_register
    PlatformRegister,  // platform_driver_register
    CharDevRegister,   // register_chrdev, cdev_add

    // Ops 表赋值
    OpsAssignment, // .probe = xxx, .open = xxx

    // 其他
    Custom(String),
}

/// 匹配置信度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchConfidence {
    /// 精确匹配
    Certain,
    /// 高置信度
    High,
    /// 中等置信度
    Medium,
    /// 低置信度
    Low,
}

/// 模式匹配器
pub struct PatternMatcher {
    /// 异步绑定模式
    async_bind_patterns: Vec<CompiledPattern>,
    /// 异步触发模式
    async_trigger_patterns: Vec<CompiledPattern>,
    /// 框架注册模式
    framework_patterns: Vec<CompiledPattern>,
    /// Ops 表模式
    ops_patterns: Vec<CompiledPattern>,
}

/// 编译后的模式
struct CompiledPattern {
    /// 模式类型
    pattern_type: PatternType,
    /// 编译的正则表达式
    regex: Regex,
    /// 变量捕获组索引
    variable_group: Option<usize>,
    /// 处理函数捕获组索引
    handler_group: Option<usize>,
    /// 描述（保留用于调试）
    #[allow(dead_code)]
    description: String,
}

impl PatternMatcher {
    /// 创建新的模式匹配器
    pub fn new() -> Self {
        let mut matcher = Self {
            async_bind_patterns: Vec::new(),
            async_trigger_patterns: Vec::new(),
            framework_patterns: Vec::new(),
            ops_patterns: Vec::new(),
        };
        matcher.load_builtin_patterns();
        matcher
    }

    /// 加载内置模式
    fn load_builtin_patterns(&mut self) {
        // ========== WorkQueue 模式 ==========

        // INIT_WORK(&dev->work, handler)
        // INIT_WORK(&work, handler)
        self.async_bind_patterns.push(CompiledPattern {
            pattern_type: PatternType::WorkQueueBind,
            regex: Regex::new(r"INIT_WORK\s*\(\s*&?\s*([\w\.\->]+)\s*,\s*(\w+)\s*\)").unwrap(),
            variable_group: Some(1),
            handler_group: Some(2),
            description: "WorkQueue INIT_WORK".into(),
        });

        // INIT_DELAYED_WORK(&dev->dwork, handler)
        self.async_bind_patterns.push(CompiledPattern {
            pattern_type: PatternType::WorkQueueBind,
            regex: Regex::new(r"INIT_DELAYED_WORK\s*\(\s*&?\s*([\w\.\->]+)\s*,\s*(\w+)\s*\)")
                .unwrap(),
            variable_group: Some(1),
            handler_group: Some(2),
            description: "WorkQueue INIT_DELAYED_WORK".into(),
        });

        // schedule_work(&dev->work)
        self.async_trigger_patterns.push(CompiledPattern {
            pattern_type: PatternType::WorkQueueTrigger,
            regex: Regex::new(r"schedule_work\s*\(\s*&?\s*([\w\.\->]+)\s*\)").unwrap(),
            variable_group: Some(1),
            handler_group: None,
            description: "WorkQueue schedule_work".into(),
        });

        // queue_work(wq, &dev->work)
        self.async_trigger_patterns.push(CompiledPattern {
            pattern_type: PatternType::WorkQueueTrigger,
            regex: Regex::new(r"queue_work\s*\(\s*\w+\s*,\s*&?\s*([\w\.\->]+)\s*\)").unwrap(),
            variable_group: Some(1),
            handler_group: None,
            description: "WorkQueue queue_work".into(),
        });

        // queue_delayed_work(wq, &dev->dwork, delay)
        self.async_trigger_patterns.push(CompiledPattern {
            pattern_type: PatternType::WorkQueueTrigger,
            regex: Regex::new(r"queue_delayed_work\s*\(\s*\w+\s*,\s*&?\s*([\w\.\->]+)\s*,")
                .unwrap(),
            variable_group: Some(1),
            handler_group: None,
            description: "WorkQueue queue_delayed_work".into(),
        });

        // ========== Timer 模式 ==========

        // timer_setup(&dev->timer, handler, flags)
        self.async_bind_patterns.push(CompiledPattern {
            pattern_type: PatternType::TimerBind,
            regex: Regex::new(r"timer_setup\s*\(\s*&?\s*([\w\.\->]+)\s*,\s*(\w+)\s*,").unwrap(),
            variable_group: Some(1),
            handler_group: Some(2),
            description: "Timer timer_setup".into(),
        });

        // setup_timer(&timer, handler, data) - 旧 API
        self.async_bind_patterns.push(CompiledPattern {
            pattern_type: PatternType::TimerBind,
            regex: Regex::new(r"setup_timer\s*\(\s*&?\s*([\w\.\->]+)\s*,\s*(\w+)\s*,").unwrap(),
            variable_group: Some(1),
            handler_group: Some(2),
            description: "Timer setup_timer (legacy)".into(),
        });

        // mod_timer(&dev->timer, jiffies + HZ)
        self.async_trigger_patterns.push(CompiledPattern {
            pattern_type: PatternType::TimerTrigger,
            regex: Regex::new(r"mod_timer\s*\(\s*&?\s*([\w\.\->]+)\s*,").unwrap(),
            variable_group: Some(1),
            handler_group: None,
            description: "Timer mod_timer".into(),
        });

        // add_timer(&dev->timer)
        self.async_trigger_patterns.push(CompiledPattern {
            pattern_type: PatternType::TimerTrigger,
            regex: Regex::new(r"add_timer\s*\(\s*&?\s*([\w\.\->]+)\s*\)").unwrap(),
            variable_group: Some(1),
            handler_group: None,
            description: "Timer add_timer".into(),
        });

        // ========== IRQ 模式 ==========

        // request_irq(irq, handler, flags, name, dev)
        self.async_bind_patterns.push(CompiledPattern {
            pattern_type: PatternType::IrqBind,
            regex: Regex::new(r"request_irq\s*\(\s*[\w\.\->]+\s*,\s*(\w+)\s*,").unwrap(),
            variable_group: None,
            handler_group: Some(1),
            description: "IRQ request_irq".into(),
        });

        // request_threaded_irq(irq, handler, thread_fn, flags, name, dev)
        self.async_bind_patterns.push(CompiledPattern {
            pattern_type: PatternType::IrqBind,
            regex: Regex::new(
                r"request_threaded_irq\s*\(\s*[\w\.\->]+\s*,\s*(\w+)\s*,\s*(\w+)\s*,",
            )
            .unwrap(),
            variable_group: None,
            handler_group: Some(1), // 主 handler
            description: "IRQ request_threaded_irq".into(),
        });

        // ========== Tasklet 模式 ==========

        // tasklet_setup(&dev->tasklet, handler)
        self.async_bind_patterns.push(CompiledPattern {
            pattern_type: PatternType::TaskletBind,
            regex: Regex::new(r"tasklet_setup\s*\(\s*&?\s*([\w\.\->]+)\s*,\s*(\w+)\s*\)").unwrap(),
            variable_group: Some(1),
            handler_group: Some(2),
            description: "Tasklet tasklet_setup".into(),
        });

        // tasklet_init(&tasklet, handler, data) - 旧 API
        self.async_bind_patterns.push(CompiledPattern {
            pattern_type: PatternType::TaskletBind,
            regex: Regex::new(r"tasklet_init\s*\(\s*&?\s*([\w\.\->]+)\s*,\s*(\w+)\s*,").unwrap(),
            variable_group: Some(1),
            handler_group: Some(2),
            description: "Tasklet tasklet_init (legacy)".into(),
        });

        // ========== 框架注册模式 ==========

        // usb_register(&my_driver)
        self.framework_patterns.push(CompiledPattern {
            pattern_type: PatternType::UsbDriverRegister,
            regex: Regex::new(r"usb_register\s*\(\s*&?\s*(\w+)\s*\)").unwrap(),
            variable_group: Some(1),
            handler_group: None,
            description: "USB driver register".into(),
        });

        // platform_driver_register(&my_driver)
        self.framework_patterns.push(CompiledPattern {
            pattern_type: PatternType::PlatformRegister,
            regex: Regex::new(r"platform_driver_register\s*\(\s*&?\s*(\w+)\s*\)").unwrap(),
            variable_group: Some(1),
            handler_group: None,
            description: "Platform driver register".into(),
        });

        // ========== Ops 表赋值模式 ==========

        // .probe = my_probe,
        // .open = my_open,
        self.ops_patterns.push(CompiledPattern {
            pattern_type: PatternType::OpsAssignment,
            regex: Regex::new(r"\.(\w+)\s*=\s*(\w+)\s*[,}]").unwrap(),
            variable_group: Some(1), // 字段名
            handler_group: Some(2),  // 函数名
            description: "Ops table assignment".into(),
        });
    }

    /// 在代码中查找所有匹配
    pub fn find_all_matches(&self, code: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        // 匹配异步绑定
        for pattern in &self.async_bind_patterns {
            matches.extend(self.apply_pattern(code, pattern));
        }

        // 匹配异步触发
        for pattern in &self.async_trigger_patterns {
            matches.extend(self.apply_pattern(code, pattern));
        }

        // 匹配框架注册
        for pattern in &self.framework_patterns {
            matches.extend(self.apply_pattern(code, pattern));
        }

        // 匹配 Ops 表
        for pattern in &self.ops_patterns {
            matches.extend(self.apply_pattern(code, pattern));
        }

        matches
    }

    /// 查找异步绑定
    pub fn find_async_bindings(&self, code: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        for pattern in &self.async_bind_patterns {
            matches.extend(self.apply_pattern(code, pattern));
        }
        matches
    }

    /// 查找异步触发
    pub fn find_async_triggers(&self, code: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        for pattern in &self.async_trigger_patterns {
            matches.extend(self.apply_pattern(code, pattern));
        }
        matches
    }

    /// 查找 Ops 表赋值
    pub fn find_ops_assignments(&self, code: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        for pattern in &self.ops_patterns {
            matches.extend(self.apply_pattern(code, pattern));
        }
        matches
    }

    /// 关联绑定和触发
    ///
    /// 根据变量名将 INIT_WORK 和 schedule_work 关联起来
    pub fn correlate_bindings_and_triggers(
        &self,
        bindings: &[PatternMatch],
        triggers: &[PatternMatch],
    ) -> Vec<AsyncCorrelation> {
        let mut correlations = Vec::new();

        // 构建变量到绑定的映射
        let mut binding_map: HashMap<String, &PatternMatch> = HashMap::new();
        for binding in bindings {
            if let Some(ref var) = binding.variable {
                binding_map.insert(var.clone(), binding);
            }
        }

        // 查找触发对应的绑定
        for trigger in triggers {
            if let Some(ref var) = trigger.variable {
                if let Some(binding) = binding_map.get(var) {
                    correlations.push(AsyncCorrelation {
                        variable: var.clone(),
                        binding: (*binding).clone(),
                        triggers: vec![trigger.clone()],
                        handler: binding.handler.clone(),
                    });
                }
            }
        }

        correlations
    }

    /// 应用单个模式
    fn apply_pattern(&self, code: &str, pattern: &CompiledPattern) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for cap in pattern.regex.captures_iter(code) {
            let full_match = cap.get(0).unwrap();

            let variable = pattern
                .variable_group
                .and_then(|i| cap.get(i))
                .map(|m| m.as_str().to_string());

            let handler = pattern
                .handler_group
                .and_then(|i| cap.get(i))
                .map(|m| m.as_str().to_string());

            matches.push(PatternMatch {
                pattern_type: pattern.pattern_type.clone(),
                matched_text: full_match.as_str().to_string(),
                start: full_match.start(),
                end: full_match.end(),
                variable,
                handler,
                confidence: MatchConfidence::Certain,
            });
        }

        matches
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// 异步绑定-触发关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncCorrelation {
    /// 关联的变量名
    pub variable: String,
    /// 绑定匹配
    pub binding: PatternMatch,
    /// 触发匹配列表
    pub triggers: Vec<PatternMatch>,
    /// 处理函数名
    pub handler: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workqueue_init_work() {
        let matcher = PatternMatcher::new();
        let code = r#"
            INIT_WORK(&dev->work, my_work_handler);
            INIT_WORK(&priv_data.work, another_handler);
        "#;

        let matches = matcher.find_async_bindings(code);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].handler, Some("my_work_handler".into()));
        assert_eq!(matches[0].variable, Some("dev->work".into()));
        assert_eq!(matches[1].handler, Some("another_handler".into()));
    }

    #[test]
    fn test_workqueue_schedule() {
        let matcher = PatternMatcher::new();
        let code = r#"
            schedule_work(&dev->work);
            queue_work(system_wq, &priv->delayed_work);
        "#;

        let matches = matcher.find_async_triggers(code);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].variable, Some("dev->work".into()));
    }

    #[test]
    fn test_timer_setup() {
        let matcher = PatternMatcher::new();
        let code = r#"
            timer_setup(&dev->timer, my_timer_handler, 0);
        "#;

        let matches = matcher.find_async_bindings(code);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].handler, Some("my_timer_handler".into()));
        assert_eq!(matches[0].pattern_type, PatternType::TimerBind);
    }

    #[test]
    fn test_request_irq() {
        let matcher = PatternMatcher::new();
        let code = r#"
            request_irq(irq, my_irq_handler, IRQF_SHARED, "my_dev", dev);
        "#;

        let matches = matcher.find_async_bindings(code);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].handler, Some("my_irq_handler".into()));
        assert_eq!(matches[0].pattern_type, PatternType::IrqBind);
    }

    #[test]
    fn test_ops_assignment() {
        let matcher = PatternMatcher::new();
        let code = r#"
            static struct usb_driver my_driver = {
                .probe = my_probe,
                .disconnect = my_disconnect,
            };
        "#;

        let matches = matcher.find_ops_assignments(code);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].variable, Some("probe".into()));
        assert_eq!(matches[0].handler, Some("my_probe".into()));
    }

    #[test]
    fn test_correlate_bindings_and_triggers() {
        let matcher = PatternMatcher::new();
        let code = r#"
            INIT_WORK(&dev->work, my_handler);
            // ... later ...
            schedule_work(&dev->work);
        "#;

        let bindings = matcher.find_async_bindings(code);
        let triggers = matcher.find_async_triggers(code);
        let correlations = matcher.correlate_bindings_and_triggers(&bindings, &triggers);

        assert_eq!(correlations.len(), 1);
        assert_eq!(correlations[0].handler, Some("my_handler".into()));
        assert_eq!(correlations[0].variable, "dev->work");
    }
}
