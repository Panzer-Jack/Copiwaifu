# Copiwaifu Website

This directory contains the public Copiwaifu website source for https://copiwaifu.panzer-jack.cn/.

## Commands

```bash
pnpm dev      # start the website dev server on port 2233
pnpm build    # build the static site and RSS output
pnpm preview  # preview the built site
pnpm fix      # run ESLint auto-fix for src files
```

## Notes

- The main landing page lives in `src/App.vue`.
- The Live2D preview uses assets from `public/Core/` and `public/Resources/Yulia/`.
- The hero links to the Copiwaifu GitHub repository and latest release page.
- `index.html` owns the basic SEO description and keywords for the static page.
- `scripts/rss.ts` writes Copiwaifu RSS metadata for the static build.
