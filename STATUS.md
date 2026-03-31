# Sunclaw Progress Tracker

> This file is updated at the end of each implementation step.

## Progress Snapshot

- Overall completion (toward v0.1 MVP): **45%**
- Current focus: **Model routing + runtime hardening**
- Last completed milestone: **Multi-model profile routing and fallback provider registry**

## Last Completed Work

- Added provider routing crate (`sunclaw-provider`) with profile routes and fallback behavior.
- Added `model_profile` support in runtime context.
- Wired demo app + CLI to select model profile (`--profile`).

## Next Plan (Immediate)

1. Implement real provider adapters (OpenRouter-compatible, Gemini-compatible, xAI-compatible) using HTTP clients.
2. Add provider configuration loader from `configs/default.toml` instead of hardcoded routes.
3. Add integration test covering route selection by profile and failover error classes.

## Reporting Rule

For every completed task, report in this format:

1. **Tiến trình hiện tại** (what % done and what was completed)
2. **Kế hoạch tiếp theo** (top 3 next tasks)
3. **Rủi ro/chặn** (if any)
