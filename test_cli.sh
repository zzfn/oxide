#!/bin/bash
# 测试 Oxide CLI 的 AI 集成

echo "🧪 测试 Oxide CLI AI 集成"
echo ""

# 检查环境变量
if [ -z "$OXIDE_AUTH_TOKEN" ] && [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "❌ 错误: 未设置 API Key"
    echo "   请运行: export OXIDE_AUTH_TOKEN=your_api_key"
    exit 1
fi

echo "✅ API Key 已设置"
echo ""

# 构建项目
echo "📦 构建项目..."
cargo build --bin oxide --quiet
if [ $? -ne 0 ]; then
    echo "❌ 构建失败"
    exit 1
fi
echo "✅ 构建成功"
echo ""

# 运行 CLI（非交互模式测试）
echo "🚀 启动 Oxide CLI"
echo "   提示: 输入 'hello' 测试 AI 响应"
echo "   提示: 输入 '/help' 查看命令"
echo "   提示: 按 Ctrl+C 两次退出"
echo ""

./target/debug/oxide
