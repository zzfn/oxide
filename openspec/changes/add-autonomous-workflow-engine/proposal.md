# Change: Add Autonomous Workflow Engine

## Why

Current Oxide implementation relies on simple persona switching (SubagentManager) and a linear message loop. It lacks a self-correcting, multi-step execution cycle compared to state-of-the-art agents like Claude Code. This proposal introduces an autonomous "Plan-Act-Observe-Reflect" workflow to enable deep reasoning and reliable multi-step task completion.

## What Changes

- **Autonomous Loop Orchestrator**: Implement a state machine that drives the Plan -> Act -> Observe -> Reflect cycle.
- **Task Decomposition**: Main agent can now create a plan (task tree) and track progress.
- **Enhanced HITL integration**: Move HITL from simple tool interception to a collaborative feedback mechanism within the workflow loop.
- **Result Aggregation**: Automatically summarize intermediate tool outputs and sub-agent findings into a cohesive user response.

## Impact

- Affected specs: `workflow-engine` (New)
- Affected code: `src/agent/mod.rs`, `src/agent/builder.rs`, `src/agent/subagent.rs`, `src/cli/mod.rs`
