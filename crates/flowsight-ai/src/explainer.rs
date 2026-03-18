//! Business Explainer
//!
//! Provides business semantics explanations for functions and execution contexts.

use super::{AiConfig, AiTask, FlowSightAi};
use serde::{Deserialize, Serialize};

/// Business explanation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessExplanation {
    /// Trigger condition (when is this function called)
    pub trigger_condition: String,
    /// What the function does in business terms
    pub business_meaning: String,
    /// What happens when the function executes
    pub execution_result: String,
    /// Related functions
    pub related_functions: Vec<String>,
    /// Common errors
    pub common_errors: Vec<String>,
    /// Execution context annotation
    pub context_annotation: Option<ContextAnnotation>,
}

/// Execution context annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAnnotation {
    /// Process/SoftIRQ/HardIRQ
    pub context_type: String,
    /// Whether can sleep
    pub can_sleep: bool,
    /// Timing description
    pub timing: String,
    /// Notes for developers
    pub notes: Vec<String>,
}

/// Business explainer
#[derive(Debug)]
pub struct BusinessExplainer {
    /// AI instance
    ai: FlowSightAi,
}

impl BusinessExplainer {
    /// Create new explainer
    pub fn new(ai_config: AiConfig) -> Self {
        Self {
            ai: FlowSightAi::new(ai_config),
        }
    }

    /// Initialize the explainer
    pub async fn initialize(&mut self) -> Result<(), anyhow::Error> {
        self.ai.initialize().await?;
        Ok(())
    }

    /// Explain business semantics of a function
    pub async fn explain(
        &mut self,
        code: &str,
        function_name: &str,
        execution_context: &str,
    ) -> Result<BusinessExplanation, anyhow::Error> {
        let task = AiTask::ExplainBusiness {
            code: code.to_string(),
            function_name: function_name.to_string(),
            context: execution_context.to_string(),
        };

        let result = self.ai.infer(task).await?;
        self.parse_explanation_response(&result.response)
    }

    /// Parse AI response into BusinessExplanation
    fn parse_explanation_response(
        &self,
        response: &str,
    ) -> Result<BusinessExplanation, anyhow::Error> {
        // Try to extract JSON
        let json_start = response.find("{");
        let json_end = response.rfind("}");

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &response[start..=end];
            let parsed: serde_json::Value = serde_json::from_str(json_str)?;

            let related: Vec<String> = parsed["related_functions"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            let errors: Vec<String> = parsed["common_errors"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            return Ok(BusinessExplanation {
                trigger_condition: parsed["trigger_condition"]
                    .as_str()
                    .unwrap_or("未知")
                    .to_string(),
                business_meaning: parsed["business_meaning"]
                    .as_str()
                    .unwrap_or("未知")
                    .to_string(),
                execution_result: parsed["execution_result"]
                    .as_str()
                    .unwrap_or("未知")
                    .to_string(),
                related_functions: related,
                common_errors: errors,
                context_annotation: None,
            });
        }

        // Fallback
        Ok(BusinessExplanation {
            trigger_condition: "未知".to_string(),
            business_meaning: response.to_string(),
            execution_result: "".to_string(),
            related_functions: vec![],
            common_errors: vec![],
            context_annotation: None,
        })
    }
}

/// Execution context annotator
#[derive(Debug)]
pub struct ExecutionContextAnnotator {
    /// AI instance
    ai: FlowSightAi,
}

impl ExecutionContextAnnotator {
    /// Create new annotator
    pub fn new(ai_config: AiConfig) -> Self {
        Self {
            ai: FlowSightAi::new(ai_config),
        }
    }

    /// Initialize the annotator
    pub async fn initialize(&mut self) -> Result<(), anyhow::Error> {
        self.ai.initialize().await?;
        Ok(())
    }

    /// Annotate execution context
    pub async fn annotate(
        &mut self,
        mechanism: &str,
        handler_code: &str,
    ) -> Result<ContextAnnotation, anyhow::Error> {
        let task = AiTask::AnnotateContext {
            mechanism: mechanism.to_string(),
            handler_code: handler_code.to_string(),
        };

        let result = self.ai.infer(task).await?;
        self.parse_annotation_response(&result.response)
    }

