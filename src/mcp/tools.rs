use serde_json::{json, Value};

use crate::actions::ActionRequest;

use super::AppState;

/// Thin shim — parse args, call service, return Value. No logic here.
pub(super) async fn execute_tool(
    state: &AppState,
    name: &str,
    args: Value,
) -> anyhow::Result<Value> {
    match name {
        "unifi" => dispatch(state, args).await,
        _ => Err(anyhow::anyhow!("unknown tool: {name}")),
    }
}

async fn dispatch(state: &AppState, args: Value) -> anyhow::Result<Value> {
    let action =
        string_arg(&args, "action").ok_or_else(|| anyhow::anyhow!("action is required"))?;
    match action.as_str() {
        "help" => Ok(json!({ "help": HELP_TEXT })),
        _ => {
            let mut params = args.get("params").cloned().unwrap_or_else(|| json!({}));
            if let Some(limit) = usize_arg(&args, "limit")? {
                merge_param(&mut params, "limit", json!(limit));
            }
            let confirm = args
                .get("confirm")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            state
                .service
                .execute(ActionRequest {
                    action,
                    params,
                    confirm,
                })
                .await
        }
    }
}

fn string_arg(args: &Value, name: &str) -> Option<String> {
    args.get(name).and_then(|v| v.as_str()).map(String::from)
}

fn merge_param(params: &mut Value, key: &str, value: Value) {
    if !params.is_object() {
        *params = json!({});
    }
    if let Some(object) = params.as_object_mut() {
        object.insert(key.to_string(), value);
    }
}

fn usize_arg(args: &Value, name: &str) -> anyhow::Result<Option<usize>> {
    let Some(v) = args.get(name) else {
        return Ok(None);
    };
    v.as_u64()
        .map(|n| Some(n as usize))
        .ok_or_else(|| anyhow::anyhow!("`{name}` must be a non-negative integer"))
}

const HELP_TEXT: &str = r#"# unifi MCP Tool

Read-only access to UniFi network controllers via REST API.
Set the required `action` argument to select the operation.

## Network
- `clients`   — Connected wireless and wired clients
- `devices`   — Network devices: APs, switches, gateways
- `wlans`     — WiFi network configurations
- `health`    — Site health summary
- `alarms`    — Active alarms and alerts
- `events`    — Recent events (optional `limit` integer)
- `sysinfo`   — Controller system information
- `me`        — Authenticated user info

## Meta
- `help`      — This documentation
"#;
