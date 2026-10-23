//! Loopback JSON API for agents. Runs inside the Slate process and calls the
//! same command functions the UI uses. Bind is 127.0.0.1 only.

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Value};
use tauri::{Emitter, Manager};
use zeroize::Zeroize;

use crate::filesystem;

pub const DEFAULT_PORT: u16 = 47321;
const MAX_BODY: usize = 4 * 1024 * 1024;
const MAX_HEADERS: usize = 32 * 1024;

struct Action {
    name: &'static str,
    method: &'static str,
    path: &'static str,
}

const ACTIONS: &[Action] = &[
    Action {
        name: "capabilities",
        method: "GET",
        path: "/v1/capabilities",
    },
    Action {
        name: "version",
        method: "GET",
        path: "/v1/version",
    },
    Action {
        name: "shortcut",
        method: "GET",
        path: "/v1/shortcut",
    },
    Action {
        name: "window_show",
        method: "POST",
        path: "/v1/window/show",
    },
    Action {
        name: "window_hide",
        method: "POST",
        path: "/v1/window/hide",
    },
    Action {
        name: "window_toggle",
        method: "POST",
        path: "/v1/window/toggle",
    },
    Action {
        name: "window_minimize",
        method: "POST",
        path: "/v1/window/minimize",
    },
    Action {
        name: "quit",
        method: "POST",
        path: "/v1/app/quit",
    },
    Action {
        name: "notes_list",
        method: "GET",
        path: "/v1/notes",
    },
    Action {
        name: "notes_read",
        method: "GET",
        path: "/v1/notes/{filename}",
    },
    Action {
        name: "notes_write",
        method: "PUT",
        path: "/v1/notes",
    },
    Action {
        name: "notes_delete",
        method: "DELETE",
        path: "/v1/notes/{filename}",
    },
    Action {
        name: "notes_archive",
        method: "POST",
        path: "/v1/notes/archive",
    },
    Action {
        name: "clipboard_read",
        method: "GET",
        path: "/v1/clipboard",
    },
    Action {
        name: "clipboard_write",
        method: "PUT",
        path: "/v1/clipboard",
    },
    Action {
        name: "clipboard_history",
        method: "GET",
        path: "/v1/clipboard/history",
    },
    Action {
        name: "clipboard_filter_get",
        method: "GET",
        path: "/v1/clipboard/filter",
    },
    Action {
        name: "clipboard_filter_set",
        method: "PUT",
        path: "/v1/clipboard/filter",
    },
    Action {
        name: "vault_status",
        method: "GET",
        path: "/v1/vault",
    },
    Action {
        name: "vault_init",
        method: "POST",
        path: "/v1/vault/init",
    },
    Action {
        name: "vault_unlock",
        method: "POST",
        path: "/v1/vault/unlock",
    },
    Action {
        name: "vault_lock",
        method: "POST",
        path: "/v1/vault/lock",
    },
    Action {
        name: "vault_secrets_list",
        method: "GET",
        path: "/v1/vault/secrets",
    },
    Action {
        name: "vault_secrets_add",
        method: "POST",
        path: "/v1/vault/secrets",
    },
    Action {
        name: "vault_secret_reveal",
        method: "POST",
        path: "/v1/vault/secrets/{index}/reveal",
    },
    Action {
        name: "vault_secret_copy",
        method: "POST",
        path: "/v1/vault/secrets/{index}/copy",
    },
    Action {
        name: "vault_secret_delete",
        method: "DELETE",
        path: "/v1/vault/secrets/{index}",
    },
    Action {
        name: "settings_get",
        method: "GET",
        path: "/v1/settings",
    },
    Action {
        name: "settings_put",
        method: "PUT",
        path: "/v1/settings",
    },
];

struct AgentCtx {
    token: String,
    state: crate::AppState,
    app: Option<tauri::AppHandle>,
}

struct Reply {
    status: u16,
    body: Value,
    quit_after: bool,
}

struct Incoming {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: String,
}

