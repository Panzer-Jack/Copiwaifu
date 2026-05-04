[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

# Copiwaifu

你的 Live2D AI 桌面导航娘。

Copiwaifu 是一个基于 Tauri 的桌宠应用，会把 AI 编程工具的运行状态同步成桌面上的 Live2D 角色反馈。目前主要对接 Claude Code、GitHub Copilot、Codex、Gemini CLI 和 OpenCode，并支持在一轮任务完成后基于已保存的 session 元信息生成可选的 AI Talk 气泡反馈。

## 官网

- Copiwaifu：https://copiwaifu.panzer-jack.cn/

<img width="1512" height="824" alt="image" src="https://github.com/user-attachments/assets/bc2bead0-5c3b-4d61-9924-5c61455d9ce0" />

## 在 macOS 安装后 （重要！！！）

请在终端执行：

```bash
xattr -dr com.apple.quarantine /Applications/copiwaifu.app
```

然后打开 Copiwaifu 就可以正常使用了!

## 它能做什么

- 在桌面上显示一个会跟随 AI 会话状态变化的 Live2D 桌宠。
- 同步 `idle`、`thinking`、`tool_use`、`error`、`complete`、`needs_attention` 等状态。
- 根据当前状态和活跃 Agent 显示气泡文案。
- 可选启用 AI Talk，在会话完成或出错后基于当前 session 摘要生成一句短反馈。
- 支持内置 Live2D 模型，也支持导入自定义模型目录。
- 支持为不同状态绑定对应的动作组。
- 提供设置窗口，可配置语言、名字、开机自启、模型、窗口尺寸、AI Talk 模型参数和动作绑定等。
- 提供托盘菜单，可控制显示/隐藏、打开设置和退出应用。
- 支持通过 GitHub Releases 检查应用更新。

## 工作方式

应用启动后会执行这几件事：

1. 在本机 `127.0.0.1` 上启动一个 HTTP 服务，默认从端口 `23333` 开始监听，端口被占用时会继续尝试后续端口。
2. 为受支持的 AI CLI 安装本地 hooks。
3. 接收 hook 上报的事件，并转换成 Copiwaifu 内部状态。
4. 把运行时数据写入 `~/.copiwaifu`，其中 session 摘要会写入 `~/.copiwaifu/sessions`。
5. 当 AI Talk 已开启，且完成或失败的一轮任务有足够上下文时，通过 Vercel AI SDK 调用本地 AI runtime，并用生成短句替换静态的 complete/error 气泡。

当前会接入这些本地配置文件：

- Claude Code：`~/.claude/settings.json`
- GitHub Copilot：`~/.config/github-copilot/config.json`
- Codex：`~/.codex/config.toml`
- Gemini CLI：`~/.gemini/settings.json`
- OpenCode：`~/.config/opencode/opencode.json`

原有 hook 配置会备份到 `~/.copiwaifu/hooks/original-hooks.json`。

## 平台说明

Copiwaifu 当前主要面向 macOS 开发和发布。应用里使用了 macOS 特有的窗口能力，仓库里的发布工作流也只产出 macOS 安装包。

## 技术栈

- Vue 3
- TypeScript
- Vite
- Tauri 2
- Rust
- PixiJS
- `easy-live2d`
- Vercel AI SDK
- Node.js sidecar runtime

## 本地开发环境

本地运行前请先准备：

- Node.js
- `pnpm`
- Rust toolchain
- Tauri 在 macOS 下的系统依赖
- 至少一个受支持的 AI CLI（如果你需要实时同步状态）
- 一个模型服务的 API Key（如果你需要 AI Talk 生成气泡）

另外，hook 桥接脚本和 AI Talk runtime 都依赖命令行里的 `node` 可执行文件。

## 快速开始

```bash
pnpm install
pnpm run
```

`pnpm run` 会直接启动 Tauri 开发环境。你也可以显式执行：

```bash
pnpm tauri dev
```

## 常用命令

```bash
pnpm dev                   # 只启动 Vite
pnpm build                 # 类型检查、构建前端产物并打包 AI runtime
pnpm sidecar:build         # 将 sidecar/ai-runtime 打包到 sidecar/ai-runtime/bundle/main.mjs
pnpm tauri dev             # 启动桌面开发环境
pnpm run                   # pnpm tauri dev 的快捷方式
pnpm tauri build           # 本地构建桌面安装包
pnpm release               # 执行 release-it
pnpm release:sync-version  # 同步 package.json / tauri.conf.json / Cargo.toml 版本号
```

## 设置项

设置窗口里目前可以调整：

- 桌宠名字
- 界面语言：英文 / 中文 / 日语
- 开机自启
- Live2D 模型目录
- 窗口尺寸预设
- AI Talk 开关、模型供应商、模型 ID、API Key、可选 Base URL 和可选自定义请求头
- 各运行状态对应的动作组绑定

如果没有选择自定义模型，Copiwaifu 会使用内置的 `Yulia` 模型。

## AI Talk

AI Talk 默认关闭。开启并配置模型供应商后，Copiwaifu 会在 session 进入 `complete` 或 `error` 状态时生成一句短气泡。它不是独立聊天窗口，不会在 idle 状态随机触发，也不会把指令写回原 AI CLI。

AI Talk 只使用 Copiwaifu 已保存的 session 元信息：Agent 类型、session id、工作目录、session 标题、最近事件/工具和可用摘要。它不会读取完整对话记录、项目文件或源码内容。

当前支持 OpenAI、Anthropic、Google Gemini、DeepSeek、阿里云百炼 / Qwen、Moonshot Kimi、智谱 GLM、火山方舟 / 豆包、百度千帆 / ERNIE、腾讯混元、MiniMax，以及 OpenAI-Compatible API。OpenAI-Compatible 配置必须填写 Base URL；需要代理或服务商额外参数时，可以在高级设置里填写自定义请求头。

如果 AI Talk 未开启、API Key 或模型缺失、没有有效 session 上下文，或模型调用失败，Copiwaifu 会回退到原有静态状态气泡。

## 自定义 Live2D 模型

保存设置前，应用会先校验模型目录。可用的模型目录至少需要包含合法的 `.model3.json` 入口文件，以及它引用到的资源文件。

如果希望桌宠在不同状态下有明显动作反馈，建议在模型里准备对应的 motion group，并在设置页里逐项绑定。某个状态没有单独绑定时，应用会尝试直接匹配常见动作组名，例如 `Idle`、`Thinking`、`ToolUse`、`Complete`；如果没有匹配到，该状态就保持未绑定。

## 更新与发布

- 应用通过 Tauri Updater 检查更新。
- 更新元数据来自 GitHub Releases。
- 当推送 `app-v*` 格式的 tag 时，仓库工作流会构建并发布 macOS Apple Silicon 与 Intel 两套产物。

## 注意事项

- Copiwaifu 会修改本地 AI CLI 配置文件以安装 hooks。如果你已经维护了自己的 hook 链，建议先检查差异。
- 会话状态、端口号等运行时文件会写入 `~/.copiwaifu`。
- AI Talk 设置会保存在本机应用设置文件中，包括 API Key；这些信息只用于调用当前选择的模型服务。
- session 文件会保留有限的元信息用于恢复和 AI Talk，包括最近事件摘要、`lastMeaningfulSummary` 和 `aiTalkContext`。
- 同时还会在 `/tmp/copiwaifu-port` 写一个兜底端口文件。

## 许可证

本项目使用 MIT License，见 [LICENSE](LICENSE)。

仓库内附带的 Live2D Cubism Core 文件使用其各自的许可证，见 [public/Core/LICENSE.md](public/Core/LICENSE.md) 和 [public/Core/README.md](public/Core/README.md)。
