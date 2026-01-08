# FlowSight 目标架构选择

> 文档版本: 1.0
> 最后更新: 2026-01-08

---

## 1. 问题背景

Linux 内核代码大量使用条件编译，不同架构的实现差异很大：

```c
#ifdef CONFIG_X86_64
    // x86_64 特定实现
    asm volatile("mfence" ::: "memory");
#elif defined(CONFIG_ARM64)
    // ARM64 特定实现
    asm volatile("dmb sy" ::: "memory");
#elif defined(CONFIG_RISCV)
    // RISC-V 特定实现
    asm volatile("fence rw, rw" ::: "memory");
#endif
```

**如果不指定目标架构**：
- 执行流会包含所有架构的分支
- 调用图变得巨大且不唯一
- 用户无法理解实际执行路径

---

## 2. 解决方案

### 2.1 用户配置目标架构

在项目打开时或设置中，用户选择目标架构：

```yaml
# .flowsight/config.yaml
target:
  arch: x86_64              # 目标架构
  kernel_version: "6.1"     # 内核版本
  config_file: .config      # 可选：使用实际内核配置

  # 预定义配置（可选）
  defconfig: defconfig      # 或 allmodconfig, allyesconfig
```

### 2.2 支持的架构

| 架构 | CONFIG 宏 | 优先级 |
|------|----------|--------|
| x86_64 | `CONFIG_X86_64` | P0 (v1.0) |
| arm64 | `CONFIG_ARM64` | P0 (v1.0) |
| riscv64 | `CONFIG_RISCV` | P1 (v1.x) |
| arm32 | `CONFIG_ARM` | P2 (未来) |
| x86_32 | `CONFIG_X86_32` | P2 (未来) |

### 2.3 条件编译处理

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      条件编译处理流程                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  源代码                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ #ifdef CONFIG_X86_64                                             │    │
│  │     x86_impl();                                                  │    │
│  │ #elif defined(CONFIG_ARM64)                                      │    │
│  │     arm64_impl();                                                │    │
│  │ #endif                                                           │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│                              ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  预处理器模拟                                                    │    │
│  │  • 读取用户配置的目标架构                                       │    │
│  │  • 设置对应的 CONFIG_* 宏                                       │    │
│  │  • 评估 #ifdef/#elif/#else 条件                                 │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│                              ▼                                           │
│  用户选择 arch: x86_64                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ // 只保留 x86_64 分支                                            │    │
│  │ x86_impl();                                                      │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  结果：执行流唯一确定！                                                 │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 实现方案

### 3.1 预定义宏集合

为每个架构预定义一组宏：

```rust
// flowsight-parser/src/arch_config.rs

pub struct ArchConfig {
    pub name: &'static str,
    pub defines: Vec<(&'static str, Option<&'static str>)>,
}

pub fn get_arch_config(arch: &str) -> ArchConfig {
    match arch {
        "x86_64" => ArchConfig {
            name: "x86_64",
            defines: vec![
                ("CONFIG_X86_64", Some("1")),
                ("CONFIG_X86", Some("1")),
                ("CONFIG_64BIT", Some("1")),
                ("__x86_64__", Some("1")),
                ("__LP64__", Some("1")),
            ],
        },
        "arm64" => ArchConfig {
            name: "arm64",
            defines: vec![
                ("CONFIG_ARM64", Some("1")),
                ("CONFIG_64BIT", Some("1")),
                ("__aarch64__", Some("1")),
                ("__LP64__", Some("1")),
            ],
        },
        "riscv64" => ArchConfig {
            name: "riscv64",
            defines: vec![
                ("CONFIG_RISCV", Some("1")),
                ("CONFIG_64BIT", Some("1")),
                ("__riscv", Some("1")),
                ("__riscv_xlen", Some("64")),
            ],
        },
        _ => panic!("Unsupported architecture: {}", arch),
    }
}
```

### 3.2 .config 文件解析

支持导入实际内核配置：

```rust
// flowsight-parser/src/kconfig.rs

use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct KernelConfig {
    pub options: HashMap<String, ConfigValue>,
}

pub enum ConfigValue {
    Bool(bool),      // CONFIG_FOO=y or # CONFIG_FOO is not set
    String(String),  // CONFIG_FOO="bar"
    Number(i64),     // CONFIG_FOO=123
    Module,          // CONFIG_FOO=m
}

impl KernelConfig {
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let content = fs::read_to_string(path)?;
        let mut options = HashMap::new();

        for line in content.lines() {
            if line.starts_with('#') {
                // # CONFIG_FOO is not set
                if let Some(name) = parse_not_set(line) {
                    options.insert(name, ConfigValue::Bool(false));
                }
            } else if let Some((name, value)) = parse_config_line(line) {
                options.insert(name, value);
            }
        }

        Ok(Self { options })
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        matches!(
            self.options.get(name),
            Some(ConfigValue::Bool(true)) | Some(ConfigValue::Module)
        )
    }
}
```

### 3.3 条件编译评估

在解析时评估条件编译指令：