    /// Parse annotation response
    fn parse_annotation_response(
        &self,
        response: &str,
    ) -> Result<ContextAnnotation, anyhow::Error> {
        // Simple parsing - look for keywords
        let context_type = if response.contains("进程") || response.contains("Process") {
            "进程上下文 (Process)"
        } else if response.contains("软中断") || response.contains("SoftIRQ") {
            "软中断上下文 (SoftIRQ)"
        } else if response.contains("硬中断") || response.contains("HardIRQ") {
            "硬中断上下文 (HardIRQ)"
        } else {
            "未知上下文"
        };

        let can_sleep = response.contains("可以睡眠")
            || response.contains("可睡眠")
            || response.contains("can sleep");

        // Extract notes (lines starting with - or *)
        let notes: Vec<String> = response
            .lines()
            .filter(|l| l.trim_start().starts_with('-') || l.trim_start().starts_with('*'))
            .map(|l| {
                l.trim_start()
                    .trim_start_matches('-')
                    .trim_start_matches('*')
                    .trim()
                    .to_string()
            })
            .collect();

        Ok(ContextAnnotation {
            context_type: context_type.to_string(),
            can_sleep,
            timing: "根据调度器决定".to_string(),
            notes,
        })
    }
}

/// Common function explanations
pub mod common_explanations {
    use super::BusinessExplanation;

    /// Explanation for USB probe
    pub fn usb_probe() -> BusinessExplanation {
        BusinessExplanation {
            trigger_condition: "USB 设备插入且设备 ID 与驱动匹配".to_string(),
            business_meaning: "当匹配的 USB 设备连接到系统时，驱动进行初始化配置".to_string(),
            execution_result: "分配设备结构体，初始化硬件，注册设备节点".to_string(),
            related_functions: vec![
                "usb_alloc_dev".into(),
                "usb_set_intfdata".into(),
                "device_create".into(),
            ],
            common_errors: vec![
                "内存分配失败 (kzalloc 返回 NULL)".into(),
                "硬件初始化超时".into(),
                "中断请求失败".into(),
            ],
            context_annotation: None,
        }
    }

    /// Explanation for interrupt handler
    pub fn irq_handler() -> super::ContextAnnotation {
        super::ContextAnnotation {
            context_type: "硬中断上下文 (HardIRQ)".to_string(),
            can_sleep: false,
            timing: "硬件中断触发时立即执行".to_string(),
            notes: vec![
                "禁止睡眠".into(),
                "尽快处理完成".into(),
                "不要调用可能阻塞的函数".into(),
                "使用 schedule_work() 延迟处理".into(),
            ],
        }
    }

    /// Explanation for workqueue handler
    pub fn workqueue_handler() -> super::ContextAnnotation {
        super::ContextAnnotation {
            context_type: "进程上下文 (Process)".to_string(),
            can_sleep: true,
            timing: "内核调��器选择合适的时机".to_string(),
            notes: vec![
                "可以睡眠".into(),
                "可以调用可能阻塞的函数".into(),
                "执行时间不应过长".into(),
            ],
        }
    }

    /// Explanation for timer callback
    pub fn timer_callback() -> super::ContextAnnotation {
        super::ContextAnnotation {
            context_type: "软中断上下文 (SoftIRQ)".to_string(),
            can_sleep: false,
            timing: "定时器到期时在软中断上下文中执行".to_string(),
            notes: vec![
                "不可睡眠".into(),
                "快速执行完成".into(),
                "注意与其他软中断的并发".into(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_explanations() {
        let probe = common_explanations::usb_probe();
        assert!(probe.trigger_condition.contains("USB"));
        assert!(!probe.related_functions.is_empty());

        let irq = common_explanations::irq_handler();
        assert!(!irq.can_sleep);
        assert!(irq.context_type.contains("硬中断"));
    }
}
