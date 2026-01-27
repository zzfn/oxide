# Change: 移除 /agent switch 命令并明确主 agent 单实例模式

## Why

当前 CLI 暴露了 `/agent switch`，但实现是占位且不会真正重建 Agent，容易造成误解。目标是移除该命令，并明确主 agent 为单实例模式，子 agent 只用于后续主 agent 内部调用。

## What Changes

- 移除 CLI 的 `/agent switch` 子命令入口与帮助提示
- `/agent` 仅保留 `list` 与 `capabilities` 用于查看能力
- 文档中明确不支持手动切换 Agent（仅主 agent 单实例）

## Impact

- Affected specs: agent-cli
- Affected code: `src/cli/command.rs`, `src/cli/mod.rs`, `docs/agent-system.md`, `README.md`
