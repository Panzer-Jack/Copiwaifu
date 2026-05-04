// copiwaifu-opencode-plugin
// version: v1
import fs from 'node:fs'
import http from 'node:http'
import os from 'node:os'
import path from 'node:path'

const PORT_FILES = [
  path.join(os.homedir(), '.copiwaifu', 'port'),
  '/tmp/copiwaifu-port',
]
const SESSION_DIR = path.join(os.homedir(), '.copiwaifu', 'sessions')

function readPort() {
  for (const file of PORT_FILES) {
    try {
      const port = Number(fs.readFileSync(file, 'utf8').trim())
      if (Number.isInteger(port) && port > 0) {
        return port
      }
    }
    catch {}
  }
  return null
}

function postJson(port, payload) {
  return new Promise((resolve) => {
    const body = JSON.stringify(payload)
    const req = http.request({
      host: '127.0.0.1',
      port,
      path: '/event',
      method: 'POST',
      timeout: 1000,
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(body),
      },
    }, (res) => {
      res.resume()
      resolve(res.statusCode >= 200 && res.statusCode < 300)
    })

    req.on('error', () => resolve(false))
    req.on('timeout', () => {
      req.destroy()
      resolve(false)
    })
    req.end(body)
  })
}

function truncate(value) {
  if (!value) {
    return undefined
  }
  const text = String(value).trim()
  if (!text) {
    return undefined
  }
  return text.length > 180 ? `${text.slice(0, 180)}...` : text
}

function tryReadJson(file) {
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'))
  }
  catch {
    return {}
  }
}

function writeSession(sessionId, event, data) {
  try {
    fs.mkdirSync(SESSION_DIR, { recursive: true })
    const safeId = sessionId.replace(/[^a-zA-Z0-9_-]/g, '_')
    const file = path.join(SESSION_DIR, `opencode_${safeId}.json`)
    const existing = tryReadJson(file)
    const now = Date.now()
    const statusMap = {
      session_start: 'idle',
      session_end: 'idle',
      thinking: 'working',
      tool_use: 'working',
      tool_result: 'working',
      permission_request: 'working',
      error: 'error',
      complete: 'completed',
    }
    const events = appendEventHistory(
      event === 'session_start' ? [] : existing.events,
      {
        type: event,
        timestamp: now,
        toolName: data.tool_name,
        summary: data.summary,
        turnStart: data.turn_start,
        turnFingerprint: data.turn_fingerprint,
      },
    )
    const sessionTitle = data.session_title || existing.sessionTitle
    const lastMeaningfulSummary = bestMeaningfulSummary(events, sessionTitle)
      || (event === 'session_start' ? undefined : existing.lastMeaningfulSummary)
    const session = {
      sessionId,
      agent: 'opencode',
      status: statusMap[event] || 'working',
      startedAt: existing.startedAt || now,
      lastUpdated: now,
      workingDirectory: data.working_directory || existing.workingDirectory,
      sessionTitle,
      needsAttention: data.needs_attention ?? existing.needsAttention ?? false,
      lastEvent: events[events.length - 1],
      events,
      lastMeaningfulSummary,
      aiTalkContext: event === 'session_start' ? undefined : existing.aiTalkContext,
    }
    if (event === 'session_end') {
      session.endedAt = now
    }
    const tmp = `${file}.tmp`
    fs.writeFileSync(tmp, JSON.stringify(session, null, 2))
    fs.renameSync(tmp, file)
  }
  catch {}
}

function appendEventHistory(existingEvents, event) {
  const summary = truncate(event.summary)
  const toolName = truncate(event.toolName)
  const next = Array.isArray(existingEvents) ? existingEvents.slice(-19) : []
  next.push({
    type: event.type,
    eventType: event.type,
    timestamp: event.timestamp || Date.now(),
    timestampMs: event.timestamp || Date.now(),
    toolName,
    summary,
    turnStart: Boolean(event.turnStart),
    turnFingerprint: event.turnFingerprint,
    informative: isMeaningfulSummary(summary, toolName, event.type),
  })
  return next
}

