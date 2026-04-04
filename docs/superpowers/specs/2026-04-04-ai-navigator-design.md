# AI Navigator 设计方案

## 概述

Copiwaifu 桌宠作为 AI Navigator，与 Claude Code、Codex CLI、Copilot CLI 协同工作。采用交互控制型方案：监听 AI 工具事件，Live2D 角色做出状态反应，并支持权限审批交互。

第一期仅支持 macOS。Windows/Linux 支持留待后续迭代。

## 架构：事件驱动单体（方案 A）

```
Hook 脚本 → HTTP POST → Rust 状态机 → Tauri Event → Vue 响应 → Live2D 动作 + 气泡
```

```
┌─────────────────────────────────────────────────┐
│                   Copiwaifu                      │
│                                                  │
│  ┌──────────── Rust (Tauri) ──────────────┐     │
│  │                                         │     │
│  │  HTTP Server (:23333)                   │     │
│  │    ├── POST /event    ← hook 脚本推送   │     │
│  │    ├── GET  /permission/{id}  ← hook轮询│     │
│  │    └── GET  /status                     │     │
│  │                                         │     │
│  │  Agent Manager                          │     │
│  │    ├── 会话注册/清理                     │     │
│  │    └── 进程存活检测                      │     │
│  │                                         │     │
│  │  State Machine                          │     │
│  │    ├── 状态优先级解析                    │     │
│  │    └── 最小显示时间控制                  │     │
│  │                                         │     │
│  │  Codex Log Monitor                      │     │
│  │    └── JSONL 增量轮询                   │     │
│  │                                         │     │
│  │  Hook Installer                         │     │
│  │    └── 自动注册到各工具配置              │     │
│  │                                         │     │
│  └──── Tauri Event ──────────────────┬─────┘     │
│                                      ▼           │
│  ┌──────────── Vue 3 ────────────────────┐      │
│  │                                        │      │
│  │  App.vue                               │      │
│  │    ├── Live2D (Pixi.js + easy-live2d)  │      │
│  │    ├── SpeechBubble (状态文字)          │      │
│  │    └── PermissionBubble (审批交互)      │      │
│  │                                        │      │
│  │  Composables                           │      │
│  │    ├── useAgentState    (监听状态事件)  │      │
│  │    ├── usePermission    (审批交互逻辑)  │      │
│  │    └── useSpeechBubble  (已有)         │      │
│  │                                        │      │
│  └────────────────────────────────────────┘      │
└─────────────────────────────────────────────────┘
```

## Rust 侧模块

Rust 新增模块统一放入 `src-tauri/src/navigator/` 子模块，避免 `src/` 根目录文件超过 8 个。

HTTP crate 选型：使用 `tiny_http`（轻量同步 HTTP server），配合 `std::thread` 在独立线程运行，不引入额外异步运行时，与 Tauri 事件循环无冲突。

### HTTP Server（`navigator/server.rs`）

- 绑定端口 23333（被占用则递增，最多尝试 10 个端口），端口写入 `~/.copiwaifu/port`
- 如果所有端口都失败，日志警告但应用正常启动（降级为普通桌宠模式，无 AI 集成）
- 路由：
  - `POST /event` — 接收 hook 事件
  - `GET /permission/{permission_id}` — hook 脚本轮询审批结果，返回 `{ "status": "pending" | "approved" | "denied" }`
  - `GET /status` — 健康检查

审批结果的两条路径：
- 前端 → Rust：通过 Tauri Command `respond_permission(id, approved)` 进程内 IPC
- Hook 脚本 → Rust：通过 `GET /permission/{id}` HTTP 轮询获取结果

### State Machine（`navigator/state.rs`）

5 种核心状态及优先级（高→低）：

| 优先级 | 状态 | 最小显示时间 |
|--------|------|-------------|
| 1 | `WaitingPermission` | 无（由审批结果决定） |
| 2 | `Error` | 2s |
| 3 | `ToolUse` | 1s |
| 4 | `Thinking` | 1.5s |
| 5 | `Idle` | 无 |

