#!/usr/bin/env node

const fs = require('node:fs')
const http = require('node:http')
const os = require('node:os')
const path = require('node:path')
const { spawn } = require('node:child_process')

const args = process.argv.slice(2)
const agent = args[args.indexOf('--agent') + 1]
const rawEvent = args[args.indexOf('--event') + 1]
const argvJson = args.find(a => a.startsWith('{'))

if (!agent || !rawEvent) process.exit(0)

const CLAUDE_MAP = {
  SessionStart: 'session_start', SessionEnd: 'session_end',
  UserPromptSubmit: 'thinking', PreToolUse: 'tool_use',
  PostToolUse: 'tool_result', PostToolUseFailure: 'error',
  Stop: 'complete', Notification: 'permission_request', PermissionRequest: 'permission_request',
  Elicitation: 'tool_use', SubagentStart: 'tool_use', SubagentStop: 'tool_use',
  PreCompact: 'tool_use', PostCompact: 'tool_use', WorktreeCreate: 'tool_use',
}
const COPILOT_MAP = {
  sessionStart: 'session_start', sessionEnd: 'session_end',
  userPromptSubmitted: 'thinking', preToolUse: 'tool_use',
  postToolUse: 'tool_result', errorOccurred: 'error', agentStop: 'complete',
}
const GEMINI_MAP = {
  SessionStart: 'session_start', SessionEnd: 'session_end',
  BeforeTool: 'tool_use', AfterTool: 'tool_result',
  BeforeAgent: 'tool_use', AfterAgent: 'tool_result',
}
const CODEX_MAP = { notify: 'tool_use' }
const PASSTHROUGH = new Set(['session_start','session_end','thinking','tool_use','tool_result','error','complete','permission_request'])

function normalizeEvent(ev) {
  if (PASSTHROUGH.has(ev)) return ev
  if (agent === 'claude-code') return CLAUDE_MAP[ev] || null
  if (agent === 'copilot') return COPILOT_MAP[ev] || null
  if (agent === 'gemini') return GEMINI_MAP[ev] || null
  if (agent === 'codex') return CODEX_MAP[ev] || null
  return null
}

const PORT_FILES = [
  path.join(os.homedir(), '.copiwaifu', 'port'),
  path.join(os.tmpdir(), 'copiwaifu-port'),
]
const SESSION_DIR = path.join(os.homedir(), '.copiwaifu', 'sessions')
const HOOKS_FILE = path.join(os.homedir(), '.copiwaifu', 'hooks', 'original-hooks.json')

let handled = false
let rawInput = ''

if (agent === 'codex' && argvJson) {
  setImmediate(() => handle(argvJson))
} else {
  const chunks = []
  process.stdin.on('data', c => chunks.push(c))
  process.stdin.on('end', () => handle(Buffer.concat(chunks).toString('utf8')))
  setTimeout(() => handle(''), 300)
}

function handle(input) {
  if (handled) return
  handled = true
  rawInput = input

  const ctx = parseJson(input)
  const mappedEvent = resolveMappedEvent(ctx)
  if (!mappedEvent) {
    chainHook(agent, rawEvent, rawInput)
    return process.exit(0)
  }
  const sessionId = ctx.session_id || ctx.sessionId || ctx['thread-id'] || `${agent}-${process.ppid}`
  const toolName = ctx.tool_name || ctx.toolName || ctx.name || agent
  const summary = resolveSummary(ctx, agent, mappedEvent)
  const workingDirectory = ctx.cwd || ctx.workingDirectory || ctx.working_directory
  const sessionTitle = resolveSessionTitle(ctx, mappedEvent)
  const needsAttention = mappedEvent === 'permission_request' || ['bash', 'execute_command'].includes(toolName.toLowerCase())
  const turnStart = isTurnStartEvent(agent, rawEvent, mappedEvent)
  const turnFingerprint = turnStart ? (sessionTitle || summary) : undefined

  const data = {
    tool_name: toolName,
    summary,
    working_directory: workingDirectory,
    session_title: sessionTitle,
    needs_attention: needsAttention,
    turn_start: turnStart,
    turn_fingerprint: turnFingerprint,
  }
  const payload = { agent, session_id: sessionId, event: mappedEvent, data }

  writeSession(sessionId, agent, mappedEvent, workingDirectory, sessionTitle, needsAttention, { type: mappedEvent, timestamp: Date.now(), toolName, summary, turnStart, turnFingerprint })
  chainHook(agent, rawEvent, rawInput)

  const port = readPort()
  if (!port) return process.exit(0)
  postJson(port, '/event', payload, 800, () => process.exit(0), () => process.exit(0))
}

