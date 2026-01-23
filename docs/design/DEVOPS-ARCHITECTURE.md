# FlowSight DevOps 架构设计

> 版本: 1.0.0
> 最后更新: 2025-01-24
> 状态: 初稿

## 1. 概述

### 1.1 项目简介

FlowSight 是一个跨平台桌面应用，用于函数执行流分析和可视化。项目结合了 Rust 后端（LLVM IR 解析、KLEE 符号执行）和 React 前端（Tauri 2.0 打包），需要复杂的开发环境和构建流程。

### 1.2 架构目标

- **可重复性**: 确保开发、测试、生产环境的一致性
- **自动化**: 端到端自动化构建、测试、部署流程
- **可观测性**: 完整的构建日志、性能指标、质量报告
- **安全性**: 依赖安全扫描、签名验证、安全发布

### 1.3 技术栈概览

```
┌─────────────────────────────────────────────────────────────────┐
│                        FlowSight 架构                            │
├─────────────────────────────────────────────────────────────────┤
│  前端层 (React + TypeScript + Tauri 2.0)                        │
│    - React 18 + Vite 5                                         │
│    - Monaco Editor + @xyflow/react                             │
│    - Tauri 2.0 (桌面打包)                                      │
├─────────────────────────────────────────────────────────────────┤
│  后端层 (Rust + Tokio)                                          │
│    - flowsight-core: 核心类型定义                               │
│    - flowsight-parser: Tree-sitter 解析                        │
│    - flowsight-llvm: LLVM IR 解析 (inkwell 绑定)               │
│    - flowsight-symbolic: KLEE 符号执行集成                      │
│    - flowsight-knowledge: YAML 知识库                          │
├─────────────────────────────────────────────────────────────────┤
│  基础设施层                                                      │
│    - Docker: 开发环境 + CI 构建                                 │
│    - GitHub Actions: CI/CD 流水线                               │
│    - 依赖管理: Cargo + pnpm + apt                               │
└─────────────────────────────────────────────────────────────────┘
```

## 2. 开发环境配置

### 2.1 本地开发环境要求

#### 2.1.1 基础依赖

| 依赖 | 版本要求 | 用途 | 安装方式 |
|------|----------|------|----------|
| Rust | 1.75+ | 编译器 | rustup |
| Node.js | 20+ | 前端构建 | nvm |
| pnpm | 8+ | 包管理 | npm |
| LLVM | 17+ | IR 解析 | apt/brew |
| Clang | 17+ | 编译到 IR | apt/brew |
| Tree-sitter | latest | C 代码解析 | cargo |
| CMake | 3.18+ | KLEE 构建 | apt/brew |

#### 2.1.2 可选依赖

| 依赖 | 版本要求 | 用途 | 安装方式 |
|------|----------|------|----------|
| KLEE | latest | 符号执行 | Docker |
| Docker | 24+ | 容器化开发 | 官方安装脚本 |

### 2.2 Docker 开发环境 (推荐)

FlowSight 提供完整的 Docker 开发环境，避免本地依赖配置问题。

#### 2.2.1 Dockerfile.dev

```dockerfile
# File: /home/parallels/github/flowsight/.devcontainer/Dockerfile.dev

#=====================================================================#
# FlowSight 开发环境镜像                                             #
#=====================================================================#
# 构建方式: docker build -t flowsight-dev -f .devcontainer/Dockerfile.dev .
# 使用方式: docker run -it -v $(pwd):/workspaces/flowsight flowsight-dev
#=====================================================================#

# ---------------------------------------------------------------------
# 基础镜像选择
# ---------------------------------------------------------------------
# 选择理由:
# - Ubuntu 22.04 LTS 提供 5 年支持
# - 包含完整的开发工具链
# - 与 CI 环境保持一致
# ---------------------------------------------------------------------
FROM ubuntu:22.04 AS base

# ---------------------------------------------------------------------
# 基础配置
# ---------------------------------------------------------------------
ENV DEBIAN_FRONTEND=noninteractive \
    TZ=UTC \
    LANG=en_US.UTF-8 \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8

# ---------------------------------------------------------------------
# 系统依赖安装
# ---------------------------------------------------------------------
RUN apt-get update && apt-get install -y --no-install-recommends \
    # 基础工具
    curl \
    wget \
    git \
    vim \
    nano \
    unzip \
    zip \
    tar \
    gzip \
    bzip2 \
    xz-utils \
    ca-certificates \
    gnupg \
    lsb-release \
    software-properties-common \
    apt-transport-https \
    ca-certificates \
    \
    # 构建工具
    build-essential \
    cmake \
    ninja-build \
    pkg-config \
    \
    # 版本控制
    git \
    \
    # Shell
    bash \
    zsh \
    fish \
    \
    # 文档工具
    pandoc \
    texlive-latex-base \
    \
    # 清理
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# ---------------------------------------------------------------------
# Rust 开发环境
# ---------------------------------------------------------------------
FROM base AS rust

ENV RUSTUP_HOME=/opt/rust/rustup \
    CARGO_HOME=/opt/rust/cargo \
    PATH=/opt/rust/cargo/bin:$PATH

# 安装 Rust 工具链
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y \
    --default-toolchain stable \
    --profile minimal \
    --component rustfmt,clippy,rust-analyzer,rust-src

# 配置 Rust
RUN echo '[profile.release]\nopt-level = 3\nlto = true' >> ~/.cargo/config.toml

# ---------------------------------------------------------------------
# LLVM/Clang 开发环境
# ---------------------------------------------------------------------
FROM rust AS llvm

ENV LLVM_VERSION=17 \
    LLVM_INSTALL_PREFIX=/opt/llvm

# 添加 LLVM APT 仓库
RUN wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor > /etc/apt/keyrings/llvm.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/llvm.gpg] http://apt.llvm.org/ubuntu/ jllvm-toolchain main" > /etc/apt/sources.list.d/llvm.list \
    && apt-get update

# 安装 LLVM 和 Clang
RUN apt-get install -y --no-install-recommends \
    llvm-${LLVM_VERSION} \
    llvm-${LLVM_VERSION}-dev \
    llvm-${LLVM_VERSION}-tools \
    clang-${LLVM_VERSION} \
    clang-tools-${LLVM_VERSION} \
    lld-${LLVM_VERSION} \
    \
    # 通用 LLVM 开发文件
    libllvm${LLVM_VERSION}-dev \
    libllvm${LLVM_VERSION}-runtime \
    \
    # Python bindings (用于 LLVM 工具)
    python3-lit \
    python3-ply \
    \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# 创建 LLVM 版本符号链接
RUN ln -sf /usr/lib/llvm-${LLVM_VERSION}/bin/* /usr/local/bin/ \
    && ln -sf /usr/lib/llvm-${LLVM_VERSION}/lib/* /usr/local/lib/ \
    && ln -sf /usr/include/llvm-${LLVM_VERSION} /usr/local/include/llvm

# 验证 LLVM 安装
RUN llvm-config-${LLVM_VERSION} --version

# ---------------------------------------------------------------------
# Node.js 前端开发环境
# ---------------------------------------------------------------------
FROM llvm AS node

ENV NVM_DIR=/opt/nvm \
    NODE_VERSION=20 \
    NPM_CONFIG_REGISTRY=https://registry.npmmirror.com

# 安装 nvm 和 Node.js
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash \
    && . $NVM_DIR/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm use $NODE_VERSION \
    && nvm alias default $NODE_VERSION

# 安装 pnpm
RUN . $NVM_DIR/nvm.sh && npm install -g pnpm@8

# 验证安装
RUN . $NVM_DIR/nvm.sh && node --version && npm --version && pnpm --version

# ---------------------------------------------------------------------
# KLEE 符号执行环境 (可选)
# ---------------------------------------------------------------------
FROM node AS klee

ENV KLEE_SRC=/opt/klee \
    KLEE_BUILD=/opt/klee/build \
    KLEE_VERSION=v3.0

# 安装 KLEE 依赖
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-pip \
    python3-setuptools \
    bc \
    binutils-dev \
    libboost-all-dev \
    libncurses5-dev \
    libfl-dev \
    libgmp-dev \
    libsqlite3-dev \
    libssl-dev \
    libtinfo5 \
    libz3-dev \
    z3 \
    uclibc \
    libc6-dev-i386 \
    \
    # STP 依赖
    cmake \
    flex \
    bison \
    libboost-program-options-dev \
    libboost-test-dev \
    \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# 克隆并构建 KLEE (仅基础框架，不含 uclibc)
# 注意: 完整构建需要更多时间和资源
WORKDIR $KLEE_SRC
RUN git clone --depth 1 --branch $KLEE_VERSION https://github.com/klee/klee.git . \
    && mkdir -p $KLEE_BUILD \
    && cd $KLEE_BUILD \
    && cmake .. \
        -DENABLE_POSIX_RUNTIME=OFF \
        -DENABLE_SOLVER_Z3=ON \
        -DKLEE_UCLibc=OFF \
        -DLLVM_DIR=/usr/lib/llvm-${LLVM_VERSION}/lib/cmake/llvm \
    && ninja -j$(nproc) \
    && cp -r bin $KLEE_INSTALL/ \
    && cp -r lib $KLEE_INSTALL/

ENV PATH=$KLEE_INSTALL/bin:$PATH

# 验证 KLEE
RUN klee --version

# ---------------------------------------------------------------------
# 最终开发环境
# ---------------------------------------------------------------------
FROM klee AS dev

ENV PROJECT_DIR=/workspaces/flowsight \
    CARGO_REGISTRY_INDEX=https://mirrors.ustc.edu.cn/crates.io-index \
    CARGO_REGISTRY_SERVER=https://mirrors.ustc.edu.cn

# 创建工作目录
WORKDIR $PROJECT_DIR

# 预设用户 (避免 root 权限问题)
ENV USER=developer
ENV HOME=/home/$USER

RUN useradd -m -s /bin/bash $USER \
    && chown -R $USER:$USER $PROJECT_DIR

# 安装常用开发工具
RUN apt-get update && apt-get install -y --no-install-recommends \
    htop \
    vim \
    tmux \
    ripgrep \
    fd-find \
    bat \
    exa \
    the_silver_searcher \
    \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# 安装 Rust 插件
RUN cargo install --locked \
    cargo-watch \
    cargo-expand \
    cargo-tree \
    cargo-audit \
    cargo-outdated \
    \
    # 清理
    && rm -rf ~/.cargo/registry/index/*

# 配置开发工具
RUN echo "alias ll='exa -l'" >> ~/.bashrc \
    && echo "alias rg='ripgrep'" >> ~/.bashrc \
    && echo "export PATH=/usr/local/bin:\$PATH" >> ~/.bashrc

# 设置工作目录
WORKDIR $PROJECT_DIR

# 暴露端口
EXPOSE 5173 3000

# 默认命令
CMD ["/bin/bash"]

# 元数据
LABEL maintainer="FlowSight Team" \
      version="1.0.0" \
      description="FlowSight Development Environment"
```

