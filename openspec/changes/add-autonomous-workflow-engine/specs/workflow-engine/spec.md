## ADDED Requirements

### Requirement: Autonomous Execution Loop

The system SHALL implement a "Plan-Act-Observe-Reflect" (PAOR) loop for processing user requests. This loop allows the agent to execute multiple tool calls autonomously until a goal is reached or it requires user intervention.

#### Scenario: Multi-step file modification

- **WHEN** user asks "Find all usages of function X and rename to Y"
- **THEN** the system SHALL first PLAN (search for usages), then ACT (edit files), then OBSERVE (check for errors), and REFLECT (verify if all instances are handled) before finishing.

### Requirement: Task Decomposition and Delegation

The main Agent MUST be able to break down complex goals into smaller sub-tasks and delegate them to specialized sub-agents or tool sequences.

#### Scenario: Delegating exploration

- **WHEN** the Main Agent determines a large codebase needs to be understood
- **THEN** it SHALL delegate the task to an "Explore" sub-agent and wait for the summarized findings.

### Requirement: Collaborative HITL Feedback

Human-In-The-Loop (HITL) SHALL NOT just be a binary "Allow/Deny" for tools, but a mechanism for the user to provide corrective guidance during the Observe/Reflect phases.

#### Scenario: User corrects search path

- **WHEN** the Agent observes a search returned too many results
- **AND** the user during HITL suggests "Search only in /src/utils"
- **THEN** the Agent SHALL update its NEXT PLAN based on this feedback.

### Requirement: Result Aggregation

The system SHALL aggregate the history of the PAOR loop into a concise final summary for the user, hiding excessive technical intermediate outputs unless requested.

#### Scenario: Summarizing a bug fix

- **WHEN** a 5-step loop (search, read, fix, test, verify) completes
- **THEN** the final response SHALL summarize the change and confirmation of fix, rather than dumping all terminal outputs.
