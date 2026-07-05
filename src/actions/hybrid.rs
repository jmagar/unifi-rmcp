use anyhow::{bail, Result};
use serde_json::{json, Value};

pub fn resolve(action: &str, params: &Value) -> Result<(&'static str, Value)> {
    let prefer_internal = params
        .get("prefer")
        .and_then(Value::as_str)
        .map(|prefer| prefer.eq_ignore_ascii_case("internal"))
        .unwrap_or(false);
    let target = if prefer_internal {
        internal_target(action)
    } else {
        official_target(action)
    };
    let Some(target) = target else {
        bail!("unknown hybrid action: {action}");
    };
    Ok((target, normalize_params(params)))
}

fn official_target(action: &str) -> Option<&'static str> {
    match action {
        "list_clients" => Some("official_list_clients"),
        "list_devices" => Some("official_list_devices"),
        "list_networks" => Some("official_list_networks"),
        "list_wifi" => Some("official_list_wifi"),
        "get_system_info" => Some("official_get_info"),
        _ => None,
    }
}

fn internal_target(action: &str) -> Option<&'static str> {
    match action {
        "list_clients" => Some("clients"),
        "list_devices" => Some("devices"),
        "list_networks" => Some("internal_list_networks"),
        "list_wifi" => Some("wlans"),
        "get_system_info" => Some("sysinfo"),
        _ => None,
    }
}

fn normalize_params(params: &Value) -> Value {
    let mut value = params.clone();
    if let Some(object) = value.as_object_mut() {
        object.remove("prefer");
    }
    if value.is_null() {
        json!({})
    } else {
        value
    }
}