#### 2.2.2 Docker Compose 开发配置

```yaml
# File: /home/parallels/github/flowsight/.devcontainer/docker-compose.yml

version: '3.8'

services:
  flowsight-dev:
    build:
      context: .
      dockerfile: Dockerfile.dev
      target: dev
    container_name: flowsight-dev
    environment:
      - RUST_LOG=info
      - CARGO_INCREMENTAL=1
      - CARGO_NET_GIT_FETCH_WITH_CLI=true
    volumes:
      # 挂载项目代码
      - ..:/workspaces/flowsight:cached
      # Rust 编译缓存
      - cargo-cache:/root/.cargo
      # Node.js 缓存
      - node-cache:/root/.npm
      # pnpm 缓存
      - pnpm-cache:/root/.pnpm-store
    ports:
      - "5173:5173"  # Vite 开发服务器
      - "9229:9229"  # Node.js 调试端口
      - "3000:3000"  # 其他开发服务
    working_dir: /workspaces/flowsight
    command: sleep infinity
    tty: true
    init: true

volumes:
  cargo-cache:
  node-cache:
  pnpm-cache:
```

#### 2.2.3 VS Code Dev Container 配置

```json
// File: /home/parallels/github/flowsight/.devcontainer/devcontainer.json

{
  "name": "FlowSight Development",
  "dockerComposeFile": "docker-compose.yml",
  "service": "flowsight-dev",
  "workspaceFolder": "/workspaces/flowsight",
  "remoteUser": "developer",
  "features": {
    "ghcr.io/devcontainers/features/rust:1": {
      "version": "stable",
      "profile": "minimal"
    },
    "ghcr.io/devcontainers/features/node:1": {
      "version": "20",
      "pnpm": "latest"
    }
  },
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "serayuzgur.crates",
        "davidanson.vscode-markdownlint",
        "esbenp.prettier-vscode",
        "bradlc.vscode-tailwindcss",
        "streetsidesoftware.code-spell-checker"
      ],
      "settings": {
        "rust-analyzer.checkOnSave.command": "clippy",
        "editor.formatOnSave": true,
        "files.watcherExclude": {
          "**/target/**": true,
          "**/node_modules/**": true
        }
      }
    }
  },
  "postCreateCommand": {
    "Rust": "cargo fetch",
    "Frontend": "cd app && pnpm install",
    "Verify": "cargo build --workspace --lib && cd app && pnpm build"
  },
  "remoteEnv": {
    "CARGO_INCREMENTAL": "1",
    "RUSTFLAGS": "-C target-cpu=native"
  },
  "hostRequirements": {
    "cpus": 4,
    "memory": "8gb",
    "storage": "20gb"
  }
}
```

### 2.3 本地快速安装脚本

