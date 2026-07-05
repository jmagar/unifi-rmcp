use serde::Deserialize;

use crate::api::ApiSourceFamily;
use crate::capabilities::Capability;

#[derive(Debug, Deserialize)]
struct Inventory {
    operations: Vec<Operation>,
}

#[derive(Debug, Deserialize)]
struct Operation {
    method: String,
    path: String,
    operation_id: String,
    summary: String,
}

pub fn capabilities() -> Vec<Capability> {
    let inventory: Inventory = serde_json::from_str(include_str!(
        "../../data/unifi_official_network_v10_3_58.json"
    ))
    .expect("official UniFi inventory should be valid JSON");
    inventory
        .operations
        .into_iter()
        .map(|operation| {
            let mutating = !operation.method.eq_ignore_ascii_case("GET");
            Capability {
                action: action_name(&operation),
                title: operation.summary,
                source: ApiSourceFamily::Official,
                method: Some(operation.method),
                path: Some(operation.path),
                mutating,
                requires_confirmation: mutating,
            }
        })
        .collect()
}

fn action_name(operation: &Operation) -> String {
    let override_name = match operation.operation_id.as_str() {
        "ConnectorDelete" => Some("official_connector_delete"),
        "ConnectorGet" => Some("official_connector_get"),
        "ConnectorPatch" => Some("official_connector_patch"),
        "ConnectorPost" => Some("official_connector_post"),
        "ConnectorPut" => Some("official_connector_put"),
        "getSiteOverviewPage" => Some("official_list_sites"),
        "getConnectedClientOverviewPage" => Some("official_list_clients"),
        "getAdoptedDeviceOverviewPage" => Some("official_list_devices"),
        "getNetworksOverviewPage" => Some("official_list_networks"),
        "getWifiBroadcastPage" => Some("official_list_wifi"),
        _ => None,
    };
    override_name
        .map(str::to_string)
        .unwrap_or_else(|| format!("official_{}", camel_to_snake(&operation.operation_id)))
}

fn camel_to_snake(input: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if idx > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}
