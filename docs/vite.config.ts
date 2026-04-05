import path from 'node:path'
import Vue from '@vitejs/plugin-vue'
import VueJsx from '@vitejs/plugin-vue-jsx'
import fs from 'fs-extra'
import matter from 'gray-matter'
import MarkdownItAnchor from 'markdown-it-anchor'
import MarkdownItPrism from 'markdown-it-prism'
// @ts-expect-error missing types
import MarkdownItTOC from 'markdown-it-table-of-contents'
import readingTime from 'reading-time'
import UnoCSS from 'unocss/vite'
import AutoImport from 'unplugin-auto-import/vite'
import { FileSystemIconLoader } from 'unplugin-icons/loaders'
import IconsResolver from 'unplugin-icons/resolver'
import Icons from 'unplugin-icons/vite'
import Components from 'unplugin-vue-components/vite'
import Markdown from 'unplugin-vue-markdown/vite'
import { VueRouterAutoImports } from 'unplugin-vue-router'
import VueRouter from 'unplugin-vue-router/vite'
import { defineConfig } from 'vite'

function _resolve(dir: string) {
  return path.resolve(__dirname, dir)
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    Vue({
      include: [/\.vue$/, /\.md$/],
    }),
    VueJsx(),
    UnoCSS(),
    VueRouter({
      extensions: ['.vue', '.md'],
      routesFolder: 'pages',
      // logs: true,
      extendRoute(route) {
        const path = route.components.get('default')
        if (!path)
          return

        if (path.endsWith('.md')) {
          const rawContent = fs.readFileSync(path, 'utf-8')
          const { data } = matter(rawContent)
          route.addToMeta({
            frontmatter: {
              duration: Number(readingTime(rawContent).minutes.toFixed(0)) + 1,
              ...data,
            },
          })
        }
      },
    }),
    Markdown({
      headEnabled: true,
      markdownItOptions: {
        html: true,
        linkify: true,
        typographer: true,
      },
      markdownItSetup(md) {
        md.use(MarkdownItAnchor)
        md.use(MarkdownItPrism)
        md.use(MarkdownItTOC, {
          includeLevel: [1, 2, 3, 4],
          containerHeaderHtml: '<div class="table-of-contents-anchor"><div class="i-ri-menu-2-fill" /></div>',
        })
      },
      wrapperClasses: 'markdown-body',
    }),
    AutoImport({
      imports: [
        'vue',
        VueRouterAutoImports,
        '@vueuse/core',
      ],
    }),
    // https://github.com/unplugin/unplugin-vue-components
    Components({
      dirs: ['src/components', 'src/demos'],
      dts: true,
      extensions: ['vue', 'md'],
      include: [/\.vue$/, /\.vue\?vue/, /\.md$/],
      resolvers: [
        IconsResolver({ customCollections: ['text', 'object'] }),
      ],
    }),
    // https://github.com/unplugin/unplugin-icons/blob/main/examples/vite-vue3/vite.config.ts
    Icons({
      autoInstall: true,
      compiler: 'vue3',
      customCollections: {
        text: FileSystemIconLoader(
          './src/assets/icons/text',
          svgLoader,
        ),
        object: FileSystemIconLoader(
          './src/assets/icons/object',
        ),
        project: FileSystemIconLoader(
          './src/assets/icons/project',
        ),
      },
    }),
  ],
  resolve: {
    alias: {
      '@': _resolve('./src'),
    },
  },
  server: {
    allowedHosts: true,
    host: '0.0.0.0',
    port: 2233,
  },
})

function svgLoader(svg: string) {
  // 1. 移除 <style>...</style> 块（如果想保留可去掉这一行）
  svg = svg.replace(/<style[^>]*>[\s\S]*?<\/style>/gi, '')

  // 2. 处理包含 style="...": 把其中的 fill / stroke 属性值替换为 currentColor
  svg = svg.replace(/style="([^"]*)"/gi, (m, content) => {
    const replaced = content
      // fill: xxx -> fill:currentColor
      .replace(/fill\s*:[^;"}]+/gi, 'fill:currentColor')
      // stroke: xxx -> stroke:currentColor
      .replace(/stroke\s*:[^;"}]+/gi, 'stroke:currentColor')
    return `style="${replaced}"`
  })

  // 3. 单独的 fill / stroke 属性（避免已经是 currentColor 再替换）
  svg = svg.replace(/\sfill="(?!currentColor)[^"]*"/gi, ' fill="currentColor"')
    .replace(/\sstroke="(?!currentColor)[^"]*"/gi, ' stroke="currentColor"')

  // 4. 给 <svg> 补充 fill="currentColor"（若不存在）
  svg = svg.replace(/<svg\b([^>]*)>/i, (m, attrs) => {
    if (!/fill=/.test(attrs)) {
      return `<svg${attrs} fill="currentColor">`
    }
    return m
  })

  return svg
}
