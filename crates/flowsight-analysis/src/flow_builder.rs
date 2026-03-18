//! 执行流构建器
//!
//! 从代码分析结果构建完整的 ExecutionFlow
//!
//! ## 构建流程
//!
//! 1. 解析入口函数
//! 2. 递归分析调用关系
//! 3. 识别异步边界
//! 4. 注入知识库调用链
//! 5. 构建完整执行流

use chrono::Utc;
use flowsight_core::location::Location;
use flowsight_core::types::{
    AnalysisInfo, AnalysisWarning, AsyncBoundary, AsyncMechanism, CallConfidence, ConfidenceLevel,
    ExecutionContext, ExecutionFlow, FlowNode, FlowNodeType, WarningKind,
};
use flowsight_knowledge::{
    matcher::{PatternMatcher, PatternType},
    KnowledgeBase,
};
use std::collections::{HashMap, HashSet};

/// 执行流构建器
pub struct FlowBuilder {
    /// 知识库
    knowledge_base: KnowledgeBase,
    /// 模式匹配器
    pattern_matcher: PatternMatcher,
    /// 最大递归深度
    max_depth: usize,
    /// 已访问的函数（防止循环）
    visited: HashSet<String>,
    /// 函数定义缓存
    function_defs: HashMap<String, FunctionInfo>,
    /// 异步绑定信息
    async_bindings: HashMap<String, AsyncBindingInfo>,
}

/// 函数信息（简化版）
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// 函数名
    pub name: String,
    /// 位置
    pub location: Option<Location>,
    /// 调用的函数列表
    pub calls: Vec<String>,
    /// 是否是内核函数
    pub is_kernel: bool,
}

/// 异步绑定信息
#[derive(Debug, Clone)]
struct AsyncBindingInfo {
    /// 变量名
    variable: String,
    /// 处理函数名
    handler: String,
    /// 机制类型
    mechanism: AsyncMechanism,
    /// 绑定位置
    _bind_location: Option<Location>,
}

/// 构建选项
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// 最大深度
    pub max_depth: usize,
    /// 是否包含内核调用链
    pub include_kernel_chains: bool,
    /// 是否展开异步回调
    pub expand_async: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            max_depth: 50,
            include_kernel_chains: true,
            expand_async: true,
        }
    }
}

