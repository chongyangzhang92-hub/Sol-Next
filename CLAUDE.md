# CLAUDE.md

See [`AGENTS.md`](AGENTS.md) and [`llms.txt`](llms.txt) for complete, authoritative instructions for **Sol Next** (`v9.3.1-public`, package `ai.sol.next`, independent Apache-2.0 Android AI OS Agent).

## Quick Reference for Claude Code / AI Coding Assistants

- **Goal**: Install, verify, or automate **Sol Next Public Edition** (`ai.sol.next`, free built-in Rust-encrypted DeepSeek V4.1 Flash + universal BYOK) on an ADB-connected Android 10–16 or HarmonyOS device.
- **Zero-dependency Rust 2024 CLI** (always prefer `--json`, never interactive; build once with `cargo build --release --bins`):
  - Check device: `./target/release/sol-cli status --json`
  - Install verified APK: `./target/release/sol-cli install --verify-sha256 --grant-permissions --json`
  - Start 1:1 VirtualDisplay helper: `./target/release/sol-cli start-helper --json`
  - Run phone task: `./target/release/sol-cli run --intent "<task>" --json`
- **MCP Server**: `./target/release/sol-mcp` (zero-dependency Rust 2024 stdio JSON-RPC 2.0).
- **Hard Guardrails**:
  - Never hardcode or commit plaintext API keys, secrets, or private server URLs.
  - Package ID is `ai.sol.next`.
  - Sol Next is a completely independent product created by 许昌北海医院张重阳 (`chongyangzhang92-hub`).
