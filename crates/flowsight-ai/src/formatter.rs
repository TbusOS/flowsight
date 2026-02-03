//! Flow Formatter Module
//!
//! 将分析结果转换为用户友好的格式:
//! - Mermaid 流程图
//! - 结构化表格 (ASCII/Markdown)
//! - 自然语言解释 (AI 生成)
//!
//! ## 使用示例
//!
//! ```ignore
//! let formatter = FlowFormatter::new();
//!
//! // 生成 Mermaid 图 (不需要 AI)
//! let mermaid = formatter.to_mermaid(&analysis_result);
//!
//! // 生成结构化表格 (不需要 AI)
//! let table = formatter.to_table(&analysis_result);
//!
//! // 生成自然语言解释 (需要 AI)
//! let explanation = formatter.explain(&analysis_result, &model).await;
//! ```

use flowsight_core::{
    AnalysisInfo, AsyncBinding, AsyncBoundary, AsyncMechanism, CallEdge, CallType, ConfidenceLevel,
    ExecutionContext, ExecutionFlow, FlowNode, FlowNodeType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{AiConfig, AiResult, FlowSightAi};

/// 格式化输出类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Mermaid 流程图
    Mermaid,
    /// ASCII 表格
    AsciiTable,
    /// Markdown 表格
    MarkdownTable,
    /// JSON (结构化数据)
    Json,
    /// 自然语言解释
    NaturalLanguage,
}

/// 执行流格式化器
pub struct FlowFormatter {
    /// 是否包含内核内部节点
    include_kernel_internal: bool,
    /// 最大显示深度
    max_depth: usize,
    /// 是否使用中文
    use_chinese: bool,
}

impl Default for FlowFormatter {
    fn default() -> Self {
        Self {
            include_kernel_internal: true,
            max_depth: 10,
            use_chinese: true,
        }
    }
}

impl FlowFormatter {
    /// 创建新的格式化器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否包含内核内部节点
    pub fn with_kernel_internal(mut self, include: bool) -> Self {
        self.include_kernel_internal = include;
        self
    }

    /// 设置最大显示深度
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// 设置语言
    pub fn with_chinese(mut self, chinese: bool) -> Self {
        self.use_chinese = chinese;
        self
    }

    // ========================================================================
    // Mermaid 图生成 (不需要 AI)
    // ========================================================================

    /// 将执行流转换为 Mermaid 流程图
    pub fn to_mermaid(&self, flow: &ExecutionFlow) -> String {
        let mut output = String::new();
        output.push_str("flowchart TB\n");

        // 收集所有节点和边
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut visited = HashSet::new();

        self.collect_mermaid_nodes(&flow.root, &mut nodes, &mut edges, &mut visited, 0);

        // 按执行上下文分组
        let mut process_nodes = Vec::new();
        let mut softirq_nodes = Vec::new();
        let mut hardirq_nodes = Vec::new();
        let mut other_nodes = Vec::new();

        for node in &nodes {
            match node.context {
                Some(ExecutionContext::Process) => process_nodes.push(node),
                Some(ExecutionContext::SoftIrq) => softirq_nodes.push(node),
                Some(ExecutionContext::HardIrq) => hardirq_nodes.push(node),
                _ => other_nodes.push(node),
            }
        }

        // 生成子图
        if !process_nodes.is_empty() {
            output.push_str("    subgraph process[\"📦 进程上下文 (可睡眠)\"]\n");
            for node in &process_nodes {
                output.push_str(&format!("        {}[\"{}\"]\n", node.id, node.label));
            }
            output.push_str("    end\n");
        }

        if !softirq_nodes.is_empty() {
            output.push_str("    subgraph softirq[\"⚡ 软中断上下文\"]\n");
            for node in &softirq_nodes {
                output.push_str(&format!("        {}[\"{}\"]\n", node.id, node.label));
            }
            output.push_str("    end\n");
        }

        if !hardirq_nodes.is_empty() {
            output.push_str("    subgraph hardirq[\"🔴 硬中断上下文 (不可睡眠!)\"]\n");
            for node in &hardirq_nodes {
                output.push_str(&format!("        {}[\"{}\"]\n", node.id, node.label));
            }
            output.push_str("    end\n");
        }

        for node in &other_nodes {
            output.push_str(&format!("    {}[\"{}\"]\n", node.id, node.label));
        }

        // 生成边
        output.push_str("\n");
        for edge in &edges {
            let arrow = match &edge.edge_type {
                EdgeType::Direct => "-->",
                EdgeType::Async(_) => "-.->",
                EdgeType::Indirect => "-->|间接|",
            };

            let label = match &edge.edge_type {
                EdgeType::Async(mechanism) => {
                    format!("|{}|", self.mechanism_to_label(mechanism))
                }
                _ => String::new(),
            };

            output.push_str(&format!(
                "    {} {}{} {}\n",
                edge.from, arrow, label, edge.to
            ));
        }

        // 添加样式
        output.push_str("\n");
        output.push_str("    style process fill:#cce5ff\n");
        output.push_str("    style softirq fill:#fff3cd\n");
        output.push_str("    style hardirq fill:#f8d7da\n");

        output
    }

