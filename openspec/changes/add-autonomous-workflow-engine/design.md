# Autonomous Workflow Engine - Design Document

## Context

The current Oxide implementation uses a simple persona-switching mechanism (`SubagentManager`) and a linear message loop. This lacks the self-correcting, multi-step execution capabilities seen in state-of-the-art agents like Claude Code. Users need a system that can autonomously break down complex tasks, execute them in multiple steps, observe results, and adapt its approach based on feedback.

## Goals

- **Autonomous Task Execution**: Implement a Plan-Act-Observe-Reflect (PAOR) loop that can execute multi-step tasks without constant user intervention
- **Intelligent Task Decomposition**: Break down complex requests into manageable sub-tasks with dependency tracking
- **Adaptive Feedback Loop**: Learn from execution results and adjust plans accordingly
- **Enhanced HITL**: Transform Human-In-The-Loop from simple allow/deny into collaborative guidance mechanism
- **Result Aggregation**: Provide concise, useful summaries instead of raw tool outputs

## Non-Goals

- **Full Autonomy**: The system still requires user approval for potentially destructive operations (HITL remains)
- **AGI Capabilities**: This is a practical engineering tool, not a general AI system
- **Real-time Streaming**: Initial implementation focuses on correctness over low-latency streaming

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────┐
│                  WorkflowOrchestrator               │
│  - Manages PAOR cycle                               │
│  - Tracks workflow state                            │
│  - Coordinates sub-agents and tools                 │
└────────────┬────────────────────────────────────────┘
             │
    ┌────────┴────────┬───────────────┬───────────────┐
    │                 │               │               │
┌───▼────┐      ┌────▼─────┐   ┌────▼──────┐  ┌─────▼─────┐
│ State  │      │Observation│   │ Task      │  │ Subagent  │
│Machine │      │Collector  │   │ Registry  │  │ Manager   │
└────────┘      └───────────┘   └───────────┘  └───────────┘
```

### State Machine

The workflow progresses through these phases:

```
Idle → Planning → Acting → Observing → Reflecting
         ↑                                  │
         └──────────(loop if not done)──────┘
                        │
                   ┌────┴────┐
                   ↓         ↓
               Complete   Failed
```

**State Transitions**:

- **Idle → Planning**: User initiates a request
- **Planning → Acting**: Plan is generated
- **Acting → Observing**: Tools/sub-agents execute
- **Observing → Reflecting**: Results are collected
- **Reflecting → Planning**: Goal not yet achieved, replan
- **Reflecting → Complete**: Goal achieved successfully
- **Reflecting → Failed**: Unrecoverable error or max iterations reached

### Data Structures

#### WorkflowState

```rust
pub struct WorkflowState {
    phase: WorkflowPhase,           // Current phase
    iteration: u32,                  // Current iteration count
    max_iterations: u32,             // Maximum allowed iterations
    user_request: String,            // Original user request
    started_at: SystemTime,          // Workflow start time
    updated_at: SystemTime,          // Last update time
    requires_user_intervention: bool,// Needs user input?
    failure_reason: Option<String>,  // Why it failed
}
```

#### Task

```rust
pub struct Task {
    id: TaskId,
    description: String,
    parent_id: Option<TaskId>,       // Sub-task relationship
    dependencies: Vec<TaskId>,       // Task dependencies
    status: TaskStatus,              // Pending/Running/Completed/Failed
    result: Option<String>,          // Execution result
    error: Option<String>,           // Error if failed
}
```

#### Observation

```rust
pub struct Observation {
    observation_type: String,        // tool_execution, subagent_result, etc.
    source: String,                  // Which tool/agent
    input: HashMap<...>,             // Input parameters
    output: Option<Value>,           // Output data
    success: bool,                   // Success flag
    error: Option<String>,           // Error message if any
    execution_time_ms: Option<u64>,  // How long it took
}
```

#### Reflection

```rust
pub struct Reflection {
    goal_achieved: bool,             // Is the goal met?
    progress: f32,                   // Progress 0.0 to 1.0
    content: String,                 // Reflection analysis
    next_action: Option<String>,     // Suggested next step
    requires_user_intervention: bool,// Need human help?
    issues: Vec<String>,             // Problems encountered
}
```

## Decisions

### Decision 1: PAOR Loop over Event-Driven Architecture

**Chosen**: Implement a synchronous PAOR loop with explicit state transitions

**Alternatives Considered**:

- Event-driven async architecture (more complex, harder to debug)
- Reactive streams (overkill for current use case)

**Rationale**:

- Easier to reason about and debug
- Simpler integration with existing codebase
- Clear state transitions make the workflow transparent
- Can add async capabilities later if needed

### Decision 2: Embedded State Machine vs External Workflow Engine

**Chosen**: Embedded state machine within `WorkflowOrchestrator`

**Alternatives Considered**:

- Use external workflow engine like Temporal (too heavyweight)
- Actor model framework (unnecessary complexity)

**Rationale**:

- Keeps dependencies minimal
- Full control over behavior
- Easier to customize for AI agent use case
- Lower operational complexity

### Decision 3: Observation Collection Strategy

**Chosen**: Thread-safe collector with Arc<RwLock<>>

**Alternatives Considered**:

- Event bus/message queue
- Database-backed observation store

**Rationale**:

- In-memory is fast enough for current scale
- Simple implementation
- Easy to extend to persistent storage later

### Decision 4: Task Decomposition Format

**Chosen**: Tree structure with explicit dependencies

**Alternatives Considered**:

- Flat list with ordering constraints
- DAG (Directed Acyclic Graph) representation

**Rationale**:

- Tree is intuitive for AI to generate
- Supports hierarchical task breakdown
- Simple dependency tracking
- Can be visualized easily

## Integration Points

### 1. SubagentManager Enhancement

```rust
// Before: Simple switching
subagent_manager.switch_to(AgentType::Explore)?;

