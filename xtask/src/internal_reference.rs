use anyhow::{Result, bail};
use serde::Serialize;

const OUTPUT: &str = "data/unifi_internal_reference_tools.json";
const SOURCE: &str = "live-verified-internal-network-reference";

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
struct Inventory {
    source: &'static str,
    count: usize,
    tools: Vec<InternalTool>,
}

pub fn refresh() -> Result<()> {
    let tools = curated_tools();
    if tools.len() != 12 {
        bail!("expected 12 internal reference tools, got {}", tools.len());
    }
    std::fs::create_dir_all("data")?;
    let inventory = Inventory {
        source: SOURCE,
        count: tools.len(),
        tools,
    };
    let body = serde_json::to_string_pretty(&inventory)?;
    std::fs::write(OUTPUT, format!("{body}\n"))?;
    Ok(())
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
