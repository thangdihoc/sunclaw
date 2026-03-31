# Sunclaw Architecture Overview

Sunclaw uses a trait-first architecture so integrations stay replaceable:

1. `sunclaw-core` defines contracts for provider, policy, tool, memory, and audit.
2. `sunclaw-provider` handles model routing profiles and provider failover.
3. `sunclaw-runtime` executes a guarded loop with turn/tool limits.
4. `sunclaw-policy` validates tool permissions globally and by skill profile.
5. `sunclaw-skills` validates manifest metadata (risk level, required tools).
6. `sunclaw-orchestrator` models planner -> executor -> reviewer handoff flow.
7. `sunclaw-app` wires in-memory demo adapters and exposes a runnable runtime.

## Runtime loop

`ingest user message -> choose model route -> provider decision -> policy gate -> tool execution -> persist system message -> provider decision ... -> final reply`

The runtime stores audit events whenever tool calls are allowed/denied to keep decision history inspectable.
