# Sunclaw Architecture Overview

Sunclaw uses a trait-first architecture so integrations stay replaceable:

1. `sunclaw-core`: defines contracts for provider, policy, tool, memory, and audit.
2. `sunclaw-provider`: handles model routing profiles and provider failover.
3. `sunclaw-runtime`: executes a guarded loop with turn/tool limits.
4. `sunclaw-policy`: validates tool permissions globally and by skill profile.
5. `sunclaw-orchestrator`: coordinate multiple agents using Hierarchical (Supervisor) or Sequential (TeamFlow) patterns.
6. `sunclaw-tools`: official tool ecosystem (WebSearch, etc.) using the Plugin SDK.
7. `sunclaw-app`: composition root for the CLI and server.

## Architectural Layers

### 1. Core Layer
Contains the building blocks: `Tool`, `MemoryStore`, `ModelProvider`, etc. All other crates depend on this.

### 2. Runtime Layer
The heart of Sunclaw. It orchestrates a single agent's reasoning loop, enforcing guardrails and policies.

### 3. Orchestration Layer (`sunclaw-orchestrator`)
Supports complex workflows by treating agents as tools. This allows for:
- **Hierarchical Teams**: A Supervisor agent planning and delegating tasks.
- **Sequential Teams**: Fixed chains of responsibility.

### 4. Integration Layer
Contains adapters for storage (SQLite), communication (Telegram, Discord), and tools (Search, Code Exec).

## Data Flow

`User Input -> Orchestrator (Optional) -> Runtime -> Provider -> Policy Gate -> Tool Exec -> Audit -> Memory -> Final Output`

The runtime stores audit events whenever tool calls are allowed/denied to keep decision history inspectable.
