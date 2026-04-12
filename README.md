[English](README.md) | [简体中文](README.zh-CN.md)

# Copiwaifu

Your Live2D AI navigator for everyday coding sessions.

Copiwaifu is a Tauri desktop pet that mirrors the state of your AI coding tools and turns that activity into a small Live2D companion on your desktop. It currently focuses on syncing with Claude Code, GitHub Copilot, Codex, Gemini CLI, and OpenCode.

## Website

- Copiwaifu: https://copiwaifu.panzer-jack.cn/

<img width="1512" height="824" alt="image" src="https://github.com/user-attachments/assets/7c387482-58ba-4e61-b500-0b7c0b01404a" />


## After Installing On macOS（important！！！）

After moving the app into `/Applications`, run:

```bash
xattr -dr com.apple.quarantine /Applications/copiwaifu.app
```

## What It Does

- Shows a Live2D desktop companion that reacts to AI session state changes.
- Syncs status such as `idle`, `thinking`, `tool_use`, `error`, `complete`, and `needs_attention`.
- Displays short speech bubbles based on the current agent and state.
- Supports a built-in Live2D model and custom model folders.
- Lets you bind model motion groups to each runtime state.
- Includes a settings window for language, name, auto-start, model selection, and window size.
- Provides a tray menu for visibility, settings, and exit.
- Checks for app updates from GitHub Releases.

## How It Works

When the app starts, it:

1. Launches a local HTTP server on `127.0.0.1` using port `23333` and falls back across the next available ports if needed.
2. Installs local hooks for supported AI CLIs.
3. Receives hook events and converts them into Copiwaifu state changes.
4. Persists local runtime data under `~/.copiwaifu`.

The hook installer currently integrates with:

- Claude Code via `~/.claude/settings.json`
- GitHub Copilot via `~/.config/github-copilot/config.json`
- Codex via `~/.codex/config.toml`
- Gemini CLI via `~/.gemini/settings.json`
- OpenCode via `~/.config/opencode/opencode.json`

Original hook definitions are backed up to `~/.copiwaifu/hooks/original-hooks.json`.

## Platform

Copiwaifu is currently built and released for macOS. The app uses macOS-specific window behavior and the release workflow only publishes macOS artifacts.

## Tech Stack

- Vue 3
- TypeScript
- Vite
- Tauri 2
- Rust
- PixiJS
- `easy-live2d`

## Requirements

Before running the project locally, make sure you have:

- Node.js
- `pnpm`
- Rust toolchain
- Tauri system prerequisites for macOS
- At least one supported AI CLI if you want live session sync

The hook bridge also expects `node` to be available in a standard shell path.

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
pnpm build                 # build the frontend bundle
pnpm tauri dev             # run the desktop app in dev mode
pnpm run                   # shortcut for pnpm tauri dev
pnpm tauri build           # build desktop artifacts locally
pnpm release               # run release-it
pnpm release:sync-version  # sync version across package.json / tauri.conf.json / Cargo.toml
```

## Settings

From the settings window you can configure:

- Pet name
- UI language: English or Chinese
- Auto start on login
- Live2D model directory
- Window size preset
- Motion group binding for each agent state

If no custom model is selected, Copiwaifu uses the bundled `Yulia` model.

## Custom Live2D Models

Custom model folders are validated before saving. A usable model directory should include a valid `.model3.json` entry file and its referenced assets.

To make state animations useful, define motion groups in your model and bind them in settings. If a state has no explicit binding, Copiwaifu falls back to the idle binding when possible.

## Updates And Release Flow

- The app checks for updates through the Tauri updater plugin.
- Update metadata is fetched from GitHub Releases.
- The repository workflow publishes release artifacts for Apple Silicon and Intel macOS when pushing tags matching `app-v*`.

## Notes

- Copiwaifu modifies local AI CLI config files to install hooks. Review those changes if you already maintain custom hook chains.
- Runtime session files and port files are written under `~/.copiwaifu`.
- A temporary fallback port file is also written to `/tmp/copiwaifu-port`.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).

Bundled Live2D Cubism Core files are distributed under their own license terms. See [public/Core/LICENSE.md](public/Core/LICENSE.md) and [public/Core/README.md](public/Core/README.md).
