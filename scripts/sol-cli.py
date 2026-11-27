#!/usr/bin/env python3
"""
Sol Next Public Edition — Non-Interactive Agent-Ready CLI (Python 3.9+ stdlib only).

Zero external dependencies, zero interactive prompts (`input()` is never called),
and deterministic `--json` structured output on every subcommand for AI agents
(Claude Code, Cursor, Codex, Antigravity) and CI scripts.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path
from typing import Any

PACKAGE_ID = "ai.sol.next"
MAIN_ACTIVITY = f"{PACKAGE_ID}/ai.sol.next.canvas.SolCanvasActivity"
RELEASE_TAG = "v9.0.9-public"
REPO_SLUG = "chongyangzhang92-hub/Sol-Next"

ARTIFACTS = {
    "arm64": {
        "filename": "Sol-Next-v9.0.9-public-arm64.apk",
        "url": f"https://github.com/{REPO_SLUG}/releases/download/{RELEASE_TAG}/Sol-Next-v9.0.9-public-arm64.apk",
        "china_url": "https://webui.67-230-181-51.sslip.io/assets/sol-public/v9.0.9/Sol-Next-v9.0.9-public-arm64.apk",
        "ghfast_url": f"https://ghfast.top/https://github.com/{REPO_SLUG}/releases/download/{RELEASE_TAG}/Sol-Next-v9.0.9-public-arm64.apk",
        "sha256": "4630a09068f65fd75bc0575bf7baef27c0735c6385d7dd3be480384c05210277",
    },
    "universal": {
        "filename": "Sol-Next-v9.0.9-public-universal.apk",
        "url": f"https://github.com/{REPO_SLUG}/releases/download/{RELEASE_TAG}/Sol-Next-v9.0.9-public-universal.apk",
        "china_url": "https://webui.67-230-181-51.sslip.io/assets/sol-public/v9.0.9/Sol-Next-v9.0.9-public-universal.apk",
        "ghfast_url": f"https://ghfast.top/https://github.com/{REPO_SLUG}/releases/download/{RELEASE_TAG}/Sol-Next-v9.0.9-public-universal.apk",
        "sha256": "1aff7e9944fdf4223c9567260f0f7fffb650cfd2ef0c0a0ff2625cd4cccb6b41",
    },
}

RUNTIME_PERMISSIONS = [
    "android.permission.POST_NOTIFICATIONS",
    "android.permission.RECORD_AUDIO",
    "android.permission.CALL_PHONE",
    "android.permission.READ_PHONE_STATE",
    "android.permission.ANSWER_PHONE_CALLS",
    "android.permission.READ_CONTACTS",
    "android.permission.READ_CALENDAR",
    "android.permission.WRITE_CALENDAR",
    "android.permission.CAMERA",
    "android.permission.ACCESS_FINE_LOCATION",
    "android.permission.ACCESS_COARSE_LOCATION",
    "android.permission.BLUETOOTH_CONNECT",
]


def find_adb() -> str | None:
    found = shutil.which("adb")
    if found:
        return found
    for candidate in [
        "/opt/homebrew/share/android-commandlinetools/platform-tools/adb",
        os.path.expanduser("~/Library/Android/sdk/platform-tools/adb"),
        os.path.expanduser("~/Android/Sdk/platform-tools/adb"),
    ]:
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            return candidate
    return None


def run_adb(args: list[str], serial: str | None = None, timeout: int = 30) -> subprocess.CompletedProcess[str]:
    adb = find_adb()
    if not adb:
        raise RuntimeError("adb binary not found in PATH or standard Android SDK locations")
    cmd = [adb]
    if serial:
        cmd += ["-s", serial]
    cmd += args
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout, check=False)


def list_devices() -> list[dict[str, str]]:
    res = run_adb(["devices", "-l"], timeout=10)
    devices: list[dict[str, str]] = []
    for line in res.stdout.splitlines()[1:]:
        line = line.strip()
        if not line:
            continue
        parts = line.split()
        if len(parts) >= 2:
            info = {"serial": parts[0], "state": parts[1], "detail": " ".join(parts[2:])}
            devices.append(info)
    return devices


def resolve_serial(explicit: str | None) -> str:
    if explicit:
        return explicit
    env_serial = os.environ.get("SOL_ADB_SERIAL", "").strip()
    if env_serial:
        return env_serial
    online = [d["serial"] for d in list_devices() if d["state"] == "device"]
    if not online:
        raise RuntimeError("No online ADB Android device found. Connect via USB or `adb connect <ip>:<port>`.")
    if len(online) > 1:
        raise RuntimeError(f"Multiple ADB devices online ({', '.join(online)}). Pass --serial <id>.")
    return online[0]


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(1024 * 1024):
            h.update(chunk)
    return h.hexdigest()


def parse_display_specs(dumpsys_text: str) -> dict[str, Any]:
    builtin = None
    virtual = None
    for line in dumpsys_text.splitlines():
        if "DisplayDeviceInfo{" not in line:
            continue
        m_dim = re.search(r"(\d+)\s*x\s*(\d+).*?density\s+(\d+)", line)
        if not m_dim:
            continue
        spec = {
            "width": int(m_dim.group(1)),
            "height": int(m_dim.group(2)),
            "densityDpi": int(m_dim.group(3)),
            "scaleOne": "scale=1.0" in line,
        }
        if '"Built-in Screen"' in line and builtin is None:
            builtin = spec
        elif '"sol-next-bg"' in line:
            virtual = spec
    exact_1to1 = bool(
        builtin and virtual and
        builtin["width"] == virtual["width"] and
        builtin["height"] == virtual["height"] and
        builtin["densityDpi"] == virtual["densityDpi"]
    )
    return {"display0": builtin, "virtualDisplay": virtual, "exact1to1": exact_1to1}


def cmd_status(args: argparse.Namespace) -> dict[str, Any]:
    adb_path = find_adb()
    if not adb_path:
        return {"ok": False, "error": "adb not found", "packageId": PACKAGE_ID}
    devices = list_devices()
    online = [d for d in devices if d["state"] == "device"]
    result: dict[str, Any] = {
        "ok": True,
        "adb": adb_path,
        "packageId": PACKAGE_ID,
        "releaseTag": RELEASE_TAG,
        "devices": devices,
    }
    target = args.serial or os.environ.get("SOL_ADB_SERIAL", "").strip() or (online[0]["serial"] if len(online) == 1 else None)
    if target:
        pkg_res = run_adb(["shell", "dumpsys", "package", PACKAGE_ID], serial=target, timeout=15)
        installed = "versionName=" in pkg_res.stdout
        ver_m = re.search(r"versionName=([^\s]+)", pkg_res.stdout)
        code_m = re.search(r"versionCode=(\d+)", pkg_res.stdout)
        disp_res = run_adb(["shell", "dumpsys", "display"], serial=target, timeout=15)
        result["selectedSerial"] = target
        result["installed"] = installed
        result["versionName"] = ver_m.group(1) if ver_m else None
        result["versionCode"] = int(code_m.group(1)) if code_m else None
        result["displays"] = parse_display_specs(disp_res.stdout)
    return result


def cmd_install(args: argparse.Namespace) -> dict[str, Any]:
    serial = resolve_serial(args.serial)
    abi = args.abi or os.environ.get("SOL_APK_ABI", "arm64").strip() or "arm64"
    if abi not in ARTIFACTS:
        raise ValueError(f"Unsupported ABI '{abi}'. Choose from: {', '.join(ARTIFACTS)}")
    meta = ARTIFACTS[abi]

    if args.apk:
        apk_path = Path(args.apk).expanduser().resolve()
        if not apk_path.is_file():
            raise FileNotFoundError(f"APK file not found: {apk_path}")
        downloaded = False
    else:
        cache_dir = Path(tempfile.gettempdir()) / "sol-next-public-cache"
        cache_dir.mkdir(parents=True, exist_ok=True)
        apk_path = cache_dir / meta["filename"]
        if not apk_path.is_file() or sha256_file(apk_path) != meta["sha256"]:
            mirror = getattr(args, "mirror", "github") or "github"
            dl_url = meta["china_url"] if mirror == "china" else (meta["ghfast_url"] if mirror == "ghfast" else meta["url"])
            urllib.request.urlretrieve(dl_url, apk_path)
            downloaded = True
        else:
            downloaded = False

    actual_sha = sha256_file(apk_path)
    if args.verify_sha256 and not args.apk and actual_sha != meta["sha256"]:
        raise RuntimeError(f"SHA-256 mismatch! expected={meta['sha256']} actual={actual_sha}")

    inst = run_adb(["install", "-r", "-d", str(apk_path)], serial=serial, timeout=120)
    if inst.returncode != 0 or "Success" not in (inst.stdout + inst.stderr):
        raise RuntimeError(f"adb install failed: {(inst.stdout + inst.stderr).strip()}")

    granted: list[str] = []
    if args.grant_permissions:
        for perm in RUNTIME_PERMISSIONS:
            g = run_adb(["shell", "pm", "grant", PACKAGE_ID, perm], serial=serial, timeout=10)
            if g.returncode == 0:
                granted.append(perm)

    return {
        "ok": True,
        "serial": serial,
        "packageId": PACKAGE_ID,
        "abi": abi,
        "apkPath": str(apk_path),
        "downloaded": downloaded,
        "sha256": actual_sha,
        "sha256Verified": (actual_sha == meta["sha256"]) if not args.apk else None,
        "permissionsGrantedCount": len(granted),
        "nextStep": "Open Sol Next v9.0.9 -> works immediately with free built-in Rust-encrypted DeepSeek V4.1 Flash, or override in Settings with your own BYOK API Key.",
    }


def cmd_launch(args: argparse.Namespace) -> dict[str, Any]:
    serial = resolve_serial(args.serial)
    am_args = ["shell", "am", "start", "-n", MAIN_ACTIVITY]
    if args.tab:
        am_args += ["--es", "sol_next_open_tab", args.tab]
    if args.open_calls:
        am_args += ["--ez", "sol_next_open_calls", "true"]
    if args.open_virtual_phone:
        am_args += ["--ez", "sol_next_open_virtual_phone", "true"]
    res = run_adb(am_args, serial=serial, timeout=15)
    return {
        "ok": res.returncode == 0,
        "serial": serial,
        "packageId": PACKAGE_ID,
        "output": (res.stdout + res.stderr).strip(),
    }


def cmd_start_helper(args: argparse.Namespace) -> dict[str, Any]:
    serial = resolve_serial(args.serial)
    # Launch app briefly so usb-start.sh is staged in external files dir
    run_adb(["shell", "am", "start", "-n", MAIN_ACTIVITY], serial=serial, timeout=15)
    script_path = f"/sdcard/Android/data/{PACKAGE_ID}/files/usb-start.sh"
    res = run_adb(["shell", "sh", script_path], serial=serial, timeout=20)
    return {
        "ok": res.returncode == 0,
        "serial": serial,
        "scriptPath": script_path,
        "output": (res.stdout + res.stderr).strip(),
    }


def cmd_run(args: argparse.Namespace) -> dict[str, Any]:
    serial = resolve_serial(args.serial)
    res = run_adb(
        ["shell", "am", "start", "-n", MAIN_ACTIVITY, "--es", "sol_next_initial_intent", args.intent],
        serial=serial,
        timeout=15,
    )
    return {
        "ok": res.returncode == 0,
        "serial": serial,
        "intent": args.intent,
        "output": (res.stdout + res.stderr).strip(),
    }


def cmd_verify_vdisplay(args: argparse.Namespace) -> dict[str, Any]:
    serial = resolve_serial(args.serial)
    res = run_adb(["shell", "dumpsys", "display"], serial=serial, timeout=15)
    specs = parse_display_specs(res.stdout)
    return {"ok": True, "serial": serial, **specs}


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="sol-cli.py",
        description="Sol Next Public Edition — Non-interactive CLI for AI agents and developers.",
    )
    p.add_argument("--serial", help="ADB device serial (optional when single device connected)")
    p.add_argument("--json", action="store_true", help="Emit machine-readable JSON output")
    sub = p.add_subparsers(dest="command", required=True)

    sub.add_parser("status", help="Inspect ADB devices, Sol Next install state, and display specs")

    p_inst = sub.add_parser("install", help="Download, SHA-256 verify, and install Sol Next Public Edition APK")
    p_inst.add_argument("--abi", choices=["arm64", "universal"], default="arm64")
    p_inst.add_argument("--mirror", choices=["github", "china", "ghfast"], default="github", help="Download channel (default github; use china for direct CN mirror)")
    p_inst.add_argument("--apk", help="Install from a local APK path instead of downloading from GitHub")
    p_inst.add_argument("--verify-sha256", action="store_true", default=True, help="Verify official SHA-256 (default true)")
    p_inst.add_argument("--grant-permissions", action="store_true", help="Grant runtime permissions via pm grant")

    p_launch = sub.add_parser("launch", help="Launch Sol Next on connected Android device")
    p_launch.add_argument("--tab", choices=["chat", "personal_twin", "brain_cockpit", "control"])
    p_launch.add_argument("--open-calls", action="store_true")
    p_launch.add_argument("--open-virtual-phone", action="store_true")

    sub.add_parser("start-helper", help="Start uid=2000 1:1 Background VirtualDisplay shell helper")

    p_run = sub.add_parser("run", help="Dispatch a natural-language task intent to Sol Next")
    p_run.add_argument("--intent", required=True, help="Natural-language task instruction for Sol Next")

    sub.add_parser("verify-vdisplay", help="Verify 1:1 VirtualDisplay resolution & DPI alignment with Display 0")
    return p


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    handlers = {
        "status": cmd_status,
        "install": cmd_install,
        "launch": cmd_launch,
        "start-helper": cmd_start_helper,
        "run": cmd_run,
        "verify-vdisplay": cmd_verify_vdisplay,
    }
    try:
        payload = handlers[args.command](args)
        exit_code = 0 if payload.get("ok", False) else 1
    except Exception as exc:
        payload = {"ok": False, "error": str(exc), "command": args.command}
        exit_code = 1

    if args.json:
        print(json.dumps(payload, ensure_ascii=False, indent=2))
    else:
        for k, v in payload.items():
            print(f"{k}: {v}")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