pub fn start(app: tauri::AppHandle, state: crate::AppState) -> Result<(), String> {
    let token = load_or_create_token()?;
    let listener = bind_loopback(DEFAULT_PORT)?;
    if let Err(e) = write_port_file(&port_file_path(), DEFAULT_PORT) {
        tracing::error!("agent api port file not written: {e}");
    }
    let ctx = AgentCtx {
        token,
        state,
        app: Some(app),
    };
    std::thread::Builder::new()
        .name("slate-agent-api".into())
        .spawn(move || accept_loop(listener, ctx))
        .map_err(|e| format!("agent api thread failed: {e}"))?;
    tracing::info!("agent api listening on 127.0.0.1:{DEFAULT_PORT}");
    Ok(())
}

pub fn bind_loopback(port: u16) -> Result<TcpListener, String> {
    let addr = loopback_addr(port)?;
    TcpListener::bind(addr).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AddrInUse {
            format!("agent api port {port} is already in use on 127.0.0.1")
        } else {
            format!("agent api failed to bind 127.0.0.1:{port}: {e}")
        }
    })
}

fn loopback_addr(port: u16) -> Result<SocketAddr, String> {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    require_loopback(addr)?;
    Ok(addr)
}

fn require_loopback(addr: SocketAddr) -> Result<(), String> {
    if addr.ip() == IpAddr::V4(Ipv4Addr::LOCALHOST) {
        Ok(())
    } else {
        Err(format!("refusing non-loopback bind {}", addr.ip()))
    }
}

fn peer_allowed(peer: SocketAddr) -> bool {
    peer.ip().is_loopback()
}

fn accept_loop(listener: TcpListener, ctx: AgentCtx) {
    loop {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer_allowed(peer) {
                    continue;
                }
                if let Err(e) = serve_client(&mut stream, &ctx) {
                    tracing::debug!("agent api connection closed: {e}");
                }
            }
            Err(e) => {
                tracing::error!("agent api accept failed: {e}");
                break;
            }
        }
    }
}

fn serve_client(stream: &mut TcpStream, ctx: &AgentCtx) -> Result<(), String> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));
    let mut req = match read_incoming(stream) {
        Ok(req) => req,
        Err(e) => {
            let reply = err(400, e);
            write_reply(stream, &reply)?;
            return Ok(());
        }
    };
    let reply = dispatch(ctx, &req);
    zeroize_incoming(&mut req);
    write_reply(stream, &reply)?;
    if reply.quit_after {
        if let Some(app) = &ctx.app {
            crate::perform_confirm_quit(app, &ctx.state);
        }
    }
    Ok(())
}

fn zeroize_incoming(req: &mut Incoming) {
    req.body.zeroize();
    for (_, value) in &mut req.headers {
        value.zeroize();
    }
}

fn dispatch(ctx: &AgentCtx, req: &Incoming) -> Reply {
    if !bearer_ok(&req.headers, &ctx.token) {
        return err(401, "unauthorized");
    }
    let method = req.method.as_str();
    let path = req.path.as_str();

    if let Some(reply) = dispatch_exact(ctx, method, path, &req.body) {
        return reply;
    }
    if let Some(reply) = dispatch_pattern(ctx, method, path, &req.body) {
        return reply;
    }
    if ACTIONS.iter().any(|action| action.path == path) {
        return err(405, "method not allowed");
    }
    err(404, "not found")
}