多 Agent 并发规则：
- 每个 Agent 独立维护自己的状态
- 展示时取所有 Agent 中最高优先级的状态
- 权限请求排队：同时只展示一个 PermissionBubble，后续请求进入 FIFO 队列，前一个处理完自动弹出下一个
- 状态降级：高优先级状态结束后，自动回退到当前所有 Agent 中次高优先级状态

### Agent Manager（`navigator/agent.rs`）

- 按 `(agent_type, session_id)` 跟踪会话
- 会话超时清理（60s 无事件则标记为 stale 并移除）
- 应用启动时扫描进程表，检测已运行的 Agent

session_id 生成规则：
- Claude Code：hook context 中包含 `session_id` 字段，直接使用
- Copilot CLI：hook context 中如有 session 标识则使用，否则用 `{agent}-{ppid}` 作为 session_id（父进程 PID 在同一会话内稳定）
- Codex：用日志文件名作为 session 标识

### Codex Log Monitor（`navigator/codex_monitor.rs`）

- 监控 `~/.codex/` 目录下的日志文件（具体路径需在实现时确认 Codex CLI 实际输出位置）
- 记录文件偏移量，增量读取 JSONL
- 文件 rotate/删除处理：检测到文件 inode 变化或大小缩小时，重置偏移量从头读取
- 轮询策略：活跃时 500ms，空闲超过 30s 后降频到 2s，检测到新事件后恢复 500ms
- JSONL 字段映射在实现时根据 Codex 实际日志格式定义

### Hook Installer（`navigator/hook_installer.rs`）

启动时自动检测并注册：

**Claude Code：**
- 读取 `~/.claude/settings.json`
- 合并写入 hooks 字段（保留用户已有的 hook 配置，追加 Copiwaifu 的 hook）
- 每个 hook entry 添加 `"source": "copiwaifu"` 标记，便于识别和清理

**Copilot CLI：**
- 读取 `~/.config/github-copilot/config.json`（macOS 路径）
- 合并策略同上

**卸载/清理：**
- 提供 Tauri Command `uninstall_hooks()` 供设置界面调用
- 清理逻辑：移除所有带 `"source": "copiwaifu"` 标记的 hook entries
- 删除 `~/.copiwaifu/` 目录

Hook 脚本打包在应用资源中，安装时复制到 `~/.copiwaifu/hooks/`。

### 统一事件类型（`navigator/events.rs`）

```rust
pub enum AgentType {
    ClaudeCode,
    Copilot,
    Codex,
}

pub enum EventType {
    SessionStart,
    SessionEnd,
    Thinking,
    ToolUse,
    ToolResult,
    Error,
    PermissionRequest,
    Complete,
}

pub struct AgentEvent {
    pub agent: AgentType,
    pub session_id: String,
    pub event: EventType,
    pub data: Option<EventData>,
}

pub struct EventData {
    pub tool_name: Option<String>,
    pub summary: Option<String>,
    pub permission_id: Option<String>,
}
```

### Tauri Commands（`navigator/commands.rs`）

- `respond_permission(permission_id: String, approved: bool)` — 前端调用，将审批结果写入状态，同时更新 HTTP 轮询接口的返回值
- `get_agent_status()` — 查询当前 Agent 状态（备用）
- `uninstall_hooks()` — 清理所有已安装的 hooks

## 前端模块

### useAgentState（`composables/useAgentState.ts`）

- 监听 Tauri Event `agent:state-change`
- 暴露：`currentState`、`activeAgent`、`sessionInfo`
- 状态变化时触发 Live2D 动作映射（具体 motion/expression 名称需在实现时查阅 Hiyori 模型的 `.model3.json`）：
  - `Idle` → 默认待机 motion group
  - `Thinking` → 思考类 motion（如有）或降低眨眼频率
  - `ToolUse` → 活跃类 motion
  - `Error` → 惊讶类 expression
  - `WaitingPermission` → 注视用户方向