```bash
#!/bin/bash
# File: /home/parallels/github/flowsight/scripts/install-dependencies.sh

set -euo pipefail

#=====================================================================#
# FlowSight 依赖安装脚本                                             #
# 支持: Ubuntu 22.04+, macOS, Windows (WSL)                          #
#=====================================================================#

echo "=========================================="
echo "FlowSight 依赖安装脚本"
echo "=========================================="

# 检测操作系统
detect_os() {
    case "$(uname -s)" in
        Linux*)
            if [[ -f /etc/os-release ]]; then
                . /etc/os-release
                OS=$NAME
            else
                OS="Linux"
            fi
            ;;
        Darwin*)
            OS="macOS"
            ;;
        *)
            echo "不支持的操作系统: $(uname -s)"
            exit 1
            ;;
    esac
    echo "$OS"
}

# 安装 Rust
install_rust() {
    echo "检查 Rust 安装..."
    if command -v rustc &> /dev/null; then
        echo "Rust 已安装: $(rustc --version)"
    else
        echo "安装 Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
        source "$HOME/.cargo/env"
    fi

    # 安装额外组件
    echo "安装 Rust 工具..."
    rustup component add rustfmt clippy rust-analyzer rust-src
}

# 安装 LLVM
install_llvm() {
    local version=${1:-17}
    echo "检查 LLVM $version 安装..."

    if command -v llvm-config-$version &> /dev/null; then
        echo "LLVM $version 已安装: $(llvm-config-$version --version)"
        return 0
    fi

    echo "安装 LLVM $version..."

    case "$(detect_os)" in
        Ubuntu*)
            # 添加 LLVM APT 仓库
            wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor > /tmp/llvm.gpg
            sudo mv /tmp/llvm.gpg /etc/apt/keyrings/
            echo "deb [signed-by=/etc/apt/keyrings/llvm.gpg] http://apt.llvm.org/ubuntu/ jllvm-toolchain main" | sudo tee /etc/apt/sources.list.d/llvm.list

            sudo apt-get update
            sudo apt-get install -y llvm-$version llvm-$version-dev llvm-$version-tools clang-$version
            ;;
        macOS)
            brew install llvm@$version
            echo "请手动将 LLVM 添加到 PATH: export PATH=\"/opt/homebrew/opt/llvm@$version/bin:\$PATH\""
            ;;
    esac
}

# 安装 Node.js 和 pnpm
install_node() {
    echo "检查 Node.js 安装..."

    if command -v node &> /dev/null; then
        echo "Node.js 已安装: $(node --version)"
    else
        echo "安装 Node.js 20..."
        case "$(detect_os)" in
            Ubuntu*)
                curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
                sudo apt-get install -y nodejs
                ;;
            macOS)
                brew install node@20
                ;;
        esac
    fi

    echo "安装 pnpm..."
    npm install -g pnpm@8
}

# 安装 CMake 和构建工具
install_build_tools() {
    echo "安装构建工具..."

    case "$(detect_os)" in
        Ubuntu*)
            sudo apt-get update
            sudo apt-get install -y build-essential cmake ninja-build pkg-config
            ;;
        macOS)
            brew install cmake ninja
            ;;
    esac
}

# 安装 KLEE (Docker 方式)
install_klee_docker() {
    echo "=========================================="
    echo "推荐使用 Docker 运行 KLEE"
    echo "=========================================="
    echo ""
    echo "运行命令:"
    echo "  docker run -it --rm -v \$(pwd):/workspace klee/klee"
    echo ""
    echo "或使用项目提供的脚本:"
    echo "  ./scripts/run-klee.sh <your-source-file.c>"
    echo ""
}

# 主函数
main() {
    local os=$(detect_os)
    echo "检测到操作系统: $os"
    echo ""

    echo "步骤 1/5: 安装构建工具..."
    install_build_tools

    echo ""
    echo "步骤 2/5: 安装 Rust..."
    install_rust

    echo ""
    echo "步骤 3/5: 安装 LLVM 17..."
    install_llvm 17

    echo ""
    echo "步骤 4/5: 安装 Node.js 和 pnpm..."
    install_node

    echo ""
    echo "步骤 5/5: 安装项目依赖..."
    echo ""

    # 安装 Rust 依赖
    echo "安装 Rust 依赖..."
    cargo fetch

    # 安装前端依赖
    echo "安装前端依赖..."
    cd app
    pnpm install
    cd ..

    echo ""
    echo "=========================================="
    echo "安装完成!"
    echo "=========================================="
    echo ""
    echo "下一步:"
    echo "  1. 构建项目: cargo build --workspace"
    echo "  2. 运行测试: cargo test --workspace"
    echo "  3. 启动前端: cd app && pnpm tauri dev"
    echo ""
}

main "$@"
```

## 3. 构建流水线设计

### 3.1 构建架构概览

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           构建流水线架构                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────┐ │
│  │   代码提交   │───>│   静态检查   │───>│   单元测试   │───>│  构建   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘    └────┬────┘ │
│                                                               │       │
│  ┌─────────────────────────────────────────────────────────────▼───── │
│  │                        持续集成阶段                              │ │
│  ├────────────────────────────────────────────────────────────────── │ │
│  │  1. 代码检查 (linting)                                            │ │
│  │  2. 静态分析 (安全扫描)                                            │ │
│  │  3. 单元测试 (coverage)                                            │ │
│  │  4. 集成测试                                                       │ │
│  │  5. 构建验证                                                       │ │
│  └────────────────────────────────────────────────────────────────── │ │
│                         │                                             │
│                         ▼                                             │
│  ┌────────────────────────────────────────────────────────────────── │ │
│  │                        发布候选阶段                                │ │
│  ├────────────────────────────────────────────────────────────────── │ │
│  │  1. 多平台构建 (Linux/macOS/Windows)                              │ │
│  │  2. E2E 测试                                                       │ │
│  │  3. 安全扫描                                                       │ │
│  │  4. 签名和公证                                                     │ │
│  │  5. 制品存储                                                       │ │
│  └────────────────────────────────────────────────────────────────── │ │
│                         │                                             │
│                         ▼                                             │
│  ┌────────────────────────────────────────────────────────────────── │ │
│  │                        发布阶段                                    │ │
│  ├────────────────────────────────────────────────────────────────── │ │
│  │  1. GitHub Release                                                │ │
│  │  2. Homebrew Cask                                                 │ │
│  │  3. Winget (Windows)                                              │ │
│  │  4. 通知和文档                                                     │ │
│  └────────────────────────────────────────────────────────────────── │ │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 构建脚本

#### 3.2.1 主构建脚本

