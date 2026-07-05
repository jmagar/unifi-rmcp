use rustifi::api::ApiSourceFamily;
use rustifi::capabilities::{all_capabilities, find_capability};

#[test]
fn internal_registry_contains_reference_count() {
    let internal = all_capabilities()
        .iter()
        .filter(|cap| cap.source == ApiSourceFamily::Internal)
        .collect::<Vec<_>>();
    assert_eq!(internal.len(), 180);
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
