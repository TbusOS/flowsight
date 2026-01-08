# FlowSight 预处理器集成方案

> **核心洞察**：不需要完整编译，只需要预处理 + 解析 + 语义分析，就能实现准确的代码分析。

---

## 目录

1. [问题分析](#1-问题分析)
2. [技术方案](#2-技术方案)
3. [实现架构](#3-实现架构)
4. [详细设计](#4-详细设计)
5. [依赖管理](#5-依赖管理)
6. [实现步骤](#6-实现步骤)

---

## 1. 问题分析

### 1.1 为什么需要预处理？

C 语言的复杂性主要来自预处理器：

```c
// 原始代码 - 无法直接解析
#ifdef CONFIG_X86_64
    typedef unsigned long size_t;
#else
    typedef unsigned int size_t;
#endif

#define container_of(ptr, type, member) \
    ((type *)((char *)(ptr) - offsetof(type, member)))

static inline void *
#ifdef CONFIG_DEBUG
__attribute__((noinline))
#endif
kmalloc(size_t size, gfp_t flags);
```

问题：
- 条件编译 (`#ifdef`) 导致多个代码分支
- 宏展开 (`#define`) 隐藏真实代码
- 头文件包含 (`#include`) 引入类型定义
- 不同架构有不同的类型大小

### 1.2 编译器的工作阶段

```
源代码 → 预处理 → 词法分析 → 语法分析 → 语义分析 → 优化 → 代码生成
         ↑         ↑          ↑           ↑
         │         │          │           │
      展开宏    Token流      AST       类型检查
      处理#ifdef                       数据流分析
      包含头文件
         │
         └── FlowSight 只需要这四个阶段，不需要优化和代码生成
```

### 1.3 关键洞察

| 阶段 | 需要完整编译器？ | FlowSight 需要？ |
|------|-----------------|-----------------|
| 预处理 | 否，可独立运行 | ✅ 必需 |
| 词法分析 | 否 | ✅ 必需 |
| 语法分析 | 否 | ✅ 必需 |
| 语义��析 | 否 | ✅ 必需 |
| 优化 | 是 | ❌ 不需要 |
| 代码生成 | 是 | ❌ 不需要 |

---

## 2. 技术方案

### 2.1 方案对比

#### 方案 A：使用 GCC 预处理器

```bash
# 预处理单个文件
gcc -E -D__x86_64__ -DCONFIG_X86_64 -I/path/to/headers driver.c -o driver.i

# 优点：
# - 与内核编译完全兼容
# - 处理所有 GCC 扩展

# 缺点：
# - 需要为每个架构安装交叉编译器
# - arm-linux-gnueabi-gcc, aarch64-linux-gnu-gcc, riscv64-linux-gnu-gcc...
```

#### 方案 B：使用 Clang 预处理器（推荐）

```bash
# 单一二进制支持多架构
clang -E -target x86_64-linux-gnu -D... driver.c -o driver.i
clang -E -target aarch64-linux-gnu -D... driver.c -o driver.i
clang -E -target riscv64-linux-gnu -D... driver.c -o driver.i

# 优点：
# - 单一二进制支持所有架构
# - libclang API 更友好
# - 更好的错误信息
# - 可以同时获取 AST

# 缺点：
# - 少数 GCC 特有扩展可能不支持
```

### 2.2 选择 Clang 的理由

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Clang vs GCC 对比                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  特性              │ GCC                    │ Clang                     │
│  ─────────────────┼────────────────────────┼───────────────────────────│
│  多架构支持        │ 需要多个二进制         │ 单一二进制                │
│  安装大小          │ ~200MB/架构            │ ~150MB 总共               │
│  API 友好度        │ 较差                   │ libclang 非常友好         │
│  AST 访问          │ 困难                   │ 简单                      │
│  错误信息          │ 一般                   │ 优秀                      │
│  内核兼容性        │ 原生                   │ 99%+ 兼容                 │
│  跨平台            │ 需要交叉编译器         │ 内置                      │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.3 最终方案

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      FlowSight 预处理方案                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  用户操作：                                                              │
│  1. 打开 Linux kernel 源码目录                                          │
│  2. 选择架构: x86_64 / arm64 / riscv64                                 │
│  3. 选择配置: defconfig / 自定义                                        │
│                                                                          │
│  FlowSight 后台处理：                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  1. 读取 Kconfig/Makefile，提取配置宏                           │   │
│  │     CONFIG_X86_64=y → -DCONFIG_X86_64                           │   │
│  │                                                                  │   │
│  │  2. 调用 Clang 预处理器                                         │   │
│  │     clang -E -target x86_64-linux-gnu \                         │   │
│  │           -D__KERNEL__ -DCONFIG_X86_64 \                        │   │
│  │           -I./include -I./arch/x86/include \                    │   │
│  │           driver.c -o driver.i                                  │   │
│  │                                                                  │   │
│  │  3. 解析预处理后的代码                                          │   │
│  │     - 使用 libclang 获取完整 AST                                │   │
│  │     - 或使用 tree-sitter 快速解析                               │   │
│  │                                                                  │   │
│  │  4. 语义分析                                                    │   │
│  │     - 类型推断                                                  │   │
│  │     - 指针分析                                                  │   │
│  │     - 数据流分析                                                │   │
│  │                                                                  │   │
│  │  5. 缓存结果                                                    │   │
│  │     - 预处理结果缓存                                            │   │
│  │     - AST 缓存                                                  │   │
│  │     - 分析结果缓存                                              │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 实现架构

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FlowSight 分析引擎                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐              │
│  │   配置管理    │    │   预处理器    │    │   解析器     │              │
│  │              │    │              │    │              │              │
│  │ - Kconfig    │───▶│ - Clang -E   │───▶│ - libclang   │              │
│  │ - Makefile   │    │ - 宏展开     │    │ - tree-sitter│              │
│  │ - .config    │    │ - 头文件     │    │              │              │
│  └──────────────┘    └──────────────┘    └──────────────┘              │
│         │                   │                   │                       │
│         ▼                   ▼                   ▼                       │
│  ┌──────────────────────────────────────────────────────────┐          │
│  │                      分析引擎                             │          │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │          │
│  │  │  类型分析   │  │  指针分析   │  │  数据流分析 │         │          │
│  │  └────────────┘  └────────────┘  └────────────┘         │          │
│  └──────────────────────────────────────────────────────────┘          │
│         │                                                               │
│         ▼                                                               │
│  ┌──────────────────────────────────────────────────────────┐          │
│  │                      知识库                               │          │
│  │  - 函数调用图                                             │          │
│  │  - 类型信息                                               │          │
│  │  - 指针指向集                                             │          │
│  │  - 数据流信息                                             │          │
│  └──────────────────────────────────────────────────────────┘          │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 模块划分

```typescript
// 模块结构
flowsight/
├── src/
│   ├── preprocessor/           // 预处理器模块
│   │   ├── ClangPreprocessor.ts    // Clang 预处理器封装
│   │   ├── ConfigExtractor.ts      // 配置提取器
│   │   ├── HeaderResolver.ts       // 头文件解析器
│   │   └── PreprocessorCache.ts    // 预处理缓存
│   │
│   ├── parser/                 // 解析器模块
│   │   ├── LibclangParser.ts       // libclang 解析器
│   │   ├── TreeSitterParser.ts     // tree-sitter 解析器
│   │   └── ASTCache.ts             // AST 缓存
│   │
│   ├── analyzer/               // 分析器模块
│   │   ├── TypeAnalyzer.ts         // 类型分析
│   │   ├── PointerAnalyzer.ts      // 指针分析
│   │   ├── DataFlowAnalyzer.ts     // 数据流分析
│   │   └── CallGraphBuilder.ts     // 调用图构建
│   │
│   └── knowledge/              // 知识库模块
│       ├── KnowledgeBase.ts        // 知识库接口
│       ├── SQLiteStorage.ts        // SQLite 存储
│       └── QueryEngine.ts          // 查询引擎
```

---

## 4. 详细设计

### 4.1 配置提取器

```typescript
/**
 * 从 Linux 内核配置中提取预处理宏
 */
interface ConfigExtractor {
  /**
   * 从 .config 文件提取配置
   * @param configPath .config 文件路径
   * @returns 宏定义列表
   */
  extractFromDotConfig(configPath: string): MacroDefinition[];

  /**
   * 从 Makefile 提取编译选项
   * @param makefilePath Makefile 路径
   * @returns 编译选项
   */
  extractFromMakefile(makefilePath: string): CompileOptions;

  /**
   * 获取架构特定的宏定义
   * @param arch 目标架构
   * @returns 架构宏定义
   */
  getArchMacros(arch: Architecture): MacroDefinition[];
}

// 示例输出
const macros: MacroDefinition[] = [
  { name: '__KERNEL__', value: '1' },
  { name: 'CONFIG_X86_64', value: '1' },
  { name: 'CONFIG_SMP', value: '1' },
  { name: 'CONFIG_PREEMPT', value: undefined },  // 未定义
  // ...
];
```

### 4.2 Clang 预处理器封装

```typescript
/**
 * Clang 预处理器封装
 */
interface ClangPreprocessor {
  /**
   * 预处理单个文件
   * @param sourcePath 源文件路径
   * @param options 预处理选项
   * @returns 预处理结果
   */
  preprocess(sourcePath: string, options: PreprocessOptions): PreprocessResult;

  /**
   * 批量预处理
   * @param sourcePaths 源文件路径列表
   * @param options 预处理选项
   * @returns 预处理结果列表
   */
  preprocessBatch(sourcePaths: string[], options: PreprocessOptions): PreprocessResult[];
}

interface PreprocessOptions {
  target: string;           // 目标架构，如 'x86_64-linux-gnu'
  defines: MacroDefinition[];  // 宏定义
  includes: string[];       // 头文件搜索路径
  systemIncludes: string[]; // 系统头文件路径
  outputPath?: string;      // 输出路径（可选）
}

interface PreprocessResult {
  success: boolean;
  preprocessedCode?: string;  // 预处理后的代码
  errors?: string[];          // 错误信息
  warnings?: string[];        // 警告信息
  includedFiles?: string[];   // 包含的头文件列表
}
```

### 4.3 预处理命令生成

```typescript
/**
 * 生成 Clang 预处理命令
 */
function generateClangCommand(
  sourcePath: string,
  options: PreprocessOptions
): string {
  const args: string[] = [
    'clang',
    '-E',                           // 只预处理
    `-target ${options.target}`,    // 目标架构
    '-nostdinc',                    // 不使用标准头文件
    '-fno-builtin',                 // 禁用内置函数
  ];

  // 添加宏定义
  for (const macro of options.defines) {
    if (macro.value !== undefined) {
      args.push(`-D${macro.name}=${macro.value}`);
    } else {
      args.push(`-U${macro.name}`);
    }
  }

  // 添加头文件路径
  for (const include of options.includes) {
    args.push(`-I${include}`);
  }

  // 添加系统头文件路径
  for (const sysInclude of options.systemIncludes) {
    args.push(`-isystem ${sysInclude}`);
  }

  args.push(sourcePath);

  if (options.outputPath) {
    args.push(`-o ${options.outputPath}`);
  }

  return args.join(' ');
}

// 示例输出
// clang -E -target x86_64-linux-gnu -nostdinc -fno-builtin \
//   -D__KERNEL__=1 -DCONFIG_X86_64=1 -DCONFIG_SMP=1 \
//   -I./include -I./arch/x86/include \
//   -isystem /usr/lib/clang/15/include \
//   drivers/usb/core/hub.c -o hub.i
```

### 4.4 Linux 内核头文件路径

```typescript
/**
 * Linux 内核头文件路径配置
 */
interface KernelHeaderPaths {
  /**
   * 获取内核头文件搜索路径
   * @param kernelRoot 内核源码根目录
   * @param arch 目标架构
   * @returns 头文件路径列表
   */
  getIncludePaths(kernelRoot: string, arch: Architecture): string[];
}

// 实现
function getKernelIncludePaths(
  kernelRoot: string,
  arch: Architecture
): string[] {
  const archDir = getArchDir(arch);  // x86, arm64, riscv 等

  return [
    // 通用头文件
    `${kernelRoot}/include`,
    `${kernelRoot}/include/uapi`,

    // 架构特定头文件
    `${kernelRoot}/arch/${archDir}/include`,
    `${kernelRoot}/arch/${archDir}/include/uapi`,
    `${kernelRoot}/arch/${archDir}/include/generated`,
    `${kernelRoot}/arch/${archDir}/include/generated/uapi`,

    // 生成的头文件（如果存在）
    `${kernelRoot}/include/generated`,
    `${kernelRoot}/include/generated/uapi`,
  ];
}

function getArchDir(arch: Architecture): string {
  const archMap: Record<Architecture, string> = {
    'x86_64': 'x86',
    'i386': 'x86',
    'arm64': 'arm64',
    'arm': 'arm',
    'riscv64': 'riscv',
    'riscv32': 'riscv',
    'mips': 'mips',
    'powerpc': 'powerpc',
  };
  return archMap[arch] || arch;
}
```

### 4.5 预处理缓存

```typescript
/**
 * 预处理结果缓存
 *
 * 缓存策略：
 * - 基于文件内容哈希 + 预处理选项哈希
 * - 文件修改时自动失效
 * - 配置变更时批量失效
 */
interface PreprocessorCache {
  /**
   * 获取缓存的预处理结果
   */
  get(sourcePath: string, options: PreprocessOptions): PreprocessResult | null;

  /**
   * 存储预处理结果
   */
  set(sourcePath: string, options: PreprocessOptions, result: PreprocessResult): void;

  /**
   * 检查缓存是否有效
   */
  isValid(sourcePath: string, options: PreprocessOptions): boolean;

  /**
   * 清除指定文件的缓存
   */
  invalidate(sourcePath: string): void;

  /**
   * 清除所有缓存
   */
  clear(): void;
}

// 缓存键生成
function generateCacheKey(
  sourcePath: string,
  options: PreprocessOptions
): string {
  const fileHash = computeFileHash(sourcePath);
  const optionsHash = computeOptionsHash(options);
  return `${sourcePath}:${fileHash}:${optionsHash}`;
}
```

---

## 5. 依赖管理

### 5.1 必需依赖

| 组件 | 用途 | 大小 | 安装方式 |
|------|------|------|----------|
| Clang/LLVM | 预处理器 + 解析器 | ~150MB | 系统包管理器 |
| 内核头文件 | 类型定义 | ~50MB | 内核源码或 linux-headers 包 |

### 5.2 安装脚本

```bash
#!/bin/bash
# install-dependencies.sh

# 检测操作系统
OS=$(uname -s)

case $OS in
  Linux)
    if command -v apt-get &> /dev/null; then
      # Debian/Ubuntu
      sudo apt-get update
      sudo apt-get install -y clang libclang-dev
    elif command -v dnf &> /dev/null; then
      # Fedora/RHEL
      sudo dnf install -y clang clang-devel
    elif command -v pacman &> /dev/null; then
      # Arch Linux
      sudo pacman -S clang
    fi
    ;;
  Darwin)
    # macOS
    if command -v brew &> /dev/null; then
      brew install llvm
    else
      echo "Please install Homebrew first: https://brew.sh"
      exit 1
    fi
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

echo "Clang installed successfully!"
clang --version
```

### 5.3 Node.js 绑定

```typescript
// 使用 node-llvm 或 libclang 的 Node.js 绑定
// 推荐使用 @aspect-build/rules_ts 的 libclang 绑定

// 或者通过子进程调用 clang
import { spawn } from 'child_process';

async function runClang(args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    const proc = spawn('clang', args);
    let stdout = '';
    let stderr = '';

    proc.stdout.on('data', (data) => { stdout += data; });
    proc.stderr.on('data', (data) => { stderr += data; });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve(stdout);
      } else {
        reject(new Error(`Clang failed: ${stderr}`));
      }
    });
  });
}
```

---

## 6. 实现步骤

### 阶段 1：基础预处理器集成

1. **实现 ClangPreprocessor 类**
   - 封装 Clang 命令行调用
   - 支持基本的预处理选项
   - 错误处理和日志

2. **实现 ConfigExtractor 类**
   - 解析 .config 文件
   - 提取 CONFIG_* 宏定义
   - 支持 defconfig 预设

3. **实现 HeaderResolver 类**
   - 计算头文件搜索路径
   - 支持多架构

### 阶段 2：缓存和优化

1. **实现 PreprocessorCache**
   - 基于文件哈希的缓存
   - 增量更新支持
   - 缓存失效策略

2. **批量预处理优化**
   - 并行预处理多个文件
   - 共享头文件解析结果

### 阶段 3：与分析引擎集成

1. **连接预处理器和解析器**
   - 预处理后直接解析
   - 保留源码位置映射

2. **更新分析流程**
   - 使用预处理后的代码进行分析
   - 支持条件编译分支选择

### 阶段 4：用户界面

1. **架构选择 UI**
   - 支持选择目标架构
   - 支持选择内核配置

2. **预处理状态显示**
   - 显示预处理进度
   - 显示错误和警告

---

## 附录 A：常见问题

### Q1: 没有编译过的内核源码能分析吗？

可以，但需要：
1. 选择一个默认配置（如 defconfig）
2. 生成必要的头文件（`make headers_install`）

或者 FlowSight 可以使用预置的头文件包。

### Q2: 如何处理内联汇编？

内联汇编 (`asm volatile(...)`) 会被保留，但不会深入分析。
FlowSight 会标记这些位置，让用户知道这里有汇编代码。

### Q3: 如何处理编译器特定扩展？

Clang 支持大部分 GCC 扩展。对于不支持的扩展：
1. 使用 `-fgnuc-version=` 模拟 GCC 版本
2. 定义兼容宏来处理特殊语法

### Q4: 预处理会很慢吗？

首次预处理可能需要几秒到几十秒（取决于文件大小和头文件数量）。
但结果会被缓存，后续访问几乎是即时的。

---

## 附录 B：参考资料

- [Clang 预处理器文档](https://clang.llvm.org/docs/ClangCommandLineReference.html)
- [libclang API 文档](https://clang.llvm.org/doxygen/group__CINDEX.html)
- [Linux 内核构建系统](https://www.kernel.org/doc/html/latest/kbuild/index.html)
- [tree-sitter C 语法](https://github.com/tree-sitter/tree-sitter-c)