fn dispatch_exact(ctx: &AgentCtx, method: &str, path: &str, body: &str) -> Option<Reply> {
    Some(match (method, path) {
        ("GET", "/v1/capabilities") => reply_capabilities(),
        ("GET", "/v1/version") => from_value(crate::get_app_version_info()),
        ("GET", "/v1/shortcut") => ok_json(json!(crate::get_summon_shortcut_label())),
        ("POST", "/v1/window/show") => window_op(ctx, |app| {
            crate::menu_show_main_window(app);
            Ok(())
        }),
        ("POST", "/v1/window/hide") => window_op(ctx, |app| {
            crate::menu_hide_main_window(app);
            Ok(())
        }),
        ("POST", "/v1/window/toggle") => window_op(ctx, |app| {
            crate::toggle_window(app);
            Ok(())
        }),
        ("POST", "/v1/window/minimize") => window_op(ctx, |app| {
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| "main window unavailable".to_string())?;
            window
                .minimize()
                .map_err(|e| format!("minimize failed: {e}"))?;
            Ok(())
        }),
        ("POST", "/v1/app/quit") => quit(ctx),
        ("GET", "/v1/notes") => from_result(crate::list_notes()),
        ("PUT", "/v1/notes") => write_note(body),
        ("POST", "/v1/notes/archive") => from_result(crate::archive_notes()),
        ("GET", "/v1/clipboard") => from_result(crate::read_text_from_clipboard()),
        ("PUT", "/v1/clipboard") => write_clipboard(body),
        ("GET", "/v1/clipboard/history") => {
            ok_json(json!(crate::clipboard_history_entries(&ctx.state)))
        }
        ("GET", "/v1/clipboard/filter") => {
            ok_json(json!(crate::clipboard_filter_enabled(&ctx.state)))
        }
        ("PUT", "/v1/clipboard/filter") => set_filter(ctx, body),
        ("GET", "/v1/vault") => ok_json(json!({
            "initialized": crate::is_vault_initialized(),
            "unlocked": crate::vault_is_unlocked(&ctx.state),
        })),
        ("POST", "/v1/vault/init") => vault_password(ctx, body, true),
        ("POST", "/v1/vault/unlock") => vault_password(ctx, body, false),
        ("POST", "/v1/vault/lock") => {
            crate::lock_vault_session(&ctx.state);
            ok_json(json!({ "unlocked": false }))
        }
        ("GET", "/v1/vault/secrets") => from_result(crate::list_secret_metadata_for(&ctx.state)),
        ("POST", "/v1/vault/secrets") => add_secret(ctx, body),
        ("GET", "/v1/settings") => get_settings(ctx),
        ("PUT", "/v1/settings") => put_settings(ctx, body),
        _ => return None,
    })
}

fn dispatch_pattern(ctx: &AgentCtx, method: &str, path: &str, _body: &str) -> Option<Reply> {
    if let Some(name) = path.strip_prefix("/v1/notes/") {
        if name.is_empty() {
            return None;
        }
        return Some(match method {
            "GET" => from_result(crate::read_note(name.to_string())),
            "DELETE" => from_result(crate::delete_note(name.to_string())),
            _ => err(405, "method not allowed"),
        });
    }

    if let Some(index) = secret_index(path, "/reveal") {
        return Some(match method {
            "POST" => from_result(crate::reveal_secret_for(index, &ctx.state)),
            _ => err(405, "method not allowed"),
        });
    }
    if let Some(index) = secret_index(path, "/copy") {
        return Some(match method {
            "POST" => from_result(crate::copy_secret_for(index, &ctx.state)),
            _ => err(405, "method not allowed"),
        });
    }
    if let Some(index) = secret_index(path, "") {
        if path.ends_with("/reveal") || path.ends_with("/copy") {
            return None;
        }
        return Some(match method {
            "DELETE" => from_result(crate::delete_secret_for(index, &ctx.state)),
            _ => err(405, "method not allowed"),
        });
    }
    None
}

fn secret_index(path: &str, suffix: &str) -> Option<usize> {
    let rest = path.strip_prefix("/v1/vault/secrets/")?;
    let rest = if suffix.is_empty() {
        rest
    } else {
        rest.strip_suffix(suffix)?
    };
    if rest.is_empty() || rest.contains('/') {
        return None;
    }
    rest.parse().ok()
}

fn reply_capabilities() -> Reply {
    let actions: Vec<Value> = ACTIONS
        .iter()
        .map(|action| {
            json!({
                "name": action.name,
                "method": action.method,
                "path": action.path,
            })
        })
        .collect();
    ok_json(json!({
        "bind": "127.0.0.1",
        "actions": actions,
        "ui_only": "Caret, undo stack, command palette, editor view, and menu events that only update the webview are not in this API.",
    }))
}

