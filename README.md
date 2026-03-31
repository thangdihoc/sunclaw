# Sunclaw

Sunclaw is a Rust-first AI agent runtime with:
- plugin-based core contracts,
- runtime guardrails (turn/tool limits),
- allowlist policy with optional skill-level restrictions,
- auditable tool decisions,
- role-based team flow primitives.

## Workspace crates

- `sunclaw-core`: shared contracts, domain types, and error types.
- `sunclaw-runtime`: runtime execution loop (provider/policy/tool/memory/audit).
- `sunclaw-policy`: allowlist-based policy engine.
- `sunclaw-orchestrator`: planner/executor/reviewer team flow schema.
- `sunclaw-skills`: skill manifest schema + validation.
- `sunclaw-app`: composition root using in-memory adapters.
- `sunclaw-cli`: executable demo.

## Quick start

```bash
cargo run -p sunclaw-cli -- "hello"
cargo run -p sunclaw-cli -- "tool: ping"
```

Sample output:

```txt
Sunclaw received: hello
(turns=1, tool_calls=0)
```

## Development checks

```bash
cargo fmt --all
cargo test --workspace
cargo run -p sunclaw-cli -- "tool: ping"
```

## Near-term roadmap

1. Add OpenAI-compatible and Anthropic-compatible provider adapters.
2. Add SQLite memory and durable audit storage.
3. Add HTTP adapter and request tracing.
4. Add sandboxed tool execution wrappers.
