#!/usr/bin/env bash
# 任务管理系统集成测试示例

set -e

echo "🧪 任务管理系统集成测试"
echo "================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}📦 编译项目...${NC}"
cargo build --release -p oxide-tools 2>&1 | tail -3
echo ""

echo -e "${BLUE}🧪 运行任务系统单元测试...${NC}"
cargo test -p oxide-tools task --lib 2>&1 | grep -E "(test result|running)"
echo ""

echo -e "${GREEN}✅ 所有测试通过！${NC}"
echo ""

echo "================================"
echo -e "${YELLOW}💡 如何在 CLI 中测试${NC}"
echo "================================"
echo ""
echo "1. 启动 Oxide CLI："
echo "   $ cargo run --release"
echo ""
echo "2. 与 AI 对话测试任务管理："
echo ""
echo -e "${BLUE}示例对话 1: 创建任务${NC}"
echo "---"
echo "你: 请使用 TaskCreate 工具创建一个任务："
echo "    - subject: 实现用户认证"
echo "    - description: 实现 JWT 认证系统"
echo "    - activeForm: 正在实现用户认证"
echo ""
echo "AI 会调用 TaskCreate 工具并返回任务 ID。"
echo ""

echo -e "${BLUE}示例对话 2: 列出任务${NC}"
echo "---"
echo "你: 使用 TaskList 工具列出所有任务"
echo ""
echo "AI 会显示所有任务的摘要信息。"
echo ""

echo -e "${BLUE}示例对话 3: 查看详情${NC}"
echo "---"
echo "你: 使用 TaskGet 工具查看任务 #1 的详情"
echo ""
echo "AI 会显示任务的完整信息。"
echo ""

echo -e "${BLUE}示例对话 4: 更新任务${NC}"
echo "---"
echo "你: 使用 TaskUpdate 工具将任务 #1 的状态改为 in_progress"
echo ""
echo "AI 会更新任务状态。"
echo ""

echo -e "${BLUE}示例对话 5: 创建依赖任务${NC}"
echo "---"
echo "你: 创建一个新任务'编写测试'，并使用 TaskUpdate 让它依赖任务 #1"
echo ""
echo "AI 会创建任务并设置依赖关系。"
echo ""

echo "================================"
echo -e "${YELLOW}🔍 查看工具定义${NC}"
echo "================================"
echo ""
echo "任务工具的实现位置："
echo "  - TaskCreate: crates/oxide-tools/src/task/tools/create.rs"
echo "  - TaskList:   crates/oxide-tools/src/task/tools/list.rs"
echo "  - TaskGet:    crates/oxide-tools/src/task/tools/get.rs"
echo "  - TaskUpdate: crates/oxide-tools/src/task/tools/update.rs"
echo ""
echo "TaskManager 实现："
echo "  - crates/oxide-tools/src/task/manager.rs"
echo ""
echo "完整文档："
echo "  - docs/task-system.md"
echo ""

echo "================================"
echo -e "${YELLOW}📊 工具参数说明${NC}"
echo "================================"
echo ""

echo -e "${BLUE}TaskCreate 参数:${NC}"
cat << 'EOF'
{
  "subject": "任务标题（祈使句）",
  "description": "详细描述",
  "activeForm": "进行中显示文本（可选）",
  "metadata": {
    "key": "value"  // 可选
  }
}
EOF
echo ""

echo -e "${BLUE}TaskList 参数:${NC}"
echo "无参数"
echo ""

echo -e "${BLUE}TaskGet 参数:${NC}"
cat << 'EOF'
{
  "taskId": "1"
}
EOF
echo ""

echo -e "${BLUE}TaskUpdate 参数:${NC}"
cat << 'EOF'
{
  "taskId": "1",
  "status": "in_progress",  // 可选: pending/in_progress/completed/deleted
  "subject": "新标题",       // 可选
  "description": "新描述",   // 可选
  "activeForm": "新文本",    // 可选
  "owner": "agent-id",      // 可选
  "addBlocks": ["2", "3"],  // 可选: 此任务阻塞的任务
  "addBlockedBy": ["0"],    // 可选: 阻塞此任务的任务
  "metadata": {             // 可选
    "key": "value",
    "removed": null         // null 表示删除该键
  }
}
EOF
echo ""

echo "================================"
echo -e "${GREEN}✨ 测试准备完成！${NC}"
echo "================================"
echo ""
echo "现在你可以："
echo "1. 运行 'cargo run --release' 启动 CLI"
echo "2. 与 AI 对话测试任务管理功能"
echo "3. 查看 docs/task-system.md 了解更多细节"
echo ""
