# AGENTS.md — AI Agent Guide for Sol Next (Public Edition)

> Sol Next 9.1.2-public (Build 912, package `ai.sol.next`, zero built-in API keys) ships the rebuilt grounded multimodal OS agent (`SolGroundedAgent`, 10 orthogonal typed tools, live Accessibility handle grounding, 1:1 Set-of-Mark vision), Studio minimalist UI, Rust v5.00 16KB JNI kernel, and 9 verified WeChat 8.0.80 reliability fixes (99/99 unit tests passing). Same signing certificate as prior public releases for in-place upgrades.

This repository distributes **Sol Next `v9.1.2-public`** — a zero-built-in-key (BYOK) Android 10–16 AI phone agent (Apache-2.0) — a brand-new, independent next-generation mobile AI OS Agent created by 许昌北海医院张重阳 with free built-in Rust-encrypted DeepSeek V4.1 Flash, 20-language phone-language auto-detection, Wi-Fi-off Shell persistence, and multi-stream accelerated OTA.

Read this file before installing, automating, documenting, or integrating with Sol Next.

---

## 1. Identity & Key Facts (Do Not Hallucinate)

- **Product Name**: Sol Next (`Public Edition`, BYOK)
- **Android Package ID**: `ai.sol.next`
  - **IMPORTANT**: Do **NOT** use `com.wangzan101.mika.dev` or `com.wangzan101.mika` when automating the Public Edition. The Public Edition package ID is strictly `ai.sol.next`.
- **Main Activity**: `ai.sol.next/ai.sol.next.canvas.SolCanvasActivity`
- **Minimum / Target SDK**: Android 10 (`minSdk 29`) → Android 16 (`targetSdk 36`)
- **Core Stack**: Kotlin, Jetpack Compose (Studio 8.0 Dark UI), `SolGroundedAgent` (10 orthogonal typed OS tools + live `#1..#N` Accessibility handles + `<2ms` zero-cache state fingerprinting + `64ms` 1:1 Set-of-Mark vision), Tier-0A `72ms` native System APIs + `120+` app deep-link teleports, **Rust v5.00 16KB page-aligned JNI** (`libsol_kernel.so`, `-74% to -97%` UI tree pruning), Android 14–16 `android.net.http.HttpEngine` (HTTP/3 QUIC 0-RTT), verified **1:1 Display-0 `VirtualDisplay`** (`1080×2280 @ 440dpi, scale=1.0`, `0.0px` drift), 9 WeChat 8.0.80 reliability fixes (including Single-Shot Bubble-Verified Send Contract), Android KeyStore hardware-encrypted personal twin, and non-default-dialer Telecom Companion `InCallService`.
- **Zero Built-In Secrets**: Public APKs contain **no** `assets/sol-models.properties`, **no** API keys, and **no** private relay URLs. Users configure their own OpenAI-compatible LLM key (DeepSeek, OpenAI, OpenRouter, SiliconFlow, Qwen, or custom) directly inside the app's **Control → BYOK** card.
- **License & Attribution**: Apache-2.0. Always preserve `LICENSE` and `NOTICE` crediting `wangzan101/Mika`.

---

## 2. Repository Layout

```text
Sol-Next/
├── llms.txt                           # Compact LLM index (llmstxt.org standard)
├── llms-full.txt                      # Single-file full documentation bundle
├── AGENTS.md                          # This file (universal AI agent instructions)
├── CLAUDE.md                          # Claude Code entrypoint
├── .env.example                       # Self-documenting BYOK & ADB env reference
├── SHA256SUMS.txt                     # Authoritative SHA-256 checksums for Public APKs
├── README.md / README.en.md           # Chinese & English product launch pages
├── LICENSE / NOTICE                   # Apache-2.0 license + Mika attribution
├── scripts/
│   └── sol-cli.py                     # Zero-dependency non-interactive CLI (--json)
├── mcp/
│   └── sol_mcp_server.py              # Zero-dependency stdio MCP Server (JSON-RPC 2.0)
└── docs/
    ├── index.html                     # Live GitHub Pages product landing page
    ├── llms.txt / llms-full.txt       # Web-served copies for AI search crawlers
    ├── robots.txt / sitemap.xml       # Crawler & AI search engine discovery
    ├── guides/
    │   ├── BYOK_SETUP.md              # LLM & Voice Realtime BYOK guide
    │   ├── COMPARISON_WITH_MIKA.md    # Deep engineering comparison vs Mika
    │   └── INSTALL_WITH_AGENT.md      # Deterministic ADB install runbook
    └── media/                         # 2K comparison charts, screenshots, MP4 & GIF
```

