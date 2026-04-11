[English](README.md) | [简体中文](README.zh-CN.md)

# Copiwaifu

你的 Live2D AI 桌面导航娘。

Copiwaifu 是一个基于 Tauri 的桌宠应用，会把 AI 编程工具的运行状态同步成桌面上的 Live2D 角色反馈。目前主要对接 Claude Code、GitHub Copilot、Codex、Gemini CLI 和 OpenCode。

## 官网

- Copiwaifu：https://copiwaifu.panzer-jack.cn/

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
- 支持内置 Live2D 模型，也支持导入自定义模型目录。
- 支持为不同状态绑定对应的动作组。
- 提供设置窗口，可配置语言、名字、开机自启、模型、窗口尺寸等。
- 提供托盘菜单，可控制显示/隐藏、打开设置和退出应用。
- 支持通过 GitHub Releases 检查应用更新。

## 工作方式

应用启动后会执行这几件事：

1. 在本机 `127.0.0.1` 上启动一个 HTTP 服务，默认从端口 `23333` 开始监听，端口被占用时会继续尝试后续端口。
2. 为受支持的 AI CLI 安装本地 hooks。
3. 接收 hook 上报的事件，并转换成 Copiwaifu 内部状态。
4. 把运行时数据写入 `~/.copiwaifu`。

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

## 本地开发环境

本地运行前请先准备：

- Node.js
- `pnpm`
- Rust toolchain
- Tauri 在 macOS 下的系统依赖
- 至少一个受支持的 AI CLI（如果你需要实时同步状态）

另外，hook 桥接脚本依赖命令行里的 `node` 可执行文件。

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
pnpm build                 # 构建前端产物
pnpm tauri dev             # 启动桌面开发环境
pnpm run                   # pnpm tauri dev 的快捷方式
pnpm tauri build           # 本地构建桌面安装包
pnpm release               # 执行 release-it
pnpm release:sync-version  # 同步 package.json / tauri.conf.json / Cargo.toml 版本号
```

## 设置项

设置窗口里目前可以调整：

- 桌宠名字
- 界面语言：英文 / 中文
- 开机自启
- Live2D 模型目录
- 窗口尺寸预设
- 各运行状态对应的动作组绑定

如果没有选择自定义模型，Copiwaifu 会使用内置的 `Hiyori` 模型。

## 自定义 Live2D 模型

保存设置前，应用会先校验模型目录。可用的模型目录至少需要包含合法的 `.model3.json` 入口文件，以及它引用到的资源文件。

如果希望桌宠在不同状态下有明显动作反馈，建议在模型里准备对应的 motion group，并在设置页里逐项绑定。某个状态没有单独绑定时，应用会尽量回退到 `idle` 的绑定。

## 更新与发布

- 应用通过 Tauri Updater 检查更新。
- 更新元数据来自 GitHub Releases。
- 当推送 `app-v*` 格式的 tag 时，仓库工作流会构建并发布 macOS Apple Silicon 与 Intel 两套产物。

## 注意事项

- Copiwaifu 会修改本地 AI CLI 配置文件以安装 hooks。如果你已经维护了自己的 hook 链，建议先检查差异。
- 会话状态、端口号等运行时文件会写入 `~/.copiwaifu`。
- 同时还会在 `/tmp/copiwaifu-port` 写一个兜底端口文件。

## 许可证

本项目使用 MIT License，见 [LICENSE](LICENSE)。

仓库内附带的 Live2D Cubism Core 文件使用其各自的许可证，见 [public/Core/LICENSE.md](public/Core/LICENSE.md) 和 [public/Core/README.md](public/Core/README.md)。
