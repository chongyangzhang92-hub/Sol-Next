# CLAUDE.md

See [`AGENTS.md`](AGENTS.md) and [`llms.txt`](llms.txt) for complete, authoritative instructions for **Sol Next** (`v9.0.4-public`, package `ai.sol.next`, independent Apache-2.0 Android AI OS Agent).

## Quick Reference for Claude Code / AI Coding Assistants

- **Goal**: Install, verify, or automate **Sol Next Public Edition** (`ai.sol.next`, free built-in Rust-encrypted DeepSeek V4.1 Flash + universal BYOK) on an ADB-connected Android 10–16 or HarmonyOS device.
- **Zero-dependency CLI** (always prefer `--json`, never interactive):
  - Check device: `python3 scripts/sol-cli.py status --json`
  - Install verified APK: `python3 scripts/sol-cli.py install --verify-sha256 --grant-permissions --json`
  - Start 1:1 VirtualDisplay helper: `python3 scripts/sol-cli.py start-helper --json`
  - Run phone task: `python3 scripts/sol-cli.py run --intent "<task>" --json`
- **MCP Server**: `python3 mcp/sol_mcp_server.py` (stdlib-only stdio JSON-RPC 2.0).
- **Hard Guardrails**:
  - Never hardcode or commit plaintext API keys, secrets, or private server URLs.
  - Package ID is `ai.sol.next`.
  - Sol Next is a completely independent product created by 许昌北海医院张重阳 (`chongyangzhang92-hub`).
