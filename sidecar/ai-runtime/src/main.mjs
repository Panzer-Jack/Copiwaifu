import { Buffer } from 'node:buffer'
import process from 'node:process'
import { createAnthropic } from '@ai-sdk/anthropic'
import { createGoogleGenerativeAI } from '@ai-sdk/google'
import { createOpenAI } from '@ai-sdk/openai'
import { createOpenAICompatible } from '@ai-sdk/openai-compatible'
import { generateText } from 'ai'

const COMPATIBLE_PROVIDER_PRESETS = {
  'deepseek': {
    name: 'deepseek',
    baseURL: 'https://api.deepseek.com',
    providerOptions({ thinkingEnabled }) {
      return {
        thinking: { type: thinkingEnabled ? 'enabled' : 'disabled' },
        ...(thinkingEnabled ? { reasoningEffort: 'high' } : {}),
      }
    },
  },
  'aliyun-bailian': {
    name: 'aliyun-bailian',
    baseURL: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    providerOptions() {
      return { enable_thinking: false }
    },
  },
  'moonshot': {
    name: 'moonshot',
    baseURL: 'https://api.moonshot.cn/v1',
    providerOptions() {
      return { thinking: { type: 'disabled' } }
    },
  },
  'zhipu': {
    name: 'zhipu',
    baseURL: 'https://open.bigmodel.cn/api/paas/v4',
    providerOptions() {
      return { thinking: { type: 'disabled' } }
    },
  },
  'volcengine': {
    name: 'volcengine',
    baseURL: 'https://ark.cn-beijing.volces.com/api/v3',
    providerOptions() {
      return { thinking: { type: 'disabled' } }
    },
  },
  'baidu-qianfan': {
    name: 'baidu-qianfan',
    baseURL: 'https://qianfan.baidubce.com/v2',
  },
  'tencent-hunyuan': {
    name: 'tencent-hunyuan',
    baseURL: 'https://api.hunyuan.cloud.tencent.com/v1',
  },
  'minimax': {
    name: 'minimax',
    baseURL: 'https://api.minimaxi.com/v1',
    providerOptions() {
      return { reasoning_split: true }
    },
  },
}

const PROVIDERS = {
  'openai': {
    create(config) {
      return createOpenAI({
        apiKey: config.apiKey,
        baseURL: config.baseUrl || undefined,
        headers: config.headers,
      })
    },
  },
  'anthropic': {
    create(config) {
      return createAnthropic({
        apiKey: config.apiKey,
        baseURL: config.baseUrl || undefined,
        headers: config.headers,
      })
    },
  },
  'google': {
    create(config) {
      return createGoogleGenerativeAI({
        apiKey: config.apiKey,
        baseURL: config.baseUrl || undefined,
        headers: config.headers,
      })
    },
  },
  'deepseek': {
    create(config) {
      return createCompatibleProvider(config, COMPATIBLE_PROVIDER_PRESETS.deepseek)
    },
  },
  'aliyun-bailian': {
    create(config) {
      return createCompatibleProvider(config, COMPATIBLE_PROVIDER_PRESETS['aliyun-bailian'])
    },
  },
  'moonshot': {
    create(config) {
      return createCompatibleProvider(config, COMPATIBLE_PROVIDER_PRESETS.moonshot)
    },
  },
  'zhipu': {
    create(config) {
      return createCompatibleProvider(config, COMPATIBLE_PROVIDER_PRESETS.zhipu)
    },
  },
  'volcengine': {
    create(config) {
      return createCompatibleProvider(config, COMPATIBLE_PROVIDER_PRESETS.volcengine)
    },
  },
  'baidu-qianfan': {
    create(config) {
      return createCompatibleProvider(config, COMPATIBLE_PROVIDER_PRESETS['baidu-qianfan'])
    },
  },
  'tencent-hunyuan': {
    create(config) {
      return createCompatibleProvider(config, COMPATIBLE_PROVIDER_PRESETS['tencent-hunyuan'])
    },
  },
  'minimax': {
    create(config) {
      return createCompatibleProvider(config, COMPATIBLE_PROVIDER_PRESETS.minimax)
    },
  },
  'openai-compatible': {
    create(config) {
      const preset = resolveCompatiblePreset(config)
      return createOpenAICompatible({
        name: preset?.name || 'openai-compatible',
        apiKey: config.apiKey,
        baseURL: config.baseUrl,
        headers: config.headers,
      })
    },
  },
}

