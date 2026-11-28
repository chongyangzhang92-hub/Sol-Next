# 🤖 让 AI 编程助手一键帮你安装并验证 Sol Next（USB / 无线 ADB）

如果你手边有电脑和任意 AI 编程助手（**Claude Code / Cursor / Codex / Antigravity / Windsurf**），直接把下面这句话发给它：

```text
请按 https://github.com/chongyangzhang92-hub/Sol-Next/blob/main/docs/guides/INSTALL_WITH_AGENT.md 的步骤，把零内置 Key 公开版 Sol Next (ai.sol.next) 安装到当前通过 ADB 连接的安卓手机上，校验 SHA-256 并启动 1:1 后台虚拟屏助手。
```

---

## 给 AI Agent 的确定性执行步骤（零交互 · 零幻觉）

1. **检查设备与当前状态**
   ```bash
   python3 scripts/sol-cli.py status --json
   ```
   - 确认输出中 `devices` 至少有一台 `state == "device"`。若有多台设备，在后续命令加上 `--serial <serial>`。

2. **下载官方公开版 APK、校验 SHA-256、安装并授予运行时权限**
   ```bash
   python3 scripts/sol-cli.py install --verify-sha256 --grant-permissions --json
   ```
   - ARM64 预期 SHA-256：`4f8ddaa1f001d916742e033d88f6eef6c13d3246b7ab77c98c9f939400e4071f`
   - 安装包名：`ai.sol.next`（公开版不含任何内置模型 Key）

3. **启动 `uid=2000` 1:1 后台虚拟屏助手**
   ```bash
   python3 scripts/sol-cli.py start-helper --json
   ```

4. **打开 BYOK 配置页，提示用户填入自有模型 API Key**
   ```bash
   python3 scripts/sol-cli.py launch --tab control --json
   ```
   - 请用户直接在手机上「模型 API 配置（BYOK）」卡片中选择预设（DeepSeek / OpenAI / OpenRouter / 硅基流动 / 通义千问）并填入自己的 API Key，点击「保存并测速」。

5. **（可选）验证 1:1 虚拟屏参数与发送测试任务**
   ```bash
   python3 scripts/sol-cli.py run --intent "查看当前手机电量与网络状态" --json
   python3 scripts/sol-cli.py verify-vdisplay --json
   ```