注：Hiyori 模型的可用 motion groups 和 expressions 需在实现阶段从 `public/Resources/Hiyori/Hiyori.model3.json` 中提取，建立具体映射表。

### usePermission（`composables/usePermission.ts`）

- 监听 Tauri Event `permission:request`
- 暴露：`pendingRequest`、`approve()`、`deny()`
- `approve/deny` 调用 Tauri Command `respond_permission` 回传结果
- 前端超时 30s 自动拒绝

### PermissionBubble（`components/PermissionBubble.vue`）

- 在 Live2D 角色旁弹出，与 SpeechBubble 统一风格
- 显示：工具名称 + 操作摘要（如 "执行命令: `git status`"）
- 两个按钮：✓ 允许 / ✗ 拒绝
- 入场/退场动画复用 pop-in/pop-out
- `pointer-events: auto`（需要接收点击）

### App.vue 改动

- 引入 `useAgentState` 和 `usePermission`
- 根据 `currentState` 切换 Live2D 动作/表情
- 状态变化时通过 `useSpeechBubble.say()` 显示状态文字
- 权限请求时显示 PermissionBubble，隐藏 SpeechBubble

### 状态气泡文字映射

| 状态 | 气泡文字 |
|------|----------|
| `Thinking` | "思考中..." |
| `ToolUse` | "正在执行: {tool_name}" |
| `Error` | "出错了..." |
| `WaitingPermission` | （PermissionBubble 接管） |
| `Idle` | 随机问候语（保留现有逻辑） |

### 类型定义（`types/agent.ts`）

```typescript
type AgentType = 'claude-code' | 'copilot' | 'codex'
type AgentState = 'idle' | 'thinking' | 'tool_use' | 'error' | 'waiting_permission'

interface StateChangeEvent {
  state: AgentState
  agent: AgentType
  session_id: string
  tool_name?: string
  summary?: string
}

interface PermissionRequest {
  permission_id: string
  agent: AgentType
  tool_name: string
  summary: string
}
```

## Hook 脚本

### 统一事件协议

```json
{
  "agent": "claude-code | copilot | codex",
  "session_id": "string",
  "event": "thinking | tool_use | tool_result | error | permission_request | complete | session_start | session_end",
  "data": {
    "tool_name": "bash",
    "summary": "git status",
    "permission_id": "perm-xxx"
  }
}
```

### Claude Code Hook（`hooks/claude-hook.js`）

注册到 `~/.claude/settings.json`：
```json
{
  "hooks": {
    "PreToolUse": [{ "command": "node ~/.copiwaifu/hooks/claude-hook.js pre_tool_use", "source": "copiwaifu" }],
    "PostToolUse": [{ "command": "node ~/.copiwaifu/hooks/claude-hook.js post_tool_use", "source": "copiwaifu" }],
    "Notification": [{ "command": "node ~/.copiwaifu/hooks/claude-hook.js notification", "source": "copiwaifu" }]
  }
}
```

- 零依赖，只用 Node.js 内置 `http` 模块
- 从 `~/.copiwaifu/port` 读取端口
- 通过 stdin 读取 Claude Code 传入的 hook context JSON
- session_id：从 hook context 中提取 `session_id` 字段
- 权限审批流程：
  1. `PreToolUse` hook 发送 `permission_request` 事件到 `POST /event`
  2. 轮询 `GET /permission/{permission_id}`（间隔 500ms）
  3. 收到 `approved` → exit code 0（允许）
  4. 收到 `denied` → exit code 2（拒绝，输出 JSON `{"decision":"block","reason":"用户拒绝"}`)
  5. hook 脚本自身超时 35s（大于前端 30s），超时后默认拒绝
