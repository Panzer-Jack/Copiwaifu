import { confirm } from '@tauri-apps/plugin-dialog'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { APP_LANGUAGE } from './types/agent'
import type { AppLanguage } from './types/agent'

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

export async function checkForAppUpdates(language: AppLanguage) {
  const update = await check()

  if (!update) {
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
