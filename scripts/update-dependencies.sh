#!/bin/bash
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
cargo outdated --workspace --root || true

echo ""
echo "步骤 2/4: 更新 Cargo.lock..."
cargo update --workspace

# 更新前端依赖
echo ""
echo "步骤 3/4: 检查前端依赖更新..."
cd app
pnpm up --latest || true
cd ..

# 更新 lock 文件
echo ""
echo "步骤 4/4: 更新 lock 文件..."
git add Cargo.lock app/package-lock.json app/pnpm-lock.yaml 2>/dev/null || true

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
