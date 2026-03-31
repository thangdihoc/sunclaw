# Sunclaw

Sunclaw is a Rust-first AI agent runtime with a plugin core, policy enforcement, and role-based orchestration.

## Current scaffold

- `sunclaw-core`: shared contracts and domain types.
- `sunclaw-runtime`: single-pass runtime loop.
- `sunclaw-policy`: allowlist policy implementation.
- `sunclaw-orchestrator`: planner/executor/reviewer flow model.
- `sunclaw-skills`: skill manifest schema.
- `sunclaw-app`: composition root with mock provider + memory + tool.
- `sunclaw-cli`: executable demo.

## Quick start

```bash
cargo run -p sunclaw-cli -- "hello"
cargo run -p sunclaw-cli -- "tool: ping"
```

## Next milestones

1. Replace mock provider with OpenAI-compatible provider crate.
2. Replace in-memory store with SQLite + migrations.
3. Add HTTP channel adapter and tracing.
4. Add sandboxed tool execution.
