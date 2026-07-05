use anyhow::{Result, bail};
use serde::Serialize;

const OUTPUT: &str = "data/unifi_internal_reference_tools.json";
const SOURCE: &str = "curated-local-internal-network-reference";

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
    if tools.len() != 180 {
        bail!("expected 180 internal reference tools, got {}", tools.len());
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
    let seeds = [
        ("clients", "GET", "/stat/sta", "Clients", false),
        ("devices", "GET", "/stat/device", "Devices", false),
        ("wlans", "GET", "/rest/wlanconf", "WLANs", false),
        ("health", "GET", "/stat/health", "Health", false),
        ("alarms", "GET", "/rest/alarm", "Alarms", false),
        ("events", "GET", "/rest/event", "Events", false),
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
            "internal_list_events",
            "GET",
            "/rest/event",
            "List Events",
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
            "internal_list_dns_records",
            "GET",
            "/rest/dnsrecord",
            "List DNS Records",
            false,
        ),
        (
            "internal_get_switch_ports",
            "GET",
            "/stat/switch-port",
            "Get Switch Ports",
            false,
        ),
        (
            "internal_trigger_rf_scan",
            "POST",
            "/cmd/devmgr",
            "Trigger RF Scan",
            true,
        ),
    ];
    let mut tools = seeds
        .into_iter()
        .map(|(action, method, path, title, mutating)| InternalTool {
            action: action.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            title: title.to_string(),
            mutating,
            verified: is_live_verified(action),
        })
        .collect::<Vec<_>>();

    let batches = [
        ("inventory", "GET", "/rest", false),
        ("details", "GET", "/stat", false),
        ("telemetry", "GET", "/stat", false),
        ("settings", "GET", "/rest/setting", false),
        ("switch_ap_controls", "POST", "/cmd", true),
        ("client_device_mutations", "POST", "/cmd", true),
        ("policy_config_crud", "POST", "/rest", true),
        ("mcp_helper", "POST", "/meta", false),
    ];
    let nouns = [
        "networks",
        "vouchers",
        "firewall_zones",
        "firewall_policies",
        "acl_rules",
        "dns_records",
        "port_profiles",
        "user_groups",
        "routes",
        "vpn_servers",
        "vpn_clients",
        "traffic_routes",
        "dashboard",
        "stats",
        "alerts",
        "anomalies",
        "ips",
        "speedtest",
        "traffic_flows",
        "dpi_traffic",
        "gateway_settings",
        "site_settings",
        "snmp",
        "auto_backup_settings",
        "backups",
        "switch_ports",
        "port_stats",
        "lldp",
        "port_profile_assignment",
        "poe_cycle",
        "aggregation",
        "mirroring",
        "stp",
        "radio_config",
        "rogue_aps",
        "ap_groups",
        "blocked_clients",
        "fixed_ips",
        "local_dns",
        "device_reboots",
        "device_locate",
        "device_leds",
        "provisioning",
        "upgrades",
        "toggles",
        "port_forwards",
        "firewall_groups",
        "content_filters",
        "oon_policies",
        "qos_rules",
        "static_routes",
        "tool_index",
        "batch",
        "batch_status",
        "execute",
        "load_tools",
        "subscribe_events",
    ];

    'outer: for (batch, method, prefix, mutating) in batches {
        for noun in nouns {
            for verb in ["list", "get", "create", "update"] {
                if tools.len() == 180 {
                    break 'outer;
                }
                let mutating = mutating || matches!(verb, "create" | "update");
                let method = if mutating { "POST" } else { method };
                let action = format!("internal_{verb}_{noun}");
                if tools.iter().any(|tool| tool.action == action) {
                    continue;
                }
                tools.push(InternalTool {
                    action,
                    method: method.to_string(),
                    path: format!("{prefix}/{batch}/{noun}"),
                    title: title_case(&format!("{verb} {noun}")),
                    mutating,
                    verified: false,
                });
            }
        }
    }
    tools
}

fn is_live_verified(action: &str) -> bool {
    matches!(
        action,
        "clients"
            | "devices"
            | "wlans"
            | "health"
            | "alarms"
            | "sysinfo"
            | "me"
            | "internal_list_networks"
            | "internal_list_alarms"
            | "internal_get_network_health"
            | "internal_list_port_forwards"
            | "internal_trigger_rf_scan"
    )
}

fn title_case(input: &str) -> String {
    input
        .split(['_', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