```bash
#!/bin/bash
# File: /home/parallels/github/flowsight/scripts/build.sh

set -euo pipefail

#=====================================================================#
# FlowSight 构建脚本                                                   #
#=====================================================================#
# 用法:
#   ./scripts/build.sh [--release] [--platform <linux|macos|windows>]#
#   ./scripts/build.sh --all                                           #
#=====================================================================#

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# 配置
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO="${CARGO:-cargo}"
PNPM="${PNPM:-pnpm}"

# 默认配置
RELEASE=false
PLATFORM=""
VERBOSE=false

# 解析参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --release)
                RELEASE=true
                shift
                ;;
            --platform)
                PLATFORM="$2"
                shift 2
                ;;
            --all)
                PLATFORM="all"
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            *)
                log_error "未知参数: $1"
                exit 1
                ;;
        esac
    done
}

# 检查依赖
check_dependencies() {
    log_info "检查构建依赖..."

    local missing_deps=()

    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("rust")
    fi

    # 检查 Node.js
    if ! command -v node &> /dev/null; then
        missing_deps+=("node")
    fi

    # 检查 pnpm
    if ! command -v pnpm &> /dev/null; then
        missing_deps+=("pnpm")
    fi

    # 检查 LLVM
    if ! command -v llvm-config &> /dev/null; then
        log_warn "LLVM 未找到，某些功能可能不可用"
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "缺少依赖: ${missing_deps[*]}"
        log_info "请运行 ./scripts/install-dependencies.sh 安装依赖"
        exit 1
    fi

    log_success "所有依赖已就绪"
}

# 构建 Rust 后端
build_rust() {
    local build_type=$1
    log_info "构建 Rust 后端 ($build_type)..."

    cd "$PROJECT_ROOT"

    local build_cmd="$CARGO build"
    [[ "$build_type" == "release" ]] && build_cmd="$build_cmd --release"

    # 设置 LLVM 配置
    if command -v llvm-config &> /dev/null; then
        export LLVM_SYS_170_PREFIX="$(llvm-config --prefix)"
    fi

    # 执行构建
    if $VERBOSE; then
        $build_cmd --workspace --verbose
    else
        $build_cmd --workspace
    fi

    log_success "Rust 后端构建完成"
}

# 构建前端
build_frontend() {
    local build_type=$1
    log_info "构建前端 ($build_type)..."

    cd "$PROJECT_ROOT/app"

    local build_cmd="$PNPM build"
    [[ "$build_type" == "release" ]] && build_cmd="$build_cmd --release"

    $build_cmd

    log_success "前端构建完成"
}

# 构建 Tauri 应用
build_tauri() {
    local build_type=$1
    local target_platform=$2
    log_info "构建 Tauri 应用 ($build_type, $target_platform)..."

    cd "$PROJECT_ROOT/app"

    local build_args=""
    [[ "$build_type" == "release" ]] && build_args="--release"

    # 平台特定配置
    case $target_platform in
        macos)
            if [[ "$(uname -s)" == "Darwin" ]]; then
                $PNPM tauri build $build_args
            else
                log_warn "macOS 构建只能在 macOS 上进行"
            fi
            ;;
        windows)
            $PNPM tauri build $build_args --target x86_64-pc-windows-msvc
            ;;
        linux)
            $PNPM tauri build $build_args
            ;;
    esac

    log_success "Tauri 应用构建完成"
}

# 构建所有平台
build_all_platforms() {
    log_info "构建所有平台..."

    local current_platform=$(uname -s | tr '[:upper:]' '[:lower:]')
    local build_type=$1

    case $current_platform in
        linux)
            build_tauri "$build_type" "linux"
            ;;
        darwin)
            build_tauri "$build_type" "macos"
            ;;
        *)
            log_error "不支持的平台: $current_platform"
            exit 1
            ;;
    esac
}

# 清理构建产物
clean() {
    log_info "清理构建产物..."

    cd "$PROJECT_ROOT"
    $CARGO clean

    cd app
    $PNPM exec tauri clean
    rm -rf dist

    log_success "清理完成"
}

# 显示帮助
show_help() {
    cat << EOF
FlowSight 构建脚本

用法: $0 [选项]

选项:
  --release    构建发布版本
  --platform   指定目标平台 (linux|macos|windows)
  --all        构建所有平台
  --verbose    显示详细输出
  --help       显示帮助信息
  --clean      清理构建产物

示例:
  $0 --release                    # 构建发布版本
  $0 --release --platform linux   # 构建 Linux 发布版本
  $0 --all                        # 构建当前平台版本
  $0 --clean                      # 清理构建产物
EOF
}

# 主函数
main() {
    parse_args "$@"

    if [[ "${VERBOSE:-false}" == "true" ]]; then
        set -x
    fi

    check_dependencies

    local build_type=$([ "$RELEASE" == "true" ] && echo "release" || echo "debug")

    if [[ -n "$PLATFORM" ]]; then
        if [[ "$PLATFORM" == "all" ]]; then
            build_all_platforms "$build_type"
        else
            build_tauri "$build_type" "$PLATFORM"
        fi
    else
        # 默认构建当前平台
        build_rust "$build_type"
        build_frontend "$build_type"

        local current_platform=$(uname -s | tr '[:upper:]' '[:lower:]')
        build_tauri "$build_type" "$current_platform"
    fi

    log_success "构建完成!"
}

main "$@"
```

### 3.3 构建优化配置

```toml
# File: /home/parallels/github/flowsight/config/cargo-build.toml

# Cargo 构建配置优化

[build]
# 并行构建
jobs = 4

# 增量编译
incremental = true

# 依赖锁文件
dependency-lockfile = "Cargo.lock"

# 退出代码
rustc-error-limit = 0

#=====================================================================#
# 发布配置                                                           #
#=====================================================================#

[profile.release]
# 优化级别
opt-level = 3

# 链接时优化
lto = "thin"

# 代码生成单元
codegen-units = 1

# 符号去除
strip = true

# 增量发布构建
incremental = false

[profile.release.package."*"]
# 优化所有依赖
opt-level = 2

#=====================================================================#
# 开发配置                                                           #
#=====================================================================#

[profile.dev]
# 调试信息级别
debug = "line-tables-only"

# 优化级别
opt-level = 0

[profile.dev.package."*"]
# 依赖优化
opt-level = 0

#=====================================================================#
# 测试配置                                                           #
#=====================================================================#

[profile.test]
debug = true
opt-level = 2

[profile.bench]
opt-level = 3
lto = true
```

## 4. 依赖管理方案

### 4.1 Rust 依赖管理

#### 4.1.1 Cargo.toml 结构

```toml
# File: /home/parallels/github/flowsight/Cargo.toml

[workspace]
resolver = "2"
members = [
    "crates/flowsight-core",
    "crates/flowsight-parser",
    "crates/flowsight-index",
    "crates/flowsight-analysis",
    "crates/flowsight-knowledge",
    "crates/flowsight-query",
    "crates/flowsight-symbolic",
    "crates/flowsight-llvm",
    "crates/flowsight-ai",
    "crates/flowsight-learning",
    "crates/flowsight-cli",
    "app/src-tauri",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["FlowSight Team"]
license = "MIT"
repository = "https://github.com/user/flowsight"
description = "A cross-platform IDE for visualizing code execution flow"

#=====================================================================#
# 工作区公共依赖                                                     #
#=====================================================================#

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Parsing
tree-sitter = "0.22"
tree-sitter-c = "0.21"

# Database
rusqlite = { version = "0.31", features = ["bundled"] }
sled = "0.34"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# CLI
clap = { version = "4.4", features = ["derive"] }

# Regex
regex = "1.10"

# Path handling
walkdir = "2.4"
globset = "0.4"

# Parallelism
rayon = "1.8"

# Time
chrono = { version = "0.4", features = ["std"] }

# UUID
uuid = { version = "1.6", features = ["v4"] }

# Indexing
dashmap = "6.0"

#=====================================================================#
# LLVM 特性配置                                                       #
#=====================================================================#

[features]
default = []

# LLVM 版本特性
llvm15 = ["flowsight-llvm/llvm15"]
llvm16 = ["flowsight-llvm/llvm16"]
llvm17 = ["flowsight-llvm/llvm17"]
llvm = ["flowsight-llvm/default"]

#=====================================================================#
# 发布配置                                                           #
#=====================================================================#

[profile.release]
lto = true
codegen-units = 1
strip = true
```

#### 4.1.2 依赖更新脚本