function bestMeaningfulSummary(events, sessionTitle) {
  const candidates = events
    .filter(item => item.informative && item.summary)
    .map(item => ({ item, priority: summaryPriority(item) }))
    .sort((a, b) => b.priority - a.priority || (b.item.timestampMs || 0) - (a.item.timestampMs || 0))
  const best = candidates[0]
  if (best?.priority >= 4) {
    return best.item.summary
  }
  if (sessionTitle) {
    return sessionTitle
  }
  return best?.item.summary
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

function isMeaningfulSummary(summary, toolName, event) {
  if (!summary || !summary.trim()) {
    return false
  }

  const normalized = normalizeSummary(summary)
  if (!normalized) {
    return false
  }
  if (normalized === normalizeSummary(toolName || '')) {
    return false
  }
  if (normalized === 'opencode' || normalized === normalizeSummary(event)) {
    return false
  }
  if (['idle', 'working', 'complete', 'completed', 'error', 'thinking', 'tooluse', 'toolresult'].includes(normalized)) {
    return false
  }

  const lower = summary.trim().toLowerCase()
  if (lower.startsWith('waiting ') || lower.startsWith('waiting for ')) {
    return false
  }
  if (summary.trim().startsWith('等') && summary.includes('操作')) {
    return false
  }
  if (lower.startsWith('running ') || lower.startsWith('finished ')) {
    return false
  }
  if (lower.endsWith(' session started') || lower.endsWith(' session closed') || lower.endsWith(' session archived') || lower.endsWith(' finished this turn')) {
    return false
  }

  return true
}

function normalizeSummary(value) {
  return String(value || '').trim().toLowerCase().replace(/[^\p{Letter}\p{Number}]/gu, '')
}

function buildPayload(agent, sessionId, event, data) {
  return {
    agent,
    session_id: sessionId,
    event,
    data,
  }
}

function mapToolName(tool) {
  if (!tool) {
    return 'OpenCode'
  }
  return `${tool}`.charAt(0).toUpperCase() + `${tool}`.slice(1)
}

export default {
  id: 'copiwaifu',
  server: async ({ serverUrl }) => {
    const sessionCwd = new Map()
    const sessionTitle = new Map()
    const messageRoles = new Map()
    const latestAssistantText = new Map()
    const localServerPort = serverUrl ? Number(serverUrl.port || 0) || null : null

    async function emit(event, sessionId, data = {}) {
      const payload = buildPayload('opencode', sessionId, event, data)
      writeSession(sessionId, event, data)

      const port = localServerPort || readPort()
      if (!port) {
        return
      }
      await postJson(port, payload)
    }

    function getSessionId(raw) {
      return raw ? `opencode-${raw}` : null
    }

    return {
      event: async ({ event }) => {
        const type = event?.type
        const properties = event?.properties || {}

        if (type === 'session.created' && properties.info?.id) {
          const sessionId = getSessionId(properties.info.id)
          const cwd = properties.info.directory || undefined
          sessionCwd.set(properties.info.id, cwd)
          await emit('session_start', sessionId, {
            working_directory: cwd,
            session_title: truncate(properties.info.title),
            summary: truncate(properties.info.title) || 'OpenCode session started',
            tool_name: 'OpenCode',
            needs_attention: false,
          })
          return
        }

        if (type === 'session.updated' && properties.info?.id) {
          if (properties.info.directory) {
            sessionCwd.set(properties.info.id, properties.info.directory)
          }
          if (properties.info.title && !properties.info.title.startsWith('New session')) {
            sessionTitle.set(properties.info.id, properties.info.title)
          }
          if (properties.info.time?.archived) {
            const sessionId = getSessionId(properties.info.id)
            await emit('session_end', sessionId, {
              working_directory: sessionCwd.get(properties.info.id),
              session_title: truncate(sessionTitle.get(properties.info.id) || properties.info.title),
              summary: 'OpenCode session archived',
              tool_name: 'OpenCode',
              needs_attention: false,
            })
          }
          return
        }

        if (type === 'session.deleted' && properties.info?.id) {
          const sessionId = getSessionId(properties.info.id)
          await emit('session_end', sessionId, {
            working_directory: sessionCwd.get(properties.info.id),
            session_title: truncate(sessionTitle.get(properties.info.id)),
            summary: 'OpenCode session closed',
            tool_name: 'OpenCode',
            needs_attention: false,
          })
          return
        }

        if (type === 'session.status' && properties.sessionID && properties.status?.type === 'idle') {
          const sessionId = getSessionId(properties.sessionID)
          await emit('complete', sessionId, {
            working_directory: sessionCwd.get(properties.sessionID),
            session_title: truncate(sessionTitle.get(properties.sessionID)),
            summary: truncate(latestAssistantText.get(properties.sessionID)) || 'OpenCode finished this turn',
            tool_name: 'OpenCode',
            needs_attention: false,
          })
          return
        }

        if (type === 'message.updated' && properties.info?.id && properties.info?.sessionID) {
          messageRoles.set(properties.info.id, {
            role: properties.info.role,
            sessionID: properties.info.sessionID,
          })
          if (messageRoles.size > 300) {
            messageRoles.delete(messageRoles.keys().next().value)
          }
          return
        }

        if (type === 'message.part.updated' && properties.part?.messageID && properties.part?.type === 'text') {
          const meta = messageRoles.get(properties.part.messageID)
          if (!meta) {
            return
          }
          const sessionId = getSessionId(meta.sessionID)
          const text = truncate(properties.part.text)
          if (meta.role === 'user' && text) {
            await emit('thinking', sessionId, {
              working_directory: sessionCwd.get(meta.sessionID),
              session_title: truncate(sessionTitle.get(meta.sessionID)),
              summary: text,
              tool_name: 'OpenCode',
              needs_attention: false,
              turn_start: true,
              turn_fingerprint: text,
            })
            return
          }
          if (meta.role === 'assistant' && text) {
            latestAssistantText.set(meta.sessionID, text)
          }
          return
        }

        if (type === 'message.part.updated' && properties.part?.sessionID && properties.part?.type === 'tool') {
          const sessionId = getSessionId(properties.part.sessionID)
          const toolName = mapToolName(properties.part.tool)
          const status = properties.part.state?.status
          if (status === 'running' || status === 'pending') {
            await emit('tool_use', sessionId, {
              working_directory: sessionCwd.get(properties.part.sessionID),
              session_title: truncate(sessionTitle.get(properties.part.sessionID)),
              summary: truncate(
                properties.part.state?.input?.command
                || properties.part.state?.input?.file_path
                || properties.part.state?.input?.path
                || properties.part.state?.input?.prompt,
              ) || `Running ${toolName}`,
              tool_name: toolName,
              needs_attention: false,
            })
            return
          }
          if (status === 'completed') {
            await emit('tool_result', sessionId, {
              working_directory: sessionCwd.get(properties.part.sessionID),
              session_title: truncate(sessionTitle.get(properties.part.sessionID)),
              summary: `Finished ${toolName}`,
              tool_name: toolName,
              needs_attention: false,
            })
            return
          }
          if (status === 'error') {
            await emit('error', sessionId, {
              working_directory: sessionCwd.get(properties.part.sessionID),
              session_title: truncate(sessionTitle.get(properties.part.sessionID)),
              summary: `Failed ${toolName}`,
              tool_name: toolName,
              needs_attention: false,
            })
          }
          return
        }

        if (type === 'permission.asked' && properties.sessionID) {
          const sessionId = getSessionId(properties.sessionID)
          const toolName = mapToolName(properties.permission)
          await emit('permission_request', sessionId, {
            working_directory: sessionCwd.get(properties.sessionID),
            session_title: truncate(sessionTitle.get(properties.sessionID)),
            summary: truncate(properties.patterns?.join(' && ')) || `OpenCode requests ${toolName}`,
            tool_name: toolName,
            needs_attention: true,
          })
          return
        }

        if (type === 'question.asked' && properties.sessionID) {
          const sessionId = getSessionId(properties.sessionID)
          const firstQuestion = properties.questions?.find?.(question => question?.question)?.question
          await emit('permission_request', sessionId, {
            working_directory: sessionCwd.get(properties.sessionID),
            session_title: truncate(sessionTitle.get(properties.sessionID)),
            summary: truncate(firstQuestion) || 'OpenCode needs your input',
            tool_name: 'AskUserQuestion',
            needs_attention: true,
          })
        }
      },
    }
  },
}
