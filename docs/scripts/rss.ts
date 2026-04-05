import type { FeedOptions, Item } from 'feed'
import { dirname } from 'node:path'
import fg from 'fast-glob'
import { Feed } from 'feed'
import fs from 'fs-extra'
import matter from 'gray-matter'
// @ts-expect-error missing types
import MarkdownIt from 'markdown-it'

const DOMAIN = 'https://blog.panzer-jack.cn'
const AUTHOR = {
  name: 'Panzer-Jack',
  email: 'panzer_jack@panzer-jack.cn',
  link: DOMAIN,
}

const markdown = MarkdownIt({
  html: true,
  breaks: true,
  linkify: true,
})

async function run() {
  await buildBlogRSS()
}

async function buildBlogRSS() {
  const files = await fg('pages/posts/*.md')

  const options = {
    title: '老船长PZ_Jack の 博客',
    description: 'Panzer-Jack\'s Blog',
    id: `${DOMAIN}/`,
    link: `${DOMAIN}/`,
    copyright: `CC BY-NC-SA 4.0 2026 © Panzer-Jack`,
    feedLinks: {
      json: `${DOMAIN}/rss.json`,
      atom: `${DOMAIN}/rss.atom`,
      rss: `${DOMAIN}/rss.xml`,
    },
  }

  const posts: any[] = (
    await Promise.all(
      files.filter(i => !i.includes('index'))
        .map(async (i) => {
          const raw = await fs.readFile(i, 'utf-8')
          const { data, content } = matter(raw)

          if (data.draft)
            return

          const html = markdown.render(content)
            .replace('src="/', `src="${DOMAIN}/`)

          if (data.cover?.startsWith('/'))
            data.cover = DOMAIN + data.cover

          return {
            ...data,
            date: new Date(data.date),
            content: html,
            author: [AUTHOR],
            link: DOMAIN + i.replace(/^pages(.+)\.md$/, '$1'),
          }
        }),
    ))
    .filter(Boolean)

  posts.sort((a, b) => +new Date(b.date) - +new Date(a.date))

  await writeFeed('rss', options, posts)
}

async function writeFeed(name: string, options: FeedOptions, items: Item[]) {
  options.author = AUTHOR
  options.favicon = `${DOMAIN}/favicon.ico`

  const feed = new Feed(options)

  items.forEach(item => feed.addItem(item))

  await fs.ensureDir(dirname(`./dist/${name}`))
  await fs.writeFile(`./dist/${name}.xml`, feed.rss2(), 'utf-8')
  await fs.writeFile(`./dist/${name}.atom`, feed.atom1(), 'utf-8')
  await fs.writeFile(`./dist/${name}.json`, feed.json1(), 'utf-8')
}

run()
