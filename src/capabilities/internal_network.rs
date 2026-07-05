use serde::Deserialize;

use crate::api::ApiSourceFamily;
use crate::capabilities::Capability;

#[derive(Debug, Deserialize)]
struct Inventory {
    tools: Vec<Tool>,
}

#[derive(Debug, Deserialize)]
struct Tool {
    action: String,
    method: String,
    path: String,
    title: String,
    mutating: bool,
    verified: bool,
}

pub fn capabilities() -> Vec<Capability> {
    let inventory: Inventory = serde_json::from_str(include_str!(
        "../../data/unifi_internal_reference_tools.json"
    ))
    .expect("internal UniFi inventory should be valid JSON");
    let mut caps = inventory
        .tools
        .into_iter()
        .filter(|tool| tool.verified)
        .map(|tool| Capability {
            action: tool.action,
            title: tool.title,
            source: ApiSourceFamily::Internal,
            method: Some(tool.method),
            path: Some(tool.path),
            mutating: tool.mutating,
            requires_confirmation: tool.mutating,
        })
        .collect::<Vec<_>>();
    caps.extend([
        hybrid("list_clients", "List Clients"),
        hybrid("list_devices", "List Devices"),
        hybrid("list_networks", "List Networks"),
        hybrid("list_wifi", "List WiFi"),
        hybrid("get_system_info", "Get System Info"),
    ]);
    caps
}

fn hybrid(action: &str, title: &str) -> Capability {
    Capability {
        action: action.to_string(),
        title: title.to_string(),
        source: ApiSourceFamily::Hybrid,
        method: None,
        path: None,
        mutating: false,
        requires_confirmation: false,
    }
}
