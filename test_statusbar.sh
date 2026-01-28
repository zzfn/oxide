#!/bin/bash
# 自动化测试脚本

echo "启动测试..."
echo ""

# 使用 expect 自动化交互
expect << 'EOF'
set timeout 10
spawn cargo run --example statusbar_test

expect "按 Ctrl+D 退出测试"
sleep 2

# 发送第一条输入
send "hello\r"
expect "输入: hello"
sleep 1

# 发送第二条输入
send "test\r"
expect "输入: test"
sleep 1

# 发送 check 命令
send "check\r"
expect "检查滚动区域"
sleep 1

# 发送多行测试
send "line1\r"
sleep 0.5
send "line2\r"
sleep 0.5
send "line3\r"
sleep 1

# 退出
send "\x04"
expect eof
EOF

echo ""
echo "测试完成"
