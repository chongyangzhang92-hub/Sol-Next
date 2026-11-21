# 📊 Sol Next vs Mika 深度工程架构对比（致敬原作者 `@wangzan101`）

> 本项目在 [**Mika (`github.com/wangzan101/Mika`, Apache-2.0)**](https://github.com/wangzan101/Mika) 的优秀开源基础上深度演进。本文档面向开发者与极客用户，逐项透明拆解 Sol Next 在底层内核、协议栈、虚拟屏精度、稳定性与 AI 电话上的工程演进，并客观列出两者的定位差异。

---

## 一、核心工程突破拆解

### 1. 端侧 Rust v3.10 原生内核（`rust/sol-kernel`）
- **UI 控件树极限剪枝（`prune_ui_tree_tsv`）**：传统 Java/Kotlin 遍历整棵无障碍节点树并序列化后往往达数千字符，既拖慢首字延迟（TTFT）又浪费输入 Token。Sol Next 在 Rust 侧实现毫秒级单遍剪枝，自动剔除不可见容器、重复包裹层与无关装饰文本，将节点编号压缩为 `[1]C"按钮"@x,y` 极简格式——**Android 16 实测复杂设置页 `58 → 25` 节点（`-74%`，仅 `45 µs`），常规页面 `22 → 3` 节点（`-97%`，仅 `17 µs`）**。
- **System-0 端侧语义直出路由（`route_system0`）**：高频任务编译为技能后，由 Rust BM25 + Bigram + 反义词极性拦截在 **`51 µs`（0.051 ms）内完成本地路由，消耗 0 Token**，避免「打开 Wi-Fi」误匹配「关闭 Wi-Fi」。
- **SSE 边吐边执行增量提取（`extract_streamed_batch_steps`）**：模型仍在流式输出 `os_batch_actions` JSON 数组时，Rust 每收到 `}` 仅花 `80 µs` 即可安全拆出已完成步骤，通过安全门控后与模型尾流并行抢跑执行。

### 2. Display 0 1:1 原生手机屏克隆（0.0px 零漂移）
- **为什么传统缩放虚拟屏容易点偏？** 720p/320dpi 固定虚拟屏在 1080p/2K/高刷旗舰机上会导致字体折行不同，且 `Bounds → 0..999 → 虚拟屏像素` 的二次整数取整在密集图标或小按钮上会产生 1.5–3.0px 偏移。
- **Sol Next 的做法**：
  - 在 `ISolBgShell` 中新增 `getDefaultDisplaySpec()`，通过 `DisplayManagerGlobal.getDisplayInfo(0)` 直读主屏物理宽、高、DPI 与刷新率，动态创建完全 1:1 的 `sol-next-bg`（实测 `dumpsys display`：`1080×2400 @ 420dpi, 60Hz, scale=1.0, physicalPixelDisplaySizeRatio=1.0`）。
  - 无障碍命中走 `injectTapPx(px, py)` 原始像素直投（**0.0px 误差**）；悬浮 PiP 触控与 ML Kit OCR 走 `injectTapFraction` / `injectSwipeFraction` 亚像素浮点直投；全量注入升级标准 `PointerProperties(TOOL_TYPE_FINGER)` + `PointerCoords`，杜绝国产 ROM/游戏拒收模拟点击。

### 3. 根治「长任务无故暂停与中断」的 7 层防护
1. **隔离 `TimeoutCancellationException`**：全链路在 `CancellationException` 前优先捕获超时异常并转为 `IOException`，绝不再把单次网络超时误判为协程取消而直接显示「任务意外暂停」。
2. **HTTP/3 QUIC 0-RTT + 智能熔断器**：Android 14–16 启用原生 Chromium `HttpEngine` QUIC 加速；针对国内移动网络偶发 UDP 443 黑洞，内置 12s 首包 + 18s 帧间看门狗，连续 2 次失败自动熔断 5 分钟直切 HTTP/2，切网立即重置。
3. **6 轮退避重试 + 自动断点续跑**：单轮最多 6 次退避重试；若仍遇瞬时断网，在保留已完成步骤的基础上自动清理死连接并无缝续跑 1 次。
4. **精准区分常规设置与高风险弹窗**：不再因小米设置首页出现「密码与安全」「清理缓存」等字样而误触发高风险暂停；真正遇到支付/删除确认弹窗时仍 100% 拦截，且 App 在后台时立即发出 `IMPORTANCE_HIGH` 顶部通知提醒确认。
5. **进程内兜底 `PARTIAL_WAKE_LOCK` + `WifiLock`**：即便从悬浮胶囊/后台触发任务，依然确保 CPU 与 Wi-Fi 在长任务期间不被休眠冻结。
6. **无线调试助手掉线自愈 + 失败附屏**：任一步操作遇到助手瞬断自动静默重连；批量步失败立刻附带 `[失败时当前屏幕]` 供模型下一轮无缝换路。
7. **小米 HyperOS 三级返回兜底 & 64 轮防提前收尾**：`KEYCODE_BACK` 失效自动回落左上角像素返回键与左边缘手势返回；严防多步任务只打开 App 就提前结束。

### 4. 完整 AI 接打电话驾驶舱 + 加密个人助理
- 完整保留系统原生拨号器，助手连通即自动补齐 Telecom Companion 权限，同时兼容 `uid=2000` Shell 与 `uid=0` Root 音频桥接；支持多路来电隔离防串音、联系人名称解析、5–15 位客服短号、3×4 交互式 DTMF 拨号盘、免提开关、来电/通话中/挂断通知闭环与通话纪要重试。
- 独立「个人助理」采用 Android KeyStore AES-GCM 本地加密存储个人偏好，支持低电量与重要日程/通知主动提醒，且后台代办顺序排队绝不打断主任务。

---

## 二、如何选择：原版 Mika 还是 Sol Next？

| 你的核心诉求 | 推荐选择 |
|---|---|
| 追求**极致速度**（Rust 内核 + QUIC + 零动画）、**1:1 虚拟屏零漂移操控**、**小米 HyperOS 长任务零暂停**、**完整 AI 接打电话与主动个人助理** | **Sol Next 公开版** |
| 重度需要**内置桌面级浏览器**、**每对话独立 Linux 沙箱（Python/Node）**、**MCP 电脑联动**与丰富的社区预设网站/App 知识库 | **原版 [Mika](https://github.com/wangzan101/Mika)** |

两者均使用 **Apache-2.0** 协议，公开版包名不同（`ai.sol.next` vs `com.wangzan101.mika`），甚至可以**在同一台手机上和平共存、按需切换使用**！