// After: Delegated execution
let result = orchestrator.delegate_to_subagent(
    AgentType::Explore,
    task_description,
).await?;
```

### 2. Tool Execution Hooks

All tools will report back through observation collector:

```rust
// Pseudo-code for tool wrapper
fn execute_tool(tool: &Tool, params: Params) -> Result<Output> {
    let start = Instant::now();
    let result = tool.run(params.clone());

    observation_collector. add_tool_execution(
        tool.name(),
        params,
        result.as_ref().ok(),
        result.is_ok(),
        result.as_ref().err().map(|e| e.to_string()),
        Some(start.elapsed().as_millis() as u64),
    );

    result
}
```

### 3. HITL Integration

Enhanced HITL that supports guidance:

```rust
pub enum HitlResponse {
    Allow,                          // Proceed as planned
    Deny(String),                   // Block with reason
    Suggest(String),                // Alternative approach
    ModifyPlan(Plan),               // Updated plan
    AskForClarification(String),    // Need more info
}
```

## Implementation Phases

### Phase 1: Core Infrastructure ✅

- [x] Define state machine and data structures
- [x] Implement `WorkflowOrchestrator` skeleton
- [x] Create `ObservationCollector`
- [x] Set up module structure

### Phase 2: Planning & Reflection (Current)

- [ ] Integrate LLM calls for plan generation
- [ ] Implement reflection logic
- [ ] Add goal achievement detection

### Phase 3: Execution & Observation

- [ ] Tool execution wrapper with observation
- [ ] Sub-agent delegation mechanism
- [ ] Result aggregation

### Phase 4: HITL Enhancement

- [ ] Extend HITL to support suggestions
- [ ] Implement path correction feedback
- [ ] Add collaborative decision making

### Phase 5: Testing & Refinement

- [ ] Integration tests for multi-step tasks
- [ ] Termination condition validation
- [ ] Performance optimization

## Risks & Mitigations

### Risk 1: Infinite Loops

**Risk**: Agent gets stuck in Planning-Acting-Reflecting loop without progress

**Mitigation**:

- Hard limit on iterations (default: 15)
- Progress tracking in reflections
- Detect repeated states and trigger intervention

### Risk 2: Context Window Exhaustion

**Risk**: Long-running workflows accumulate too much history for LLM context

**Mitigation**:

- Summarization at each reflection step
- Keep only recent N observations
- Hierarchical summarization for deep task trees

### Risk 3: Tool Execution Failures

**Risk**: Tool failures derail entire workflow

**Mitigation**:

- Automatic retry logic (configurable)
- Graceful degradation
- Clear error reporting in observations
- Allow user to fix and resume

### Risk 4: Performance Degradation

**Risk**: Synchronous PAOR loop blocks user interaction

**Mitigation**:

- Phase 1: Accept synchronous behavior
- Phase 2: Add async execution where beneficial
- Use progress indicators
- Allow user to pause/resume

## Migration Plan

### Backward Compatibility

Existing code continues to work unchanged:

- `SubagentManager` API remains stable
- `AgentBuilder` maintains current interface
- New workflow features are opt-in

### Adoption Path

1. **Internal Use**: Test with development team
2. **Opt-in Flag**: Add `--autonomous` CLI flag
3. **Gradual Rollout**: Enable for specific task types
4. **Full Integration**: Make default after validation

### Rollback Strategy

If issues arise:

- Feature flag can disable workflow engine
- Falls back to simple message loop
- No data loss (observations optional)

## Open Questions

1. **Context Management**: How should we handle very long conversation histories?
   - Current thinking: Summarize after each iteration
2. **Parallel Execution**: Should independent tasks run in parallel?
   - Current thinking: Phase 1 is sequential, add parallel later
3. **Persistence**: Should workflow state persist across restarts?
   - Current thinking: Not in v1, add in future if needed

4. **Metrics**: What metrics should we collect?
   - Success rate, average iterations, time per phase, etc.

## Success Criteria

- [ ] Successfully execute multi-step refactoring tasks (e.g., "rename function X to Y across codebase")
- [ ] Demonstrate self-correction (plan adjustment based on observations)
- [ ] User feedback indicates improved task completion rate
- [ ] No regression in existing functionality
- [ ] Loop termination works reliably (no stuck agents)

## References

- OpenSpec Workflow Engine Spec: `openspec/changes/add-autonomous-workflow-engine/specs/workflow-engine/spec.md`
- Implementation: `src/agent/workflow/`
- Tests: `src/agent/workflow/*/tests`
