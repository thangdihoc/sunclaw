# Sunclaw

Sunclaw is a Rust-first AI agent runtime with:
- plugin-based core contracts,
- runtime guardrails (turn/tool limits),
- allowlist policy with optional skill-level restrictions,
- auditable tool decisions,
- role-based team flow primitives,
- multi-model routing profiles (OpenRouter/Gemini/xAI style backends).

## Workspace crates

- `sunclaw-core`: shared contracts, domain types, and error types.
- `sunclaw-provider`: provider routing + fallback registry.
- `sunclaw-runtime`: runtime execution loop (provider/policy/tool/memory/audit).
- `sunclaw-policy`: allowlist-based policy engine.
- `sunclaw-orchestrator`: planner/executor/reviewer team flow schema.
- `sunclaw-skills`: skill manifest schema + validation.
- `sunclaw-app`: composition root using in-memory adapters.
- `sunclaw-cli`: executable demo.

## Quick start

```bash
cargo run -p sunclaw-cli -- "hello"
cargo run -p sunclaw-cli -- --profile reasoning "hello"
cargo run -p sunclaw-cli -- --profile cheap "tool: ping"
```

Sample output:

```txt
profile=reasoning
[openrouter:deepseek-r1] Sunclaw received: hello
(turns=1, tool_calls=0)
```

## Development checks

```bash
cargo fmt --all
cargo test --workspace
cargo run -p sunclaw-cli -- --profile default "force_provider_fail"
```

## Near-term roadmap

1. Replace mock backends with real OpenRouter/Gemini/xAI provider HTTP clients.
2. Add SQLite memory and durable audit storage.
3. Add HTTP adapter and request tracing.
4. Add sandboxed tool execution wrappers.