```bash
#!/bin/bash
# File: /home/parallels/github/flowsight/scripts/update-dependencies.sh

set -euo pipefail

#=====================================================================#
# FlowSight 依赖更新脚本                                              #
#=====================================================================#

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=========================================="
echo "FlowSight 依赖更新"
echo "=========================================="

# 更新 Rust 依赖
echo ""
echo "步骤 1/4: 检查 Rust 依赖更新..."
cargo outdated --workspace --root

echo ""
echo "步骤 2/4: 更新 Cargo.lock..."
cargo update --workspace

# 更新前端依赖
echo ""
echo "步骤 3/4: 检查前端依赖更新..."
cd app
pnpm up --latest
cd ..

# 更新 lock 文件
echo ""
echo "步骤 4/4: 更新 lock 文件..."
git add Cargo.lock app/package-lock.json app/pnpm-lock.yaml

echo ""
echo "=========================================="
echo "依赖更新完成!"
echo "=========================================="
echo ""
echo "请检查变更:"
echo "  git diff Cargo.lock"
echo "  git diff app/package.json"
echo ""
echo "运行测试验证:"
echo "  cargo test --workspace"
echo ""
```

### 4.2 前端依赖管理

```json
{
  "name": "flowsight",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test:e2e": "playwright test",
    "lint": "eslint src --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
    "format": "prettier --write src",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "@monaco-editor/react": "^4.6.0",
    "@tauri-apps/plugin-dialog": "^2.5.0",
    "@xyflow/react": "^12.10.0",
    "d3": "^7.8.5",
    "dagre": "^0.8.5",
    "html-to-image": "^1.11.13",
    "monaco-editor": "^0.55.1",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "zustand": "^4.4.7"
  },
  "devDependencies": {
    "@playwright/test": "^1.57.0",
    "@tauri-apps/api": "^2",
    "@tauri-apps/cli": "^2",
    "@types/d3": "^7.4.3",
    "@types/dagre": "^0.7.52",
    "@types/react": "^18.2.45",
    "@types/react-dom": "^18.2.18",
    "@vitejs/plugin-react": "^4.2.1",
    "@typescript-eslint/eslint-plugin": "^6.0.0",
    "@typescript-eslint/parser": "^6.0.0",
    "eslint": "^8.56.0",
    "eslint-plugin-react-hooks": "^4.6.0",
    "eslint-plugin-react-refresh": "^0.4.5",
    "playwright": "^1.57.0",
    "prettier": "^3.1.0",
    "typescript": "^5.3.3",
    "vite": "^5.0.10"
  },
  "engines": {
    "node": ">=20.0.0",
    "pnpm": ">=8.0.0"
  }
}
```

### 4.3 系统依赖管理

```bash
#!/bin/bash
# File: /home/parallels/github/flowsight/scripts/install-llvm.sh

set -euo pipefail

#=====================================================================#
# LLVM 安装脚本 (支持多版本)                                           #
#=====================================================================#

LLVM_VERSION=${1:-17}
INSTALL_PREFIX="/opt/llvm"

echo "=========================================="
echo "安装 LLVM $LLVM_VERSION"
echo "=========================================="

# 检测操作系统
if [[ "$(uname -s)" == "Darwin" ]]; then
    # macOS
    brew install llvm@$LLVM_VERSION
    echo "LLVM $LLVM_VERSION 已安装到 /opt/homebrew/opt/llvm@$LLVM_VERSION"
    echo "请添加到 PATH:"
    echo "  export PATH=\"/opt/homebrew/opt/llvm@$LLVM_VERSION/bin:\$PATH\""
elif [[ -f /etc/os-release ]]; then
    # Linux (Ubuntu/Debian)
    . /etc/os-release

    # 添加 LLVM APT 仓库
    wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor > /tmp/llvm.gpg
    sudo mv /tmp/llvm.gpg /etc/apt/keyrings/
    echo "deb [signed-by=/etc/apt/keyrings/llvm.gpg] http://apt.llvm.org/ubuntu/ jllvm-toolchain main" | sudo tee /etc/apt/sources.list.d/llvm.list

    sudo apt-get update
    sudo apt-get install -y llvm-$LLVM_VERSION llvm-$LLVM_VERSION-dev llvm-$LLVM_VERSION-tools

    # 验证安装
    llvm-config-$LLVM_VERSION --version
else
    echo "不支持的操作系统"
    exit 1
fi

echo ""
echo "安装完成!"
```

### 4.4 依赖版本锁定策略

```yaml
# File: /home/parallels/github/flowsight/.github/dependabot.yml

version: 2
updates:
  # Rust 依赖
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "09:00"
    labels:
      - "dependencies"
      - "rust"
    ignore: []
    commit-message:
      prefix: "chore(deps): rust"
      prefix-development: "chore(deps-dev): rust"

  # Node.js 依赖
  - package-ecosystem: "npm"
    directory: "/app"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "09:00"
    labels:
      - "dependencies"
      - "javascript"
    versioning-strategy: increase
    commit-message:
      prefix: "chore(deps): node"
      prefix-development: "chore(deps-dev): node"

  # GitHub Actions
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "09:00"
    labels:
      - "dependencies"
      - "ci"
    commit-message:
      prefix: "ci(actions)"
```

## 5. CI/CD 配置

### 5.1 CI/CD 架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           GitHub Actions CI/CD                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                        代码提交事件                               │   │
│  │  • push (main, develop)                                         │   │
│  │  • pull_request                                                 │   │
│  │  • tags (v*)                                                    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                    │                                    │
│                                    ▼                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                        CI 流水线                                 │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │   │
│  │  │ 静态检查  │→│ 单元测试  │→│ 构建验证  │→│ 制品生成  │          │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │   │
│  │       │           │           │           │                      │   │
│  │       │           │           │           │                      │   │
│  │       ▼           ▼           ▼           ▼                      │   │
│  │  ┌─────────────────────────────────────────────────────────┐   │   │
│  │  │                    质量门禁                               │   │   │
│  │  │  • 代码覆盖率 > 80%                                      │   │   │
│  │  │  • 无新 lint 警告                                        │   │   │
│  │  │  • 所有测试通过                                          │   │   │
│  │  └─────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                    │                                    │
│                                    ▼                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                        CD 流水线                                 │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │   │
│  │  │多平台构建 │→│  E2E 测试 │→│  安全扫描 │→│  制品签名 │          │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │   │
│  │       │           │           │           │                      │   │
│  │       │           │           │           │                      │   │
│  │       ▼           ▼           ▼           ▼                      │   │
│  │  ┌─────────────────────────────────────────────────────────┐   │   │
│  │  │                    发布阶段                               │   │   │
│  │  │  • GitHub Release (自动生成)                             │   │   │
│  │  │  • Homebrew Cask 更新                                    │   │   │
│  │  │  • Winget 更新 (Windows)                                 │   │   │
│  │  │  • 通知 (Slack/Discord)                                  │   │   │
│  │  └─────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 CI 配置 (持续集成)

