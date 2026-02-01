#!/bin/bash
# 安装 Git Hooks
#
# 运行方式：
#   ./scripts/install-hooks.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
GIT_HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "🔧 Installing FlowSight Git Hooks..."
echo ""

# 检查是否在 Git 仓库中
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo "❌ Not a Git repository"
    exit 1
fi

# 创建 hooks 目录（如果不存在）
mkdir -p "$GIT_HOOKS_DIR"

# 安装 pre-commit hook
echo "📝 Installing pre-commit hook..."
cp "$SCRIPT_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
chmod +x "$GIT_HOOKS_DIR/pre-commit"
echo "   ✅ pre-commit installed"

# 创建 pre-push hook
echo "📝 Creating pre-push hook..."
cat > "$GIT_HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash
# FlowSight Pre-push Hook
#
# 在推送前运行完整测试

set -e

echo "🔍 FlowSight Pre-push Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 运行完整的 Rust 测试
echo ""
echo "🧪 Running full Rust tests..."
if ! cargo test --workspace 2>/dev/null; then
    echo "❌ Rust tests failed"
    exit 1
fi
echo "✅ Rust tests passed"

# 运行后端集成测试
echo ""
echo "🔴 Running backend integration tests..."
cd app
if ! npx tsx tests/integration/real-backend-test.ts 2>/dev/null; then
    echo "❌ Backend integration tests failed"
    echo "   This is critical - real backend functionality may be broken"
    exit 1
fi
echo "✅ Backend integration tests passed"
cd ..

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All pre-push checks passed!"
echo ""
EOF
chmod +x "$GIT_HOOKS_DIR/pre-push"
echo "   ✅ pre-push installed"

echo ""
echo "🎉 Git hooks installed successfully!"
echo ""
echo "Installed hooks:"
echo "  - pre-commit: Quick checks before commit"
echo "  - pre-push: Full tests before push"
echo ""
echo "To skip hooks (not recommended):"
echo "  git commit --no-verify"
echo "  git push --no-verify"
echo ""