function writeSession(sessionId, ag, ev, workDir, title, attention, lastEvent) {
  try {
    fs.mkdirSync(SESSION_DIR, { recursive: true })
    const safeId = sessionId.replace(/[^a-zA-Z0-9_-]/g, '_')
    const file = path.join(SESSION_DIR, `${ag}_${safeId}.json`)
    const existing = parseJson(tryRead(file))
    const STATUS_MAP = {
      session_start: 'idle', session_end: 'idle',
      thinking: 'working', tool_use: 'working', tool_result: 'working', permission_request: 'working',
      error: 'error', complete: 'completed',
    }
    const now = Date.now()
    const eventHistory = appendEventHistory(
      ev === 'session_start' ? [] : existing.events,
      { ...lastEvent, timestamp: lastEvent.timestamp || now },
      ag,
      ev,
    )
    const sessionTitle = title || existing.sessionTitle
    const lastMeaningfulSummary = bestMeaningfulSummary(eventHistory, sessionTitle)
      || (ev === 'session_start' ? undefined : existing.lastMeaningfulSummary)
    const session = {
      sessionId,
      agent: ag,
      status: STATUS_MAP[ev] || 'working',
      startedAt: existing.startedAt || now,
      lastUpdated: now,
      workingDirectory: workDir || existing.workingDirectory,
      sessionTitle,
      needsAttention: attention,
      lastEvent: eventHistory[eventHistory.length - 1] || lastEvent,
      events: eventHistory,
      lastMeaningfulSummary,
      aiTalkContext: ev === 'session_start' ? undefined : existing.aiTalkContext,
    }
    if (ev === 'session_end') session.endedAt = now
    const tmp = `${file}.tmp`
    fs.writeFileSync(tmp, JSON.stringify(session, null, 2))
    fs.renameSync(tmp, file)
  } catch {}
}

function appendEventHistory(existingEvents, event, ag, ev) {
  const summary = typeof event.summary === 'string' ? truncate(event.summary.trim()) : undefined
  const toolName = typeof event.toolName === 'string' ? event.toolName.trim() : undefined
  const next = Array.isArray(existingEvents) ? existingEvents.slice(-19) : []
  next.push({
    type: ev,
    eventType: ev,
    timestamp: event.timestamp || Date.now(),
    timestampMs: event.timestamp || Date.now(),
    toolName,
    summary,
    turnStart: Boolean(event.turnStart),
    turnFingerprint: event.turnFingerprint,
    informative: isMeaningfulSummary(summary, toolName, ag, ev),
  })
  return next
}

function bestMeaningfulSummary(events, sessionTitle) {
  const candidates = events
    .filter(event => event.informative && event.summary)
    .map(event => ({ event, priority: summaryPriority(event) }))
    .sort((a, b) => b.priority - a.priority || (b.event.timestampMs || 0) - (a.event.timestampMs || 0))

  const best = candidates[0]
  if (best?.priority >= 4) {
    return best.event.summary
  }

  if (sessionTitle) {
    return sessionTitle
  }

  return best?.event.summary
}

function summaryPriority(event) {
  if (event.type === 'complete' || event.eventType === 'complete') return 5
  if (event.type === 'error' || event.eventType === 'error') return 5
  if (event.type === 'thinking' || event.eventType === 'thinking') return 4
  if (event.type === 'permission_request' || event.eventType === 'permission_request') return 3
  if (event.type === 'tool_result' || event.eventType === 'tool_result') return 2
  if (event.type === 'tool_use' || event.eventType === 'tool_use') return 1
  return 0
}

function isMeaningfulSummary(summary, toolName, ag, ev) {
  if (!summary || !summary.trim()) return false
  const normalized = normalizeSummary(summary)
  if (!normalized) return false
  if (normalized === normalizeSummary(toolName || '')) return false
  if (normalized === normalizeSummary(ag) || normalized === normalizeSummary(ev)) return false
  if (['idle', 'working', 'complete', 'completed', 'error', 'thinking', 'tooluse', 'toolresult'].includes(normalized)) return false

  const lower = summary.trim().toLowerCase()
  if (lower.startsWith('waiting ') || lower.startsWith('waiting for ')) return false
  if (summary.trim().startsWith('等') && summary.includes('操作')) return false
  if (lower.startsWith('running ') || lower.startsWith('finished ')) return false
  if (lower.endsWith(' session started') || lower.endsWith(' session closed') || lower.endsWith(' session archived') || lower.endsWith(' finished this turn')) return false
  return true
}

function normalizeSummary(value) {
  return String(value || '').trim().toLowerCase().replace(/[^\p{Letter}\p{Number}]/gu, '')
}

function chainHook(ag, ev, input) {
  try {
    const hooks = parseJson(tryRead(HOOKS_FILE))
    const agentHooks = hooks[ag]
    if (!agentHooks) return
    // Claude/Copilot already support multiple hook entries natively.
    // Replaying their saved hooks here causes duplicate execution.
    if (ag !== 'codex') return

    const cmd = agentHooks[ev]
    if (!Array.isArray(cmd) || !cmd.length) return
    spawn(cmd[0], cmd.slice(1).concat([input]), { stdio: 'ignore', detached: true }).unref()
  } catch {}
}