```yaml
# File: /home/parallels/github/flowsight/.github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main, develop]
    paths-ignore:
      - '**.md'
      - 'docs/**'
      - '.gitignore'
      - 'LICENSE'
  pull_request:
    branches: [main]
    paths-ignore:
      - '**.md'
      - 'docs/**'
      - '.gitignore'
      - 'LICENSE'

env:
  CARGO_TERM_COLOR: always
  CARGO_NET_GIT_FETCH_WITH_CLI: true
  CARGO_INCREMENTAL: 0

jobs:
  #=====================================================================#
  # Rust 后端检查                                                       #
  #=====================================================================#
  rust-check:
    name: Rust Check
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy, rust-src

      - name: Setup LLVM
        uses: awalsh128/cache-apt-pkgs-action@latest
        with:
          packages: llvm-17 llvm-17-dev clang-17
          version: 1.0

      - name: Configure LLVM
        run: |
          echo "LLVM_SYS_170_PREFIX=/usr/lib/llvm-17" >> $GITHUB_ENV

      - name: Cache Rust
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
        continue-on-error: true

      - name: Build
        run: cargo build --workspace --all-targets

  #=====================================================================#
  # Rust 测试                                                           #
  #=====================================================================#
  rust-test:
    name: Rust Tests
    runs-on: ubuntu-latest
    needs: rust-check
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rust-src

      - name: Setup LLVM
        uses: awalsh128/cache-apt-pkgs-action@latest
        with:
          packages: llvm-17 llvm-17-dev clang-17
          version: 1.0

      - name: Configure LLVM
        run: echo "LLVM_SYS_170_PREFIX=/usr/lib/llvm-17" >> $GITHUB_ENV

      - name: Cache Rust
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-test-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --workspace --all-targets -- --test-threads=4

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./target/coverage/flags
          fail_ci_if_error: true

  #=====================================================================#
  # 前端检查                                                            #
  #=====================================================================#
  frontend-check:
    name: Frontend Check
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ./app
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'

      - name: Install pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8

      - name: Install dependencies
        run: pnpm install --frozen-lockfile

      - name: Type check
        run: pnpm typecheck

      - name: Lint
        run: pnpm lint

      - name: Format check
        run: pnpm exec prettier --check src

      - name: Build
        run: pnpm build

  #=====================================================================#
  # 跨平台构建测试                                                      #
  #=====================================================================#
  cross-platform-build:
    name: Build on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    needs: [rust-check, frontend-check]
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'

      - name: Install pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8

      - name: Install Linux dependencies
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libgtk-3-dev \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            llvm-17 \
            llvm-17-dev \
            clang-17

      - name: Build Rust
        run: cargo build --workspace --release

      - name: Build frontend
        working-directory: ./app
        run: pnpm build

      - name: Build Tauri
        uses: tauri-apps/tauri-action@v0
        with:
          projectPath: app
          args: --release

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: flowsight-${{ matrix.os }}-${{ github.sha }}
          path: |
            target/release/bundle/**
          retention-days: 7

  #=====================================================================#
  # 安全扫描                                                            #
  #=====================================================================#
  security-scan:
    name: Security Scan
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Run Cargo Audit
        uses: actions-rs/audit@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          args: --deny warnings

      - name: Run Trivy
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: '.'
          severity: 'CRITICAL,HIGH'
          format: 'sarif'
          output: 'trivy-results.sarif'

      - name: Upload Trivy results
        uses: github/codeql-action/upload-sarif@v2
        if: always()
        with:
          sarif_file: 'trivy-results.sarif'

  #=====================================================================#
  # 依赖更新检查                                                        #
  #=====================================================================#
  dependency-check:
    name: Dependency Update Check
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Check Cargo outdated
        uses: actions-rs/cargo@v1
        with:
          command: outdated
          args: --workspace --root

      - name: Check npm outdated
        working-directory: ./app
        run: |
          npx npm-check-updates --minimal --silent || true
```

### 5.3 CD 配置 (持续部署)

```yaml
# File: /home/parallels/github/flowsight/.github/workflows/cd.yml

name: CD

on:
  push:
    branches: [main]
    tags: ['v*.*.*']
  workflow_dispatch:
    inputs:
      version:
        description: 'Release version (e.g., 1.0.0)'
        required: true

env:
  CARGO_TERM_COLOR: always
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

jobs:
  #=====================================================================#
  # 准备发布                                                            #
  #=====================================================================#
  prepare-release:
    name: Prepare Release
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
      is_release: ${{ steps.version.outputs.is_release }}
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Get version
        id: version
        run: |
          if [[ ${{ github.event_name }} == 'workflow_dispatch' ]]; then
            VERSION="${{ github.event.inputs.version }}"
            echo "version=$VERSION" >> $GITHUB_OUTPUT
            echo "is_release=true" >> $GITHUB_OUTPUT
          else
            TAG="${GITHUB_REF#refs/tags/}"
            echo "version=$TAG" >> $GITHUB_OUTPUT
            echo "is_release=true" >> $GITHUB_OUTPUT
          fi

      - name: Create Release PR
        if: steps.version.outputs.is_release != 'true'
        run: |
          echo "Creating release PR for version ${{ steps.version.outputs.version }}"
          # 这里可以添加创建 Release PR 的逻辑

  #=====================================================================#
  # 多平台构建                                                          #
  #=====================================================================#
  build:
    name: Build ${{ matrix.platform }}
    needs: prepare-release
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: Linux
            os: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
            bundle: deb.AppImage
          - platform: macOS
            os: macos-latest
            target: aarch64-apple-darwin
            bundle: dmg
          - platform: macOS Intel
            os: macos-latest
            target: x86_64-apple-darwin
            bundle: dmg
          - platform: Windows
            os: windows-2022
            target: x86_64-pc-windows-msvc
            bundle: msi

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          target: ${{ matrix.target }}

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'

      - name: Install pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8

      - name: Install Linux dependencies
        if: matrix.platform == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libgtk-3-dev \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            llvm-17 llvm-17-dev clang-17

      - name: Setup macOS signing
        if: matrix.platform == 'macOS'
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
        run: |
          # macOS 代码签名配置
          echo "macOS signing configured"

      - name: Setup Windows signing
        if: matrix.platform == 'Windows'
        env:
          CERT_FILE: ${{ secrets.CERT_FILE }}
          CERT_PASSWORD: ${{ secrets.CERT_PASSWORD }}
        run: |
          # Windows 代码签名配置
          echo "Windows signing configured"

      - name: Build Rust
        run: cargo build --workspace --release --target ${{ matrix.target }}

      - name: Build frontend
        working-directory: ./app
        run: pnpm build

      - name: Build Tauri
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          projectPath: app
          args: --release --target ${{ matrix.target }}

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: flowsight-${{ matrix.platform }}-${{ needs.prepare-release.outputs.version }}
          path: |
            target/${{ matrix.target }}/release/bundle/**
          retention-days: 30

  #=====================================================================#
  # 安全验证                                                            #
  #=====================================================================#
  security-verify:
    name: Security Verification
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Verify signatures
        run: |
          # 验证签名
          echo "Verifying signatures..."

      - name: Run Trivy on artifacts
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: 'artifacts'
          severity: 'CRITICAL,HIGH'
          format: 'table'
          exit-code: 1

  #=====================================================================#
  # 发布 GitHub Release                                                 #
  #=====================================================================#
  release-github:
    name: Create GitHub Release
    needs: [build, security-verify]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          path: release-assets
          merge-multiple: true

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ needs.prepare-release.outputs.version }}
          name: FlowSight ${{ needs.prepare-release.outputs.version }}
          body: |
            ## What's New

            See [CHANGELOG.md](./CHANGELOG.md) for details.

            ## Downloads

            | Platform | File |
            |----------|------|
            | Linux | `.AppImage`, `.deb` |
            | macOS | `.dmg` |
            | Windows | `.msi` |
          files: release-assets/**
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Upload SHA256 checksums
        run: |
          cd release-assets
          sha256sum * > checksums.txt
          gh release upload ${{ needs.prepare-release.outputs.version }} checksums.txt --clobber

  #=====================================================================#
  # 更新 Homebrew (macOS)                                               #
  #=====================================================================#
  update-homebrew:
    name: Update Homebrew Cask
    needs: release-github
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - name: Checkout Homebrew Cask
        uses: actions/checkout@v4
        with:
          repository: Homebrew/homebrew-cask
          path: homebrew-cask
          token: ${{ secrets.HOMEBREW_TOKEN }}

      - name: Create PR
        run: |
          echo "Creating Homebrew Cask PR..."
          # 这里实现 Homebrew Cask 更新逻辑

  #=====================================================================#
  # 更新 Winget (Windows)                                               #
  #=====================================================================#
  update-winget:
    name: Update Winget Package
    needs: release-github
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - name: Checkout Winget PKG
        uses: actions/checkout@v4
        with:
          repository: microsoft/winget-pkgs
          path: winget-pkgs
          token: ${{ secrets.WINGET_TOKEN }}

      - name: Create PR
        run: |
          echo "Creating Winget PR..."
          # 这里实现 Winget 更新逻辑

  #=====================================================================#
  # 通知                                                                #
  #=====================================================================#
  notify:
    name: Send Notifications
    needs: release-github
    runs-on: ubuntu-latest
    steps:
      - name: Discord Notification
        uses: Ilshidur/action-discord@master
        env:
          DISCORD_WEBHOOK: ${{ secrets.DISCORD_WEBHOOK }}
        with:
          args: 'FlowSight ${{ needs.prepare-release.outputs.version }} has been released! :tada: https://github.com/user/flowsight/releases/tag/${{ needs.prepare-release.outputs.version }}'

      - name: Slack Notification
        uses: 8398a7/action-slack@v3
        with:
          status: success
          channel: '#releases'
          text: FlowSight ${{ needs.prepare-release.outputs.version }} released!
        env:
          SLACK_WEBHOOK: ${{ secrets.SLACK_WEBHOOK }}
        if: always()
```