    /// 从节点递归收集 Mermaid 图数据
    fn collect_mermaid_nodes(
        &self,
        node: &FlowNode,
        nodes: &mut Vec<MermaidNode>,
        edges: &mut Vec<MermaidEdge>,
        visited: &mut HashSet<String>,
        depth: usize,
    ) {
        if depth > self.max_depth {
            return;
        }

        if visited.contains(&node.id) {
            return;
        }
        visited.insert(node.id.clone());

        // 跳过内核内部节点（如果配置）
        if !self.include_kernel_internal && node.is_kernel_internal {
            // 但仍然处理子节点
            for child in &node.children {
                self.collect_mermaid_nodes(child, nodes, edges, visited, depth + 1);
            }
            return;
        }

        // 创建节点
        let icon = self.node_type_icon(&node.node_type);
        let label = format!("{} {}", icon, node.display_name);

        nodes.push(MermaidNode {
            id: node.id.clone(),
            label,
            context: node.execution_context.clone(),
        });

        // 创建边
        for child in &node.children {
            let edge_type = match &child.node_type {
                FlowNodeType::AsyncCallback { mechanism } => EdgeType::Async(mechanism.clone()),
                _ => {
                    if child.confidence.as_ref().map(|c| c.level) == Some(ConfidenceLevel::Possible)
                    {
                        EdgeType::Indirect
                    } else {
                        EdgeType::Direct
                    }
                }
            };

            edges.push(MermaidEdge {
                from: node.id.clone(),
                to: child.id.clone(),
                edge_type,
            });

            self.collect_mermaid_nodes(child, nodes, edges, visited, depth + 1);
        }
    }

