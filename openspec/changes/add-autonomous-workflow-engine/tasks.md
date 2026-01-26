## 1. Design & Specification

- [x] 1.1 Define the `workflow-engine` capability spec
- [x] 1.2 Design the state machine for the autonomous loop

## 2. Core Implementation

- [x] 2.1 Refactor `AgentBuilder` to support loop-aware agents
- [x] 2.2 Implement the `WorkflowOrchestrator` to manage the cycle
- [x] 2.3 Update `SubagentManager` to support delegation rather than just switching

## 3. Tool & HITL Integration

- [x] 3.1 Update HITL system to support path-correction feedback
- [x] 3.2 Ensure all tools report observation data back to the orchestrator

## 4. Verification

- [x] 4.1 Create integration tests for multi-step tasks (e.g., "Find bug and fix it")
- [x] 4.2 Validate the loop termination logic