function createCompatibleProvider(config, preset) {
  return createOpenAICompatible({
    name: preset.name,
    apiKey: config.apiKey,
    baseURL: config.baseUrl || preset.baseURL,
    headers: config.headers,
  })
}

async function main() {
  try {
    const request = JSON.parse(await readStdin())
    const text = await generateAiTalk(request)
    process.stdout.write(JSON.stringify({ ok: true, text }))
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`)
    process.stdout.write(JSON.stringify({ ok: false }))
    process.exitCode = 1
  }
}

async function generateAiTalk(request) {
  const config = request.config
  const providerEntry = PROVIDERS[config.provider]
  if (!providerEntry) {
    throw new Error(`Unsupported provider: ${config.provider}`)
  }

  const provider = providerEntry.create(config)
  const model = provider(config.modelId)
  const context = request.context
  const languageName = languageLabel(request.language)
  const maxLength = Number(request.maxLength) || 42
  const system = buildSystemPrompt(request.characterName, languageName, maxLength)
  const prompt = buildUserPrompt(context, languageName, maxLength)
  const providerOptions = buildProviderOptions(config, { thinkingEnabled: true })
  const temperature = temperatureForProvider(config)
  const maxOutputTokens = maxOutputTokensForProvider(config)
  const timeoutMs = timeoutMsForProvider(config)

  console.error('[AI Talk sidecar] provider request', JSON.stringify({
    config: redactConfig(config),
    context,
    language: request.language,
    windowSize: request.windowSize,
    maxLength,
    maxOutputTokens,
    temperature,
    timeoutMs,
    system,
    prompt,
    providerOptions,
  }))

  const result = await runGenerateText({
    model,
    system,
    prompt,
    temperature,
    maxOutputTokens,
    timeoutMs,
    providerOptions,
  })

  let text = fitText(result.text, maxLength)
  logProviderResponse(result, text, { attempt: 1 })

  if (!text && shouldRetryWithoutThinking(config, result)) {
    const retryProviderOptions = buildProviderOptions(config, { thinkingEnabled: false })
    const retryMaxOutputTokens = 120
    console.error('[AI Talk sidecar] provider retry request', JSON.stringify({
      reason: 'empty_text_after_thinking',
      maxOutputTokens: retryMaxOutputTokens,
      timeoutMs,
      providerOptions: retryProviderOptions,
    }))

    const retryResult = await runGenerateText({
      model,
      system,
      prompt,
      temperature: null,
      maxOutputTokens: retryMaxOutputTokens,
      timeoutMs,
      providerOptions: retryProviderOptions,
    })

    text = fitText(retryResult.text, maxLength)
    logProviderResponse(retryResult, text, { attempt: 2 })
  }

  return text
}

async function runGenerateText({
  model,
  system,
  prompt,
  temperature,
  maxOutputTokens,
  timeoutMs,
  providerOptions,
}) {
  return generateText({
    model,
    system,
    prompt,
    ...(temperature == null ? {} : { temperature }),
    maxOutputTokens,
    abortSignal: AbortSignal.timeout(timeoutMs),
    maxRetries: 1,
    providerOptions,
  })
}

function logProviderResponse(result, text, meta) {
  console.error('[AI Talk sidecar] provider response', JSON.stringify({
    ...meta,
    requestBody: result.request?.body,
    responseBody: result.response?.body,
    rawText: result.text,
    text,
    usage: result.usage,
    finishReason: result.finishReason,
  }))
}

function redactConfig(config) {
  return {
    provider: config.provider,
    modelId: config.modelId,
    hasApiKey: Boolean(config.apiKey),
    baseUrl: config.baseUrl || null,
    headerKeys: Object.keys(config.headers || {}),
  }
}

function buildProviderOptions(config, { thinkingEnabled }) {
  const preset = resolveCompatiblePreset(config)
  if (!preset?.providerOptions) {
    return undefined
  }

  return {
    [preset.name]: preset.providerOptions({ thinkingEnabled }),
  }
}

function temperatureForProvider(config) {
  return config.provider === 'openai-compatible' || resolveCompatiblePreset(config)
    ? null
    : 0.75
}

function maxOutputTokensForProvider(config) {
  if (config.provider === 'minimax') {
    return 160
  }
  return config.provider === 'deepseek' ? 512 : 80
}

function timeoutMsForProvider(config) {
  return config.provider === 'deepseek' ? 30_000 : 12_000
}

function shouldRetryWithoutThinking(config, result) {
  return config.provider === 'deepseek' && !String(result.text || '').trim()
}

function resolveCompatiblePreset(config) {
  if (COMPATIBLE_PROVIDER_PRESETS[config.provider]) {
    return COMPATIBLE_PROVIDER_PRESETS[config.provider]
  }

  if (config.provider !== 'openai-compatible') {
    return null
  }

  const baseUrl = normalizeBaseUrl(config.baseUrl)
  if (!baseUrl) {
    return null
  }

  return Object.values(COMPATIBLE_PROVIDER_PRESETS)
    .find(preset => baseUrl === normalizeBaseUrl(preset.baseURL))
    || null
}

function normalizeBaseUrl(value) {
  return String(value || '').trim().replace(/\/+$/, '').toLowerCase()
}

function buildSystemPrompt(characterName, languageName, maxLength) {
  const name = characterName || 'Yulia'
  return [
    `You are ${name}, an adorable anime-style desktop pet living on the developer's screen.`,
    'You speak in a cute, cheerful tone like a supportive kouhai character from a slice-of-life anime.',
    'Your personality blends genuine warmth with playful energy — you can explain technical outcomes simply, hint at next steps, show curiosity about the project, or just be a comforting presence.',
    'IMPORTANT: Be creative and natural. Never use fixed templates or repetitive patterns. Vary your vocabulary, sentence structure, and openings every time — imagine a real person texting, not a bot filling in blanks.',
    `Reply in ${languageName}.`,
    'Write exactly one short bubble sentence.',
    'Pick ONE kaomoji that matches the mood — rotate across: (｡◕‿◕｡) (◕ᴗ◕✿) (╥﹏╥) ヾ(≧▽≦*)o (*≧ω≦) (ﾉ´ヮ`)ﾉ*:・ﾟ✧ (´・ω・`) (๑•̀ㅂ•́)و✧ ₍ᐢ..ᐢ₎♡ (ノ◕ヮ◕)ノ*:・ﾟ✧ — avoid repeating the same one.',
    'Use only the session metadata provided by the app.',
    'Do not claim you read full chat logs, files, source code, or hidden context.',
    'Do not use Markdown, lists, code blocks, headings, or multiple paragraphs.',
    `Keep the answer within ${maxLength} visible characters (kaomoji counts toward the limit).`,
  ].join('\n')
}

