import { confirm } from '@tauri-apps/plugin-dialog'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { APP_LANGUAGE } from './types/agent'
import type { AppLanguage } from './types/agent'

const UPDATE_CHECK_MAX_ATTEMPTS = 3
const UPDATE_CHECK_RETRY_DELAY_MS = 5_000

type UpdateCopy = {
  availableTitle: string
  availableMessage: (version: string, date: string | null, notes: string | null) => string
  installNow: string
  installLater: string
}

const UPDATE_COPY: Record<AppLanguage, UpdateCopy> = {
  [APP_LANGUAGE.ENGLISH]: {
    availableTitle: 'Update Available',
    availableMessage: (version, date, notes) => {
      const lines = [`Version ${version} is available.`]

      if (date) {
        lines.push(`Release date: ${date}`)
      }
      if (notes) {
        lines.push('', 'Release notes:', notes)
      }

      lines.push('', 'Install now and restart the app?')
      return lines.join('\n')
    },
    installNow: 'Install Now',
    installLater: 'Later',
  },
  [APP_LANGUAGE.CHINESE]: {
    availableTitle: '发现新版本',
    availableMessage: (version, date, notes) => {
      const lines = [`检测到新版本 ${version}。`]

      if (date) {
        lines.push(`发布日期：${date}`)
      }
      if (notes) {
        lines.push('', '更新说明：', notes)
      }

      lines.push('', '是否现在安装并重启应用？')
      return lines.join('\n')
    },
    installNow: '立即安装',
    installLater: '稍后',
  },
}

function getUpdateCopy(language: AppLanguage) {
  return UPDATE_COPY[language] ?? UPDATE_COPY[APP_LANGUAGE.ENGLISH]
}

function normalizeText(value: string | null | undefined) {
  const trimmed = value?.trim()
  return trimmed ? trimmed : null
}

function wait(ms: number) {
  return new Promise((resolve) => window.setTimeout(resolve, ms))
}

async function checkWithRetry() {
  let lastError: unknown = null

  for (let attempt = 1; attempt <= UPDATE_CHECK_MAX_ATTEMPTS; attempt += 1) {
    try {
      console.info(`[updater] checking for updates (attempt ${attempt}/${UPDATE_CHECK_MAX_ATTEMPTS})`)
      return await check()
    }
    catch (error) {
      lastError = error
      console.warn(`[updater] update check failed on attempt ${attempt}`, error)

      if (attempt < UPDATE_CHECK_MAX_ATTEMPTS) {
        await wait(UPDATE_CHECK_RETRY_DELAY_MS)
      }
    }
  }

  throw lastError instanceof Error ? lastError : new Error(String(lastError))
}

export async function checkForAppUpdates(language: AppLanguage) {
  const update = await checkWithRetry()

  if (!update) {
    console.info('[updater] app is already up to date')
    return false
  }

  const copy = getUpdateCopy(language)
  const shouldInstall = await confirm(
    copy.availableMessage(
      update.version,
      normalizeText(update.date),
      normalizeText(update.body),
    ),
    {
      title: copy.availableTitle,
      kind: 'info',
      okLabel: copy.installNow,
      cancelLabel: copy.installLater,
    },
  )

  if (!shouldInstall) {
    console.info(`[updater] postponed update ${update.version}`)
    await update.close()
    return true
  }

  let downloaded = 0
  let contentLength = 0

  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case 'Started':
        contentLength = event.data.contentLength ?? 0
        console.info(`[updater] started downloading ${contentLength} bytes`)
        break
      case 'Progress':
        downloaded += event.data.chunkLength
        console.info(`[updater] downloaded ${downloaded} / ${contentLength}`)
        break
      case 'Finished':
        console.info('[updater] download finished')
        break
    }
  })

  console.info(`[updater] installed version ${update.version}`)
  await update.close()
  await relaunch()
  return true
}
