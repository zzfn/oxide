## ADDED Requirements

### Requirement: Autonomous Execution Loop

The system SHALL implement a "Plan-Act-Observe-Reflect" (PAOR) loop for processing user requests. This loop allows the agent to execute multiple tool calls autonomously until a goal is reached or it requires user intervention.

#### Scenario: Multi-step file modification

- **WHEN** user asks "Find all usages of function X and rename to Y"
- **THEN** the system SHALL first PLAN (search for usages), then ACT (edit files), then OBSERVE (check for errors), and REFLECT (verify if all instances are handled) before finishing.

#### Scenario: Maximum iterations reached

- **WHEN** the workflow loop executes for more than MAX_ITERATIONS (default: 15)
- **THEN** the system SHALL terminate and report progress to the user

### Requirement: Workflow State Machine

The system SHALL maintain a state machine with the following states: Idle, Planning, Acting, Observing, Reflecting, Complete, Failed.

#### Scenario: State transitions

- **WHEN** the orchestrator starts a new task
- **THEN** it SHALL transition from Idle -> Planning -> Acting -> Observing -> Reflecting
- **AND** it SHALL loop back to Planning if the goal is not achieved
- **AND** it SHALL transition to Complete when the goal is achieved
- **AND** it SHALL transition to Failed if an unrecoverable error occurs

### Requirement: Task Decomposition and Delegation

The main Agent MUST be able to break down complex goals into smaller sub-tasks and delegate them to specialized sub-agents or tool sequences.

#### Scenario: Delegating exploration

- **WHEN** the Main Agent determines a large codebase needs to be understood
- **THEN** it SHALL delegate the task to an "Explore" sub-agent and wait for the summarized findings.

#### Scenario: Sub-task dependency tracking

- **WHEN** a task is decomposed into sub-tasks
- **THEN** the system SHALL track dependencies between sub-tasks
- **AND** it SHALL execute sub-tasks in dependency order

### Requirement: Observation Data Collection

The system SHALL collect observation data from all tool executions and sub-agent completions, including success/failure status, output, and metadata.

#### Scenario: Tool execution observation

- **WHEN** a tool completes execution
- **THEN** the system SHALL record the tool name, input parameters, output, execution time, and success status
- **AND** it SHALL make this observation data available to the Reflect phase

#### Scenario: Error observation

- **WHEN** a tool or sub-agent fails
- **THEN** the system SHALL capture the error message and context
- **AND** it SHALL include this in the observation data for reflection

### Requirement: Collaborative HITL Feedback

Human-In-The-Loop (HITL) SHALL NOT just be a binary "Allow/Deny" for tools, but a mechanism for the user to provide corrective guidance during the Observe/Reflect phases.

#### Scenario: User corrects search path

- **WHEN** the Agent observes a search returned too many results
- **AND** the user during HITL suggests "Search only in /src/utils"
- **THEN** the Agent SHALL update its NEXT PLAN based on this feedback.

#### Scenario: User provides alternative approach

- **WHEN** the Agent is stuck in a reflection loop
- **AND** the user provides an alternative approach during HITL
- **THEN** the Agent SHALL incorporate the suggestion into its next plan

### Requirement: Result Aggregation

The system SHALL aggregate the history of the PAOR loop into a concise final summary for the user, hiding excessive technical intermediate outputs unless requested.

#### Scenario: Summarizing a bug fix

- **WHEN** a 5-step loop (search, read, fix, test, verify) completes
- **THEN** the final response SHALL summarize the change and confirmation of fix, rather than dumping all terminal outputs.

#### Scenario: Verbose mode requested

- **WHEN** the user requests verbose output
- **THEN** the system SHALL include detailed intermediate steps and observations in the final summary

### Requirement: Loop Termination Conditions

The system SHALL terminate the PAOR loop when one of the following conditions is met: goal achieved, maximum iterations reached, user intervention required, or unrecoverable error occurred.

#### Scenario: Goal achieved

- **WHEN** the Reflect phase determines the goal is fully achieved
- **THEN** the system SHALL transition to Complete state and report success

#### Scenario: User intervention required

- **WHEN** the Reflect phase determines it cannot proceed without user input
- **THEN** the system SHALL pause and prompt the user for guidance
