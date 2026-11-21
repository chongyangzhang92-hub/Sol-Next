# 🔑 Sol Next 公开版 BYOK（自带模型 API Key）配置指南

Sol Next 公开版（包名 `ai.sol.next`）**不包含任何内置 API Key，也不连接任何私有中转服务器**。所有模型请求均从你的手机直连你在本机填写的官方或兼容接口，密钥仅保存在本机应用私有目录。

---

## 一、配置主推理模型（对话与手机操作，30 秒完成）

1. 打开 Sol Next，点击首页提示条或底部 **「后台手机」** 标签页。
2. 在最顶部的 **「模型 API 配置（公开版 · 自带 Key / BYOK）」** 卡片中选择预设或手动填写：

| 服务商 | Base URL | 推荐模型（Model ID） | 特点 |
|---|---|---|---|
| **DeepSeek 官方（推荐）** | `https://api.deepseek.com/v1` | `deepseek-chat` | 性价比极高，中文与 Tool Calling 能力强，支持原生 QUIC/H2 直连 |
| **OpenAI 官方** | `https://api.openai.com/v1` | `gpt-4o-mini` / `gpt-4.1` | 通用能力与多模态看屏均衡 |
| **OpenRouter** | `https://openrouter.ai/api/v1` | `deepseek/deepseek-chat` / `google/gemini-2.5-flash` | 一个 Key 切换全球主流模型 |
| **硅基流动 SiliconFlow** | `https://api.siliconflow.cn/v1` | `deepseek-ai/DeepSeek-V3` | 国内直连极快，兼容 OpenAI 接口 |
| **阿里云通义千问** | `https://dashscope.aliyuncs.com/compatible-mode/v1` | `qwen-plus` / `qwen-vl-max` | 国内高稳定，中文场景表现出色 |
| **任意自定义端点** | 填写你的 `https://.../v1` | 填写对应模型名 | 兼容任何标准 OpenAI Chat Completions 接口 |

3. 粘贴你的 `sk-...` API Key，点击 **「保存并测速」**。看到 `✓ 连接成功 · <模型> · h2/h3 · xxxms` 即配置完成，无需重启 App。

---

## 二、配置 AI 接打电话实时语音（可选）

打开左上角侧边栏 → **「AI 接打电话助手」** → **「代接设置」** → 展开自定义实时语音凭据，支持以下两种协议：

1. **豆包实时语音（Volcengine Seeduplex 3.0，推荐中文通话）**
   - 协议选择：`豆包 Seeduplex 3.0`
   - 填入你在火山引擎控制台申请的 `App ID` 与 `Access Key`。
2. **OpenAI Realtime API**
   - 协议选择：`OpenAI Realtime`
   - 填入支持 Realtime WebSocket 的 OpenAI API Key。

配置完成后，回到「拨打与通话」页点击 **「一键全链路就绪」**，即可自动完成系统伴侣权限授权、音频通道自检（Direct/Redirect）与实时语音握手验证。

---

## 三、常见问题

- **为什么测速提示 HTTP 401 / 403？**
  请检查 API Key 是否完整复制（不要多出空格），以及对应账号余额是否充足。
- **为什么提示 HTTP 404？**
  请确认 Base URL 以 `/v1`（或服务商要求的兼容路径，如通义千问 `/compatible-mode/v1`）结尾，不要额外多写 `/chat/completions`。
- **想看屏识别纯图片/图标界面怎么选模型？**
  填入支持视觉多模态（Vision / `image_url`）的模型 ID（例如 `gpt-4o-mini`、`qwen-vl-max`、`gemini-2.5-flash` 等）即可获得最佳视觉看屏效果；即便使用纯文本模型，Sol Next 端侧也内置 **ML Kit 中文 OCR + 二维码扫描 + Rust 控件树剪枝**，依然可流畅操作绝大多数 App！