---

## 3. Standard Non-Interactive Commands (Zero Prompts, `--json` Output)

All scripts use **Python 3.9+ standard library only** (`pip install` is NOT required) and NEVER block on `input()`.

```bash
# 1. Inspect connected Android device, package state, and 1:1 VirtualDisplay status
python3 scripts/sol-cli.py status --json

# 2. Download verified v9.1.2-public ARM64 APK, verify SHA-256, install via ADB & grant permissions
python3 scripts/sol-cli.py install --verify-sha256 --grant-permissions --json

# 3. Launch Sol Next UI on the connected device
python3 scripts/sol-cli.py launch --tab control --json

# 4. Activate the uid=2000 1:1 Background VirtualDisplay shell helper (over USB/wireless ADB)
python3 scripts/sol-cli.py start-helper --json

# 5. Dispatch a natural-language task directly to Sol Next
python3 scripts/sol-cli.py run --intent "打开系统设置并查看电池电量" --json

# 6. Verify 1:1 VirtualDisplay resolution & DPI match Display 0
python3 scripts/sol-cli.py verify-vdisplay --json
```

Pass `--serial <device-serial>` whenever multiple ADB devices are connected.

---

## 4. Android Intent Automation Contract (`ai.sol.next`)

Agents automating a connected phone directly via `adb shell am start` can use these stable extras on `ai.sol.next/ai.sol.next.canvas.SolCanvasActivity`:

| Extra Key | Type | Values / Purpose |
|---|---|---|
| `sol_next_initial_intent` | `String` | Submit a natural-language task immediately |
| `sol_next_open_tab` | `String` | `chat` \| `personal_twin` \| `brain_cockpit` \| `control` |
| `sol_next_open_virtual_phone` | `Boolean` | `true` to expand the 1:1 Background Virtual Phone PiP |
| `sol_next_open_calls` | `Boolean` | `true` to open the AI Phone Call Center cockpit |
| `sol_next_voice_trigger` | `Boolean` | `true` to trigger voice capture |

Example:
```bash
adb shell am start -n ai.sol.next/ai.sol.next.canvas.SolCanvasActivity \
  --es sol_next_initial_intent "检查当前手机电池与网络状态"
```

---

## 5. MCP Integration (Model Context Protocol)

Start the zero-dependency stdio MCP server:
```bash
python3 mcp/sol_mcp_server.py
```

Or add to Claude Desktop / Cursor / Claude Code / Codex / Antigravity config:
```json
{
  "mcpServers": {
    "sol-next": {
      "command": "python3",
      "args": ["/ABS/PATH/TO/Sol-Next/mcp/sol_mcp_server.py"]
    }
  }
}
```

Exposed MCP tools:
- `sol_device_status`: Inspect ADB devices, `ai.sol.next` install status, and VirtualDisplay specs.
- `sol_install_public_apk`: Download, SHA-256 verify, install `ai.sol.next`, and grant runtime permissions.
- `sol_start_virtual_display_helper`: Start the `uid=2000` 1:1 background helper script on the phone.
- `sol_run_phone_task`: Dispatch a natural-language task or open a specific Sol Next view on the phone.

---

## 6. Rules & Guardrails for Contributors / Agents

1. **NEVER commit real API keys, tokens, private IPs, or personal OTA endpoints** into this public repository.
2. **NEVER remove or dilute Apache-2.0 attribution** to `wangzan101/Mika` in `LICENSE`, `NOTICE`, or README files.
3. **Always verify APK SHA-256** against `SHA256SUMS.txt` before installing on a user's device.
4. **Never add interactive `input()` prompts** to `scripts/sol-cli.py` or `mcp/sol_mcp_server.py`.
5. Keep `llms.txt`, `llms-full.txt`, and `docs/llms*.txt` synchronized whenever release versions, checksums, or CLI flags change.