```rust
// flowsight-parser/src/preprocessor.rs

pub struct PreprocessorState {
    defines: HashMap<String, Option<String>>,
    condition_stack: Vec<ConditionState>,
}

enum ConditionState {
    Active,      // 当前分支激活
    Inactive,    // 当前分支未激活
    Done,        // 已有分支激活，后续都跳过
}

impl PreprocessorState {
    pub fn should_include(&self) -> bool {
        self.condition_stack.iter().all(|s| matches!(s, ConditionState::Active))
    }

    pub fn handle_ifdef(&mut self, macro_name: &str) {
        let defined = self.defines.contains_key(macro_name);
        self.condition_stack.push(if defined {
            ConditionState::Active
        } else {
            ConditionState::Inactive
        });
    }

    pub fn handle_elif(&mut self, condition: bool) {
        if let Some(state) = self.condition_stack.last_mut() {
            *state = match state {
                ConditionState::Active => ConditionState::Done,
                ConditionState::Inactive if condition => ConditionState::Active,
                _ => ConditionState::Inactive,
            };
        }
    }

    pub fn handle_endif(&mut self) {
        self.condition_stack.pop();
    }
}
```

---

## 4. UI 设计

### 4.1 项目打开时选择架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      打开 Linux 内核项目                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  检测到 Linux 内核源码，请选择目标架构：                                │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  ○ x86_64 (Intel/AMD 64-bit)                                    │    │
│  │  ○ arm64 (ARM 64-bit / Apple Silicon)                           │    │
│  │  ○ riscv64 (RISC-V 64-bit)                                      │    │
│  │  ○ 从 .config 文件导入                                          │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  内核版本: [6.1        ▼]                                               │
│                                                                          │
│  [ ] 记住此项目的选择                                                   │
│                                                                          │
│                              [取消]  [确定]                              │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.2 状态栏显示

```
┌─────────────────────────────────────────────────────────────────────────┐
│  ✅ 索引完成: 15,234 符号 | 🎯 x86_64 | 📊 分析就绪                     │
└─────────────────────────────────────────────────────────────────────────┘
                                  ↑
                            点击可切换架构
```

### 4.3 架构切换

切换架构时需要重新索引：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         切换目标架构                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  当前架构: x86_64                                                        │
│  切换到: arm64                                                           │
│                                                                          │
│  ⚠️ 切换架构需要重新索引代码，这可能需要几分钟时间。                    │
│                                                                          │
│                              [取消]  [切换并重新索引]                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 5. 执行流中的架构标注

### 5.1 架构特定代码标注

在执行流图中标注架构特定的代码：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         执行流视图                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────┐                                                        │
│  │ my_driver_  │                                                        │
│  │ probe       │                                                        │
│  └──────┬──────┘                                                        │
│         │                                                                │
│  ┌──────┴──────┐                                                        │
│  │ init_device │                                                        │
│  └──────┬──────┘                                                        │
│         │                                                                │
│  ┌──────┴──────┐                                                        │
│  │ setup_dma   │ [x86_64]  ← 架构特定标注                               │
│  └──────┬──────┘                                                        │
│         │                                                                │
│  ┌──────┴──────┐                                                        │
│  │ dma_map_    │                                                        │
│  │ single      │                                                        │
│  └─────────────┘                                                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 多架构对比视图（v1.x）

未来可支持同时查看多个架构的执行流差异：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      多架构对比视图                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  x86_64                          arm64                                   │
│  ┌─────────────┐                ┌─────────────┐                         │
│  │ setup_dma   │                │ setup_dma   │                         │
│  └──────┬──────┘                └──────┬──────┘                         │
│         │                              │                                 │
│  ┌──────┴──────┐                ┌──────┴──────┐                         │
│  │ x86_dma_    │                │ arm64_dma_  │  ← 不同实现             │
│  │ setup       │                │ setup       │                         │
│  └──────┬──────┘                └──────┬──────┘                         │
│         │                              │                                 │
│  ┌──────┴──────┐                ┌──────┴──────┐                         │
│  │ dma_map_    │ ═══════════════│ dma_map_    │  ← 相同调用             │
│  │ single      │                │ single      │                         │
│  └─────────────┘                └─────────────┘                         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 6. 实施计划

### v1.0 必须
- [ ] 支持 x86_64 和 arm64 两种架构
- [ ] 项目打开时架构选择对话框
- [ ] 基本的条件编译处理
- [ ] 状态栏架构显示

### v1.x 增强
- [ ] 支持 riscv64
- [ ] .config 文件导入
- [ ] 架构切换功能
- [ ] 多架构对比视图

---

## 附录：常见内核架构宏

| 宏 | 含义 |
|----|------|
| `CONFIG_X86_64` | x86 64-bit |
| `CONFIG_X86_32` | x86 32-bit |
| `CONFIG_ARM64` | ARM 64-bit |
| `CONFIG_ARM` | ARM 32-bit |
| `CONFIG_RISCV` | RISC-V |
| `CONFIG_64BIT` | 64-bit 架构 |
| `CONFIG_SMP` | 多处理器支持 |
| `CONFIG_PREEMPT` | 抢占式内核 |
| `CONFIG_DEBUG_INFO` | 调试信息 |