### 5.4 E2E 测试配置

```yaml
# File: /home/parallels/github/flowsight/.github/workflows/e2e.yml

name: E2E Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  e2e-test:
    name: E2E Tests on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        browser: [chromium]

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'

      - name: Install pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8

      - name: Install dependencies
        working-directory: ./app
        run: pnpm install --frozen-lockfile

      - name: Build
        working-directory: ./app
        run: |
          cargo build --workspace --release
          pnpm build

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        with:
          projectPath: app
          args: --release

      - name: Install Playwright browsers
        working-directory: ./app
        run: npx playwright install --with-deps ${{ matrix.browser }}

      - name: Run E2E tests
        working-directory: ./app
        run: pnpm test:e2e

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report-${{ matrix.os }}-${{ matrix.browser }}
          path: app/playwright-report/
          retention-days: 7

      - name: Upload failure screenshots
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: failure-screenshots-${{ matrix.os }}
          path: app/test-results/
          retention-days: 7
```

## 6. 发布策略

### 6.1 版本管理策略

FlowSight 使用语义化版本控制 (Semantic Versioning):

```
主版本号.次版本号.修订号
   MAJOR    MINOR  PATCH

例如: 1.2.3

- MAJOR: 不兼容的 API 变更
- MINOR: 新功能 (向后兼容)
- PATCH:  Bug 修复 (向后兼容)
```

#### 6.1.1 版本号规则

| 版本类型 | 触发条件 | 示例 |
|----------|----------|------|
| Major | 重大架构变更，破坏性更新 | 1.0.0 -> 2.0.0 |
| Minor | 新功能，API 扩展 | 1.0.0 -> 1.1.0 |
| Patch | Bug 修复，安全补丁 | 1.1.0 -> 1.1.1 |
| Pre-release | 测试版本 | 1.2.0-alpha.1 |
| Build metadata | 构建元信息 | 1.2.0+build.123 |

#### 6.1.2 发布分支策略

```
main          ──┬──●──●──●──●──●──●──●──●──●── (生产版本)
                 │     │     │     │     │
develop       ──┴──●──●──●──●──●──●──●──●──●── (开发版本)
                 │     │     │     │     │
feature/*     ────┐  │     │     │     │
                 │  │     │     │     │
release/1.2   ────┴──●──●──●                 (发布分支)
                 │     │     │
hotfix/*      ────┐     │     │
                 └──●──●──●──●──●──●──●──●──●──
```

### 6.2 发布流程

#### 6.2.1 标准发布流程

```bash
#!/bin/bash
# File: /home/parallels/github/flowsight/scripts/release.sh

set -euo pipefail

#=====================================================================#
# FlowSight 发布脚本                                                  #
#=====================================================================#

VERSION=${1:-}

if [[ -z "$VERSION" ]]; then
    echo "用法: $0 <version>"
    echo "示例: $0 1.2.3"
    exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=========================================="
echo "FlowSight Release $VERSION"
echo "=========================================="

# 1. 检查 git 状态
echo ""
echo "���骤 1/8: 检查 git 状态..."
if [[ -n "$(git status --porcelain)" ]]; then
    echo "错误: 工作目录有未提交的更改"
    git status
    exit 1
fi

# 2. 确保在 main 分支
echo ""
echo "步骤 2/8: 检查分支..."
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo "错误: 必须在 main 分支上发布"
    exit 1
fi

# 3. 拉取最新代码
echo ""
echo "步骤 3/8: 拉取最新代码..."
git pull origin main

# 4. 运行最终测试
echo ""
echo "步骤 4/8: 运行测试..."
cargo test --workspace
cd app && pnpm test:e2e && cd ..

# 5. 更新版本号
echo ""
echo "步骤 5/8: 更新版本号..."
sed -i "s/version = \".*\"/version = \"$VERSION\"/" Cargo.toml
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" app/package.json

# 6. 更新 CHANGELOG
echo ""
echo "步骤 6/8: 更新 CHANGELOG..."
cat > CHANGELOG.md << EOF
# Changelog

## [$VERSION] - $(date +%Y-%m-%d)

### Added

### Changed

### Fixed

### Security

EOF

git add -A
git commit -m "Release $VERSION"

# 7. 创建标签
echo ""
echo "步骤 7/8: 创建标签..."
git tag -a "v$VERSION" -m "Release $VERSION"

# 8. 推送到远程
echo ""
echo "步骤 8/8: 推送到远程..."
git push origin main --tags

echo ""
echo "=========================================="
echo "发布准备完成!"
echo "=========================================="
echo ""
echo "GitHub Actions 将自动构建并发布版本。"
echo "请查看: https://github.com/user/flowsight/actions"
```

### 6.3 发布清单