fn window_op(ctx: &AgentCtx, op: impl FnOnce(&tauri::AppHandle) -> Result<(), String>) -> Reply {
    let Some(app) = ctx.app.as_ref() else {
        return err(500, "window unavailable");
    };
    from_result(op(app))
}

fn quit(ctx: &AgentCtx) -> Reply {
    if ctx.app.is_none() {
        return err(500, "window unavailable");
    }
    Reply {
        status: 200,
        body: json!({ "ok": true, "data": { "quitting": true } }),
        quit_after: true,
    }
}

fn write_note(body: &str) -> Reply {
    #[derive(serde::Deserialize)]
    struct Body {
        title: String,
        content: String,
        is_ephemeral: bool,
        #[serde(default)]
        previous_filename: Option<String>,
    }
    let parsed: Body = match parse_json(body) {
        Ok(v) => v,
        Err(reply) => return reply,
    };
    from_result(crate::write_note(
        parsed.title,
        parsed.content,
        parsed.is_ephemeral,
        parsed.previous_filename,
    ))
}

fn write_clipboard(body: &str) -> Reply {
    #[derive(serde::Deserialize)]
    struct Body {
        text: String,
    }
    let parsed: Body = match parse_json(body) {
        Ok(v) => v,
        Err(reply) => return reply,
    };
    from_result(crate::copy_text_to_clipboard(parsed.text))
}

fn set_filter(ctx: &AgentCtx, body: &str) -> Reply {
    #[derive(serde::Deserialize)]
    struct Body {
        enabled: bool,
    }
    let parsed: Body = match parse_json(body) {
        Ok(v) => v,
        Err(reply) => return reply,
    };
    crate::set_clipboard_filter_flag(&ctx.state, parsed.enabled);
    if let Some(app) = ctx.app.as_ref() {
        let stored = if parsed.enabled { "true" } else { "false" };
        if let Err(e) = write_setting_string(app, "gnomad-clipboard-filter", stored) {
            tracing::error!("agent api clipboard filter not saved: {e}");
        }
    }
    ok_json(json!(parsed.enabled))
}

fn vault_password(ctx: &AgentCtx, body: &str, init: bool) -> Reply {
    #[derive(serde::Deserialize)]
    struct Body {
        password: String,
    }
    let mut parsed: Body = match parse_json(body) {
        Ok(v) => v,
        Err(reply) => return reply,
    };
    let result = if init {
        crate::initialize_vault_with(&parsed.password, false, &ctx.state).map(|recovery_key| {
            json!({ "unlocked": true, "recovery_key": recovery_key })
        })
    } else {
        crate::unlock_vault_with(&parsed.password, &ctx.state).map(|_| json!({ "unlocked": true }))
    };
    parsed.password.zeroize();
    match result {
        Ok(body) => ok_json(body),
        Err(e) => err(400, e),
    }
}

fn add_secret(ctx: &AgentCtx, body: &str) -> Reply {
    #[derive(serde::Deserialize)]
    struct Body {
        name: String,
        path: String,
        value: String,
    }
    let mut parsed: Body = match parse_json(body) {
        Ok(v) => v,
        Err(reply) => return reply,
    };
    let result = crate::add_secret_for(
        std::mem::take(&mut parsed.name),
        std::mem::take(&mut parsed.path),
        std::mem::take(&mut parsed.value),
        &ctx.state,
    );
    parsed.value.zeroize();
    from_result(result)
}

fn get_settings(ctx: &AgentCtx) -> Reply {
    let Some(app) = ctx.app.as_ref() else {
        return err(500, "settings store unavailable");
    };
    match read_settings(app, &ctx.state) {
        Ok(value) => ok_json(value),
        Err(e) => err(500, e),
    }
}

