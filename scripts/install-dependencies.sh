#!/bin/bash
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

# 安装 Tauri 桌面依赖
install_tauri_deps() {
    echo "安装 Tauri 桌面依赖..."

    case "$(detect_os)" in
        Ubuntu*)
            sudo apt-get install -y \
                libgtk-3-dev \
                libwebkit2gtk-4.1-dev \
                libappindicator3-dev \
                librsvg2-dev \
                patchelf
            ;;
        macOS)
            echo "macOS 无需额外依赖"
            ;;
    esac
}

# 主函数
main() {
    local os=$(detect_os)
    echo "检测到操作系统: $os"
    echo ""

    echo "步骤 1/6: 安装构建工具..."
    install_build_tools

    echo ""
    echo "步骤 2/6: 安装 Rust..."
    install_rust

    echo ""
    echo "步骤 3/6: 安装 LLVM 17..."
    install_llvm 17

    echo ""
    echo "步骤 4/6: 安装 Tauri 依赖..."
    install_tauri_deps

    echo ""
    echo "步骤 5/6: 安装 Node.js 和 pnpm..."
    install_node

    echo ""
    echo "步骤 6/6: 安装项目依赖..."
    echo ""

    # 安装 Rust 依赖
    echo "安装 Rust 依赖..."
    cargo fetch

    # 安装前端依赖
    echo "安装前端依赖..."
    if [[ -f app/package.json ]]; then
        cd app && pnpm install && cd ..
    fi

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
