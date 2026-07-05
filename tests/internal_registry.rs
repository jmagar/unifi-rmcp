use rustifi::api::ApiSourceFamily;
use rustifi::capabilities::{all_capabilities, find_capability};
use serde_json::Value;

#[test]
fn internal_registry_contains_reference_count() {
    let inventory: Value =
        serde_json::from_str(include_str!("../data/unifi_internal_reference_tools.json"))
            .expect("internal reference inventory should parse");
    assert_eq!(inventory["count"].as_u64(), Some(180));
    assert_eq!(
        inventory["tools"].as_array().expect("tools array").len(),
        180
    );

    let internal = all_capabilities()
        .iter()
        .filter(|cap| cap.source == ApiSourceFamily::Internal)
        .collect::<Vec<_>>();
    assert_eq!(internal.len(), 16);
}

#[test]
fn existing_internal_actions_are_registered() {
    for action in [
        "clients", "devices", "wlans", "health", "alarms", "events", "sysinfo", "me",
    ] {
        let cap = find_capability(action).unwrap_or_else(|| panic!("missing {action}"));
        assert_eq!(cap.source, ApiSourceFamily::Internal);
    }
}

#[test]
fn internal_gap_examples_are_registered() {
    for action in [
        "internal_list_alarms",
        "internal_list_events",
        "internal_get_network_health",
        "internal_list_port_forwards",
        "internal_list_dns_records",
        "internal_get_switch_ports",
        "internal_trigger_rf_scan",
    ] {
        let cap = find_capability(action).unwrap_or_else(|| panic!("missing {action}"));
        assert_eq!(cap.source, ApiSourceFamily::Internal);
    }
}

#[test]
fn unverified_internal_reference_rows_are_not_runtime_capabilities() {
    for cap in all_capabilities()
        .iter()
        .filter(|cap| cap.source == ApiSourceFamily::Internal)
    {
        let path = cap.path.as_deref().unwrap_or_default();
        assert!(
            !path.contains("/inventory/"),
            "{} exposes {path}",
            cap.action
        );
        assert!(!path.contains("/details/"), "{} exposes {path}", cap.action);
        assert!(
            !path.contains("/mcp_helper/"),
            "{} exposes {path}",
            cap.action
        );
    }
    assert!(find_capability("internal_get_networks").is_none());
}
