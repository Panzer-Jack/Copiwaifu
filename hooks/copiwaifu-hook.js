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
  Stop: 'complete', Notification: 'complete', PermissionRequest: 'permission_request',
  Elicitation: 'tool_use', SubagentStart: 'tool_use', SubagentStop: 'tool_use',
  PreCompact: 'tool_use', PostCompact: 'tool_use', WorktreeCreate: 'tool_use',
}
const COPILOT_MAP = {
  sessionStart: 'session_start', sessionEnd: 'session_end',
  userPromptSubmitted: 'thinking', preToolUse: 'tool_use',
  postToolUse: 'tool_result', errorOccurred: 'error', agentStop: 'complete',
}
const CODEX_MAP = { 'agent-turn-complete': 'complete', notify: 'tool_use' }
const PASSTHROUGH = new Set(['session_start','session_end','thinking','tool_use','tool_result','error','complete','permission_request'])

function normalizeEvent(ev) {
  if (PASSTHROUGH.has(ev)) return ev
  if (agent === 'claude-code') return CLAUDE_MAP[ev] || null
  if (agent === 'copilot') return COPILOT_MAP[ev] || null
  if (agent === 'codex') return CODEX_MAP[ev] || null
  return null
}

const PORT_FILES = [path.join(os.homedir(), '.copiwaifu', 'port'), '/tmp/copiwaifu-port']
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
  if (!mappedEvent) return process.exit(0)
  const sessionId = ctx.session_id || ctx.sessionId || ctx['thread-id'] || `${agent}-${process.ppid}`
  const toolName = ctx.tool_name || ctx.toolName || ctx.name || agent
  const summary = resolveSummary(ctx, agent)
  const workingDirectory = ctx.cwd || ctx.workingDirectory || ctx.working_directory
  const sessionTitle = resolveSessionTitle(ctx)
  const needsAttention = mappedEvent === 'permission_request' || ['bash', 'execute_command'].includes(toolName.toLowerCase())

  const data = { tool_name: toolName, summary, working_directory: workingDirectory, session_title: sessionTitle, needs_attention: needsAttention }
  const payload = { agent, session_id: sessionId, event: mappedEvent, data }

  writeSession(sessionId, agent, mappedEvent, workingDirectory, sessionTitle, needsAttention, { type: mappedEvent, timestamp: Date.now(), toolName, summary })
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
    const session = {
      sessionId,
      agent: ag,
      status: STATUS_MAP[ev] || 'working',
      startedAt: existing.startedAt || now,
      lastUpdated: now,
      workingDirectory: workDir || existing.workingDirectory,
      sessionTitle: title || existing.sessionTitle,
      needsAttention: attention,
      lastEvent,
    }
    if (ev === 'session_end') session.endedAt = now
    const tmp = `${file}.tmp`
    fs.writeFileSync(tmp, JSON.stringify(session, null, 2))
    fs.renameSync(tmp, file)
  } catch {}
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

function resolveSessionTitle(ctx) {
  const msgs = ctx['input-messages']
  if (Array.isArray(msgs)) {
    const first = msgs.find(m => m.role === 'user')
    const content = first?.content
    if (typeof content === 'string') return truncate(content)
    if (Array.isArray(content)) {
      const text = content.find(c => c.type === 'text')?.text
      if (text) return truncate(text)
    }
  }
  return ctx.sessionTitle ? truncate(ctx.sessionTitle) : undefined
}

function resolveSummary(ctx, ag) {
  const explicit = ctx.summary || ctx.description || ctx['last-assistant-message']
  if (explicit) return truncate(explicit)
  const input = ctx.tool_input || ctx.toolInput || ctx.input
  if (typeof input === 'string') return truncate(input)
  if (input && typeof input === 'object') {
    const preferred = input.command || input.file_path || input.path || input.prompt || input.query
    if (typeof preferred === 'string') return truncate(preferred)
    return truncate(JSON.stringify(input))
  }
  return `等待 ${ag} 操作`
}

function truncate(v) { return v.length > 180 ? `${v.slice(0, 180)}...` : v }
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
