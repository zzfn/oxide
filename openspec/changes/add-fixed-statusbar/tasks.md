# 实现任务清单

## 阶段 1：核心状态栏模块

- [ ] 创建 `src/cli/statusbar.rs` 模块
  - 定义 `StatusBar` 结构体和 `StatusData` 数据结构
  - 实现终端尺寸查询（使用 crossterm）
  - 实现 ANSI 转义序列辅助函数（保存/恢复光标、移动光标、设置滚动区域）

- [ ] 实现 `StatusBar::init()` 方法
  - 查询终端尺寸
  - 设置滚动区域（保留底部 1 行给状态栏）
  - 清空状态栏区域

- [ ] 实现 `StatusBar::update()` 方法
  - 保存当前光标位置
  - 移动光标到底部状态栏行
  - 渲染状态信息（带背景色）
  - 恢复光标位置

- [ ] 实现 `StatusBar::cleanup()` 方法
  - 重置滚动区域到全屏
  - 清除状态栏内容
  - 移动光标到底部

## 阶段 2：集成到 OxideCli 与 Reedline

- [ ] 在 `OxideCli` 结构体中添加 `statusbar: StatusBar` 字段

- [ ] 在 `OxideCli::new()` 中初始化状态栏
  - 创建 `StatusBar` 实例
  - 调用 `init()` 设置滚动区域（DECSTBM）

- [ ] 在 `OxideCli::run()` 的 Reedline 循环中集成
  - 每次 `read_line()` 前调用 `statusbar.update()`
  - 确保状态栏在用户输入前是最新的

- [ ] 添加退出清理（CRITICAL）
  - 实现 `StatusBarCleanup` Drop guard
  - 确保 `cleanup()` 在任何退出路径（正常/Ctrl+C/panic）都被调用

- [ ] 在流式输出中集成状态栏刷新
  - 在 `stream_with_animation` 中，限流更新（每 100ms 或每 10 token）
  - 传递当前 Token 数据、会话 ID、模型名称等

## 阶段 3：状态数据收集

- [ ] 定义 `StatusData` 结构体
  - `total_tokens: u64` - 总 Token 使用量
  - `session_id: String` - 会话 ID
  - `model_name: String` - 模型名称
  - `cwd: PathBuf` - 当前工作目录

- [ ] 在 `OxideCli` 中实现 `get_status_data()` 方法
  - 从 `total_tokens` 原子变量读取
  - 从 `context_manager` 获取会话 ID
  - 返回 `StatusData` 实例

## 阶段 4：终端尺寸变化处理（CRITICAL）

- [ ] 实现 `StatusBar::handle_resize()` 方法
  - 更新内部尺寸字段
  - 重新调用 `init()` 调整滚动区域

- [ ] 使用 crossterm EventStream 监听 Resize 事件
  - 在后台 tokio 任务中监听 `Event::Resize`
  - 触发 `handle_resize()` 重新初始化

- [ ] 测试窗口缩放场景
  - 验证状态栏位置正确调整
  - 验证滚动区域正确重新划分

## 阶段 5：优雅降级与配置

- [ ] 添加终端能力检测
  - 检测是否支持 ANSI 转义序列
  - 检测终端类型（通过 `TERM` 环境变量）
  - 不支持时禁用状态栏

- [ ] 添加配置选项（可选）
  - 在 `config.toml` 中添加 `statusbar.enabled` 配置
  - 支持通过命令行参数 `--no-statusbar` 禁用

## 阶段 6：测试与验证

- [ ] 手动测试
  - 在不同终端模拟器中测试（iTerm2, Terminal.app, Alacritty 等）
  - 测试终端尺寸变化场景
  - 测试长时间运行和大量 Token 消耗场景

- [ ] 边界情况测试
  - 终端高度不足（< 5 行）时的处理
  - 非交互式环境（管道、重定向）中的行为
  - 不支持 ANSI 的终端（如 dumb terminal）

- [ ] 性能验证
  - 确保状态栏更新不影响流式输出性能
  - 验证无闪烁和撕裂现象

## 验收标准

所有任务完成后，验证以下场景：

1. 启动 Oxide，状态栏显示在底部，显示初始状态
2. 发送消息，对话内容在上方滚动，状态栏保持固定
3. Token 使用量实时更新
4. 调整终端窗口大小，状态栏自动适配
5. 退出程序，终端状态正确恢复
6. 在不支持 ANSI 的终端中，程序正常运行（无状态栏）