    fn node_type_icon(&self, node_type: &FlowNodeType) -> &'static str {
        match node_type {
            FlowNodeType::EntryPoint => "📍",
            FlowNodeType::Function => "📦",
            FlowNodeType::AsyncCallback { mechanism } => match mechanism {
                AsyncMechanism::WorkQueue { .. } => "⚙️",
                AsyncMechanism::Timer { .. } => "⏰",
                AsyncMechanism::Interrupt { .. } => "⚡",
                AsyncMechanism::Tasklet => "📋",
                AsyncMechanism::Softirq => "🔔",
                AsyncMechanism::KThread => "🧵",
                AsyncMechanism::RcuCallback => "🔄",
                _ => "🔗",
            },
            FlowNodeType::KernelApi => "🔧",
            FlowNodeType::External => "📤",
            FlowNodeType::Separator { .. } => "───",
            FlowNodeType::Branch { .. } => "❓",
        }
    }

    fn mechanism_to_label(&self, mechanism: &AsyncMechanism) -> &'static str {
        match mechanism {
            AsyncMechanism::WorkQueue { delayed: true } => "delayed_work",
            AsyncMechanism::WorkQueue { .. } => "schedule_work",
            AsyncMechanism::Timer {
                high_resolution: true,
            } => "hrtimer",
            AsyncMechanism::Timer { .. } => "mod_timer",
            AsyncMechanism::Interrupt { threaded: true } => "threaded_irq",
            AsyncMechanism::Interrupt { .. } => "IRQ",
            AsyncMechanism::Tasklet => "tasklet",
            AsyncMechanism::Softirq => "softirq",
            AsyncMechanism::KThread => "kthread",
            AsyncMechanism::RcuCallback => "call_rcu",
            AsyncMechanism::Notifier => "notifier",
            AsyncMechanism::Custom(name) => {
                // 返回 static str 需要 leak，这里简化处理
                "custom"
            }
        }
    }

    // ========================================================================
    // 结构化表格生成 (不需要 AI)
    // ========================================================================

    /// 生成 Markdown 格式的执行流表格
    pub fn to_markdown_table(&self, flow: &ExecutionFlow) -> String {
        let mut output = String::new();

        // 标题
        output.push_str(&format!("# 执行流分析: {}\n\n", flow.entry_function));

        // 基本信息
        output.push_str("## 基本信息\n\n");
        output.push_str("| 属性 | 值 |\n");
        output.push_str("|------|----|\n");
        output.push_str(&format!("| 入口函数 | `{}` |\n", flow.entry_function));
        if let Some(loc) = &flow.entry_location {
            output.push_str(&format!("| 位置 | {}:{} |\n", loc.file, loc.line));
        }
        output.push_str(&format!(
            "| 分析时间 | {} |\n",
            flow.analysis_info.analyzed_at.format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!(
            "| 节点总数 | {} |\n",
            flow.analysis_info.total_nodes
        ));
        output.push_str("\n");

        // 执行流表格
        output.push_str("## 执行流\n\n");
        output.push_str("| 层级 | 函数 | 上下文 | 类型 | 说明 |\n");
        output.push_str("|------|------|--------|------|------|\n");

        self.append_flow_rows(&flow.root, &mut output, 0);

        // 异步边界
        if !flow.async_boundaries.is_empty() {
            output.push_str("\n## 异步边界\n\n");
            output.push_str("| 机制 | 触发点 | 处理函数 | 上下文 |\n");
            output.push_str("|------|--------|----------|--------|\n");

            for boundary in &flow.async_boundaries {
                let mechanism = format!("{:?}", boundary.mechanism);
                let trigger = &boundary.trigger_call;
                let handler = &boundary.handler_function;
                let context = &boundary.context_description;
                output.push_str(&format!(
                    "| {} | `{}` | `{}` | {} |\n",
                    mechanism, trigger, handler, context
                ));
            }
        }

        // 警告
        if !flow.analysis_info.warnings.is_empty() {
            output.push_str("\n## 分析警告\n\n");
            for warning in &flow.analysis_info.warnings {
                output.push_str(&format!("- ⚠️ {}\n", warning.message));
            }
        }

        output
    }

    fn append_flow_rows(&self, node: &FlowNode, output: &mut String, depth: usize) {
        if depth > self.max_depth {
            return;
        }

        let indent = "  ".repeat(depth);
        let icon = self.node_type_icon(&node.node_type);

        let context_str = match &node.execution_context {
            Some(ExecutionContext::Process) => "process",
            Some(ExecutionContext::SoftIrq) => "softirq",
            Some(ExecutionContext::HardIrq) => "hardirq",
            _ => "-",
        };

        let type_str = match &node.node_type {
            FlowNodeType::EntryPoint => "入口",
            FlowNodeType::Function => "函数",
            FlowNodeType::AsyncCallback { .. } => "异步回调",
            FlowNodeType::KernelApi => "内核API",
            FlowNodeType::External => "外部",
            FlowNodeType::Separator { .. } => "分隔",
            FlowNodeType::Branch { .. } => "分支",
        };

        let desc = node.description.as_deref().unwrap_or("-");

        output.push_str(&format!(
            "| {} | {}`{}` | {} | {} | {} |\n",
            depth, indent, node.display_name, context_str, type_str, desc
        ));

        for child in &node.children {
            self.append_flow_rows(child, output, depth + 1);
        }
    }

    /// 生成 ASCII 格式的执行流树
    pub fn to_ascii_tree(&self, flow: &ExecutionFlow) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "╔══════════════════════════════════════════════════════════════╗\n"
        ));
        output.push_str(&format!("║  执行流分析: {:<47} ║\n", flow.entry_function));
        output.push_str(&format!(
            "╠══════════════════════════════════════════════════════════════╣\n"
        ));

        self.append_ascii_node(&flow.root, &mut output, "", true, 0);

        output.push_str(&format!(
            "╚══════════════════════════════════════════════════════════════╝\n"
        ));

        output
    }

    fn append_ascii_node(
        &self,
        node: &FlowNode,
        output: &mut String,
        prefix: &str,
        is_last: bool,
        depth: usize,
    ) {
        if depth > self.max_depth {
            return;
        }

        let connector = if is_last { "└── " } else { "├── " };
        let icon = self.node_type_icon(&node.node_type);

        let context_badge = match &node.execution_context {
            Some(ExecutionContext::Process) => "[P]",
            Some(ExecutionContext::SoftIrq) => "[S]",
            Some(ExecutionContext::HardIrq) => "[H]",
            _ => "",
        };

        output.push_str(&format!(
            "║ {}{}{} {} {}\n",
            prefix, connector, icon, node.display_name, context_badge
        ));

        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            self.append_ascii_node(child, output, &new_prefix, is_last_child, depth + 1);
        }
    }

    // ========================================================================
    // AI 辅助的自然语言生成
    // ========================================================================

    /// 使用 AI 生成执行流的自然语言解释
    pub async fn explain(
        &self,
        flow: &ExecutionFlow,
        ai: &mut FlowSightAi,
    ) -> Result<String, anyhow::Error> {
        // 构建 prompt
        let prompt = self.build_explanation_prompt(flow);

        // 调用 AI
        let task = crate::AiTask::AnswerQuestion {
            question: "请用中文解释以下执行流分析结果".to_string(),
            code: prompt,
        };

        let result = ai.infer(task).await?;
        Ok(result.response)
    }

    fn build_explanation_prompt(&self, flow: &ExecutionFlow) -> String {
        let mut prompt = String::new();

        prompt.push_str("执行流分析数据:\n\n");
        prompt.push_str(&format!("入口函数: {}\n", flow.entry_function));

        prompt.push_str("\n调用链:\n");
        self.append_call_chain(&flow.root, &mut prompt, 0);

        if !flow.async_boundaries.is_empty() {
            prompt.push_str("\n异步机制:\n");
            for boundary in &flow.async_boundaries {
                prompt.push_str(&format!(
                    "- {:?}: {} → {}\n",
                    boundary.mechanism, boundary.trigger_call, boundary.handler_function
                ));
            }
        }

        prompt.push_str("\n请解释:\n");
        prompt.push_str("1. 这段代码的主要功能是什么\n");
        prompt.push_str("2. 执行流程是怎样的\n");
        prompt.push_str("3. 异步机制的作用\n");
        prompt.push_str("4. 需要注意的地方\n");

        prompt
    }

    fn append_call_chain(&self, node: &FlowNode, output: &mut String, depth: usize) {
        if depth > 5 {
            // 限制深度避免 prompt 过长
            return;
        }

        let indent = "  ".repeat(depth);
        let context = match &node.execution_context {
            Some(ExecutionContext::Process) => "(process)",
            Some(ExecutionContext::SoftIrq) => "(softirq)",
            Some(ExecutionContext::HardIrq) => "(hardirq)",
            _ => "",
        };

        output.push_str(&format!("{}- {} {}\n", indent, node.display_name, context));

        for child in &node.children {
            self.append_call_chain(child, output, depth + 1);
        }
    }

    // ========================================================================
    // 结构化 JSON 输出
    // ========================================================================

    /// 生成用于前端展示的 JSON 数据
    pub fn to_display_json(&self, flow: &ExecutionFlow) -> DisplayFlowData {
        DisplayFlowData {
            entry_function: flow.entry_function.clone(),
            summary: self.generate_summary(flow),
            mermaid_diagram: self.to_mermaid(flow),
            nodes: self.collect_display_nodes(&flow.root, 0),
            async_patterns: flow
                .async_boundaries
                .iter()
                .map(|b| DisplayAsyncPattern {
                    mechanism: format!("{:?}", b.mechanism),
                    trigger: b.trigger_call.clone(),
                    handler: b.handler_function.clone(),
                    description: b.context_description.clone(),
                })
                .collect(),
            stats: DisplayStats {
                total_nodes: flow.analysis_info.total_nodes,
                direct_calls: flow.analysis_info.direct_calls,
                indirect_calls: flow.analysis_info.indirect_calls,
                async_calls: flow.analysis_info.async_calls,
            },
        }
    }

    fn generate_summary(&self, flow: &ExecutionFlow) -> String {
        let mut summary = String::new();

        // 分析入口类型
        let entry_type = if flow.entry_function.contains("probe") {
            "设备探测函数"
        } else if flow.entry_function.contains("init") {
            "模块初始化函数"
        } else if flow.entry_function.contains("irq") || flow.entry_function.contains("handler") {
            "中断处理函数"
        } else {
            "普通函数"
        };

        summary.push_str(&format!("`{}` 是一个{}", flow.entry_function, entry_type));

        // 添加异步模式说明
        if !flow.async_boundaries.is_empty() {
            let mechanisms: Vec<String> = flow
                .async_boundaries
                .iter()
                .map(|b| format!("{:?}", b.mechanism))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            summary.push_str(&format!("，使用了 {} 异步机制", mechanisms.join("、")));
        }

        summary.push_str("。");
        summary
    }

    fn collect_display_nodes(&self, node: &FlowNode, depth: usize) -> Vec<DisplayNode> {
        let mut nodes = Vec::new();

        if depth > self.max_depth {
            return nodes;
        }

        nodes.push(DisplayNode {
            id: node.id.clone(),
            name: node.name.clone(),
            display_name: node.display_name.clone(),
            node_type: format!("{:?}", node.node_type),
            context: node.execution_context.as_ref().map(|c| format!("{:?}", c)),
            can_sleep: node.can_sleep,
            description: node.description.clone(),
            depth,
            children_count: node.children.len(),
        });

        for child in &node.children {
            nodes.extend(self.collect_display_nodes(child, depth + 1));
        }

        nodes
    }
}