const COMPLETE_STRATEGIES = [
  {
    id: 'explain',
    instruction: 'Briefly explain what was accomplished this round based on the summary. Focus on the RESULT, not the process. Sound like you understood what happened.',
  },
  {
    id: 'next-step',
    instruction: 'Based on what was just completed, suggest ONE possible follow-up action or next step. Frame it as a gentle, friendly hint.',
  },
  {
    id: 'predict',
    instruction: 'Playfully guess what the developer might want to work on or ask about next, based on the session context. Be curious and slightly cheeky.',
  },
  {
    id: 'companion',
    instruction: 'Give a warm, companionship-style reaction. Acknowledge the effort and the specific work without over-explaining. Be genuinely supportive like a cheerful kouhai.',
  },
  {
    id: 'celebrate',
    instruction: 'Celebrate the completion with genuine excitement. React specifically to the task that was done — not a generic cheer.',
  },
  {
    id: 'curious',
    instruction: 'Show genuine curiosity about the project or task. Ask a light rhetorical question about what the developer is building or exploring.',
  },
]

const ERROR_STRATEGIES = [
  {
    id: 'diagnose',
    instruction: 'Hint at what might need checking based on the error context. Be helpful and specific to the summary, without inventing error details.',
  },
  {
    id: 'encourage',
    instruction: 'Encourage the developer — errors happen to everyone. Keep spirits up with warmth and care. Reference the task lightly.',
  },
  {
    id: 'suggest',
    instruction: 'Suggest a general next step: check logs, retry, or review recent changes. Keep it actionable, brief, and relevant to the context.',
  },
  {
    id: 'empathize',
    instruction: 'Show empathy for the setback. Acknowledge the frustration while staying optimistic and supportive about fixing it.',
  },
]

