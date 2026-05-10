[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

# Copiwaifu

Your Live2D AI navigator for everyday coding sessions.

Copiwaifu is a Tauri desktop pet that mirrors the state of your AI coding tools and turns that activity into a small Live2D companion on your desktop. It currently focuses on syncing with Claude Code, GitHub Copilot, Codex, Gemini CLI, and OpenCode, with optional AI Talk bubbles generated from saved session metadata after a coding turn finishes.

## Website

- Copiwaifu: https://copiwaifu.panzer-jack.cn/

<img width="1512" height="824" alt="image" src="https://github.com/user-attachments/assets/bc2bead0-5c3b-4d61-9924-5c61455d9ce0" />


## After Installing On macOS（important！！！）

After moving the app into `/Applications`, run:

```bash
xattr -dr com.apple.quarantine /Applications/copiwaifu.app
```

## What It Does

- Shows a Live2D desktop companion that reacts to AI session state changes.
- Syncs status such as `idle`, `thinking`, `tool_use`, `error`, `complete`, and `needs_attention`.
- Displays short speech bubbles based on the current agent and state.
- Optionally uses AI Talk to generate one short completion/error bubble from the current session summary.
- Supports a built-in Live2D model and custom model folders.
- Lets you bind model motion groups to each runtime state.
- Includes a settings window for language, name, auto-start, model selection, window size, AI Talk model settings, and motion bindings.
- Provides a tray menu for visibility, settings, and exit.
- Checks for app updates from GitHub Releases.

## How It Works

When the app starts, it:

1. Launches a local HTTP server on `127.0.0.1` using port `23333` and falls back across the next available ports if needed.
2. Installs local hooks for supported AI CLIs.
3. Receives hook events and converts them into Copiwaifu state changes.
4. Persists local runtime data under `~/.copiwaifu`, including session digests under `~/.copiwaifu/sessions`.
5. When AI Talk is enabled and a completed or failed turn has enough context, runs the local AI runtime through Vercel AI SDK and replaces the static completion/error bubble with a generated short sentence.

The hook installer currently integrates with:

- Claude Code via `~/.claude/settings.json`
- GitHub Copilot via `~/.config/github-copilot/config.json`
- Codex via `~/.codex/config.toml`
- Gemini CLI via `~/.gemini/settings.json`
- OpenCode via `~/.config/opencode/opencode.json`

Original hook definitions are backed up to `~/.copiwaifu/hooks/original-hooks.json`.

## Platform

Copiwaifu keeps the native macOS desktop-pet window behavior while adding maintainable Windows development and packaging paths:

- macOS: continues to use `tauri-nspanel`, hidden Dock behavior, and all-space window behavior.
- Windows: does not load macOS-only plugins and uses Tauri's standard transparent, borderless, always-on-top window, system tray, and autostart plugin.
- Hook and runtime files prefer the user's `.copiwaifu` directory, and fallback port files are written to the system temp directory instead of a hardcoded `/tmp`.
- Codex `notify` configuration escapes Windows backslash paths as TOML strings.

Windows maintenance notes are available in [docs/WINDOWS.zh-CN.md](docs/WINDOWS.zh-CN.md).

## Tech Stack

- Vue 3
- TypeScript
- Vite
- Tauri 2
- Rust
- PixiJS
- `easy-live2d`
- Vercel AI SDK
- Node.js sidecar runtime

## Requirements

Before running the project locally, make sure you have:

- Node.js
- `pnpm`
- Rust toolchain
- Tauri system prerequisites for macOS
- At least one supported AI CLI if you want live session sync
- A model API key if you want AI Talk-generated bubbles

The hook bridge and AI Talk runtime expect `node` to be available in a standard shell path.

## Quick Start

```bash
pnpm install
pnpm run
```

`pnpm run` starts the Tauri development app. You can also use:

```bash
pnpm tauri dev
```

## Available Scripts

```bash
pnpm dev                   # start Vite only
pnpm build                 # type-check, build the frontend bundle, and bundle the AI runtime
pnpm sidecar:build         # bundle sidecar/ai-runtime into sidecar/ai-runtime/bundle/main.mjs
pnpm tauri dev             # run the desktop app in dev mode
pnpm run                   # shortcut for pnpm tauri dev
pnpm tauri:build:windows   # build NSIS / MSI installers on Windows
pnpm tauri build           # build desktop artifacts locally
pnpm release               # run release-it
pnpm release:sync-version  # sync version across package.json / tauri.conf.json / Cargo.toml
```

## Settings

From the settings window you can configure:

- Pet name
- UI language: English, Chinese, or Japanese
- Auto start on login
- Live2D model directory
- Window size preset
- AI Talk enablement, provider, model ID, API key, optional base URL, and optional custom headers
- Motion group binding for each agent state

If no custom model is selected, Copiwaifu uses the bundled `Yulia` model.

## AI Talk

AI Talk is disabled by default. After you enable it and configure a provider, Copiwaifu can generate one short bubble when a session enters `complete` or `error`. It does not open a chat window, trigger randomly while idle, or send prompts back into your AI CLI.

AI Talk only uses the session metadata Copiwaifu already stores: agent type, session id, working directory, session title, recent event/tool, and the best available summary. It does not read full chat logs, project files, or source code for generation.

Supported provider options are OpenAI, Anthropic, Google Gemini, DeepSeek, Alibaba Bailian / Qwen, Moonshot Kimi, Zhipu GLM, Volcengine Ark / Doubao, Baidu Qianfan / ERNIE, Tencent Hunyuan, MiniMax, and OpenAI-compatible APIs. OpenAI-compatible setups require a Base URL. Custom headers are available for services or proxies that need them.

If AI Talk is off, the API key or model is missing, no useful session context exists, or the model call fails, Copiwaifu falls back to the normal static state bubble.

## Custom Live2D Models

Custom model folders are validated before saving. A usable model directory should include a valid `.model3.json` entry file and its referenced assets.

To make state animations useful, define motion groups in your model and bind them in settings. If a state has no explicit binding, Copiwaifu tries a direct auto-match for common group names such as `Idle`, `Thinking`, `ToolUse`, and `Complete`. If no match is found, that state remains unbound.

## Updates And Release Flow

- The app checks for updates through the Tauri updater plugin.
- Update metadata is fetched from GitHub Releases.
- The repository workflow publishes release artifacts for Apple Silicon macOS, Intel macOS, and Windows when pushing tags matching `app-v*`.

## Notes

- Copiwaifu modifies local AI CLI config files to install hooks. Review those changes if you already maintain custom hook chains.
- Runtime session files and primary port files are written under `~/.copiwaifu`.
- AI Talk settings, including API keys, are saved locally in the app settings file and are only used for the selected model provider.
- Session files keep bounded metadata for recovery and AI Talk, including recent event digests, `lastMeaningfulSummary`, and `aiTalkContext`.
- A temporary fallback port file is also written to the system temp directory.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).

Bundled Live2D Cubism Core files are distributed under their own license terms. See [public/Core/LICENSE.md](public/Core/LICENSE.md) and [public/Core/README.md](public/Core/README.md).
