#!/usr/bin/env python3
"""
Sol Next Public Edition — Zero-Dependency MCP Server (Model Context Protocol, JSON-RPC 2.0 over stdio).

Allows Claude Code, Claude Desktop, Cursor, Codex, and local AI agents to inspect, install,
activate 1:1 Background VirtualDisplay, and dispatch phone tasks directly to Sol Next (`ai.sol.next`)
over USB or wireless ADB using pure Python 3.9+ standard library (zero pip installs required).
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any

SERVER_INFO = {"name": "sol-next-mcp", "version": "6.3.0-public"}
PROTOCOL_VERSION = "2024-11-05"
CLI_PATH = Path(__file__).resolve().parent.parent / "scripts" / "sol-cli.py"

TOOLS = [
    {
        "name": "sol_device_status",
        "description": "Inspect connected ADB Android devices, check whether Sol Next Public Edition (ai.sol.next) is installed, and verify 1:1 VirtualDisplay alignment.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "serial": {"type": "string", "description": "Optional ADB device serial"},
            },
        },
    },
    {
        "name": "sol_install_public_apk",
        "description": "Download the official zero-API-key Sol Next v6.3.0-public APK from GitHub Releases, verify SHA-256, install via ADB, and optionally grant runtime permissions.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "serial": {"type": "string", "description": "Optional ADB device serial"},
                "abi": {"type": "string", "enum": ["arm64", "universal"], "default": "arm64"},
                "grant_permissions": {"type": "boolean", "default": True},
            },
        },
    },
    {
        "name": "sol_start_virtual_display_helper",
        "description": "Start the uid=2000 1:1 Background VirtualDisplay shell helper on the connected Android phone via ADB.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "serial": {"type": "string", "description": "Optional ADB device serial"},
            },
        },
    },
    {
        "name": "sol_run_phone_task",
        "description": "Dispatch a natural-language task intent to Sol Next (ai.sol.next) running on the connected Android phone.",
        "inputSchema": {
            "type": "object",
            "required": ["intent"],
            "properties": {
                "intent": {"type": "string", "description": "Natural-language task instruction for the Android phone"},
                "serial": {"type": "string", "description": "Optional ADB device serial"},
            },
        },
    },
]


def invoke_cli(cli_args: list[str]) -> dict[str, Any]:
    cmd = [sys.executable, str(CLI_PATH), "--json", *cli_args]
    res = subprocess.run(cmd, capture_output=True, text=True, timeout=150, check=False)
    raw = res.stdout.strip()
    try:
        return json.loads(raw)
    except Exception:
        return {"ok": False, "returncode": res.returncode, "stdout": raw, "stderr": res.stderr.strip()}


def call_tool(name: str, arguments: dict[str, Any]) -> dict[str, Any]:
    serial = str(arguments.get("serial") or "").strip()
    prefix = ["--serial", serial] if serial else []
    if name == "sol_device_status":
        data = invoke_cli([*prefix, "status"])
    elif name == "sol_install_public_apk":
        abi = str(arguments.get("abi") or "arm64")
        grant = bool(arguments.get("grant_permissions", True))
        extra = ["--abi", abi, "--verify-sha256"] + (["--grant-permissions"] if grant else [])
        data = invoke_cli([*prefix, "install", *extra])
    elif name == "sol_start_virtual_display_helper":
        data = invoke_cli([*prefix, "start-helper"])
    elif name == "sol_run_phone_task":
        intent = str(arguments.get("intent") or "").strip()
        if not intent:
            data = {"ok": False, "error": "intent is required"}
        else:
            data = invoke_cli([*prefix, "run", "--intent", intent])
    else:
        return {"isError": True, "content": [{"type": "text", "text": f"Unknown tool: {name}"}]}

    return {
        "isError": not bool(data.get("ok", False)),
        "content": [{"type": "text", "text": json.dumps(data, ensure_ascii=False, indent=2)}],
    }


def write_msg(msg: dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(msg, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def handle_request(req: dict[str, Any]) -> dict[str, Any] | None:
    method = req.get("method")
    req_id = req.get("id")
    params = req.get("params") or {}

    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {"tools": {}},
                "serverInfo": SERVER_INFO,
            },
        }
    if method == "notifications/initialized":
        return None
    if method == "ping":
        return {"jsonrpc": "2.0", "id": req_id, "result": {}}
    if method == "tools/list":
        return {"jsonrpc": "2.0", "id": req_id, "result": {"tools": TOOLS}}
    if method == "tools/call":
        name = str(params.get("name") or "")
        arguments = params.get("arguments") if isinstance(params.get("arguments"), dict) else {}
        return {"jsonrpc": "2.0", "id": req_id, "result": call_tool(name, arguments)}
    if req_id is None:
        return None
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "error": {"code": -32601, "message": f"Method not found: {method}"},
    }


def main() -> int:
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except json.JSONDecodeError as exc:
            write_msg({"jsonrpc": "2.0", "id": None, "error": {"code": -32700, "message": str(exc)}})
            continue
        resp = handle_request(req)
        if resp is not None:
            write_msg(resp)
    return 0


if __name__ == "__main__":
    sys.exit(main())
