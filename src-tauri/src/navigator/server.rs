use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use serde_json::json;
use tauri::AppHandle;
use tiny_http::{Header, Method, Response, Server, StatusCode};

use super::{
    emit_all,
    events::{IncomingHookEvent, NavigatorEmission},
    state::NavigatorState,
};

const DEFAULT_PORT: u16 = 23333;
const PORT_ATTEMPTS: u16 = 10;
const TMP_PORT_FILE: &str = "/tmp/copiwaifu-port";

pub fn start(app_handle: AppHandle, state: Arc<Mutex<NavigatorState>>) {
    thread::spawn(move || {
        let Some((server, port)) = bind_server() else {
            eprintln!("navigator server failed to bind any port");
            return;
        };

        if let Ok(mut navigator) = state.lock() {
            navigator.set_server_port(port);
        }
        emit_all(
            &app_handle,
            vec![NavigatorEmission::StateChange(
                state
                    .lock()
                    .ok()
                    .map(|navigator| navigator.snapshot().current)
                    .unwrap_or_else(|| super::events::StateChangePayload {
                        state: super::events::AgentState::Idle,
                        agent: None,
                        session_id: None,
                        tool_name: None,
                        summary: None,
                        server_port: Some(port),
                    }),
            )],
        );

        write_port_files(port);

        for mut request in server.incoming_requests() {
            match (request.method(), request.url()) {
                (&Method::Get, "/status") => {
                    let _ = request.respond(json_response(
                        StatusCode(200),
                        json!({ "ok": true, "port": port }),
                    ));
                }
                (&Method::Post, "/event") => {
                    let response = handle_event_request(&mut request, &app_handle, &state);
                    let _ = request.respond(response);
                }
                (&Method::Get, url) if url.starts_with("/permission/") => {
                    let permission_id = url.trim_start_matches("/permission/");
                    let status = match state.lock() {
                        Ok(mut navigator) => navigator.get_permission_status(permission_id),
                        Err(err) => {
                            eprintln!("navigator permission lock poisoned: {err}");
                            super::events::PermissionStatus::Denied
                        }
                    };
                    let _ = request
                        .respond(json_response(StatusCode(200), json!({ "status": status })));
                }
                _ => {
                    let _ = request.respond(json_response(
                        StatusCode(404),
                        json!({ "error": "not_found" }),
                    ));
                }
            }
        }
    });
}

fn bind_server() -> Option<(Server, u16)> {
    for port in DEFAULT_PORT..(DEFAULT_PORT + PORT_ATTEMPTS) {
        let address = format!("127.0.0.1:{port}");
        if let Ok(server) = Server::http(&address) {
            return Some((server, port));
        }
    }

    None
}

fn handle_event_request(
    request: &mut tiny_http::Request,
    app_handle: &AppHandle,
    state: &Arc<Mutex<NavigatorState>>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut body = String::new();
    if let Err(err) = request.as_reader().read_to_string(&mut body) {
        return json_response(
            StatusCode(400),
            json!({ "error": format!("failed_to_read_body: {err}") }),
        );
    }

    let parsed = serde_json::from_str::<IncomingHookEvent>(&body)
        .map_err(|err| err.to_string())
        .and_then(|payload| payload.into_agent_event());

    let event = match parsed {
        Ok(event) => event,
        Err(err) => {
            return json_response(StatusCode(400), json!({ "error": err }));
        }
    };

    let emissions = match state.lock() {
        Ok(mut navigator) => navigator.apply_event(event),
        Err(err) => {
            return json_response(
                StatusCode(500),
                json!({ "error": format!("state_lock_failed: {err}") }),
            );
        }
    };

    emit_all(app_handle, emissions);
    json_response(StatusCode(200), json!({ "ok": true }))
}

fn json_response(
    status: StatusCode,
    body: serde_json::Value,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut response = Response::from_string(body.to_string()).with_status_code(status);
    if let Ok(header) = Header::from_bytes("Content-Type", "application/json") {
        response = response.with_header(header);
    }
    response
}

fn write_port_files(port: u16) {
    let _ = write_port_file(primary_port_file().as_path(), port);
    let _ = write_port_file(Path::new(TMP_PORT_FILE), port);
}

fn write_port_file(path: &Path, port: u16) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, port.to_string()).map_err(|err| err.to_string())
}

fn primary_port_file() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".copiwaifu")
        .join("port")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