impl FlowBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            knowledge_base: KnowledgeBase::builtin(),
            pattern_matcher: PatternMatcher::new(),
            max_depth: 50,
            visited: HashSet::new(),
            function_defs: HashMap::new(),
            async_bindings: HashMap::new(),
        }
    }

    /// 使用自定义知识库
    pub fn with_knowledge_base(mut self, kb: KnowledgeBase) -> Self {
        self.knowledge_base = kb;
        self
    }

    /// 设置最大深度
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// 注册函数定义
    pub fn register_function(&mut self, func: FunctionInfo) {
        self.function_defs.insert(func.name.clone(), func);
    }

    /// 从代码中提取异步绑定
    pub fn extract_async_bindings(&mut self, code: &str) {
        let bindings = self.pattern_matcher.find_async_bindings(code);

        for binding in bindings {
            if let (Some(var), Some(handler)) = (&binding.variable, &binding.handler) {
                let mechanism = match binding.pattern_type {
                    PatternType::WorkQueueBind => AsyncMechanism::WorkQueue { delayed: false },
                    PatternType::TimerBind => AsyncMechanism::Timer {
                        high_resolution: false,
                    },
                    PatternType::IrqBind => AsyncMechanism::Interrupt { threaded: false },
                    PatternType::TaskletBind => AsyncMechanism::Tasklet,
                    _ => continue,
                };

                self.async_bindings.insert(
                    var.clone(),
                    AsyncBindingInfo {
                        variable: var.clone(),
                        handler: handler.clone(),
                        mechanism,
                        _bind_location: None,
                    },
                );
            }
        }
    }

    /// 构建执行流
    pub fn build(&mut self, entry_function: &str, options: &BuildOptions) -> ExecutionFlow {
        self.visited.clear();
        self.max_depth = options.max_depth;

        let mut warnings = Vec::new();
        let mut async_boundaries = Vec::new();

        // 构建根节点
        let root = self.build_node(
            entry_function,
            0,
            options,
            &mut warnings,
            &mut async_boundaries,
        );

        // 统计信息
        let total_nodes = count_nodes(&root);
        let (direct_calls, indirect_calls, async_calls) = count_call_types(&root);

        ExecutionFlow {
            entry_function: entry_function.to_string(),
            entry_location: self
                .function_defs
                .get(entry_function)
                .and_then(|f| f.location.clone()),
            root,
            async_boundaries,
            analysis_info: AnalysisInfo {
                analyzed_at: Utc::now(),
                source_file: None,
                knowledge_version: Some("v3.0".into()),
                total_nodes,
                direct_calls,
                indirect_calls,
                async_calls,
                warnings,
            },
        }
    }

    /// 递归构建节点
    fn build_node(
        &mut self,
        func_name: &str,
        depth: usize,
        options: &BuildOptions,
        warnings: &mut Vec<AnalysisWarning>,
        async_boundaries: &mut Vec<AsyncBoundary>,
    ) -> FlowNode {
        // 检查深度限制
        if depth >= self.max_depth {
            warnings.push(AnalysisWarning {
                kind: WarningKind::DepthLimitReached,
                message: format!("Depth limit reached at function: {}", func_name),
                location: None,
            });
            return self.create_leaf_node(func_name, "... depth limit reached");
        }

        // 检查循环
        if self.visited.contains(func_name) {
            warnings.push(AnalysisWarning {
                kind: WarningKind::RecursionDetected,
                message: format!("Recursion detected: {}", func_name),
                location: None,
            });
            return self.create_leaf_node(func_name, "... (recursive)");
        }

        self.visited.insert(func_name.to_string());

        // 获取函数信息
        let func_info = self.function_defs.get(func_name).cloned();

        // 确定节点类型
        let (node_type, is_kernel) = self.determine_node_type(func_name);

        // 构建子节点
        let mut children = Vec::new();

        if let Some(ref info) = func_info {
            for callee in &info.calls {
                // 检查是否是异步触发
                if let Some(async_info) = self.check_async_trigger(callee) {
                    // 添加异步边界
                    let boundary_id = format!("async-{}-{}", async_info.variable, depth);
                    async_boundaries.push(AsyncBoundary {
                        id: boundary_id.clone(),
                        mechanism: async_info.mechanism.clone(),
                        trigger_call: callee.clone(),
                        trigger_location: None,
                        handler_function: async_info.handler.clone(),
                        handler_node_id: None,
                        context_description: self.get_context_description(&async_info.mechanism),
                    });

                    if options.expand_async {
                        // 添加分隔符
                        children.push(FlowNode {
                            id: format!("{}-sep", boundary_id),
                            name: "... async boundary ...".into(),
                            display_name: format!(
                                "↓ {} 异步执行",
                                self.get_mechanism_name(&async_info.mechanism)
                            ),
                            location: None,
                            node_type: FlowNodeType::Separator {
                                text: format!(
                                    "稍后由 {} 执行",
                                    self.get_mechanism_name(&async_info.mechanism)
                                ),
                            },
                            children: Vec::new(),
                            description: Some("异步边界：以下代码在不同上下文执行".into()),
                            confidence: None,
                            execution_context: None,
                            can_sleep: None,
                            source_file: None,
                            is_kernel_internal: false,
                        });

                        // 递归构建异步处理函数
                        let handler_node = self.build_node(
                            &async_info.handler,
                            depth + 1,
                            options,
                            warnings,
                            async_boundaries,
                        );
                        children.push(handler_node);
                    }
                } else {
                    // 普通调用
                    let child =
                        self.build_node(callee, depth + 1, options, warnings, async_boundaries);
                    children.push(child);
                }
            }
        }

        // 如果是内核 API 且开启了知识库调用链注入
        if is_kernel && options.include_kernel_chains {
            if let Some(chain) = self.get_kernel_chain(func_name) {
                // 将知识库调用链添加到子节点
                for chain_node in chain {
                    children.push(chain_node);
                }
            }
        }

        self.visited.remove(func_name);

        FlowNode {
            id: format!("node-{}-{}", func_name, depth),
            name: func_name.to_string(),
            display_name: func_name.to_string(),
            location: func_info.and_then(|f| f.location),
            node_type,
            children,
            description: None,
            confidence: Some(CallConfidence {
                level: ConfidenceLevel::Certain,
                reason: "Direct call".into(),
            }),
            execution_context: Some(ExecutionContext::Process),
            can_sleep: Some(true),
            source_file: None,
            is_kernel_internal: is_kernel,
        }
    }

    /// 确定节点类型
    fn determine_node_type(&self, func_name: &str) -> (FlowNodeType, bool) {
        // 检查是否是内核 API
        if self.knowledge_base.get_api(func_name).is_some() {
            return (FlowNodeType::KernelApi, true);
        }

        // 检查是否是已知的内核函数
        let kernel_prefixes = ["__", "do_", "sys_", "vfs_", "device_", "driver_", "bus_"];
        let is_kernel = kernel_prefixes.iter().any(|p| func_name.starts_with(p));

        if is_kernel {
            return (FlowNodeType::KernelApi, true);
        }

        // 普通函数
        (FlowNodeType::Function, false)
    }

    /// 检查是否是异步触发
    fn check_async_trigger(&self, call: &str) -> Option<AsyncBindingInfo> {
        // 简化检查：查找 schedule_work, queue_work 等调用
        let async_triggers = [
            (
                "schedule_work",
                AsyncMechanism::WorkQueue { delayed: false },
            ),
            ("queue_work", AsyncMechanism::WorkQueue { delayed: false }),
            (
                "queue_delayed_work",
                AsyncMechanism::WorkQueue { delayed: true },
            ),
            (
                "mod_timer",
                AsyncMechanism::Timer {
                    high_resolution: false,
                },
            ),
            (
                "add_timer",
                AsyncMechanism::Timer {
                    high_resolution: false,
                },
            ),
            ("tasklet_schedule", AsyncMechanism::Tasklet),
        ];

        for (trigger, mechanism) in &async_triggers {
            if call.contains(trigger) {
                // 尝试从 async_bindings 中查找对应的 handler
                for (var, binding) in &self.async_bindings {
                    if call.contains(var) {
                        return Some(binding.clone());
                    }
                }

                // 没有找到绑定，返回一个未知 handler 的信息
                return Some(AsyncBindingInfo {
                    variable: "unknown".into(),
                    handler: format!("{}_handler", trigger),
                    mechanism: mechanism.clone(),
                    _bind_location: None,
                });
            }
        }

        None
    }

    /// 获取内核调用链
    fn get_kernel_chain(&self, _func_name: &str) -> Option<Vec<FlowNode>> {
        // TODO: 从知识库获取调用链
        None
    }

    /// 创建叶子节点
    fn create_leaf_node(&self, func_name: &str, suffix: &str) -> FlowNode {
        FlowNode {
            id: format!("leaf-{}", func_name),
            name: func_name.to_string(),
            display_name: format!("{} {}", func_name, suffix),
            location: None,
            node_type: FlowNodeType::External,
            children: Vec::new(),
            description: Some(suffix.to_string()),
            confidence: None,
            execution_context: None,
            can_sleep: None,
            source_file: None,
            is_kernel_internal: false,
        }
    }

    /// 获取上下文描述
    fn get_context_description(&self, mechanism: &AsyncMechanism) -> String {
        match mechanism {
            AsyncMechanism::WorkQueue { .. } => "进程上下文，可以睡眠".into(),
            AsyncMechanism::Timer { .. } => "软中断上下文，不可睡眠".into(),
            AsyncMechanism::Interrupt { threaded: true } => "进程上下文（threaded）".into(),
            AsyncMechanism::Interrupt { threaded: false } => "硬中断上下文，不可睡眠".into(),
            AsyncMechanism::Tasklet => "软中断上下文，不可睡眠".into(),
            AsyncMechanism::Softirq => "软中断上下文，不可睡眠".into(),
            _ => "未知上下文".into(),
        }
    }

    /// 获取机制名称
    fn get_mechanism_name(&self, mechanism: &AsyncMechanism) -> &'static str {
        match mechanism {
            AsyncMechanism::WorkQueue { .. } => "WorkQueue",
            AsyncMechanism::Timer { .. } => "Timer",
            AsyncMechanism::Interrupt { .. } => "IRQ",
            AsyncMechanism::Tasklet => "Tasklet",
            AsyncMechanism::Softirq => "SoftIRQ",
            AsyncMechanism::KThread => "KThread",
            AsyncMechanism::RcuCallback => "RCU",
            _ => "Async",
        }
    }
}

