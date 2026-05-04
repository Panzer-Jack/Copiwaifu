# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Copiwaifu 是一个基于 Tauri 2 + Vue 3 的 macOS 桌面宠物应用。它使用内置的 Yulia Live2D 模型展示 AI 编程工具状态，并通过本地 hooks 同步 Claude Code、GitHub Copilot、Codex、Gemini CLI 和 OpenCode 的会话事件。

当前分支新增 AI Talk：当原 AI CLI 的一轮 session 进入 `complete` 或 `error` 后，应用可以基于已保存的 session 元信息和摘要调用用户配置的模型服务，生成一句适合桌宠气泡展示的短反馈。AI Talk 不是主动聊天窗口，也不会读取完整对话、项目文件或源码。

## Tech Stack

- **Frontend:** Vue 3 + TypeScript + Vite 6
- **Desktop:** Tauri 2 + Rust，macOS 使用 private API / NSPanel 行为
- **Rendering:** Pixi.js 8 + easy-live2d
- **AI Runtime:** Node.js sidecar + Vercel AI SDK + esbuild bundle
- **Package manager:** pnpm

## Common Commands

```bash
# Frontend dev server
pnpm dev

# Bundle the AI Talk Node runtime
pnpm sidecar:build

# Type-check, build frontend, and bundle AI runtime
pnpm build

# Tauri desktop app in dev mode
pnpm tauri dev

# Shortcut for pnpm tauri dev
pnpm run

# Tauri desktop build
pnpm tauri build

# ESLint
pnpm eslint .
```

## Architecture

### Frontend (`src/`)

- `App.vue` loads app bootstrap data, selects the main/settings window by Tauri window label, listens for settings and visibility events, and triggers update checks.
- `windows/MainWindow.vue` renders the desktop pet, state bubbles, context menu, Live2D motion playback, and AI Talk generation fallback logic.
- `windows/SettingsWindow.vue` manages user settings: language, character name, auto-start, model import/reset, window size, motion bindings, and AI Talk provider/API configuration.
- `components/` contains reusable UI pieces such as the speech bubble and context menu.
- `composables/` contains state/event helpers for navigator state, speech bubbles, context menu, and Live2D setup.
- `live2d/` contains model scanning, runtime loading, and motion-controller logic.
- `types/agent.ts` defines shared frontend types for agent state, app settings, sessions, motion bindings, and AI Talk context.

### Backend (`src-tauri/src/`)

- `lib.rs` wires Tauri plugins, windows, commands, tray/menu behavior, and the navigator runtime.
- `shell.rs` owns app settings, settings persistence, model import/validation, main/settings window management, tray menu, and auto-start handling.
- `navigator/` owns hook installation, local HTTP server startup, provider event parsing, state reduction, session reconciliation, session recovery, and persisted session snapshots.
- `navigator/reducer.rs` converts raw hook events into session phase/state, maintains bounded event digests, filters low-information summaries, and builds `aiTalkContext`.
- `navigator/session_store.rs` persists session files under `~/.copiwaifu/sessions`.
- `ai_talk.rs` validates AI Talk settings/context, applies bubble length limits, invokes the Node sidecar, emits debug events, and falls back silently on failure.

### AI Runtime

- `sidecar/ai-runtime/src/main.mjs` is the Vercel AI SDK entrypoint.
- `pnpm sidecar:build` bundles it to `sidecar/ai-runtime/bundle/main.mjs`.
- `tauri.conf.json` runs `pnpm run sidecar:build && pnpm dev` before Tauri dev, and packages the bundled sidecar as a resource for builds.
- The runtime currently supports OpenAI, Anthropic, Google Gemini, DeepSeek, Alibaba Bailian / Qwen, Moonshot Kimi, Zhipu GLM, Volcengine Ark / Doubao, Baidu Qianfan / ERNIE, Tencent Hunyuan, MiniMax, and generic OpenAI-compatible APIs.

### Hooks And Runtime Data

- Hook scripts live in `hooks/` and are installed into local AI CLI config files.
- Original hook definitions are backed up to `~/.copiwaifu/hooks/original-hooks.json`.
- Runtime session snapshots and AI Talk metadata are written to `~/.copiwaifu/sessions`.
- Port discovery writes under `~/.copiwaifu` and also uses `/tmp/copiwaifu-port` as a fallback.
- App settings, including AI Talk provider profiles and API keys, are stored in the Tauri app config directory as `settings.json`.

## Static Assets

- `public/Core/` contains Live2D Cubism Core runtime files.
- `public/Resources/Yulia/` contains the bundled Yulia model, motions, physics, textures, and model metadata.
- `src-tauri/icons/` contains desktop app icons.

## Implementation Notes

- AI Talk should only trigger for `complete` and `error`, only once per terminal turn, and only when `aiTalkContext.hasContext` is true.
- `needs_attention` continues to use static bubbles and must not trigger AI Talk.
- Generated AI Talk text must be capped by language and window size before display.
- Missing model config, missing context, sidecar failure, network failure, or provider errors should fall back to existing static bubbles without showing model errors in the pet bubble.