// ============================================================================
// 辅助数据结构
// ============================================================================

#[derive(Debug)]
struct MermaidNode {
    id: String,
    label: String,
    context: Option<ExecutionContext>,
}

#[derive(Debug)]
struct MermaidEdge {
    from: String,
    to: String,
    edge_type: EdgeType,
}

#[derive(Debug)]
enum EdgeType {
    Direct,
    Async(AsyncMechanism),
    Indirect,
}

/// 前端展示用的执行流数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayFlowData {
    /// 入口函数名
    pub entry_function: String,
    /// 摘要说明
    pub summary: String,
    /// Mermaid 图代码
    pub mermaid_diagram: String,
    /// 扁平化的节点列表（便于表格展示）
    pub nodes: Vec<DisplayNode>,
    /// 异步模式列表
    pub async_patterns: Vec<DisplayAsyncPattern>,
    /// 统计信息
    pub stats: DisplayStats,
}

/// 展示用节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayNode {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub node_type: String,
    pub context: Option<String>,
    pub can_sleep: Option<bool>,
    pub description: Option<String>,
    pub depth: usize,
    pub children_count: usize,
}

/// 展示用异步模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayAsyncPattern {
    pub mechanism: String,
    pub trigger: String,
    pub handler: String,
    pub description: String,
}