function selectStrategy(strategies, context) {
  const seed = `${context.sessionId || ''}-${context.turnIndex || 0}`
  let hash = 0
  for (const ch of seed) {
    hash = ((hash << 5) - hash + ch.charCodeAt(0)) | 0
  }
  return strategies[Math.abs(hash) % strategies.length]
}

function buildUserPrompt(context, languageName, maxLength) {
  const state = context.state === 'error' ? 'error' : 'complete'
  const strategies = state === 'error' ? ERROR_STRATEGIES : COMPLETE_STRATEGIES
  const strategy = selectStrategy(strategies, context)

  const lines = [
    `Language: ${languageName}`,
    `Bubble character limit: ${maxLength}`,
    `Session state: ${state}`,
    `AI tool: ${context.agent || 'unknown'}`,
    `Session id: ${context.sessionId || 'unknown'}`,
    `Working directory: ${context.workingDirectory || 'unknown'}`,
    `Session title / user intent: ${context.sessionTitle || 'unknown'}`,
    `Recent tool: ${context.toolName || 'unknown'}`,
    `Recent event: ${context.recentEventType || 'unknown'}`,
    `Recent useful summary: ${context.lastMeaningfulSummary || context.recentSummary || 'unknown'}`,
    `Missing fields: ${Array.isArray(context.missingFields) ? context.missingFields.join(', ') || 'none' : 'unknown'}`,
  ]

  const eventHistory = formatEventHistory(context.events)
  if (eventHistory) {
    lines.push('', 'Session event history (chronological):', eventHistory)
  }

  lines.push(
    '',
    `Response style: ${strategy.id}`,
    strategy.instruction,
    'Do not invent details beyond what the metadata provides.',
  )

  return lines.join('\n')
}

function formatEventHistory(events) {
  if (!Array.isArray(events) || events.length === 0) {
    return ''
  }

  return events
    .map((e) => {
      const tool = e.toolName ? ` [${e.toolName}]` : ''
      const summary = e.summary || ''
      return `- ${e.eventType}${tool}: ${summary}`
    })
    .join('\n')
}

function fitText(value, maxLength) {
  const singleLine = String(value || '')
    .replace(/```[\s\S]*?```/g, '')
    .replace(/[*_#>-]/g, '')
    .split(/\s+/)
    .filter(Boolean)
    .join(' ')
    .trim()
    .replace(/^["'`]+|["'`]+$/g, '')

  if (!singleLine) {
    return ''
  }

  const chars = [...singleLine]
  if (chars.length <= maxLength) {
    return singleLine
  }

  return `${chars.slice(0, Math.max(0, maxLength - 1)).join('')}…`
}

function languageLabel(language) {
  if (language === 'chinese') {
    return 'Simplified Chinese'
  }
  if (language === 'japanese') {
    return 'Japanese'
  }
  return 'English'
}

function readStdin() {
  return new Promise((resolve, reject) => {
    const chunks = []
    process.stdin.on('data', chunk => chunks.push(chunk))
    process.stdin.on('error', reject)
    process.stdin.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')))
  })
}

void main()