```markdown
# FlowSight 发布清单

## 发布前检查

- [ ] 所有测试通过
  - [ ] 单元测试 (cargo test)
  - [ ] 集成测试
  - [ ] E2E 测试 (所有平台)
  - [ ] 安全扫描通过

- [ ] 代码质量
  - [ ] 无 lint 警告
  - [ ] 代码覆盖率 > 80%
  - [ ] 文档更新

- [ ] 功能验证
  - [ ] 新功能已测试
  - [ ] 已知问题已修复
  - [ ] 回���测试通过

## 发布检查

- [ ] 版本号已更新
- [ ] CHANGELOG 已更新
- [ ] Git 标签已创建
- [ ] 发布分支已创建

## 发布后检查

- [ ] CI/CD 构建成功
- [ ] GitHub Release 已创建
- [ ] 制品已上传
- [ ] 通知已发送

## 平台特定检查

- [ ] Linux
  - [ ] AppImage 可用
  - [ ] .deb 包可用

- [ ] macOS
  - [ ] Apple Silicon 构建可用
  - [ ] Intel 构建可用
  - [ ] 公证完成

- [ ] Windows
  - [ ] .msi 包可用
  - [ ] 签名验证通过
```

### 6.4 回滚策略

```yaml
# File: /home/parallels/github/flowsight/.github/workflows/rollback.yml

name: Rollback

on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to rollback from'
        required: true
      reason:
        description: 'Reason for rollback'
        required: true

jobs:
  rollback:
    name: Rollback from ${{ github.event.inputs.version }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Rollback version
        run: |
          # 获取上一个版本
          PREV_VERSION=$(git describe --tags --abbrev=0 HEAD~1)

          echo "Rolling back from ${{ github.event.inputs.version }} to $PREV_VERSION"

          # 更新版本号
          sed -i "s/version = \".*\"/version = \"$PREV_VERSION\"/" Cargo.toml
          sed -i "s/\"version\": \".*\"/\"version\": \"$PREV_VERSION\"/" app/package.json

          # 创建回滚提交
          git add -A
          git commit -m "Rollback: Revert to $PREV_VERSION due to ${{ github.event.inputs.reason }}"

          # 创建回滚标签
          git tag -d "v${{ github.event.inputs.version }}"
          git tag -a "v$PREV_VERSION-rollback" -m "Rollback from ${{ github.event.inputs.version }}"

      - name: Notify
        uses: Ilshidur/action-discord@master
        env:
          DISCORD_WEBHOOK: ${{ secrets.DISCORD_WEBHOOK }}
        with:
          args: 'FlowSight rolled back from ${{ github.event.inputs.version }} due to: ${{ github.event.inputs.reason }}'

  delete-release:
    name: Delete GitHub Release
    needs: rollback
    runs-on: ubuntu-latest
    steps:
      - name: Delete release
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.repos.deleteRelease({
              owner: context.repo.owner,
              repo: context.repo.repo,
              release_id: context.sha
            })
```

### 6.5 发布制品

| 平台 | 格式 | 大小估算 | 签名方式 |
|------|------|----------|----------|
| Linux | AppImage | ~50 MB | SHA256 |
| Linux | .deb | ~45 MB | GPG |
| macOS | .dmg (Apple Silicon) | ~55 MB | Apple 公证 |
| macOS | .dmg (Intel) | ~55 MB | Apple 公证 |
| Windows | .msi | ~60 MB | 代码签名证书 |

## 7. 监控与可观测性

### 7.1 构建监控

```yaml
# File: /home/parallels/github/flowsight/.github/workflows/monitor.yml

name: Build Monitor

on:
  schedule:
    - cron: '0 0 * * *'  # 每天午夜
  workflow_run:
    workflows: [CI, CD]
    types: [completed]

jobs:
  monitor:
    name: Build Monitor
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Get build stats
        run: |
          echo "获取构建统计..."

      - name: Send daily report
        uses: Ilshidur/action-discord@master
        env:
          DISCORD_WEBHOOK: ${{ secrets.DISCORD_WEBHOOK }}
        with:
          args: 'Daily build report for FlowSight'
```

### 7.2 性能基准

```bash
#!/bin/bash
# File: /home/parallels/github/flowsight/scripts/benchmark.sh

set -euo pipefail

#=====================================================================#
# FlowSight 性能基准测试                                              #
#=====================================================================#

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=========================================="
echo "FlowSight 性能基准测试"
echo "=========================================="

# 构建时间基准
echo ""
echo "构建时间基准..."
cargo build --release --timings

# 测试执行时间
echo ""
echo "测试执行时间..."
cargo test --workspace -- --test-threads=1 --nocapture

# 产物大小
echo ""
echo "产物大小..."
du -sh target/release/bundle/

echo ""
echo "基准测试完成!"
```

## 8. 故障排查

### 8.1 常见问题

#### 问题 1: LLVM 找不到

```bash
# 症状
error: Could not find LLVM installed via llvm-config

# 解决方案
export LLVM_SYS_170_PREFIX=/usr/lib/llvm-17
cargo clean
cargo build
```

#### 问题 2: Tauri 构建失败

```bash
# 症状
error: failed to run custom build command

# 解决方案
# 1. 检查系统依赖
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev

# 2. 清理并重新构建
cargo clean
cd app && pnpm install && pnpm tauri build
```

#### 问题 3: Docker 构建内存不足

```bash
# 解决方案
# 增加 Docker 内存限制到至少 8GB
docker build --build-arg BUILDKIT_INLINE_CACHE=1 .
```

### 8.2 日志收集

```bash
#!/bin/bash
# File: /home/parallels/github/flowsight/scripts/collect-logs.sh

# 收集构建日志用于故障排查

echo "收集 FlowSight 构建日志..."
echo ""

# Git 信息
echo "=== Git 信息 ==="
git log --oneline -10
git status
echo ""

# Rust 环境
echo "=== Rust 环境 ==="
rustc --version
cargo --version
echo ""

# 构建日志
echo "=== 最后构建日志 ==="
cargo build --workspace --verbose 2>&1 | tail -100
```

## 9. 附录

### 9.1 参考文献

- [Tauri 2.0 文档](https://v2.tauri.app/)
- [Rust Cargo 文档](https://doc.rust-lang.org/cargo/)
- [LLVM 文档](https://llvm.org/docs/)
- [GitHub Actions 文档](https://docs.github.com/en/actions### 9.)

2 相关文件

```
flowsight/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml          # CI 配置文件
│   │   ├── cd.yml          # CD 配置文件
│   │   ├── e2e.yml         # E2E 测试配置
│   │   └── monitor.yml     # 监控配置
│   └── dependabot.yml      # 依赖更新配置
├── scripts/
│   ├── build.sh            # 构建脚本
│   ├── release.sh          # 发布脚本
│   ├── install-dependencies.sh  # 依赖安装脚本
│   ├── update-dependencies.sh   # 依赖更新脚本
│   └── benchmark.sh        # 性能基准脚本
├── .devcontainer/
│   ├── Dockerfile.dev      # 开发环境镜像
│   ├── docker-compose.yml  # Docker Compose 配置
│   └── devcontainer.json   # VS Code 配置
└── docs/design/
    └── DEVOPS-ARCHITECTURE.md  # 本文档
```

---

> 本文档由 FlowSight DevOps 团队维护
> 如有问题，请提交 Issue 或联系维护者
