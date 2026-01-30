#!/bin/bash

# 测试工具调用功能

echo "测试 Oxide CLI 工具调用功能"
echo "================================"
echo ""

# 创建测试文件
echo "创建测试文件..."
echo "Hello from test file!" > /tmp/oxide_test.txt

# 测试 CLI（使用 expect 或手动输入）
echo "启动 Oxide CLI..."
echo "请输入: 读取 /tmp/oxide_test.txt 文件的内容"
echo ""

cd /Users/c.chen/dev/oxide
./target/debug/oxide