fn put_settings(ctx: &AgentCtx, body: &str) -> Reply {
    #[derive(serde::Deserialize)]
    struct Body {
        #[serde(default)]
        theme: Option<String>,
        #[serde(default)]
        line_numbers: Option<bool>,
        #[serde(default)]
        clipboard_filter: Option<bool>,
    }
    let Some(app) = ctx.app.as_ref() else {
        return err(500, "settings store unavailable");
    };
    let parsed: Body = match parse_json(body) {
        Ok(v) => v,
        Err(reply) => return reply,
    };
    if parsed.theme.is_none() && parsed.line_numbers.is_none() && parsed.clipboard_filter.is_none()
    {
        return err(400, "no settings provided");
    }
    if let Some(theme) = &parsed.theme {
        if !matches!(theme.as_str(), "light" | "dark" | "system") {
            return err(400, "theme must be light, dark, or system");
        }
    }
    if let Some(theme) = &parsed.theme {
        if let Err(e) = write_setting_string(app, "gnomad-theme", theme) {
            return err(500, e);
        }
        let event = match theme.as_str() {
            "light" => "menu-theme-light",
            "dark" => "menu-theme-dark",
            _ => "menu-theme-system",
        };
        let _ = app.emit(event, ());
    }
    if let Some(enabled) = parsed.line_numbers {
        let value = if enabled { "true" } else { "false" };
        if let Err(e) = write_setting_string(app, "gnomad-line-numbers", value) {
            return err(500, e);
        }
    }
    if let Some(enabled) = parsed.clipboard_filter {
        crate::set_clipboard_filter_flag(&ctx.state, enabled);
        let value = if enabled { "true" } else { "false" };
        if let Err(e) = write_setting_string(app, "gnomad-clipboard-filter", value) {
            return err(500, e);
        }
    }
    get_settings(ctx)
}

fn read_settings(app: &tauri::AppHandle, state: &crate::AppState) -> Result<Value, String> {
    let store = open_settings(app)?;
    let _ = store.reload_ignore_defaults();
    let theme = store.get("gnomad-theme").and_then(|value| match value {
        Value::String(text) => Some(text),
        _ => None,
    });
    let line_numbers = store
        .get("gnomad-line-numbers")
        .and_then(|value| json_bool(&value));
    Ok(json!({
        "theme": theme,
        "line_numbers": line_numbers,
        "clipboard_filter": crate::clipboard_filter_enabled(state),
    }))
}

fn write_setting_string(app: &tauri::AppHandle, key: &str, value: &str) -> Result<(), String> {
    let store = open_settings(app)?;
    let _ = store.reload_ignore_defaults();
    store.set(key, value);
    store
        .save()
        .map_err(|e| format!("settings save failed: {e}"))?;
    Ok(())
}

fn open_settings(
    app: &tauri::AppHandle,
) -> Result<std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    use tauri_plugin_store::StoreExt;
    app.store("settings.json")
        .map_err(|e| format!("settings store unavailable: {e}"))
}

fn json_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(flag) => Some(*flag),
        Value::String(text) if text == "true" => Some(true),
        Value::String(text) if text == "false" => Some(false),
        _ => None,
    }
}

fn parse_json<T: serde::de::DeserializeOwned>(body: &str) -> Result<T, Reply> {
    serde_json::from_str(body).map_err(|_| err(400, "invalid json"))
}

fn from_result<T: Serialize>(result: Result<T, String>) -> Reply {
    match result {
        Ok(value) => from_value(value),
        Err(error) => err(400, error),
    }
}

fn from_value<T: Serialize>(value: T) -> Reply {
    match serde_json::to_value(value) {
        Ok(data) => ok_json(data),
        Err(_) => err(500, "encode failed"),
    }
}

fn ok_json(data: Value) -> Reply {
    Reply {
        status: 200,
        body: json!({ "ok": true, "data": data }),
        quit_after: false,
    }
}

fn err(status: u16, error: impl Into<String>) -> Reply {
    Reply {
        status,
        body: json!({ "ok": false, "error": error.into() }),
        quit_after: false,
    }
}