/// 展示用统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayStats {
    pub total_nodes: usize,
    pub direct_calls: usize,
    pub indirect_calls: usize,
    pub async_calls: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_flow() -> ExecutionFlow {
        let root = FlowNode {
            id: "1".to_string(),
            name: "my_probe".to_string(),
            display_name: "my_probe".to_string(),
            location: None,
            node_type: FlowNodeType::EntryPoint,
            children: vec![
                FlowNode {
                    id: "2".to_string(),
                    name: "devm_kzalloc".to_string(),
                    display_name: "devm_kzalloc".to_string(),
                    location: None,
                    node_type: FlowNodeType::KernelApi,
                    children: vec![],
                    description: Some("分配设备内存".to_string()),
                    confidence: None,
                    execution_context: Some(ExecutionContext::Process),
                    can_sleep: Some(true),
                    source_file: None,
                    is_kernel_internal: false,
                },
                FlowNode {
                    id: "3".to_string(),
                    name: "work_handler".to_string(),
                    display_name: "work_handler".to_string(),
                    location: None,
                    node_type: FlowNodeType::AsyncCallback {
                        mechanism: AsyncMechanism::WorkQueue { delayed: false },
                    },
                    children: vec![],
                    description: Some("工作队列处理函数".to_string()),
                    confidence: None,
                    execution_context: Some(ExecutionContext::Process),
                    can_sleep: Some(true),
                    source_file: None,
                    is_kernel_internal: false,
                },
            ],
            description: Some("设备探测函数".to_string()),
            confidence: None,
            execution_context: Some(ExecutionContext::Process),
            can_sleep: Some(true),
            source_file: None,
            is_kernel_internal: false,
        };

        ExecutionFlow::new("my_probe".to_string(), root)
    }

    #[test]
    fn test_to_mermaid() {
        let flow = create_test_flow();
        let formatter = FlowFormatter::new();
        let mermaid = formatter.to_mermaid(&flow);

        assert!(mermaid.contains("flowchart TB"));
        assert!(mermaid.contains("my_probe"));
        assert!(mermaid.contains("work_handler"));
    }

    #[test]
    fn test_to_markdown_table() {
        let flow = create_test_flow();
        let formatter = FlowFormatter::new();
        let table = formatter.to_markdown_table(&flow);

        assert!(table.contains("# 执行流分析"));
        assert!(table.contains("my_probe"));
        assert!(table.contains("| 层级 |"));
    }

    #[test]
    fn test_to_ascii_tree() {
        let flow = create_test_flow();
        let formatter = FlowFormatter::new();
        let tree = formatter.to_ascii_tree(&flow);

        assert!(tree.contains("my_probe"));
        assert!(tree.contains("├──") || tree.contains("└──"));
    }

    #[test]
    fn test_to_display_json() {
        let flow = create_test_flow();
        let formatter = FlowFormatter::new();
        let display = formatter.to_display_json(&flow);

        assert_eq!(display.entry_function, "my_probe");
        assert!(!display.summary.is_empty());
        assert!(!display.mermaid_diagram.is_empty());
        assert!(!display.nodes.is_empty());
    }
}