impl Default for FlowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 统计节点数量
fn count_nodes(node: &FlowNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

/// 统计调用类型
fn count_call_types(node: &FlowNode) -> (usize, usize, usize) {
    let mut direct = 0;
    let mut indirect = 0;
    let mut async_count = 0;

    count_call_types_recursive(node, &mut direct, &mut indirect, &mut async_count);

    (direct, indirect, async_count)
}

fn count_call_types_recursive(
    node: &FlowNode,
    direct: &mut usize,
    indirect: &mut usize,
    async_count: &mut usize,
) {
    match &node.node_type {
        FlowNodeType::Function | FlowNodeType::KernelApi => *direct += 1,
        FlowNodeType::AsyncCallback { .. } => *async_count += 1,
        FlowNodeType::EntryPoint => *direct += 1,
        _ => {}
    }

    for child in &node.children {
        count_call_types_recursive(child, direct, indirect, async_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_flow() {
        let mut builder = FlowBuilder::new();

        // 注册测试函数
        builder.register_function(FunctionInfo {
            name: "main".into(),
            location: None,
            calls: vec!["helper".into()],
            is_kernel: false,
        });

        builder.register_function(FunctionInfo {
            name: "helper".into(),
            location: None,
            calls: vec![],
            is_kernel: false,
        });

        let flow = builder.build("main", &BuildOptions::default());

        assert_eq!(flow.entry_function, "main");
        assert_eq!(flow.root.name, "main");
        assert_eq!(flow.root.children.len(), 1);
        assert_eq!(flow.root.children[0].name, "helper");
    }

    #[test]
    fn test_async_detection() {
        let mut builder = FlowBuilder::new();

        let code = r#"
            INIT_WORK(&dev->work, my_work_handler);
        "#;
        builder.extract_async_bindings(code);

        builder.register_function(FunctionInfo {
            name: "my_probe".into(),
            location: None,
            calls: vec!["schedule_work(&dev->work)".into()],
            is_kernel: false,
        });

        builder.register_function(FunctionInfo {
            name: "my_work_handler".into(),
            location: None,
            calls: vec![],
            is_kernel: false,
        });

        let flow = builder.build("my_probe", &BuildOptions::default());

        // 应该有异步边界
        assert!(!flow.async_boundaries.is_empty());
    }
}
