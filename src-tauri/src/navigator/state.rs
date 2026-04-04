use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use super::{
    agent::{session_key, PERMISSION_TTL, SESSION_TTL},
    events::{
        AgentEvent, AgentState, EventType, NavigatorEmission, NavigatorStatus,
        PermissionRequestPayload, PermissionResolvedPayload, PermissionStatus, StateChangePayload,
    },
};

static PERMISSION_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
struct SessionRecord {
    agent: super::events::AgentType,
    session_id: String,
    state: AgentState,
    tool_name: Option<String>,
    summary: Option<String>,
    updated_at: Instant,
}

#[derive(Clone, Debug)]
struct PermissionRecord {
    id: String,
    agent: super::events::AgentType,
    session_id: String,
    tool_name: String,
    summary: String,
    status: PermissionStatus,
    last_touched_at: Instant,
}

pub struct NavigatorState {
    sessions: HashMap<String, SessionRecord>,
    permission_queue: VecDeque<String>,
    permissions: HashMap<String, PermissionRecord>,
    server_port: Option<u16>,
    last_snapshot: Option<StateChangePayload>,
    last_snapshot_at: Option<Instant>,
}

impl NavigatorState {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            permission_queue: VecDeque::new(),
            permissions: HashMap::new(),
            server_port: None,
            last_snapshot: None,
            last_snapshot_at: None,
        }
    }

    pub fn set_server_port(&mut self, port: u16) {
        self.server_port = Some(port);
    }

    pub fn snapshot(&self) -> NavigatorStatus {
        NavigatorStatus {
            current: self
                .last_snapshot
                .clone()
                .unwrap_or_else(|| self.compute_raw_snapshot()),
            active_permission: self.current_permission_payload(),
            server_port: self.server_port,
        }
    }

    pub fn apply_event(&mut self, event: AgentEvent) -> Vec<NavigatorEmission> {
        let previous_active = self.active_permission_id();
        let now = Instant::now();

        match event.event {
            EventType::SessionStart => self.upsert_session(
                event.agent,
                event.session_id,
                AgentState::Idle,
                event.data.tool_name,
                event.data.summary,
                now,
            ),
            EventType::SessionEnd => {
                self.sessions
                    .remove(&session_key(&event.agent, &event.session_id));
            }
            EventType::Thinking => self.upsert_session(
                event.agent,
                event.session_id,
                AgentState::Thinking,
                event.data.tool_name,
                event.data.summary,
                now,
            ),
            EventType::ToolUse | EventType::ToolResult => self.upsert_session(
                event.agent,
                event.session_id,
                AgentState::ToolUse,
                event.data.tool_name,
                event.data.summary,
                now,
            ),
            EventType::Error => self.upsert_session(
                event.agent,
                event.session_id,
                AgentState::Error,
                event.data.tool_name,
                event.data.summary,
                now,
            ),
            EventType::Complete => self.upsert_session(
                event.agent,
                event.session_id,
                AgentState::Idle,
                event.data.tool_name,
                event.data.summary,
                now,
            ),
            EventType::PermissionRequest => {
                self.upsert_session(
                    event.agent,
                    event.session_id.clone(),
                    AgentState::ToolUse,
                    event.data.tool_name.clone(),
                    event.data.summary.clone(),
                    now,
                );

                let permission_id = event
                    .data
                    .permission_id
                    .unwrap_or_else(|| self.next_permission_id());

                if !self.permissions.contains_key(&permission_id) {
                    let tool_name = event
                        .data
                        .tool_name
                        .unwrap_or_else(|| "未知操作".to_string());
                    let summary = event.data.summary.unwrap_or_else(|| "等待授权".to_string());

                    self.permissions.insert(
                        permission_id.clone(),
                        PermissionRecord {
                            id: permission_id.clone(),
                            agent: event.agent,
                            session_id: event.session_id,
                            tool_name,
                            summary,
                            status: PermissionStatus::Pending,
                            last_touched_at: now,
                        },
                    );
                    self.permission_queue.push_back(permission_id);
                }
            }
        }

        self.finish_transition(previous_active, Vec::new())
    }

    pub fn respond_permission(
        &mut self,
        permission_id: &str,
        approved: bool,
    ) -> Vec<NavigatorEmission> {
        let previous_active = self.active_permission_id();
        let status = if approved {
            PermissionStatus::Approved
        } else {
            PermissionStatus::Denied
        };
        let now = Instant::now();
        let mut emissions = Vec::new();

        if let Some(permission) = self.permissions.get_mut(permission_id) {
            permission.status = status;
            permission.last_touched_at = now;

            if let Some(session) = self
                .sessions
                .get_mut(&session_key(&permission.agent, &permission.session_id))
            {
                session.state = if approved {
                    AgentState::ToolUse
                } else {
                    AgentState::Error
                };
                session.tool_name = Some(permission.tool_name.clone());
                session.summary = Some(if approved {
                    permission.summary.clone()
                } else {
                    format!("已拒绝：{}", permission.summary)
                });
                session.updated_at = now;
            }

            emissions.push(NavigatorEmission::PermissionResolved(
                PermissionResolvedPayload {
                    permission_id: permission_id.to_string(),
                    status,
                },
            ));
        }

        self.permission_queue.retain(|id| id != permission_id);
        self.permissions
            .retain(|_, permission| permission.status == PermissionStatus::Pending);

        self.finish_transition(previous_active, emissions)
    }

    pub fn get_permission_status(&mut self, permission_id: &str) -> PermissionStatus {
        let Some(permission) = self.permissions.get_mut(permission_id) else {
            return PermissionStatus::Denied;
        };

        permission.last_touched_at = Instant::now();
        permission.status
    }

    pub fn cleanup_stale(&mut self) -> Vec<NavigatorEmission> {
        let previous_active = self.active_permission_id();
        let now = Instant::now();
        let mut emissions = Vec::new();

        self.sessions
            .retain(|_, session| now.duration_since(session.updated_at) <= SESSION_TTL);

        let stale_permissions = self
            .permissions
            .values()
            .filter(|permission| {
                permission.status == PermissionStatus::Pending
                    && now.duration_since(permission.last_touched_at) > PERMISSION_TTL
            })
            .map(|permission| permission.id.clone())
            .collect::<Vec<_>>();

        for permission_id in stale_permissions {
            self.permission_queue
                .retain(|queued| queued != &permission_id);
            self.permissions.remove(&permission_id);
            emissions.push(NavigatorEmission::PermissionResolved(
                PermissionResolvedPayload {
                    permission_id,
                    status: PermissionStatus::Denied,
                },
            ));
        }

        self.finish_transition(previous_active, emissions)
    }

    fn upsert_session(
        &mut self,
        agent: super::events::AgentType,
        session_id: String,
        state: AgentState,
        tool_name: Option<String>,
        summary: Option<String>,
        updated_at: Instant,
    ) {
        self.sessions.insert(
            session_key(&agent, &session_id),
            SessionRecord {
                agent,
                session_id,
                state,
                tool_name,
                summary,
                updated_at,
            },
        );
    }

    fn finish_transition(
        &mut self,
        previous_active: Option<String>,
        mut emissions: Vec<NavigatorEmission>,
    ) -> Vec<NavigatorEmission> {
        let current_active = self.active_permission_id();

        if previous_active != current_active {
            if let Some(payload) = self.current_permission_payload() {
                emissions.push(NavigatorEmission::PermissionRequest(payload));
            }
        }

        if let Some(payload) = self.refresh_snapshot() {
            emissions.push(NavigatorEmission::StateChange(payload));
        }

        emissions
    }

    fn refresh_snapshot(&mut self) -> Option<StateChangePayload> {
        let candidate = self.compute_raw_snapshot();
        let now = Instant::now();

        let next = match (&self.last_snapshot, self.last_snapshot_at) {
            (Some(previous), Some(previous_at))
                if candidate.state.priority() < previous.state.priority()
                    && now.duration_since(previous_at) < min_state_duration(previous.state) =>
            {
                previous.clone()
            }
            _ => candidate,
        };

        if self.last_snapshot.as_ref() == Some(&next) {
            return None;
        }

        self.last_snapshot = Some(next.clone());
        self.last_snapshot_at = Some(now);
        Some(next)
    }

    fn compute_raw_snapshot(&self) -> StateChangePayload {
        if let Some(permission) = self.current_permission_payload() {
            return StateChangePayload {
                state: AgentState::WaitingPermission,
                agent: Some(permission.agent),
                session_id: Some(permission.session_id),
                tool_name: Some(permission.tool_name),
                summary: Some(permission.summary),
                server_port: self.server_port,
            };
        }

        let best = self.sessions.values().max_by(|left, right| {
            left.state
                .priority()
                .cmp(&right.state.priority())
                .then(left.updated_at.cmp(&right.updated_at))
        });

        match best {
            Some(session) => StateChangePayload {
                state: session.state,
                agent: Some(session.agent),
                session_id: Some(session.session_id.clone()),
                tool_name: session.tool_name.clone(),
                summary: session.summary.clone(),
                server_port: self.server_port,
            },
            None => StateChangePayload {
                state: AgentState::Idle,
                agent: None,
                session_id: None,
                tool_name: None,
                summary: None,
                server_port: self.server_port,
            },
        }
    }

    fn current_permission_payload(&self) -> Option<PermissionRequestPayload> {
        let permission_id = self.permission_queue.front()?;
        let permission = self.permissions.get(permission_id)?;
        Some(PermissionRequestPayload {
            permission_id: permission.id.clone(),
            agent: permission.agent,
            session_id: permission.session_id.clone(),
            tool_name: permission.tool_name.clone(),
            summary: permission.summary.clone(),
        })
    }

    fn active_permission_id(&self) -> Option<String> {
        self.permission_queue.front().cloned()
    }

    fn next_permission_id(&self) -> String {
        let counter = PERMISSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("perm-{counter}")
    }
}

fn min_state_duration(state: AgentState) -> Duration {
    match state {
        AgentState::Error => Duration::from_secs(2),
        AgentState::ToolUse => Duration::from_secs(1),
        AgentState::Thinking => Duration::from_millis(1500),
        AgentState::Idle | AgentState::WaitingPermission => Duration::ZERO,
    }
}
