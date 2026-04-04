#!/usr/bin/env node

const fs = require('node:fs')
const http = require('node:http')
const os = require('node:os')
const path = require('node:path')

const eventName = process.argv[2]
if (!eventName) {
  process.exit(0)
}

const EVENT_MAP = {
  session_start: 'session_start',
  session_end: 'session_end',
  thinking: 'thinking',
  tool_use: 'tool_use',
  tool_result: 'tool_result',
  error: 'error',
  complete: 'complete',
}

const mappedEvent = EVENT_MAP[eventName]
if (!mappedEvent) {
  process.exit(0)
}

const PORT_FILES = [
  path.join(os.homedir(), '.copiwaifu', 'port'),
  '/tmp/copiwaifu-port',
]

const chunks = []
process.stdin.on('data', chunk => chunks.push(chunk))
process.stdin.on('end', () => handle(Buffer.concat(chunks).toString('utf8')))
setTimeout(() => handle(''), 300)

let handled = false

function handle(rawInput) {
  if (handled)
    return
  handled = true

  const context = parseJson(rawInput)
  const port = readPort()
  if (!port) {
    process.exit(0)
  }

  const payload = {
    agent: 'claude-code',
    session_id: context.session_id || 'claude-default',
    event: mappedEvent,
    data: {
      tool_name: resolveToolName(context),
      summary: resolveSummary(context),
    },
  }

  postJson(port, '/event', payload, 800, () => process.exit(0), () => process.exit(0))
}

function postJson(port, route, payload, timeout, onSuccess, onFailure) {
  const body = JSON.stringify(payload)
  const req = http.request({
    host: '127.0.0.1',
    port,
    path: route,
    method: 'POST',
    timeout,
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(body),
    },
  }, (res) => {
    if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) {
      res.resume()
      onSuccess()
      return
    }

    res.resume()
    onFailure()
  })

  req.on('error', onFailure)
  req.on('timeout', () => {
    req.destroy()
    onFailure()
  })

  req.end(body)
}

function resolveToolName(context) {
  return context.tool_name || context.toolName || context.name || 'Claude Code'
}

function resolveSummary(context) {
  const explicit = context.summary || context.description
  if (explicit) {
    return truncate(explicit)
  }

  const input = context.tool_input || context.toolInput || context.input
  if (typeof input === 'string') {
    return truncate(input)
  }
  if (input && typeof input === 'object') {
    const preferred = input.command || input.file_path || input.path || input.prompt || input.query
    if (typeof preferred === 'string') {
      return truncate(preferred)
    }
    return truncate(JSON.stringify(input))
  }

  return '等待 Claude Code 操作'
}

function truncate(value) {
  return value.length > 180 ? `${value.slice(0, 180)}...` : value
}

function parseJson(input) {
  try {
    return JSON.parse(input)
  }
  catch {
    return {}
  }
}

function readPort() {
  for (const portFile of PORT_FILES) {
    try {
      const port = Number(fs.readFileSync(portFile, 'utf8').trim())
      if (Number.isInteger(port) && port > 0) {
        return port
      }
    }
    catch {}
  }

  return null
}
