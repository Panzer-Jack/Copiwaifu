# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是 Panzer-Jack.cn 博客的新框架，一个基于 Vue 3 + Vite + TypeScript 的个人博客网站。项目特色是带有哥特暗黑风格的交互式开场动画（Prologue），由虚拟角色"尤莉娅"引导访客。

## 常用命令

```bash
# 开发服务器 (端口 2233，自动打开浏览器)
pnpm dev

# 构建 (使用 vite-ssg 静态生成)
pnpm build

# 预览构建结果
pnpm preview
```

## 技术栈

- **框架**: Vue 3 + TypeScript + Vite 7
- **SSG**: vite-ssg (静态站点生成)
- **路由**: unplugin-vue-router (文件系统路由，支持 .vue 和 .md 文件)
- **状态管理**: Pinia
- **样式**: 强调优先使用（能使用就使用除非实现不了才用style取写）UnoCSS (Wind 3 预设 + 自定义哥特主题色)写法，然后样式使用px单位去写（比如w-10px)
- **自动导入**: unplugin-auto-import (Vue/VueUse/Router API 无需手动导入)
- **组件**: unplugin-vue-components (自动注册)
- **图标**: unplugin-icons (自定义 SVG 图标集合: text, object)
- **Markdown**: unplugin-vue-markdown (Markdown 作为 Vue 组件)
- **第三方 hooks**: vueuse

## 代码架构

```
src/
├── prologue/           # 开场动画系统 (核心功能)
│   ├── PrologueContainer.vue  # 开场容器，编排精灵图、对话框
│   └── stage1/         # 各阶段视觉效果 (预留)
├── components/
│   ├── dialogue/       # 对话框组件
│   ├── sprite/         # 精灵图组件 (尤莉娅)
│   └── ui/             # UI 组件 (跳过按钮等)
├── composables/        # 组合式函数
│   ├── useDialogue.ts  # 对话流程控制
│   ├── useTypewriter.ts # 打字机效果
│   └── useYuliaSprite.ts # 精灵图动画控制
├── stores/
│   └── prologue.ts     # 开场状态 (阶段、对话索引、表情)
├── types/
│   └── prologue.ts     # 类型定义 (精灵帧、表情、对话)
├── data/
│   └── dialogueScripts.ts # 对话剧本数据
└── assets/
    ├── icons/          # SVG 图标 (text/, object/)
    ├── images/         # 图片资源 (精灵图)
    └── styles/         # 样式 (哥特主题)
```

```
pages/                  # 文件系统路由
├── index.md           # 首页
├── [...404].md        # 404 页
└── posts/             # 博客文章
```

## 关键设计模式

### 开场动画系统 (Prologue)

开场动画采用多阶段对话系统：

- `stores/prologue.ts` 管理全局状态（当前阶段、对话索引、表情、完成状态）
- `data/dialogueScripts.ts` 定义阶段脚本，每个 `StageScript` 包含 `lines[]` 和 `nextStage`
- `composables/useDialogue.ts` 封装对话流程控制逻辑
- 表情类型: `neutral | smile | happy | surprised | thinking | wave | blink | shy`

### 路径别名

- `@/` 和 `~/` 都映射到 `src/`

### 自定义 SVG 图标

图标通过 `<IText名称 />` 或 `<IObject名称 />` 使用，SVG 自动处理 fill/stroke 为 currentColor。

## UnoCSS 主题色

```ts
// 哥特暗黑风配色
gothic: { bg, surface, elevated, border }
blood: { DEFAULT, light, glow }
violet: { DEFAULT, light, glow }
text: { primary, secondary, muted }
dialogue: { bg, border, nameBg }
```