fn bearer_ok(headers: &[(String, String)], token: &str) -> bool {
    let Some(value) = header(headers, "authorization") else {
        return false;
    };
    let Some(presented) = value.trim().strip_prefix("Bearer ") else {
        return false;
    };
    ct_eq(presented.trim(), token)
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn ct_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

fn read_incoming(stream: &mut TcpStream) -> Result<Incoming, String> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    let header_end = loop {
        if buf.len() > MAX_HEADERS {
            return Err("headers too large".into());
        }
        let n = stream
            .read(&mut tmp)
            .map_err(|_| "read failed".to_string())?;
        if n == 0 {
            return Err("connection closed".into());
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(index) = find_header_end(&buf) {
            break index;
        }
    };
    let head =
        std::str::from_utf8(&buf[..header_end]).map_err(|_| "invalid request".to_string())?;
    let mut lines = head.split("\r\n");
    let request_line = lines.next().ok_or("invalid request")?;
    let mut parts = request_line.split(' ');
    let method = parts.next().unwrap_or("").to_ascii_uppercase();
    let raw_path = parts.next().unwrap_or("");
    let version = parts.next().unwrap_or("");
    if !version.starts_with("HTTP/1.") || method.is_empty() || !raw_path.starts_with('/') {
        return Err("invalid request".into());
    }
    let path_only = raw_path.split('?').next().unwrap_or(raw_path);
    let path = percent_decode(path_only);
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            return Err("invalid header".into());
        };
        headers.push((key.trim().to_string(), value.trim().to_string()));
    }
    if headers.iter().any(|(key, value)| {
        key.eq_ignore_ascii_case("transfer-encoding") && !value.eq_ignore_ascii_case("identity")
    }) {
        return Err("chunked bodies are not supported".into());
    }
    let len = match header(&headers, "content-length") {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| "invalid content-length".to_string())?,
        None => 0,
    };
    if len > MAX_BODY {
        return Err("body too large".into());
    }
    let mut body_bytes = buf[header_end + 4..].to_vec();
    while body_bytes.len() < len {
        let n = stream
            .read(&mut tmp)
            .map_err(|_| "read failed".to_string())?;
        if n == 0 {
            return Err("truncated body".into());
        }
        body_bytes.extend_from_slice(&tmp[..n]);
    }
    body_bytes.truncate(len);
    buf.zeroize();
    let body = String::from_utf8(body_bytes).map_err(|_| "body must be utf-8".to_string())?;
    Ok(Incoming {
        method,
        path,
        headers,
        body,
    })
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|window| window == b"\r\n\r\n")
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(
                std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or(""),
                16,
            ) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn write_reply(stream: &mut TcpStream, reply: &Reply) -> Result<(), String> {
    let mut bytes = serde_json::to_vec(&reply.body)
        .unwrap_or_else(|_| br#"{"ok":false,"error":"encode failed"}"#.to_vec());
    let reason = match reply.status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n",
        status = reply.status,
        len = bytes.len(),
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(&bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| "write failed".to_string())?;
    bytes.zeroize();
    Ok(())
}

fn token_file_path() -> PathBuf {
    filesystem::get_slate_dir().join("agent-api.token")
}

fn port_file_path() -> PathBuf {
    filesystem::get_slate_dir().join("agent-api.json")
}

fn load_or_create_token() -> Result<String, String> {
    let path = token_file_path();
    if path.exists() {
        let token = std::fs::read_to_string(&path)
            .map_err(|e| format!("agent api token file unreadable: {e}"))?;
        let token = token.trim().to_string();
        if token.len() < 16 {
            return Err("agent api token file is empty".into());
        }
        write_private_file(&path, &token)?;
        return Ok(token);
    }
    let token = generate_token();
    write_private_file(&path, &token)?;
    Ok(token)
}

fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut token = String::with_capacity(64);
    for byte in bytes {
        token.push(HEX[(byte >> 4) as usize] as char);
        token.push(HEX[(byte & 0xf) as usize] as char);
    }
    bytes.zeroize();
    token
}

fn write_port_file(path: &Path, port: u16) -> Result<(), String> {
    let contents = format!("{{\"port\":{port}}}\n");
    debug_assert!(!contents.contains("token"));
    write_with_mode(path, &contents, 0o644)
}

fn write_private_file(path: &Path, contents: &str) -> Result<(), String> {
    write_with_mode(path, contents, 0o600)
}

