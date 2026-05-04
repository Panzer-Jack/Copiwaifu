# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working in the `docs/` website project.

## Project Overview

`docs/` is the public Copiwaifu website, not the desktop app runtime. It is a Vue/Vite static site that presents the product, links to GitHub releases, and renders a Live2D Yulia preview using the same public model assets.

## Common Commands

```bash
# Dev server on port 2233
pnpm dev

# Static build plus RSS generation
pnpm build

# Preview build output
pnpm preview

# ESLint auto-fix for src files
pnpm fix
```

## Tech Stack

- **Framework:** Vue 3 + TypeScript + Vite 7
- **SSG:** vite-ssg
- **Styles:** UnoCSS plus scoped Vue styles
- **Rendering:** Pixi.js + easy-live2d for the Yulia preview
- **Routing/Markdown tooling:** vue-router, unplugin-vue-router, unplugin-vue-markdown
- **Package manager:** pnpm

## Code Structure

- `src/App.vue` contains the current landing page hero, release/GitHub links, background imagery, and Live2D preview mount.
- `src/components/sprite.vue` renders the Live2D sprite preview.
- `src/composables/useSpeechBubble.ts` contains the small preview bubble behavior.
- `index.html` owns the static SEO description and keyword metadata.
- `src/assets/images/main.png` is the main hero background image.
- `public/Core/` contains Live2D Cubism Core runtime files.
- `public/Resources/Yulia/` contains the public Yulia model assets used by the preview.
- `scripts/rss.ts` is run after static build and should keep Copiwaifu website metadata, not old blog metadata.

## Notes

- Keep website copy aligned with the root README and current desktop app behavior.
- The website should describe Copiwaifu as the Live2D AI desktop companion with optional AI Talk, not as a personal blog framework.
- Avoid changing generated output files unless a build step intentionally regenerates them.