function resolveMappedEvent(ctx) {
  if (agent === 'codex' && rawEvent === 'notify') {
    return normalizeEvent(ctx.type || rawEvent)
  }
  return normalizeEvent(rawEvent)
}

function isTurnStartEvent(ag, ev, mappedEvent) {
  if (mappedEvent !== 'thinking') return false
  if (ag === 'claude-code') return ev === 'UserPromptSubmit' || ev === 'thinking'
  if (ag === 'copilot') return ev === 'userPromptSubmitted' || ev === 'thinking'
  return ev === 'thinking'
}

function resolveSessionTitle(ctx, mappedEvent) {
  const limit = 180
  const msgs = ctx['input-messages']
  if (Array.isArray(msgs)) {
    const first = msgs.find(m => m.role === 'user')
    const content = first?.content
    if (typeof content === 'string') return truncate(content, limit)
    if (Array.isArray(content)) {
      const text = content.find(c => c.type === 'text')?.text
      if (text) return truncate(text, limit)
    }
  }
  if (mappedEvent === 'thinking') {
    const prompt = pickText(ctx.prompt, ctx.message, ctx.userPrompt, ctx.user_prompt)
    if (prompt) return truncate(firstNonEmptyLine(prompt), limit)
  }
  return ctx.sessionTitle ? truncate(ctx.sessionTitle, limit) : undefined
}

function resolveSummary(ctx, ag, mappedEvent) {
  const limit = TRUNCATE_LIMITS[mappedEvent] || TRUNCATE_DEFAULT
  const agentSummary = ag === 'claude-code' ? resolveClaudeSummary(ctx, mappedEvent) : undefined
  if (agentSummary) return truncate(agentSummary, limit)

  const explicit = pickText(
    ctx.summary,
    ctx.description,
    ctx['last-assistant-message'],
    ctx.prompt,
    ctx.message,
  )
  if (explicit) return truncate(explicit, limit)
  const input = ctx.tool_input || ctx.toolInput || ctx.input
  if (typeof input === 'string') return truncate(input, limit)
  if (input && typeof input === 'object') {
    const preferred = input.command || input.file_path || input.path || input.prompt || input.query
    if (typeof preferred === 'string') return truncate(preferred, limit)
    return truncate(JSON.stringify(input), limit)
  }
  return `等待 ${ag} 操作`
}

function resolveClaudeSummary(ctx, mappedEvent) {
  if (mappedEvent === 'thinking') {
    return pickText(ctx.prompt, ctx.message, ctx.userPrompt, ctx.user_prompt)
  }

  if (mappedEvent === 'complete') {
    return pickText(
      ctx.summary,
      ctx.description,
      ctx.last_assistant_message,
      ctx['last-assistant-message'],
      ctx.message,
      ctx.result,
    )
  }

  if (mappedEvent === 'error') {
    return pickText(ctx.error, ctx.message, ctx.summary, ctx.description)
  }

  if (mappedEvent === 'permission_request') {
    return pickText(ctx.message, ctx.prompt, ctx.reason)
  }

  return undefined
}

function pickText(...values) {
  for (const value of values) {
    if (typeof value !== 'string') continue
    const text = value.trim()
    if (text) return text
  }
  return undefined
}

function firstNonEmptyLine(value) {
  return value.split(/\r?\n/).map(line => line.trim()).find(Boolean) || value.trim()
}

const TRUNCATE_LIMITS = { complete: 512, error: 512, thinking: 180 }
const TRUNCATE_DEFAULT = 120
function truncate(v, limit) {
  const max = limit || TRUNCATE_DEFAULT
  return v.length > max ? `${v.slice(0, max)}...` : v
}
function parseJson(s) { try { return JSON.parse(s) } catch { return {} } }
function tryRead(f) { try { return fs.readFileSync(f, 'utf8') } catch { return '{}' } }

function readPort() {
  for (const f of PORT_FILES) {
    try {
      const p = Number(fs.readFileSync(f, 'utf8').trim())
      if (Number.isInteger(p) && p > 0) return p
    } catch {}
  }
  return null
}

function postJson(port, route, payload, timeout, onSuccess, onFailure) {
  const body = JSON.stringify(payload)
  const req = http.request({
    host: '127.0.0.1', port, path: route, method: 'POST', timeout,
    headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(body) },
  }, (res) => { res.resume(); res.statusCode >= 200 && res.statusCode < 300 ? onSuccess() : onFailure() })
  req.on('error', onFailure)
  req.on('timeout', () => { req.destroy(); onFailure() })
  req.end(body)
}