fn write_with_mode(path: &Path, contents: &str, mode: u32) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("agent api data dir: {e}"))?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    {
        let mut file = options
            .open(path)
            .map_err(|e| format!("agent api file create failed: {e}"))?;
        file.write_all(contents.as_bytes())
            .map_err(|e| format!("agent api file write failed: {e}"))?;
        let _ = file.sync_all();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .map_err(|e| format!("agent api file mode failed: {e}"))?;
    }
    let _ = mode;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;

    fn test_ctx() -> AgentCtx {
        AgentCtx {
            token: "unit-test-token".into(),
            state: crate::AppState {
                clipboard_history: Arc::new(Mutex::new(crate::clipboard::ClipboardHistory::new(
                    15,
                ))),
                clipboard_filter_enabled: Arc::new(Mutex::new(true)),
                vault_session: Arc::new(Mutex::new(None)),
                unlock_limiter: Arc::new(Mutex::new(
                    crate::unlock_limiter::UnlockRateLimiter::new(),
                )),
            },
            app: None,
        }
    }

    fn call(ctx: &AgentCtx, method: &str, path: &str, auth: Option<&str>, body: &str) -> Reply {
        let mut headers = vec![("host".into(), "127.0.0.1".into())];
        if let Some(auth) = auth {
            headers.push(("authorization".into(), auth.into()));
        }
        dispatch(
            ctx,
            &Incoming {
                method: method.into(),
                path: percent_decode(path.split('?').next().unwrap_or(path)),
                headers,
                body: body.into(),
            },
        )
    }

    #[test]
    fn rejects_missing_and_wrong_auth() {
        let ctx = test_ctx();
        let missing = call(&ctx, "GET", "/v1/capabilities", None, "");
        assert_eq!(missing.status, 401);
        assert_eq!(missing.body["ok"], false);
        assert_eq!(missing.body["error"], "unauthorized");
        assert!(missing.body.get("data").is_none());

        let wrong = call(&ctx, "GET", "/v1/capabilities", Some("Bearer nope"), "");
        assert_eq!(wrong.status, 401);

        let query = call(
            &ctx,
            "GET",
            "/v1/capabilities?access_token=unit-test-token",
            None,
            "",
        );
        assert_eq!(query.status, 401);

        let unlock = call(
            &ctx,
            "POST",
            "/v1/vault/unlock",
            None,
            r#"{"password":"not-a-real-password"}"#,
        );
        assert_eq!(unlock.status, 401);
        assert!(!unlock.body.to_string().contains("not-a-real-password"));
    }

    #[test]
    fn capabilities_lists_actions_without_token() {
        let ctx = test_ctx();
        let reply = call(
            &ctx,
            "GET",
            "/v1/capabilities",
            Some("Bearer unit-test-token"),
            "",
        );
        assert_eq!(reply.status, 200);
        assert_eq!(reply.body["ok"], true);
        let actions = reply.body["data"]["actions"].as_array().unwrap();
        assert_eq!(actions.len(), ACTIONS.len());
        assert!(actions
            .iter()
            .any(|action| { action["method"] == "GET" && action["path"] == "/v1/notes" }));
        assert!(actions
            .iter()
            .any(|action| { action["method"] == "POST" && action["path"] == "/v1/vault/unlock" }));
        assert!(actions
            .iter()
            .any(|action| { action["path"] == "/v1/clipboard/history" }));
        let rendered = reply.body.to_string();
        assert!(!rendered.contains("unit-test-token"));
        assert!(!rendered.contains("password"));
    }

    #[test]
    fn rejects_non_loopback_bind() {
        let allowed = loopback_addr(DEFAULT_PORT).unwrap();
        assert_eq!(allowed.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(require_loopback(allowed).is_ok());
        let refused: SocketAddr = "0.0.0.0:47321".parse().unwrap();
        assert!(require_loopback(refused).is_err());
        let any_v6: SocketAddr = "[::]:47321".parse().unwrap();
        assert!(require_loopback(any_v6).is_err());
        assert!(!peer_allowed(refused));

        let listener = bind_loopback(0).unwrap();
        let bound = listener.local_addr().unwrap();
        assert!(bound.ip().is_loopback());
        assert_eq!(bound.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn clipboard_filter_and_history_do_not_require_pasteboard() {
        let ctx = test_ctx();
        let auth = Some("Bearer unit-test-token");
        let set = call(
            &ctx,
            "PUT",
            "/v1/clipboard/filter",
            auth,
            r#"{"enabled":false}"#,
        );
        assert_eq!(set.status, 200);
        assert_eq!(set.body["data"], false);
        assert!(!*ctx.state.clipboard_filter_enabled.lock().unwrap());

        ctx.state
            .clipboard_history
            .lock()
            .unwrap()
            .add("history-only".into(), false);
        let history = call(&ctx, "GET", "/v1/clipboard/history", auth, "");
        assert_eq!(history.status, 200);
        assert_eq!(history.body["data"][0]["text"], "history-only");
    }

    #[test]
    fn vault_routes_stay_locked_without_session() {
        let ctx = test_ctx();
        let auth = Some("Bearer unit-test-token");
        let locked = call(&ctx, "GET", "/v1/vault/secrets", auth, "");
        assert_eq!(locked.status, 400);
        assert_eq!(locked.body["ok"], false);
        assert!(locked.body["error"].as_str().unwrap().contains("locked"));

        let reveal = call(&ctx, "POST", "/v1/vault/secrets/0/reveal", auth, "");
        assert_eq!(reveal.status, 400);
        assert_eq!(reveal.body["ok"], false);

        let add = call(
            &ctx,
            "POST",
            "/v1/vault/secrets",
            auth,
            r#"{"name":"n","path":"p","value":"v"}"#,
        );
        assert_eq!(add.status, 400);
        assert!(!crate::vault_is_unlocked(&ctx.state));

        let lock = call(&ctx, "POST", "/v1/vault/lock", auth, "");
        assert_eq!(lock.status, 200);
        assert_eq!(lock.body["data"]["unlocked"], false);
    }

    #[test]
    fn note_path_uses_existing_filename_checks() {
        let ctx = test_ctx();
        let reply = call(
            &ctx,
            "GET",
            "/v1/notes/..%2F..%2Fetc%2Fpasswd",
            Some("Bearer unit-test-token"),
            "",
        );
        assert_eq!(reply.status, 400);
        assert_eq!(reply.body["ok"], false);
        let error = reply.body["error"].as_str().unwrap();
        assert!(error.contains("Invalid") || error.contains(".md"));
        assert!(!error.contains("root:"));
    }

    #[test]
    fn port_file_is_public_and_token_file_is_private() {
        let dir = std::env::temp_dir().join(format!(
            "slate-agent-api-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let token_path = dir.join("agent-api.token");
        let port_path = dir.join("agent-api.json");
        write_private_file(&token_path, "unit-test-token-not-real").unwrap();
        write_port_file(&port_path, DEFAULT_PORT).unwrap();

        let port_body = std::fs::read_to_string(&port_path).unwrap();
        assert_eq!(port_body, "{\"port\":47321}\n");
        assert!(!port_body.contains("unit-test-token"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let token_mode = std::fs::metadata(&token_path).unwrap().permissions().mode() & 0o777;
            let port_mode = std::fs::metadata(&port_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(token_mode, 0o600);
            assert_eq!(port_mode, 0o644);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn socket_rejects_missing_auth() {
        let listener = bind_loopback(0).unwrap();
        let port = listener.local_addr().unwrap().port();
        let ctx = test_ctx();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let worker = thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while !stop_flag.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, peer)) => {
                        if peer_allowed(peer) {
                            let _ = serve_client(&mut stream, &ctx);
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .write_all(
                b"GET /v1/capabilities HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("HTTP/1.1 401"));
        assert!(text.contains("\"ok\":false"));
        assert!(!text.contains("unit-test-token"));

        stop.store(true, Ordering::SeqCst);
        worker.join().unwrap();
    }
}
