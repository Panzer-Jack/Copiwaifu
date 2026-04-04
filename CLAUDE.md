# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Copiwaifu 是一个基于 Tauri 2 + Vue 3 的桌面宠物应用，集成 Live2D 角色（Hiyori），作为 AI Navigator 与 ClaudeCode、Copilot、Codex 等代码编辑器协同工作。

## Tech Stack

- **前端:** Vue 3 + TypeScript + Vite 6
- **桌面框架:** Tauri 2（macOS 使用 private API 实现面板行为）
- **渲染:** Pixi.js 8 (WebGL) + easy-live2d 0.4
- **后端:** Rust (Tauri runtime)
- **包管理:** pnpm

## Common Commands

```bash
# 前端开发服务器 (port 1420)
pnpm dev

# 类型检查 + 构建
pnpm build

# Tauri 桌面应用开发模式
pnpm tauri dev

# Tauri 桌面应用构建
pnpm tauri build

# ESLint 检查
pnpm eslint .
```

## Architecture

### 前端 (`src/`)

单组件架构 — `App.vue` 是唯一的业务组件，负责：
- 初始化 Pixi.js Application 作为 WebGL 渲染容器
- 通过 easy-live2d 加载和管理 Live2D 模型
- 处理窗口拖拽（Tauri drag region）
- 响应式画布适配（devicePixelRatio）

### 后端 (`src-tauri/src/`)

- `lib.rs` — Tauri 主配置，包含 Rust commands、macOS 面板行为设置（NSPanel）、窗口属性配置
- `main.rs` — 入口，调用 `copiwaifu_lib::run()`
- macOS 特有：通过 `tauri-nspanel` 实现桌面宠物的置顶面板、隐藏 Dock 图标等行为

### 静态资源 (`public/`)

- `Core/` — Live2D Cubism SDK 核心库
- `Resources/Hiyori/` — Live2D 模型文件（moc3、动作、音频、纹理）

### 窗口特性

应用窗口 200x500px，不可调整大小，透明背景，始终置顶，macOS 下隐藏 Dock 图标。

## ESLint

使用 `@panzerjack/eslint-config`，启用 Vue + TypeScript + pnpm + formatters 规则集。