- Copiwaifu 未启动时（端口文件不存在或连接失败）：`PreToolUse` 默认允许（exit code 0），非审批事件静默退出

### Copilot CLI Hook（`hooks/copilot-hook.js`）

结构与 Claude Code hook 类似，适配 Copilot CLI 的 hook 格式。session_id 使用 `{agent}-{ppid}`。

### Codex 集成

无 hook 脚本，Rust 侧 `codex_monitor.rs` 直接轮询日志。

### 端口发现机制

- Rust 启动 HTTP Server 后将端口写入 `~/.copiwaifu/port`
- hook 脚本启动时读取该文件获取端口
- 端口文件写入失败时（权限问题），尝试写入 `/tmp/copiwaifu-port` 作为 fallback

## 错误处理与降级策略

| 故障场景 | 行为 |
|----------|------|
| HTTP Server 启动失败（端口耗尽） | 应用正常运行，降级为普通桌宠，日志警告 |
| 端口文件写入失败 | fallback 到 `/tmp/copiwaifu-port` |
| hook 脚本执行时 Copiwaifu 未启动 | 非审批事件静默退出；`PreToolUse` 默认允许（不阻塞 AI 工具） |
| 审批超时（前端 30s） | 自动拒绝，Rust 侧清理 permission 状态 |
| hook 脚本超时（35s） | 默认拒绝 |
| Codex 日志路径不存在 | 跳过 Codex 监控，日志提示 |
| Codex 日志文件 rotate | 检测 inode 变化，重置偏移量 |
| Permission 状态 TTL | 60s 未被轮询的 permission 记录自动清理 |

## 文件结构

```
src/
├── App.vue                          # 改动：集成 agent state + permission
├── main.ts
├── components/
│   ├── SpeechBubble.vue             # 已有
│   └── PermissionBubble.vue         # 新增
├── composables/
│   ├── useSpeechBubble.ts           # 已有
│   ├── useAgentState.ts             # 新增
│   └── usePermission.ts             # 新增
└── types/
    └── agent.ts                     # 新增

src-tauri/src/
├── main.rs                          # 不变
├── lib.rs                           # 改动：注册 navigator 模块
└── navigator/                       # 新增子模块
    ├── mod.rs                       # 模块入口
    ├── server.rs                    # HTTP Server
    ├── state.rs                     # 状态机
    ├── agent.rs                     # Agent 会话管理
    ├── codex_monitor.rs             # Codex 日志轮询
    ├── hook_installer.rs            # Hook 自动安装
    ├── events.rs                    # 统一事件类型
    └── commands.rs                  # Tauri Commands

hooks/
├── claude-hook.js                   # Claude Code hook
└── copilot-hook.js                  # Copilot CLI hook
```

## 设计决策记录

| 决策 | 选择 | 理由 |
|------|------|------|
| 功能层级 | 交互控制型 | 需要权限审批能力 |
| 第一期工具 | Claude Code + Codex + Copilot | 覆盖三种不同集成模式 |
| 通信方案 | Rust 原生 HTTP Server (tiny_http) | 轻量，无需额外异步运行时，与 Tauri 无冲突 |
| 审批交互 | 气泡弹窗式 | 与现有 SpeechBubble 风格统一 |
| 状态映射 | 基础 5 状态 | 保持简洁，够用即可 |
| Hook 安装 | 自动安装（合并策略） | 用户无感，不覆盖已有配置 |
| 架构模式 | 事件驱动单体 | 实时性好，状态集中管理 |
| Rust 模块组织 | `navigator/` 子模块 | 避免 src/ 根目录文件超 8 个 |
| 平台支持 | 第一期仅 macOS | 聚焦核心功能，后续扩展 |
| Copiwaifu 未启动时审批 | 默认允许 | 不阻塞 AI 工具正常工作 |
| 多 Agent 权限并发 | FIFO 队列 | 逐个审批，避免遗漏 |
