use std::{
    fs,
    path::Path,
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
use crate::platform;
use crate::shell;

const DEFAULT_PORT: u16 = 23333;
const PORT_ATTEMPTS: u16 = 10;

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
            state
                .lock()
                .ok()
                .map(|navigator| {
                    vec![
                        NavigatorEmission::StateChange(navigator.snapshot().current),
                        NavigatorEmission::SessionsChanged(navigator.sessions_snapshot()),
                    ]
                })
                .unwrap_or_else(|| {
                    vec![NavigatorEmission::StateChange(
                        super::events::StateChangePayload {
                            state: super::events::AgentState::Idle,
                            agent: None,
                            session_id: None,
                            tool_name: None,
                            summary: None,
                            working_directory: None,
                            session_title: None,
                            needs_attention: None,
                            server_port: Some(port),
                            ai_talk_context: None,
                        },
                    )]
                }),
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
                (&Method::Get, url) if url.starts_with("/model/current/") => {
                    let response = handle_model_request(url, &app_handle);
                    let _ = request.respond(response);
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
    if let Ok(header) = Header::from_bytes("Access-Control-Allow-Origin", "*") {
        response = response.with_header(header);
    }
    response
}

fn handle_model_request(url: &str, app_handle: &AppHandle) -> Response<std::io::Cursor<Vec<u8>>> {
    let relative = url.trim_start_matches("/model/current/");
    let Ok(decoded) = urlencoding::decode(relative) else {
        return json_response(StatusCode(400), json!({ "error": "invalid_model_path" }));
    };

    let Some(model_directory) = shell::current_model_directory(app_handle) else {
        return json_response(StatusCode(404), json!({ "error": "no_custom_model" }));
    };

    let file_path = match shell::resolve_model_resource_path(&model_directory, decoded.as_ref()) {
        Ok(path) => path,
        Err(err) => return json_response(StatusCode(404), json!({ "error": err })),
    };

    let bytes = match fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            return json_response(
                StatusCode(500),
                json!({ "error": format!("failed_to_read_resource: {err}") }),
            )
        }
    };

    let mut response = Response::from_data(bytes).with_status_code(StatusCode(200));
    if let Ok(header) = Header::from_bytes("Content-Type", content_type_for(&file_path)) {
        response = response.with_header(header);
    }
    if let Ok(header) = Header::from_bytes("Access-Control-Allow-Origin", "*") {
        response = response.with_header(header);
    }
    response
}

fn content_type_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "moc3" => "application/octet-stream",
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        _ => "application/octet-stream",
    }
}

fn write_port_files(port: u16) {
    if let Ok(path) = platform::primary_port_file() {
        let _ = write_port_file(&path, port);
    }
    let _ = write_port_file(&platform::fallback_port_file(), port);
}

fn write_port_file(path: &Path, port: u16) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, port.to_string()).map_err(|err| err.to_string())
}
