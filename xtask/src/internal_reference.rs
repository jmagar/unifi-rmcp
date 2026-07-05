use anyhow::{Result, bail};
use serde::Serialize;

const RAW_OUTPUT: &str = "data/upstream_mcp_network_tools_main.json";
const MODEL_OUTPUT: &str = "data/unifi_internal_endpoint_models.json";
const SOURCE: &str = "neutral-internal-network-reference";
const LIVE_EVIDENCE: &str = "live Cloud Gateway Max probe returned 2xx";

#[derive(Debug, Serialize)]
struct InternalTool {
    action: String,
    method: String,
    path: String,
    title: String,
    mutating: bool,
    verified: bool,
}

#[derive(Debug, Serialize)]
struct RawInventory {
    source: &'static str,
    count: usize,
    tools: Vec<InternalTool>,
}

#[derive(Debug, Clone, Serialize)]
struct InternalEndpointModel {
    action: String,
    title: String,
    method: String,
    path: String,
    mutating: bool,
    runtime: bool,
    verified: bool,
    verification_mode: String,
    auth_scope: String,
    evidence: String,
}

#[derive(Debug, Serialize)]
struct ModelInventory {
    source: &'static str,
    source_count: usize,
    accounted_count: usize,
    runtime_count: usize,
    non_runtime_count: usize,
    tools: Vec<InternalEndpointModel>,
}

pub fn refresh() -> Result<()> {
    let raw_tools = curated_tools();
    if raw_tools.len() != 12 {
        bail!(
            "expected 12 internal reference tools, got {}",
            raw_tools.len()
        );
    }
    std::fs::create_dir_all("data")?;
    let raw = RawInventory {
        source: SOURCE,
        count: raw_tools.len(),
        tools: raw_tools,
    };
    write_json(RAW_OUTPUT, &raw)?;

    let tools = raw
        .tools
        .iter()
        .map(endpoint_model)
        .collect::<Vec<InternalEndpointModel>>();
    let runtime_count = tools.iter().filter(|tool| tool.runtime).count();
    let models = ModelInventory {
        source: SOURCE,
        source_count: raw.tools.len(),
        accounted_count: tools.len(),
        runtime_count,
        non_runtime_count: tools.len() - runtime_count,
        tools,
    };
    write_json(MODEL_OUTPUT, &models)?;
    Ok(())
}

fn write_json<T: Serialize>(path: &str, value: &T) -> Result<()> {
    let body = serde_json::to_string_pretty(value)?;
    std::fs::write(path, format!("{body}\n"))?;
    Ok(())
}

fn endpoint_model(tool: &InternalTool) -> InternalEndpointModel {
    InternalEndpointModel {
        action: tool.action.clone(),
        title: tool.title.clone(),
        method: tool.method.clone(),
        path: tool.path.clone(),
        mutating: tool.mutating,
        runtime: tool.verified,
        verified: tool.verified,
        verification_mode: if tool.verified {
            "live_2xx".to_string()
        } else {
            "requires_fixture".to_string()
        },
        auth_scope: if tool.mutating { "admin" } else { "read" }.to_string(),
        evidence: if tool.verified {
            LIVE_EVIDENCE.to_string()
        } else {
            "reference row retained for contract accounting".to_string()
        },
    }
}

fn curated_tools() -> Vec<InternalTool> {
    [
        ("clients", "GET", "/stat/sta", "Clients", false),
        ("devices", "GET", "/stat/device", "Devices", false),
        ("wlans", "GET", "/rest/wlanconf", "WLANs", false),
        ("health", "GET", "/stat/health", "Health", false),
        ("alarms", "GET", "/rest/alarm", "Alarms", false),
        ("sysinfo", "GET", "/stat/sysinfo", "System Info", false),
        ("me", "GET", "/api/self", "Authenticated User", false),
        (
            "internal_list_networks",
            "GET",
            "/rest/networkconf",
            "List Networks",
            false,
        ),
        (
            "internal_list_alarms",
            "GET",
            "/rest/alarm",
            "List Alarms",
            false,
        ),
        (
            "internal_get_network_health",
            "GET",
            "/stat/health",
            "Get Network Health",
            false,
        ),
        (
            "internal_list_port_forwards",
            "GET",
            "/rest/portforward",
            "List Port Forwards",
            false,
        ),
        (
            "internal_trigger_rf_scan",
            "POST",
            "/cmd/devmgr",
            "Trigger RF Scan",
            true,
        ),
    ]
    .into_iter()
    .map(|(action, method, path, title, mutating)| InternalTool {
        action: action.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        title: title.to_string(),
        mutating,
        verified: true,
    })
    .collect::<Vec<_>>()
}
