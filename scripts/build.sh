#!/bin/bash
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
            --help)
                show_help
                exit 0
                ;;
            --clean)
                clean
                exit 0
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
