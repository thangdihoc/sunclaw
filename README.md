# Sunclaw

Sunclaw is a Rust-first AI agent runtime with:
- plugin-based core contracts,
- runtime guardrails (turn/tool limits),
- allowlist policy with optional skill-level restrictions,
- auditable tool decisions,
- role-based team flow primitives,
- multi-agent coordination (Hierarchical & Sequential),
- multi-model routing profiles (OpenRouter/Gemini/xAI style backends).

## Workspace crates

- `sunclaw-core`: shared contracts, domain types, and error types.
- `sunclaw-provider`: provider routing + fallback registry.
- `sunclaw-runtime`: runtime execution loop (provider/policy/tool/memory/audit).
- `sunclaw-policy`: allowlist-based policy engine.
- `sunclaw-orchestrator`: multi-agent coordination (Hierarchical & Sequential).
- `sunclaw-tools`: official tool ecosystem (WebSearch, etc.).
- `sunclaw-skills`: skill manifest schema + validation.
- `sunclaw-app`: composition root using in-memory adapters.
- `sunclaw-cli`: executable demo.

## Multi-Agent Orchestration

Sunclaw supports complex agentic workflows:
- **Hierarchical**: A Supervisor agent acts as a planner, delegating sub-tasks to specialized Worker agents (wrapped as tools).
- **Sequential**: A linear chain of agents (TeamFlow) where each agent's output feeds the next.

## Plugin SDK

Creating new tools is easy with the `sunclaw_tool!` macro. It automatically generates JSON Schema and handles argument parsing:

```rust
sunclaw_tool!(
    MyTool, Args, "tool_name", "Description...", 
    self_obj, args, {
        Ok(ToolResult { output: "..." })
    }
);
```

## Quick start

```bash
cargo run -p sunclaw-cli -- "hỏi gì đó"
cargo run -p sunclaw-cli -- --profile reasoning "giải quyết vấn đề"
cargo run -p sunclaw-cli -- --team "Nghiên cứu về Rust 1.80"
```

## Near-term roadmap

1. Add Bridge adapters (Discord, Zalo, Telegram).
2. Advanced tool sandboxing (WebAssembly/MicroVMs).
3. Distributed orchestrator state (Redis/Postgres).
4. v0.1 Release and crates.io publishing.

## Development checks

```bash
cargo fmt --all
cargo test --workspace
cargo run -p sunclaw-cli -- --profile default "force_provider_fail"
```
